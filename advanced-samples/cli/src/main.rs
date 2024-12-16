// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT License.

use azure_data_cosmos::CosmosClient;
use azure_identity::DefaultAzureCredential;
use clap::{Args, CommandFactory, Parser, Subcommand};
use std::error::Error;

mod create;
mod delete;
mod metadata;
mod query;
mod read;
mod replace;
mod upsert;

/// An example which shows how you might create a CLI tool for managing items in Cosmos DB using the Rust SDK.
///
/// NOTE: This is not intended to be a general-purpose CLI for managing items in Cosmos DB.
/// It exists for illustrative purposes, and could also be used to simplify ad-hoc end-to-end testing.
///
/// usage examples when running from command line:
///
///
/// Note: database must already exist as database creation is not supported when using RBAC or DefaultAzureCredential.
/// If you are using a service principal that did not create the account, you must also create containers using the Azure Portal or the Azure CLI.
///
/// First sign in using az login, then set the subscription and resource group.
///
/// Set an environment variable for your Cosmos DB endpoint: COSMOSDB_ENDPOINT
///
/// For windows (use "$COSMOSDB_ENDPOINT" for Linux):
///
/// create container:
///
/// cargo run -- "%COSMOSDB_ENDPOINT%" create container database-name --id container-name --partition-key "/id"
///
/// create item:
///
/// cargo run -- "%COSMOSDB_ENDPOINT%" create item database-name container-name --partition-key "my-item-id" --json "{\"id\": \"my-item-id\", \"name\": \"my-item-name\"}"
///
/// upsert item:
///
/// cargo run -- "%COSMOSDB_ENDPOINT%" upsert --partition-key "my-item-id" --json "{\"id\": \"my-item-id\", \"name\": \"my-item-name-changed\"}" database-name container-name
///
/// read item:
///
/// cargo run -- "%COSMOSDB_ENDPOINT%" read database-name container-name --item-id "my-item-id" --partition-key "my-item-id"
///
/// query items:
///
/// cargo run -- "%COSMOSDB_ENDPOINT%" query items database-name container-name "select * from c" --partition-key "my-item-id"
///
/// delete item:
///
/// cargo run -- "%COSMOSDB_ENDPOINT%" delete item database-name container-name --item-id "my-item-id" --partition-key "my-item-id"

///
#[derive(Clone, Parser)]
struct ProgramArgs {
    #[clap(flatten)]
    shared_args: SharedArgs,

    #[command(subcommand)]
    subcommand: Option<Subcommands>,
}

#[derive(Clone, Args)]
struct SharedArgs {
    /// The Cosmos DB endpoint to connect to.
    endpoint: String,

    /// An authentication key to use when connecting to the Cosmos DB account. If omitted, the connection will use Entra ID.
    #[clap(long)]
    key: Option<String>,
}

#[derive(Clone, Subcommand)]
enum Subcommands {
    Create(create::CreateCommand),
    Delete(delete::DeleteCommand),
    Metadata(metadata::MetadataCommand),
    Query(query::QueryCommand),
    Read(read::ReadCommand),
    Replace(replace::ReplaceCommand),
    Upsert(upsert::UpsertCommand),
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = ProgramArgs::parse();

    let Some(cmd) = args.subcommand else {
        ProgramArgs::command().print_long_help()?;
        return Ok(());
    };

    let client = create_client(&args.shared_args)?;

    match cmd {
        Subcommands::Create(cmd) => cmd.run(client).await,
        Subcommands::Delete(cmd) => cmd.run(client).await,
        Subcommands::Metadata(cmd) => cmd.run(client).await,
        Subcommands::Query(cmd) => cmd.run(client).await,
        Subcommands::Read(cmd) => cmd.run(client).await,
        Subcommands::Replace(cmd) => cmd.run(client).await,
        Subcommands::Upsert(cmd) => cmd.run(client).await,
    }
}

fn create_client(args: &SharedArgs) -> Result<CosmosClient, Box<dyn Error>> {
    if let Some(key) = args.key.as_ref() {
        #[cfg(feature = "key_auth")]
        {
            Ok(CosmosClient::with_key(&args.endpoint, key.clone(), None)?)
        }
        #[cfg(not(feature = "key_auth"))]
        {
            let _ = key; // Mark 'key' as used to make the compiler happy.
            Err("cannot authenticate with a key unless the 'key_auth' feature is enabled".into())
        }
    } else {
        let cred = DefaultAzureCredential::new().unwrap();
        Ok(CosmosClient::new(&args.endpoint, cred, None)?)
    }
}
