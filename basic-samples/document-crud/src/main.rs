use azure_core::StatusCode;
use azure_data_cosmos::{CosmosClient, PartitionKey};
use azure_identity::DefaultAzureCredential;
use futures::StreamExt;
use serde_json::json;
use std::env;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Replace with your actual Cosmos DB credentials

    // Load cosmos db environment variables
    match env::var("COSMOSDB_ENDPOINT") {
        Ok(value) => println!("COSMOSDB_ENDPOINT is: {}", value),
        Err(e) => println!("Couldn't read COSMOSDB_ENDPOINT ({})", e),
    }

    let endpoint = env::var("COSMOSDB_ENDPOINT")?;
    let credential = DefaultAzureCredential::new()?;

    // Create a Cosmos client
    let client = CosmosClient::new(&endpoint, credential, None)?;

    // Set database (database must already exist, create database not support in RBAC)
    let database = env::var("COSMOSDB_DATABASE").map_err(|_| "COSMOSDB_DATABASE not set")?;

    // Set container (ensure this container already exists with partition key of "/pk")
    let container = env::var("COSMOSDB_CONTAINER").map_err(|_| "COSMOSDB_CONTAINER not set")?;

    let db_client = client.database_client(&database);
    let container_client = db_client.container_client(&container);

    let items = vec![
        json!({
            "id": "my-item-id-value-1",
            "pk": "my-item-partition-key-value",
            "name": "my-item-name-1",
        }),
        json!({
            "id": "my-item-id-value-2",
            "pk": "my-item-partition-key-value",
            "name": "my-item-name-2",
        }),
    ];

    for item in items {
        let item_id = item["id"].clone();
        match container_client
            .create_item("my-item-partition-key-value", item, None)
            .await
        {
            Ok(_) => {
                println!("Created item");
            }
            Err(e) if e.http_status() == Some(StatusCode::Conflict) => {
                return Err(format!("Document with ID '{}' already exists.", item_id).into());
            }
            Err(e) => return Err(e.into()),
        }
    }

    // read item
    let read_response = container_client
        .read_item::<serde_json::Value>("my-item-partition-key-value", "my-item-id-value-1", None)
        .await;
    match read_response {
        Err(e) if e.http_status() == Some(StatusCode::NotFound) => println!("Item not found!"),
        Ok(r) => {
            let body_bytes = r.into_body().collect().await?;
            let item: serde_json::Value = serde_json::from_slice(&body_bytes)?;
            println!("Found item:");
            println!("{:#?}", item);
        }
        Err(e) => return Err(e.into()),
    };

    // query items
    let query = "SELECT * FROM c";

    let db_client = client.database_client(&database);
    let container_client = db_client.container_client(&container);

    let pk = PartitionKey::from("my-item-partition-key-value");
    let mut items =
        container_client.query_items::<serde_json::Value>(&query.to_string(), pk, None)?;

    while let Some(page) = items.next().await {
        let page = page?.into_body().collect().await?;
        println!("Query results page");
        println!("  Items:");
        let page_json: serde_json::Value = serde_json::from_slice(&page)?;
        if let Some(items) = page_json["Documents"].as_array() {
            for item in items {
                println!("    * {:#?}", item);
            }
        }
    }

    // upsert item
    let upsert_item = serde_json::json!({
        "id": "my-item-id-value-1",
        "pk": "my-item-partition-key-value",
        "name": "my-item-name-updated",
    });
    container_client
        .upsert_item("my-item-partition-key-value", upsert_item, None)
        .await?;
    println!("Upserted item.");

    // replace item

    let replace_item = serde_json::json!({
        "id": "my-item-id-value-2",
        "pk": "my-item-partition-key-value",
        "name": "my-item-name-replaced",
    });
    container_client
        .replace_item(
            "my-item-partition-key-value",
            "my-item-id-value-2",
            replace_item,
            None,
        )
        .await?;
    println!("Replaced item.");

    // delete items
    let item_ids = vec!["my-item-id-value-1", "my-item-id-value-2"];

    for item_id in item_ids {
        let delete_response = container_client
            .delete_item("my-item-partition-key-value", item_id, None)
            .await;

        match delete_response {
            Err(e) if e.http_status() == Some(StatusCode::NotFound) => {
                println!("Item with ID '{}' not found!", item_id);
            }
            Ok(_) => {
                println!("Item with ID '{}' deleted", item_id);
            }
            Err(e) => {
                eprintln!("Error deleting item with ID '{}': {}", item_id, e);
                return Err(e.into()); // Stop on the first error encountered
            }
        }
    }

    Ok(())
}
