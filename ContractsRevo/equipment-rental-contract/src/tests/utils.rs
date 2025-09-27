#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, String,
};

use crate::{EquipmentRentalContract, EquipmentRentalContractClient};

/// Test setup helper function
/// Returns: (env, contract_id, client, owner, renter1, renter2)
pub fn setup_test<'a>() -> (
    Env,
    Address,
    EquipmentRentalContractClient<'a>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    // Mock all auths: contract methods that require_auth will pass for tests
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let renter1 = Address::generate(&env);
    let renter2 = Address::generate(&env);

    let contract_id = env.register(EquipmentRentalContract, ());
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    (env, contract_id, client, owner, renter1, renter2)
}

/// Helper function to create a BytesN<32> equipment ID from a short string
pub fn create_equipment_id(env: &Env, id: &str) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    let id_bytes = id.as_bytes();
    assert!(
        id_bytes.len() <= 32,
        "equipment_id exceeds 32 bytes and would be truncated"
    );
    bytes[..id_bytes.len()].copy_from_slice(id_bytes);
    BytesN::from_array(env, &bytes)
}

/// Helper function to register basic equipment for testing
pub fn register_basic_equipment(
    client: &EquipmentRentalContractClient,
    env: &Env,
    id_str: &str,
    price_per_day: i128,
) -> BytesN<32> {
    let equipment_id = create_equipment_id(env, id_str);
    let equipment_type = String::from_str(env, "Agricultural Tractor");
    let location = String::from_str(env, "Farm Location A");

    client.register_equipment(
        &equipment_id,
        &equipment_type,
        &price_per_day,
        &location,
    );

    equipment_id
}

/// Advance ledger time by seconds (helper for time-based tests)
pub fn advance_time(env: &Env, seconds: u64) {
    env.ledger().with_mut(|li| {
        li.timestamp += seconds;
    });
}

/// Create a standard rental with future dates
pub fn create_standard_rental(
    client: &EquipmentRentalContractClient,
    env: &Env,
    equipment_id: &BytesN<32>,
    renter: &Address,
    days_duration: u64,
) -> (u64, u64, i128) {
    let start_date = env.ledger().timestamp() + 86400; // Tomorrow
    let end_date = start_date + (days_duration * 86400);
    let total_price = days_duration as i128 * 1000; // Assuming 1000 per day

    client.create_rental(equipment_id, renter, &start_date, &end_date, &total_price);

    (start_date, end_date, total_price)
}