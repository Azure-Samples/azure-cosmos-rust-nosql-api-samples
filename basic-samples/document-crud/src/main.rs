use azure_data_cosmos::{
    options::{BinaryEncodingOptions, ConnectionPoolOptions, ServerCertificateValidation},
    AccountEndpoint, AccountReference, CosmosClient, CosmosRuntime, FeedScope, Query,
    RoutingStrategy,
};
use azure_identity::DeveloperToolsCredential;
use futures::TryStreamExt;
use serde_json::{json, Value};
use std::env;

fn emulator_endpoint(endpoint: &str) -> Result<AccountEndpoint, Box<dyn std::error::Error>> {
    let url = url::Url::parse(endpoint)?;
    let loopback = match url.host() {
        Some(url::Host::Domain(host)) => host == "localhost",
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        None => false,
    };
    if url.scheme() != "https"
        || !loopback
        || url.port() != Some(8081)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("Emulator mode requires an HTTPS loopback URL on port 8081 with no credentials, path, query, or fragment".into());
    }
    Ok(endpoint.parse()?)
}

fn assert_item(actual: &Value, expected: &Value) {
    assert_eq!(actual["id"], expected["id"], "Unexpected item ID");
    assert_eq!(actual["pk"], expected["pk"], "Unexpected partition key");
    assert_eq!(actual["name"], expected["name"], "Unexpected item name");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = env::var("COSMOSDB_ENDPOINT")?;
    let database = env::var("COSMOSDB_DATABASE").map_err(|_| "COSMOSDB_DATABASE not set")?;
    let container = env::var("COSMOSDB_CONTAINER").map_err(|_| "COSMOSDB_CONTAINER not set")?;
    let mode = match env::var("COSMOSDB_AUTH_MODE") {
        Ok(mode) => mode,
        Err(env::VarError::NotPresent) => "hosted".into(),
        Err(error) => return Err(error.into()),
    };
    let client = match mode.as_str() {
        "hosted" => {
            let region = env::var("COSMOSDB_REGION").map_err(|_| "COSMOSDB_REGION not set")?;
            let credential = DeveloperToolsCredential::new(None)?;
            CosmosClient::builder()
                .build(
                    AccountReference::with_credential(endpoint.parse()?, credential),
                    RoutingStrategy::ProximityTo(region.into()),
                )
                .await?
        }
        "emulator" => {
            let endpoint = emulator_endpoint(&endpoint)?;
            let key = env::var("COSMOSDB_KEY").map_err(|_| "COSMOSDB_KEY not set")?;
            let runtime = CosmosRuntime::builder()
                .with_connection_pool(
                    ConnectionPoolOptions::builder()
                        .with_gateway_v2_disabled(true)
                        .with_server_certificate_validation(ServerCertificateValidation::Required)
                        .build()?,
                )
                .build()
                .await?;
            CosmosClient::builder()
                .with_runtime(runtime)
                .with_binary_encoding_options(BinaryEncodingOptions::default().with_enabled(false))
                .build(
                    AccountReference::with_authentication_key(endpoint, key),
                    RoutingStrategy::ProximityTo("West US".into()),
                )
                .await?
        }
        _ => return Err("COSMOSDB_AUTH_MODE must be hosted or emulator".into()),
    };

    let db_client = client.database_client(&database);
    let container_client = db_client.container_client(&container, None).await?;
    let partition_key = "my-item-partition-key-value";

    let items = [
        json!({
            "id": "my-item-id-value-1",
            "pk": partition_key,
            "name": "my-item-name-1",
        }),
        json!({
            "id": "my-item-id-value-2",
            "pk": partition_key,
            "name": "my-item-name-2",
        }),
    ];

    for item in &items {
        let item_id = item["id"].as_str().unwrap();
        match container_client
            .create_item(partition_key, item_id, item, None)
            .await
        {
            Ok(_) => println!("Created item '{item_id}'"),
            Err(error) if error.status().is_conflict() => {
                return Err(format!("Sample item '{item_id}' already exists; use a fresh sample container or remove only leftover sample items before rerunning.").into());
            }
            Err(error) => return Err(error.into()),
        }
        let actual: Value = container_client
            .read_item(partition_key, item_id, None)
            .await?
            .into_model()?;
        assert_item(&actual, item);
        println!("Verified created item '{item_id}'");
    }

    let query = Query::from("SELECT * FROM c WHERE c.pk = @pk AND c.id IN (@first, @second)")
        .with_parameter("@pk", partition_key)?
        .with_parameter("@first", "my-item-id-value-1")?
        .with_parameter("@second", "my-item-id-value-2")?;
    let mut query_items = container_client
        .query_items::<Value>(query, FeedScope::partition(partition_key), None)
        .await?;
    let mut found = [false; 2];
    while let Some(item) = query_items.try_next().await? {
        let position = items
            .iter()
            .position(|expected| expected["id"] == item["id"])
            .expect("Query returned an unexpected item ID");
        assert!(!found[position], "Query returned a duplicate item");
        assert_item(&item, &items[position]);
        found[position] = true;
    }
    assert!(
        found.into_iter().all(|present| present),
        "Query missed a sample item"
    );
    println!("Verified parameterized query results");

    let upsert_item = json!({
        "id": "my-item-id-value-1",
        "pk": partition_key,
        "name": "my-item-name-updated",
    });
    container_client
        .upsert_item(partition_key, "my-item-id-value-1", &upsert_item, None)
        .await?;
    let actual: Value = container_client
        .read_item(partition_key, "my-item-id-value-1", None)
        .await?
        .into_model()?;
    assert_item(&actual, &upsert_item);
    println!("Verified upserted item");

    let replace_item = json!({
        "id": "my-item-id-value-2",
        "pk": partition_key,
        "name": "my-item-name-replaced",
    });
    container_client
        .replace_item(partition_key, "my-item-id-value-2", &replace_item, None)
        .await?;
    let actual: Value = container_client
        .read_item(partition_key, "my-item-id-value-2", None)
        .await?
        .into_model()?;
    assert_item(&actual, &replace_item);
    println!("Verified replaced item");

    for item_id in ["my-item-id-value-1", "my-item-id-value-2"] {
        container_client
            .delete_item(partition_key, item_id, None)
            .await?;
        match container_client
            .read_item(partition_key, item_id, None)
            .await
        {
            Err(error) if error.status().is_not_found() => {
                println!("Verified deleted item '{item_id}' is absent");
            }
            Err(error) => return Err(error.into()),
            Ok(_) => return Err(format!("Deleted item '{item_id}' is still present").into()),
        }
    }

    println!("All document CRUD assertions passed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{assert_item, emulator_endpoint};
    use serde_json::json;

    #[test]
    fn accepts_https_loopback_emulator_endpoints() {
        for endpoint in [
            "https://localhost:8081",
            "https://127.0.0.1:8081/",
            "https://[::1]:8081/",
        ] {
            assert!(emulator_endpoint(endpoint).is_ok());
        }
    }

    #[test]
    fn rejects_non_emulator_endpoints() {
        for endpoint in [
            "http://localhost:8081",
            "https://localhost",
            "https://localhost:443",
            "https://example.documents.azure.com:8081",
            "https://localhost.example.com:8081",
            "https://192.168.1.1:8081",
            "https://user:password@localhost:8081",
            "https://localhost:8081/dbs",
            "https://localhost:8081/?key=value",
            "https://localhost:8081/#fragment",
            "not a URL",
        ] {
            assert!(emulator_endpoint(endpoint).is_err());
        }
    }

    #[test]
    fn item_assertions_allow_service_metadata() {
        let expected = json!({"id": "sample", "pk": "partition", "name": "initial"});
        let actual = json!({"id": "sample", "pk": "partition", "name": "initial", "_etag": "etag"});
        assert_item(&actual, &expected);
    }

    #[test]
    fn item_assertions_reject_incorrect_or_missing_sample_fields() {
        let expected = json!({"id": "sample", "pk": "partition", "name": "updated"});
        for field in ["id", "pk", "name"] {
            let mut actual = expected.clone();
            actual[field] = json!("incorrect");
            assert!(std::panic::catch_unwind(|| assert_item(&actual, &expected)).is_err());
            actual.as_object_mut().unwrap().remove(field);
            assert!(std::panic::catch_unwind(|| assert_item(&actual, &expected)).is_err());
        }
    }
}
