# Equipment Rental Smart Contract

A Soroban smart contract for managing equipment rental on the Stellar network. This contract allows users to list, rent, and manage equipment securely and transparently.

## Features
- Register equipment with metadata, geolocation, and owner
- Update equipment availability and maintenance status
- Create, confirm, complete, and cancel rental agreements
- Compute and validate rental pricing
- Track maintenance events and restrict rentals during maintenance

## Data Structures

```rustrust
struct Equipment {
    id: BytesN<32>,
    equipment_type: String,
    owner: Address,
    rental_price_per_day: i128,
    available: bool,
    location: String,
    maintenance_status: MaintenanceStatus,
}

struct Rental {
    equipment_id: BytesN<32>,
    renter: Address,
    start_date: u64,
    end_date: u64,
    total_price: i128,
    status: RentalStatus,
}

enum MaintenanceStatus {
    Good,
    NeedsService,
    UnderMaintenance,
}

enum RentalStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}
```rust

## Core Functions
- `register_equipment()` – Register a new equipment item
- `update_availability()` – Change equipment availability
- `update_maintenance_status()` – Mark equipment status (Good, NeedsService, UnderMaintenance)
- `create_rental()` – Initiate a rental request
- `confirm_rental()` – Confirm and activate a rental
- `complete_rental()` – Finalize rental and release equipment
- `cancel_rental()` – Cancel a rental before start date
- `compute_total_price()` – Compute total rental price for a date range
- `validate_price()` – Validate proposed price for a rental
- `log_maintenance()` – Log a maintenance event
- `get_maintenance_history()` – Retrieve maintenance history

## Usage
1. Build the contract with Soroban CLI:
   ```rustsh
   soroban contract build
   ```rust
2. Deploy to the Stellar network and interact via the Soroban CLI or SDK.

## Notes
- Rentals are blocked if equipment is under maintenance or needs service.
- Only the owner can update equipment or maintenance status.
- Designed for extensibility (e.g., future review or insurance systems).

---

MIT License
