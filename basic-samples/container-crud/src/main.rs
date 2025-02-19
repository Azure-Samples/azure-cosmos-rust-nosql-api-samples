use azure_data_cosmos::{
    models::{ContainerProperties, IndexingMode, IndexingPolicy, PartitionKeyDefinition},
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

    let endpoint = env::var("COSMOSDB_ENDPOINT")?;
    let credential = DefaultAzureCredential::new()?;

    // Create a Cosmos client
    let client = CosmosClient::new(&endpoint, credential, None)?;

    // Set database (the database must exist - create database not supported in RBAC)
    let database = env::var("COSMOSDB_DATABASE").map_err(|_| "COSMOSDB_DATABASE not set")?;

    // Create a container
    let container = env::var("COSMOSDB_CONTAINER").map_err(|_| "COSMOSDB_CONTAINER not set")?;
    let partition_key = "/pk".to_string();
    let container_id = container;
    let properties = ContainerProperties {
        id: container_id.into(),
        partition_key: PartitionKeyDefinition::new(vec![partition_key.clone()]),
        indexing_policy: Some(IndexingPolicy {
            automatic: true,
            indexing_mode: Some(IndexingMode::Consistent),
            included_paths: vec!["/".into()],
            excluded_paths: vec!["/objects/*".into()],
            composite_indexes: vec![],
            spatial_indexes: vec![],
            vector_indexes: vec![],
        }),
        ..Default::default()
    };
    client
        .database_client(&database)
        .create_container(properties, None)
        .await?
        .into_body()
        .await?;

    print!("Container created");

    Ok(())
}
