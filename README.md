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

Samples for the Azure Cosmos DB for NoSQL Rust SDK. Both packages pin
`azure_data_cosmos` and `azure_identity` to stable `1.0.0`.

## Contents

| File/folder | Description |
|-------------|-------------|
| `basic-samples/Cargo.toml` | Cargo workspace for both samples. |
| `basic-samples/container-crud` | Creates a container with partition key `/pk` in an existing database. It does not read, update, or delete containers. |
| `basic-samples/document-crud` | Creates, reads, queries, upserts, replaces, and deletes sample documents, with readback assertions. |
| `.gitignore` | Excludes build output, generated lockfiles, and local environment files. |

## Prerequisites

* Rust 1.88 or newer and Cargo. Install with [rustup](https://rustup.rs/).
* Either an Azure Cosmos DB for NoSQL account or a local Cosmos DB emulator.
* A database created beforehand using the Azure Portal, Azure Resource Manager,
  or the emulator's Data Explorer. Neither sample creates a database.
* For hosted document operations: an existing container with partition key `/pk`,
  an Azure CLI sign-in, and a Cosmos DB data-plane role such as **Cosmos DB Built-in
  Data Contributor**, scoped to the sample database or container. Azure subscription
  Contributor alone does not grant document access.
* For the emulator: a trusted HTTPS certificate valid for the chosen loopback host,
  the emulator listening on port 8081, and its key supplied through `COSMOSDB_KEY`.
  Follow the [emulator certificate instructions](https://learn.microsoft.com/azure/cosmos-db/how-to-develop-emulator)
  for your operating system. Do not disable certificate validation.

## Setup

Clone the repository and keep your shell at the repository root for every command below:

```bash
git clone https://github.com/Azure-Samples/azure-cosmos-rust-nosql-api-samples.git
cd azure-cosmos-rust-nosql-api-samples
```

## Hosted account

The default authentication mode is `hosted`. It uses
[`DeveloperToolsCredential`](https://docs.rs/azure_identity/1.0.0/azure_identity/struct.DeveloperToolsCredential.html),
not `DefaultAzureCredential`. `COSMOSDB_REGION` is required and should name an Azure
region for proximity-based routing, for example `West US`. Use the full HTTPS account
endpoint, not a bare hostname. Hosted mode does not use `COSMOSDB_KEY`.

**Container creation limitation:** Cosmos DB native data-plane RBAC does not support
the SDK's container-create operation. The SDK exposes that operation through its
`control_plane` feature and requires key authentication. For hosted accounts, create
the database and container through the Azure Portal or Azure Resource Manager and
run only `document-crud`. Running `container-crud` with the hosted credential is
expected to fail authorization, even with a data-plane contributor role. These
samples intentionally do not offer cloud key authentication.

### Bash

Replace the account, database, container, and region values with your own:

```bash
az login
export COSMOSDB_AUTH_MODE=hosted
export COSMOSDB_ENDPOINT='https://<account>.documents.azure.com:443/'
export COSMOSDB_REGION='West US'
export COSMOSDB_DATABASE='rust-samples'
export COSMOSDB_CONTAINER='rust-crud'
cargo run --manifest-path basic-samples/Cargo.toml -p document-crud
```

### PowerShell

```powershell
az login
$env:COSMOSDB_AUTH_MODE = 'hosted'
$env:COSMOSDB_ENDPOINT = 'https://<account>.documents.azure.com:443/'
$env:COSMOSDB_REGION = 'West US'
$env:COSMOSDB_DATABASE = 'rust-samples'
$env:COSMOSDB_CONTAINER = 'rust-crud'
cargo run --manifest-path basic-samples/Cargo.toml -p document-crud
```

For production on Azure, use
[`ManagedIdentityCredential`](https://docs.rs/azure_identity/1.0.0/azure_identity/struct.ManagedIdentityCredential.html)
or an appropriate workload identity instead of a developer-tool credential. Assign
that identity the required Cosmos DB data-plane role. Changing the credential in
the client builder is an application change; these samples default to developer sign-in.

## Local emulator

Emulator access is opt-in with `COSMOSDB_AUTH_MODE=emulator`. Only HTTPS loopback
URLs on port 8081 are accepted, such as `https://localhost:8081/`,
`https://127.0.0.1:8081/`, or `https://[::1]:8081/`. The URL must not contain
credentials, a non-root path, a query, or a fragment. The emulator must actually
listen on the selected address and its certificate must match that host.

The key is read from the environment, never hardcoded or printed. Emulator mode
disables gateway v2 and binary encoding, and explicitly requires TLS certificate
validation. `COSMOSDB_REGION` is not used in this mode; the client uses `West US`
as its local routing preference. These settings do not establish compatibility
with every emulator version; compilation and offline unit tests do not replace
an end-to-end run against your emulator.

Create the `rust-samples` database in Data Explorer first. Use a fresh container
name, then run the container sample followed by the document sample.

### Bash

```bash
export COSMOSDB_AUTH_MODE=emulator
export COSMOSDB_ENDPOINT='https://localhost:8081/'
export COSMOSDB_DATABASE='rust-samples'
export COSMOSDB_CONTAINER='rust-crud'
read -rsp 'Emulator key: ' COSMOSDB_KEY
printf '\n'
export COSMOSDB_KEY
cargo run --manifest-path basic-samples/Cargo.toml -p container-crud &&
	cargo run --manifest-path basic-samples/Cargo.toml -p document-crud
unset COSMOSDB_KEY
```

### PowerShell 7

```powershell
$env:COSMOSDB_AUTH_MODE = 'emulator'
$env:COSMOSDB_ENDPOINT = 'https://localhost:8081/'
$env:COSMOSDB_DATABASE = 'rust-samples'
$env:COSMOSDB_CONTAINER = 'rust-crud'
$env:COSMOSDB_KEY = Read-Host 'Emulator key' -MaskInput
try {
		cargo run --manifest-path basic-samples/Cargo.toml -p container-crud
		if ($LASTEXITCODE -ne 0) { throw 'Container creation failed' }
		cargo run --manifest-path basic-samples/Cargo.toml -p document-crud
		if ($LASTEXITCODE -ne 0) { throw 'Document CRUD failed' }
} finally {
		Remove-Item Env:COSMOSDB_KEY -ErrorAction SilentlyContinue
}
```

## Assertions and reruns

`document-crud` uses IDs `my-item-id-value-1` and `my-item-id-value-2` in partition
`my-item-partition-key-value`. Use a dedicated sample container and do not run
multiple copies concurrently. It verifies the initial documents, runs a
parameterized query restricted to those IDs and that partition, verifies both
mutations with point reads, deletes both documents, and verifies their absence.
Other documents are not counted or changed. Service-managed metadata is excluded
from value comparisons.

A successful document run prints `All document CRUD assertions passed` and leaves
the database and container in place. Run only `document-crud` on subsequent runs.
`container-crud` is create-only and returns a conflict if its container exists.
Document creation also fails on a conflict rather than overwriting existing data.
An interrupted or failed run can leave sample items behind: inspect and remove
only those sample IDs in their sample partition, or select a new dedicated
container. Do not delete unrelated documents. Unexpected missing items, query
results, mutation values, and delete failures produce a nonzero exit.

For certificate errors, install/trust the emulator certificate and verify the
hostname; do not use a TLS bypass. For hosted authorization errors, check the
signed-in identity and Cosmos DB data-plane role scope. An HTTP or protocol error
against an emulator is not a successful CRUD verification; retain the error and
emulator version when reporting compatibility issues.

## Local validation

From the repository root, these commands format-check, compile, and run offline
unit tests without executing either sample's `main` or accessing Cosmos DB:

```text
cargo fmt --manifest-path basic-samples/Cargo.toml --all -- --check
cargo check --manifest-path basic-samples/Cargo.toml --workspace --all-targets
cargo test --manifest-path basic-samples/Cargo.toml --workspace --all-targets
```

The tests cover emulator URL validation and document field comparisons. Live CRUD
assertions run only through `cargo run`. Cargo generates a local lockfile; the
existing repository policy ignores it.

## Contributing

This project welcomes contributions and suggestions. Most contributions require you to agree to a Contributor License Agreement (CLA) declaring that you have the right to, and actually do, grant us the rights to use your contribution. For details, visit https://cla.opensource.microsoft.com.

When you submit a pull request, a CLA bot will automatically determine whether you need to provide a CLA and decorate the PR appropriately (e.g., status check, comment). Simply follow the instructions provided by the bot. You will only need to do this once across all repos using our CLA.

This project has adopted the [Microsoft Open Source Code of Conduct](https://opensource.microsoft.com/codeofconduct/). For more information see the [Code of Conduct FAQ](https://opensource.microsoft.com/codeofconduct/faq/) or contact [opencode@microsoft.com](mailto:opencode@microsoft.com) with any additional questions or comments.
