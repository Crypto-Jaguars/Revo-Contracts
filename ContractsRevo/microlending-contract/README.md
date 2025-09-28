# Microlending Smart Contract

A comprehensive peer-to-peer microlending smart contract built on the Stellar blockchain using Soroban SDK. This contract enables decentralized lending with support for multiple lenders, automated repayment scheduling, collateral management, and default handling.

## Project Structure

This contract follows the recommended Soroban project structure:
```text
ContractsRevo/microlending-contract/
├── src/
│   ├── lib.rs          # Main contract configuration and exports
│   ├── datatypes.rs     # Core data structures and error definitions
│   ├── request.rs       # Loan request creation and management
│   ├── fund.rs          # Loan funding and multi-lender support
│   ├── repay.rs         # Repayment processing with remainder distribution
│   └── claim.rs         # Default handling and collateral claims
├── Cargo.toml           # Contract dependencies and configuration
└── README.md           # This documentation
```

## Core Functionality

### 🏦 Microlending Features
- **Loan Request Creation**: Borrowers can create loan requests with customizable terms
- **Multi-Lender Funding**: Multiple lenders can contribute to a single loan
- **Automated Repayment**: Supports both single payments and installment schedules
- **Collateral Management**: Secure collateral verification and liquidation
- **Default Handling**: Automated default detection and collateral distribution
- **Performance Tracking**: Comprehensive borrower and system metrics

### 🔐 Security Features
- **Authentication**: All operations require proper authorization
- **Input Validation**: Comprehensive validation for amounts, durations, and rates
- **Status Enforcement**: Prevents invalid state transitions
- **Balance Verification**: Ensures sufficient token balances
- **Rounding Protection**: Fair distribution without fund loss due to integer division

## API Documentation

### Contract Initialization
```rust
fn initialize(env: Env, token_address: Address)
```
Initializes the contract with the token address for funding and repayments.

### Loan Request Management
```rust
fn create_loan_request(
    env: Env,
    borrower: Address,
    amount: i128,
    purpose: String,
    duration_days: u32,
    interest_rate: u32,  // basis points (e.g., 1000 = 10%)
    collateral: CollateralInfo,
) -> u32
```
Creates a new loan request. Returns the loan ID.

```rust
fn get_loan_request(env: Env, loan_id: u32) -> LoanRequest
```
Retrieves loan request details by ID.

```rust
fn cancel_loan_request(env: Env, borrower: Address, loan_id: u32)
```
Allows borrowers to cancel pending loan requests.

```rust
fn update_loan_request(
    env: Env,
    borrower: Address,
    loan_id: u32,
    amount: i128,
    purpose: String,
    duration_days: u32,
    interest_rate: u32,
    collateral: CollateralInfo,
)
```
Updates loan request details before funding.

### Funding Functions
```rust
fn fund_loan(env: Env, lender: Address, loan_id: u32, amount: i128)
```
Allows lenders to fund a loan (supports partial funding).

```rust
fn get_loan_fundings(env: Env, loan_id: u32) -> Vec<FundingContribution>
```
Returns all funding contributions for a loan.

```rust
fn calculate_lender_share(env: Env, lender: Address, loan_id: u32) -> i128
```
Calculates a lender's total contribution amount.

```rust
fn calculate_lender_share_percent(env: Env, lender: Address, loan_id: u32) -> u32
```
Calculates a lender's contribution percentage (basis points).

### Repayment Functions
```rust
fn repay_loan(env: Env, borrower: Address, loan_id: u32, amount: i128)
```
Processes loan repayment with fair distribution to lenders.

```rust
fn get_loan_repayments(env: Env, loan_id: u32) -> Vec<Repayment>
```
Returns all repayment records for a loan.

```rust
fn calculate_total_repayment_due(env: Env, loan_id: u32) -> i128
```
Calculates total amount due (principal + interest).

### Default and Claim Functions
```rust
fn claim_default(env: Env, lender: Address, loan_id: u32)
```
Handles loan defaults and distributes collateral to lenders.

```rust
fn check_default_status(env: Env, loan_id: u32) -> bool
```
Checks if a loan is in default status.

### Query Functions
```rust
fn get_loan_history(env: &Env, loan_id: u32) -> LoanHistory
```
Returns complete loan history including funding, repayments, and status.

```rust
fn get_borrower_loans(env: Env, borrower: Address) -> Vec<u32>
```
Returns all loan IDs for a borrower.

```rust
fn get_lender_loans(env: Env, lender: Address) -> Vec<u32>
```
Returns all loan IDs for a lender.

## Data Structures

### LoanRequest
```rust
struct LoanRequest {
    id: u32,
    borrower: Address,
    amount: i128,
    purpose: String,
    duration_days: u32,
    interest_rate: u32,  // basis points
    collateral: CollateralInfo,
    status: LoanStatus,
    funded_amount: i128,
    creation_timestamp: u64,
    funded_timestamp: Option<u64>,
    repayment_due_timestamp: Option<u64>,
    repayment_schedule: RepaymentSchedule,
}
```

### CollateralInfo
```rust
struct CollateralInfo {
    asset_type: String,  // "Savings", "Future harvest", "Equipment"
    estimated_value: i128,
    verification_data: BytesN<32>,  // Hash of verification documents
}
```

### LoanStatus
```rust
enum LoanStatus {
    Pending,   // Created but not fully funded
    Funded,    // Fully funded, not yet repaid
    Repaying,  // Active repayment phase
    Completed, // Fully repaid
    Defaulted, // In default
    Cancelled, // Cancelled by borrower
}
```

## Usage Examples

### Creating a Loan Request
```rust
// Borrower creates a loan request
let loan_id = contract.create_loan_request(
    borrower_address,
    10000,  // 10,000 tokens
    "Agricultural equipment purchase".into(),
    90,     // 90 days
    1500,   // 15% interest (1500 basis points)
    CollateralInfo {
        asset_type: "Future harvest".into(),
        estimated_value: 15000,
        verification_data: verification_hash,
    }
);
```

### Funding a Loan
```rust
// Lender funds a loan
contract.fund_loan(
    lender_address,
    loan_id,
    5000  // 5,000 tokens contribution
);
```

### Repaying a Loan
```rust
// Borrower repays the loan
contract.repay_loan(
    borrower_address,
    loan_id,
    11500  // Principal + interest
);
```

## Key Features

### 🔄 Repayment Scheduling
- **Automatic Scheduling**: Loans ≥30 days get monthly installment schedules
- **Grace Periods**: 3-day early, 7-day late payment windows
- **Flexible Payments**: Supports both single payments and installments

### 💰 Fair Distribution
- **Proportional Repayment**: Lenders receive repayments based on contribution percentage
- **Remainder Handling**: No fund loss due to rounding errors
- **Dust Distribution**: Final fractional amounts allocated fairly

### 📊 Comprehensive Tracking
- **Borrower Metrics**: Performance history and default rates
- **System Statistics**: Platform-wide analytics and health monitoring
- **Event Logging**: Detailed transparency through event emission

### 🛡️ Security & Validation
- **Multi-layer Validation**: Input, state, and balance verification
- **Authorization Checks**: Proper authentication for all operations
- **Collateral Security**: Hash-based verification with liquidation support

## Development

### Prerequisites
1. Install Soroban CLI: `cargo install soroban-cli`
2. Set up your identity: `soroban config identity generate <name>`
3. Fund your account for deployment fees

### Building the Contract
```bash
cd ContractsRevo/microlending-contract
make build
# or manually:
cargo build --target wasm32-unknown-unknown --release
soroban contract build
```

### Testing
```bash
make test
# or manually:
cargo test
```

### Deployment

#### Deploy to Testnet
```bash
# Set your secret key
export ADMIN_SECRET=<your-secret-key>

# Deploy to testnet
make deploy-testnet
```

#### Deploy to Mainnet
```bash
# Set your secret key
export ADMIN_SECRET=<your-secret-key>

# Deploy to mainnet (use with caution)
make deploy-mainnet
```

#### Manual Deployment
```bash
# Upload contract
soroban contract upload \
  --source-account <identity-or-secret-key> \
  --network testnet \
  --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm

# Deploy contract
soroban contract deploy \
  --source-account <identity-or-secret-key> \
  --network testnet \
  --wasm target/wasm32-unknown-unknown/release/micro_lending.wasm
```

### Contract Interaction
After deployment, you can interact with the contract:

```bash
# Initialize the contract with a token address
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <your-identity> \
  --network testnet \
  -- \
  initialize \
  --token_address <TOKEN_ADDRESS>

# Create a loan request
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <your-identity> \
  --network testnet \
  -- \
  create_loan_request \
  --borrower <BORROWER_ADDRESS> \
  --amount 10000 \
  --purpose "Agricultural equipment" \
  --duration_days 90 \
  --interest_rate 1500 \
  --collateral '{"asset_type": "Future harvest", "estimated_value": 15000, "verification_data": "0x..."}'
```

### Available Makefile Targets
- `make build` - Build the contract
- `make test` - Run tests
- `make clean` - Clean build artifacts
- `make deploy-testnet` - Deploy to testnet (requires ADMIN_SECRET)
- `make deploy-mainnet` - Deploy to mainnet (requires ADMIN_SECRET)
- `make docs` - Generate documentation
- `make dev-setup` - Setup development environment
- `make help` - Show available targets

## License

This project is licensed under the MIT License - see the LICENSE file for details.