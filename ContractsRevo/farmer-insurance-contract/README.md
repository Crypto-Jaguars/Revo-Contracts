# Farmer Insurance Contract

This Soroban smart contract implements a decentralized parametric insurance system for farmers on the Stellar network. It enables the creation of insurance policies, premium activation, submission of claims, and manual payouts upon oracle/admin verification.

## ✅ Exported Functions

* **create\_pol(farmer: Address, coverage: Symbol, premium: i128) -> BytesN<32>:**
  Creates a new insurance policy for a farmer. Requires authorization from the `farmer`.

* **pay\_prem(policy\_id: BytesN<32>):**
  Activates an existing policy by marking the premium as paid. Requires authorization from the policy holder.

* **sub\_claim(policy\_id: BytesN<32>, event\_hash: BytesN<32>, payout: i128) -> BytesN<32>:**
  Submits a claim referencing a policy and an event hash. Requires authorization from the policy holder.

* **pay\_out(claim\_id: BytesN<32>, admin: Address):**
  Processes a payout for a given claim. Requires authorization from the specified `admin` address.

* **get\_policy(policy\_id: BytesN<32>) -> InsurancePolicy:**
  Returns the policy object associated with the given ID.

## 🧱 On-Chain Storage

The contract persistently stores:

* `InsurancePolicy` records by policy ID.
* `Claim` records by claim ID.
* Internal counters for:

  * Total number of policies (`PolicyCount`)
  * Total number of claims (`ClaimCount`)

These are managed via the `utils::DataKey` enum.

## 🗞 Data Structures

### InsurancePolicy

```rust
struct InsurancePolicy {
    policy_id: BytesN<32>,
    farmer: Address,
    coverage: Symbol,
    premium: i128,
    active: bool,
}
```

### Claim

```rust
struct Claim {
    claim_id: BytesN<32>,
    policy_id: BytesN<32>,
    event_hash: BytesN<32>,
    payout_amount: i128,
}
```

## 🔐 Authorization

* **Farmers** must authorize:

  * Creation of policies
  * Payment of premiums
  * Submission of claims
* **Admins** (oracles) must authorize:

  * Payouts for claims

The contract uses `require_auth()` to enforce these rules.

## 💡 Oracle & Event Hash

This version assumes the `event_hash` is generated off-chain from external data (e.g., climate APIs) and passed by the farmer when submitting a claim.
An authorized `admin` is expected to validate and approve payouts by calling `pay_out`.

## 🧪 Testing

The contract includes full and modular tests that cover:

* Policy creation
* Activation via premium payment
* Claim submission validation
* Claim payout flow
* Failure paths (e.g., duplicate payments, inactive policies)

Run tests with:

```bash
cargo test
```

## ⚙️ Build

Use:

```bash
cargo build
```

or

```bash
soroban contract build
```

to compile the contract to `.wasm`.

## 📌 Notes

* No real token transfers are currently implemented. If desired, integrate the Soroban Token Interface (CAP-46) in `pay_prem` and `pay_out`.
* Claim verification logic is delegated to external oracles/admins for now.
* Policy expiration, maximum payouts, or time-based logic can be added in future iterations.

## 🧩 Versioning

Developed with **Soroban SDK 22.0.7**.
Ensure you’re using this version or adjust the code for compatibility.

---
