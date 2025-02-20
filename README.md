---
page_type: sample
languages:
- rust
products:
- rust sdk
description: "Sample code repo for Azure Cosmos DB Rust SDK for NoSQL API"
urlFragment: ""
---

# Azure Cosmos DB Rust NoSQL API Samples

Sample code repository for Azure Cosmos DB Rust SDK for NoSQL API. By cloning and running these samples, and then studying their implementations, you will have examples for sending various requests to Azure Cosmos DB from the Rust SDK via the NoSQL API.

## Contents

| File/folder       | Description                                 |
|-------------------|---------------------------------------------|
| `basic-samples`    | Basic CRUD operations and container management samples. |
| `advanced-samples` | More advanced features and usage examples.  |
| `.gitignore`      | Defines what to ignore at commit time.      |
| `Cargo.toml`      | Cargo configuration file for Rust.          |
| `README.md`       | This README file.                           |
| `LICENSE`         | The license for the sample.                 |

## Prerequisites

* Rust (1.56 or newer)
* Setting up an Azure Cosmos DB account through the Azure Portal. The **Create a database account** section of [this guide](https://docs.microsoft.com/en-us/azure/cosmos-db/create-sql-api-rust) walks you through account creation.
* The hostname and master key for your Azure Cosmos DB account
* Azure SDK for Rust `azure_identity` crate for authentication

## Setup

Clone the sample to your PC. You can run the samples using Cargo.

```bash
git clone https://github.com/Azure-Samples/azure-cosmos-rust-nosql-api-samples.git
cd azure-cosmos-rust-nosql-api-samples/basic-samples/container-crud
```

## Running the samples

These environment variables must be set in order to give the samples read/write access to your account:

```bash
export COSMOSDB_ENDPOINT=your_account_hostname
```

### Authentication with DefaultAzureCredential

To authenticate with Azure Cosmos DB, you can use `DefaultAzureCredential` from the `azure_identity` crate. A simple way to get the sample working is to log in to Azure using the Azure CLI:

```bash
az login
```

This credential type will automatically attempt to authenticate via multiple methods, including environment variables, managed identities, and Azure CLI credentials. Ensure that you have the required authentication setup for your Azure account.

For more details on the `azure_identity` crate, refer to the documentation [here](https://docs.rs/azure_identity/latest/azure_identity/).

To run a sample, use Cargo:

```bash
cd basic-samples/document-crud 
cargo run
```

This will build and run the selected sample.

## Key concepts

These samples include:
* `basic-samples` - container and document crud

## Contributing

This project welcomes contributions and suggestions. Most contributions require you to agree to a Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
