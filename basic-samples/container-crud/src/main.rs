use azure_data_cosmos::{
    models::{ContainerProperties, IndexingMode, IndexingPolicy, PartitionKeyDefinition},
    options::{BinaryEncodingOptions, ConnectionPoolOptions, ServerCertificateValidation},
    AccountEndpoint, AccountReference, CosmosClient, CosmosRuntime, RoutingStrategy,
};
use azure_identity::DeveloperToolsCredential;
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

    let mut indexing_policy = IndexingPolicy::default();
    indexing_policy.automatic = true;
    indexing_policy.indexing_mode = Some(IndexingMode::Consistent);
    indexing_policy.included_paths = vec!["/*".into()];
    indexing_policy.excluded_paths = vec!["/objects/*".into()];
    let properties =
        ContainerProperties::new(container, PartitionKeyDefinition::new(vec!["/pk".into()]))
            .with_indexing_policy(indexing_policy);
    client
        .database_client(&database)
        .create_container(properties, None)
        .await?;

    println!("Container created");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::emulator_endpoint;

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
}
