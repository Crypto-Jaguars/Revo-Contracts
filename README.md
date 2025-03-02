<h1 align="center"> Revo Contracts</h1>

<h3 align="center"> 🛠️ Maintainer</h3>
<table align="center">
  <tr>
    <td align="center">
      <img src="https://avatars.githubusercontent.com/u/176054645?v=4" alt="maintainer 1" width="150" />
      <br /><br />
      <strong>Software Engineer | OSS contributor</strong>
      <br /><br />
      <a href="https://github.com/aguilar1x" target="_blank">Matias</a>
      <br />
      <a href="https://t.me/aguilar1x" target="_blank">Telegram</a>
    </td>    
  </tr>
</table>

## 📖 Table of Contents
1. 📜 [Prerequisites](#-prerequisites)
2. 🖥️ [Environment Setup](#environment-setup-️)
3. 💳 [Wallet Configuration](#wallet-configuration-)
4. 🚀 [Compilation and Deployment](#compilation-and-deployment-)
5. 🕵🏻 [Testing and Execution](#testing-and-execution-)
6. 🔩 [Practical Example](#practical-example-)
7. 🩺 [Troubleshooting](#troubleshooting-)


## 📝 Prerequisites
   Before getting started, make sure you have the following installed on your system:

### 1. Installing Rust 🦀:
- ### For Linux 🐧, macOS 🍎 Systems
  If you using macOS, Linux, or any other Unix-like system:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

- ### Windows 🪟:
  Download [Rust](https://www.rust-lang.org/tools/install) and run `rustup-init.exe`.

    - #### Install the wasm32 target:
      After installing Rust, add the `wasm32-unknown-unknown` target:
    ```bash
    rustup target add wasm32-unknown-unknown
    ```
### 2. Install Stellar CLI 📡:
1. There are a few ways to install the [latest version](https://github.com/stellar/stellar-cli/releases) of Stellar CLI.
2. Rust allows you to use `cargo` command in the terminal to install the Stellar CLI.

- #### Install with cargo 📦:
```sh
cargo install --locked stellar-cli --features opt
```

- #### Install with Homebrew (macOS, Linux):
```sh
brew install stellar-cli
```

## Environment Setup 🛠️

-  Clone the repository 🗂️:
   ```bash
   git clone https://github.com/<username>/Revo-Contracts.git
   cd ./Revo-Contracts
   ```

-  build the smart contract 👷‍♂️: 
   ```bash
   stellar contract build
   ```

-  Run the Tests 🕵️:
   ```bash
   cargo test
   ```

## Wallet Configuration 💳
1. Install the Stellar Wallet (e.g., [Freighter Wallet](https://www.freighter.app/)).
2. Create a wallet and save the secret keys 🔑 securely.
3. Connect wallet to the Stellar test network.
   
## Compilation and Deployment 🚀

### 1. Build contract 👷‍♂️:
To build the smart contract, run the following command:
```bash
stellar contract build
```
This command will compile the contract and generate a contract.wasm file in the target/deploy directory.

### 2. Deploy contract 🧨:
To deploy the smart contract to the Stellar testnet, run the following command:
- ### macOS/Linux 💿:
```bash
stellar contract deploy \
   --wasm-hash <wasm_hash> \
   --source <source_account> \
   --network <network>
```
This command will deploy the contract to the testnet and return the contract's address.
### Example💡:
- Assume the following values:
  - <wasm_hash>: ./target/wasm32-unknown-unknown/release/stellar_smart_contract.wasm \
  - <source_account>: GBZXN7PIRZGNWCXXFYU7KYWXX4BXZUYHZO5QUEMKRHLUVLYN53WVFG3E
  -  <network> : testnet

```bash   
stellar contract deploy \
   --wasm ./target/wasm32-unknown-unknown/release/stellar_smart_contract.wasm \
   --source GBZXN7PIRZGNWCXXFYU7KYWXX4BXZUYHZO5QUEMKRHLUVLYN53WVFG3E \
   --network testnet  
```
Where:
- `<wasm_hash>` is the hash of the `.wasm` file generated during the contract installation.
- `<source_account>` is the account from which the deployment will be made.
- `<network>` is the network you are working on (e.g., testnet).

## Testing and Execution 🔬
To run the tests, execute the following command:
   ```bash
   cargo test
   ```
Fix any errors and re-run the tests.

### Interact with contract 🤖:
- Simulate contract calls to ensure correctness:
```bash
stellar contract invoke \
   --contract-id <contract_id> \
   --source <source_account> \
   --network <network> \
   --function <function_name> \
   --args <function_arguments>
   ```
  Where:
- <contract_id> is the deployed contract ID.
- <function_name> is the function being tested.

## Practical Example 👩🏻‍💻

### Installation 📦: 
Install all [prerequisites](#-prerequisites),If not installed. 
### Create New Project 🎨:
Create a new project using the init command to create a soroban-hello-world project.
```bash
stellar contract init soroban-hello-world
```
The init command will create a Rust workspace project structure 🩻:
```bash
.
├── Cargo.lock
├── Cargo.toml
├── README.md
└── contracts
    └── hello_world
        ├── Cargo.toml
        └── src
            ├── lib.rs
            └── test.rs
 ```

- add simple contract in `contracts/hello_world/src/lib.rs` :

```bash
#![no_std]
use soroban_sdk::{contract, contractimpl, symbol_short, vec, Env, Symbol, Vec};

#[contract]
pub struct HelloContract;

#[contractimpl]
impl HelloContract {
    pub fn hello(env: Env, to: Symbol) -> Vec<Symbol> {
        vec![&env, symbol_short!("Hello"), to]
    }
}

mod test;
```

- Add test contract file `contracts/hello_world/src/test.rs`:
```bash
#![cfg(test)]

use super::*;
use soroban_sdk::{symbol_short, vec, Env};

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, HelloContract);
    let client = HelloContractClient::new(&env, &contract_id);

    let words = client.hello(&symbol_short!("Dev"));
    assert_eq!(
        words,
        vec![&env, symbol_short!("Hello"), symbol_short!("Dev"),]
    );
}
```
### Run the Tests 🕵️:
Run cargo test and watch the unit test run. You should see the following output:

```bash
cargo test
```

```bash
running 1 test
test test::test ... ok
```
### Build the contract 🏗️:
To build a smart contract to deploy or run, use the stellar contract build command.
```bash
stellar contract build
```
### Deploy to Testnet 🚀:
To deploy your HelloWorld contract, run the following command:
```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/hello_world.wasm \
  --source alice \
  --network testnet
  ```
This returns the contract id `CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN`, so replace it with your actual contract id.

### Interact 🔁:
run the following command to invoke the hello function.

```bash
stellar contract invoke \
  --id CACDYF3CYMJEJTIVFESQYZTN67GO2R5D5IUABTCUG3HXQSRXCSOROBAN \
  --source alice \
  --network testnet \
  -- \
  hello \
  --to RPC
  ```
  output should appear:
  ```bash
  ["Hello", "RPC"]
  ```
### Summary 🎯
In this example , we learned how to:
  - deploy a contract to Testnet
  - interact with a deployed contract

## Troubleshooting 🩺:
### Common Issues and Fixes🤔:
1. 🦀Rust Installation Issues:
   - Ensure `cargo` is in your system PATH.

2. 📡Stellar CLI Errors:
   - Verify the version compatibility of the Stellar CLI.
   - Use the --help flag to get details of commands:
   ```bash
    stellar --help
   ```
3. 💸Wallet Connectivity:
   - Double-check network configuration (testnet/mainnet).
---

##### **By following this guide, you should be able to set up your environment and deploy a basic Smart Contract using Stellar. Always refer to the official Stellar documentation for the most up-to-date information and best practices**
