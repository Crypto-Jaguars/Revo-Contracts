use crate::equipment::{get_equipment, MaintenanceStatus};
use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, Map, Symbol, Vec};

/// Status of a rental agreement
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[contracttype]
pub enum RentalStatus {
    /// Rental has been requested but not yet confirmed
    Pending,
    /// Rental is active and ongoing
    Active,
    /// Rental is completed and equipment released
    Completed,
    /// Rental was cancelled before starting
    Cancelled,
}

/// Rental agreement for equipment
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Rental {
    /// Equipment being rented
    pub equipment_id: BytesN<32>,
    /// Address of the renter
    pub renter: Address,
    /// Rental start date (UNIX timestamp)
    pub start_date: u64,
    /// Rental end date (UNIX timestamp)
    pub end_date: u64,
    /// Total rental price for the period
    pub total_price: i128,
    /// Current status of the rental
    pub status: RentalStatus,
}

const RENTAL_STORAGE: Symbol = symbol_short!("rental");
const RENTAL_HISTORY_BY_EQUIPMENT: Symbol = symbol_short!("rent_eq");
const RENTAL_HISTORY_BY_USER: Symbol = symbol_short!("rent_usr");

/// Initiate a rental request for a given equipment and date range
pub fn create_rental(
    env: &Env,
    equipment_id: BytesN<32>,
    renter: Address,
    start_date: u64,
    end_date: u64,
    total_price: i128,
) {
    let equipment = get_equipment(env, equipment_id.clone()).expect("Equipment not found");
    if !equipment.available {
        panic!("Equipment not available");
    }
    if equipment.maintenance_status != MaintenanceStatus::Good {
        panic!("Equipment under maintenance or needs service");
    }
    let mut rental_map: Map<BytesN<32>, Rental> = env
        .storage()
        .persistent()
        .get(&RENTAL_STORAGE)
        .unwrap_or(Map::new(env));
    if rental_map.contains_key(equipment_id.clone()) {
        panic!("Rental already exists for this equipment");
    }
    let rental = Rental {
        equipment_id: equipment_id.clone(),
        renter: renter.clone(),
        start_date,
        end_date,
        total_price,
        status: RentalStatus::Pending,
    };
    rental_map.set(equipment_id.clone(), rental.clone());
    env.storage().persistent().set(&RENTAL_STORAGE, &rental_map);
    // Track history per equipment
    let mut eq_history = env
        .storage()
        .persistent()
        .get(&(RENTAL_HISTORY_BY_EQUIPMENT, equipment_id.clone()))
        .unwrap_or(Vec::new(env));
    eq_history.push_back(rental.clone());
    env.storage().persistent().set(
        &(RENTAL_HISTORY_BY_EQUIPMENT, equipment_id.clone()),
        &eq_history,
    );
    // Track history per user
    let mut user_history = env
        .storage()
        .persistent()
        .get(&(RENTAL_HISTORY_BY_USER, renter.clone()))
        .unwrap_or(Vec::new(env));
    user_history.push_back(rental);
    env.storage()
        .persistent()
        .set(&(RENTAL_HISTORY_BY_USER, renter), &user_history);
}

/// Confirm and activate a pending rental
pub fn confirm_rental(env: &Env, equipment_id: BytesN<32>) {
    let mut rental_map: Map<BytesN<32>, Rental> = env
        .storage()
        .persistent()
        .get(&RENTAL_STORAGE)
        .unwrap_or(Map::new(env));
    let mut rental = rental_map
        .get(equipment_id.clone())
        .expect("Rental not found");
    if rental.status != RentalStatus::Pending {
        panic!("Rental not pending");
    }
    rental.status = RentalStatus::Active;
    rental_map.set(equipment_id.clone(), rental);
    env.storage().persistent().set(&RENTAL_STORAGE, &rental_map);
}

/// Finalize rental and release equipment
pub fn complete_rental(env: &Env, equipment_id: BytesN<32>) {
    let mut rental_map: Map<BytesN<32>, Rental> = env
        .storage()
        .persistent()
        .get(&RENTAL_STORAGE)
        .unwrap_or(Map::new(env));
    let mut rental = rental_map
        .get(equipment_id.clone())
        .expect("Rental not found");
    if rental.status != RentalStatus::Active {
        panic!("Rental not active");
    }
    rental.status = RentalStatus::Completed;
    rental_map.set(equipment_id.clone(), rental.clone());
    env.storage().persistent().set(&RENTAL_STORAGE, &rental_map);
    // Mark equipment as available again
    let equipment =
        crate::equipment::get_equipment(env, equipment_id.clone()).expect("Equipment not found");
    let _ = crate::equipment::update_availability(env, equipment_id, equipment.owner, true);
}

/// Cancel a rental agreement before it starts
pub fn cancel_rental(env: &Env, equipment_id: BytesN<32>) {
    let mut rental_map: Map<BytesN<32>, Rental> = env
        .storage()
        .persistent()
        .get(&RENTAL_STORAGE)
        .unwrap_or(Map::new(env));
    let mut rental = rental_map
        .get(equipment_id.clone())
        .expect("Rental not found");
    if rental.status != RentalStatus::Pending {
        panic!("Only pending rentals can be cancelled");
    }
    rental.status = RentalStatus::Cancelled;
    rental_map.set(equipment_id.clone(), rental);
    env.storage().persistent().set(&RENTAL_STORAGE, &rental_map);
}

/// Retrieve rental details by equipment ID
pub fn get_rental(env: &Env, equipment_id: BytesN<32>) -> Option<Rental> {
    let rental_map: Map<BytesN<32>, Rental> = env
        .storage()
        .persistent()
        .get(&RENTAL_STORAGE)
        .unwrap_or(Map::new(env));
    if rental_map.contains_key(equipment_id.clone()) {
        Some(rental_map.get_unchecked(equipment_id.clone()))
    } else {
        None
    }
}

/// Retrieve all rental agreements for a given equipment
pub fn get_rental_history_by_equipment(env: &Env, equipment_id: BytesN<32>) -> Vec<Rental> {
    env.storage()
        .persistent()
        .get(&(RENTAL_HISTORY_BY_EQUIPMENT, equipment_id.clone()))
        .unwrap_or(Vec::new(env))
}

/// Retrieve all rental agreements for a given renter address
pub fn get_rental_history_by_user(env: &Env, renter: Address) -> Vec<Rental> {
    env.storage()
        .persistent()
        .get(&(RENTAL_HISTORY_BY_USER, renter))
        .unwrap_or(Vec::new(env))
}
