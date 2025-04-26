#![cfg(test)]

use soroban_sdk::{Env, Address, BytesN, String};
use equipment_rental_contract::{EquipmentRentalContractClient, MaintenanceStatus};

#[test]
fn test_register_equipment() {
    let env = Env::default();
    let contract_id = env.register_contract(None, equipment_rental_contract::EquipmentRentalContract {});
    let client = EquipmentRentalContractClient::new(&env, &contract_id);

    let equipment_id = BytesN::from_array(&env, &[1u8; 32]);
    let owner = Address::random(&env);
    let equipment_type = String::from_slice(&env, "Tractor");
    let location = String::from_slice(&env, "Farm 42");
    let price = 1000_i128;

    client.register_equipment(&equipment_id, &equipment_type, &owner, &price, &location);
    let eq = client.get_equipment(&equipment_id).unwrap();
    assert_eq!(eq.equipment_type, equipment_type);
    assert_eq!(eq.owner, owner);
    assert_eq!(eq.rental_price_per_day, price);
    assert_eq!(eq.location, location);
    assert_eq!(eq.available, true);
    assert_eq!(eq.maintenance_status, MaintenanceStatus::Good);
}
