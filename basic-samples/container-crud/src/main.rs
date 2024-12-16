use azure_data_cosmos::{
    models::{
        ContainerProperties, IndexingMode, IndexingPolicy, PartitionKeyDefinition, PropertyPath,
    },
    CosmosClient,
};
use azure_identity::DefaultAzureCredential;
use std::env;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load cosmos db environment variables
    match env::var("COSMOSDB_ENDPOINT") {
        Ok(value) => println!("COSMOSDB_ENDPOINT is: {}", value),
        Err(e) => println!("Couldn't read COSMOSDB_ENDPOINT ({})", e),
    }

    let endpoint = env::var("COSMOSDB_ENDPOINT").unwrap();
    let credential = DefaultAzureCredential::new().unwrap();

    // Create a Cosmos client
    let client = CosmosClient::new(endpoint, credential, None)?;

    // Set database (the database must exist - create database not supported in RBAC)
    let database = "my-database";

    // Create a container
    let container = "my-container";
    let partition_key = "/id".to_string();
    let container_id = container;
    let properties = ContainerProperties {
        id: container_id.to_string(),
        partition_key: PartitionKeyDefinition::new(vec![partition_key.clone()]),
        indexing_policy: Some(IndexingPolicy {
            automatic: true,
            indexing_mode: Some(IndexingMode::Consistent),
            included_paths: vec![PropertyPath {
                path: "/".to_string(),
            }],
            excluded_paths: vec![PropertyPath {
                path: "/objects/*".to_string(),
            }],
            composite_indexes: vec![],
            spatial_indexes: vec![],
            vector_indexes: vec![],
        }),
        ..Default::default()
    };
    client
        .database_client(database)
        .create_container(properties, None)
        .await?
        .deserialize_body()
        .await?
        .unwrap();

    print!("Container created");

    Ok(())
}
