# Agricultural Training Smart Contract

This Soroban smart contract provides a decentralized platform for managing agricultural training programs on the Stellar network. It enables instructors to create courses, track farmer participation, and automatically issue tokenized certificates and loyalty rewards upon completion.

## 🏗 Contract Architecture

The contract is designed with a clear separation of concerns to ensure modularity and secure integration with external systems.

* **`lib.rs`**: The main entry point, defining the contract interface and importing the interfaces of external dependency contracts.
* **`training.rs`**: Contains the core logic for creating and managing the details of training programs.
* **`participation.rs`**: Handles all logic related to farmer enrollment and progress tracking.
* **`certification.rs`**: Implements the logic for issuing certificates and rewards through secure cross-contract calls.
* **`storage.rs`**: Defines all on-chain data structures (`TrainingProgram`, `ParticipantStatus`) and storage keys.
* **`utils.rs`**: Provides shared utility functions, such as deterministic ID generation.
* **`error.rs`**: Defines custom contract errors for predictable and clear error handling.

## 🗂 Features

* **Program Management**: Allows instructors to register training programs with details like title, duration, and a hash of off-chain training materials (e.g., stored on IPFS).
* **Secure Participation Tracking**: Securely tracks farmer enrollment and completion status, ensuring that only the designated instructor can update a participant's progress.
* **Automated Certification**: Upon 100% completion, the contract automatically calls an external `certificate-management-contract` to issue a unique, tokenized certificate to the farmer.
* **Integrated Reward System**: Simultaneously calls an external `loyalty-token-contract` to reward farmers with loyalty points for completing their training.
* **Scalable Design**: Built to support multiple concurrent training programs and a large number of participants.

## 🔑 Key Functions

### State-Changing Functions

* `initialize(admin: Address, certificate_contract_id: Address, loyalty_token_id: Address, loyalty_program_id: BytesN<32>)`: Initializes the contract with an admin and the on-chain addresses of the external certificate and loyalty contracts.
* `create_training_program(...)`: Creates a new training program.
* `enroll_farmer(farmer: Address, program_id: BytesN<32>)`: Enrolls a farmer in a program.
* `update_progress(instructor: Address, ...)`: Updates a farmer’s training progress. Can only be called by the program's instructor.
* `issue_certificate(instructor: Address, ...)`: Issues a certificate and rewards upon completion. Can only be called by the program's instructor.

### Read-Only Functions

* `get_program(program_id: BytesN<32>)`: Retrieves the details of a specific training program.
* `get_participant_status(program_id: BytesN<32>, farmer_id: Address)`: Retrieves the participation status of a farmer in a program.

## 📦 Deployment and Usage

### Prerequisites

* [Rust](https://www.rust-lang.org/tools/install)
* [Soroban CLI](https://soroban.stellar.org/docs/getting-started/setup)

### Build and Test
```
# Build the contract WASM
make build

# Run the test suite
make test
```

### Deployment and Interaction

1. **Deploy Dependencies**: First, deploy instances of the `certificate-management-contract` and `loyalty-token-contract` to the network.
2. **Deploy this Contract**:
   ```
   make deploy ADMIN_ACCOUNT=<your-admin-name>
   ```
3. **Initialize**: After deployment, initialize the contract, providing it with the addresses of the dependency contracts.
   ```
   # Set the variables in the Makefile first
   make init ADMIN_ACCOUNT=<your-admin-name> ...
   ```
4. **Interact**: Use the other `make` commands (`create-program`, `enroll-farmer`, etc.) to interact with the deployed contract.

NB: Contract deployed to testnet on `CA33BT2EGOVSOHFGP47HLXFDST4AXWDIG7GNHY6FVVQFOKCVCFSRYT3R`

---
This contract is designed to be a core component of a larger agricultural ecosystem, enabling verifiable, on-chain educational credentials for farmers.
