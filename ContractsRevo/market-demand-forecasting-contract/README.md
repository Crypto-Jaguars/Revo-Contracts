# Market Demand Forecasting Smart Contract

This smart contract provides a decentralized solution for forecasting market demand for agricultural products on the Stellar network. It helps farmers make informed decisions about crop planning to optimize yield and maximize profitability by analyzing recent, oracle-provided data.

## 🏗 Contract Architecture

The contract is designed with a clear separation of concerns to ensure modularity and scalability.

- **`lib.rs`**: The main entry point, defining the contract interface and exporting public functions.
- **`forecasting.rs`**: Contains the core logic for recording demand forecasts provided by an oracle.
- **`data.rs`**: Manages the registration and retrieval of product data.
- **`recommendations.rs`**: Implements the logic for generating crop planting recommendations based on an average of recent demand forecasts.
- **`storage.rs`**: Defines all on-chain data structures (`Product`, `DemandForecast`) and storage keys.
- **`utils.rs`**: Provides shared utility functions, such as deterministic ID generation.
- **`error.rs`**: Defines custom contract errors for predictable and clear error handling.

## 🗂 Features

- **Oracle-Driven Forecasts**: A trusted oracle performs complex off-chain analysis and pushes a final `predicted_demand` value to the contract.
- **Time-Windowed Recommendations**: The contract generates recommendations by averaging forecasts submitted within a specific time window (e.g., the last 7 days), providing a stable and accurate view of the current market trend.
- **On-Chain Metadata**: Stores essential forecast metadata on-chain for transparency and accessibility, while larger datasets can be stored off-chain (e.g., on IPFS) with their hashes recorded on-chain.
- **Secure and Scalable**: Uses an admin-managed oracle for data integrity and efficient, region-based indexing to handle a large number of forecasts.

## 🔑 Key Functions

### State-Changing Functions

- `initialize(admin: Address)`: Initializes the contract and sets the admin. Must be called once upon deployment.
- `set_oracle(admin: Address, oracle: Address)`: Sets the trusted oracle address. Only the admin can call this.
- `register_product(name: String, historical_demand: Vec<i128>)`: Registers a new agricultural product.
- `generate_forecast(oracle: Address, product_id: BytesN<32>, region: String, predicted_demand: i128, data_hash: BytesN<32>)`: Called by the oracle to submit data and record a new forecast.

### Read-Only Functions

- `get_forecast(forecast_id: BytesN<32>)`: Retrieves a specific demand forecast.
- `get_product(product_id: BytesN<32>)`: Retrieves details for a registered product.
- `list_forecasts(product_id: Option<BytesN<32>>, region: Option<String>)`: Returns a list of all forecasts, with optional filters.
- `generate_recommendation(region: String, time_window_days: u64)`: Generates a ranked list of crop recommendations for a given region based on forecasts within the specified number of days.

## 📦 Deployment and Usage

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)

### Build and Test

```bash
# Build the contract WASM
make build

# Run the test suite
make test
