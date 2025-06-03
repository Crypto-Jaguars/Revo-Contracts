use soroban_sdk::{Address, BytesN, Env, Symbol, Map, contracttype, symbol_short, String, Vec, Error};

/// Status of equipment maintenance
#[derive(Clone, Debug, Eq, PartialEq, Copy)]
#[contracttype]
pub enum MaintenanceStatus {
    Good,
    NeedsService,
    UnderMaintenance,
}

/// Equipment item listed for rental
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Equipment {
    /// Unique identifier
    pub id: BytesN<32>,
    /// Equipment type or description
    pub equipment_type: String,
    /// Address of the owner
    pub owner: Address,
    /// Daily rental price (in stroops or smallest currency unit)
    pub rental_price_per_day: i128,
    /// Whether equipment is available for rental
    pub available: bool,
    /// Geolocation or address string
    pub location: String,
    /// Current maintenance status
    pub maintenance_status: MaintenanceStatus,
}

const EQUIPMENT_STORAGE: Symbol = symbol_short!("equipment");

/// Register a new equipment item
pub fn register_equipment(
    env: &Env,
    id: BytesN<32>,
    equipment_type: String,
    rental_price_per_day: i128,
    location: String,
) {
    let mut equipment_map: Map<BytesN<32>, Equipment> = env
        .storage()
        .persistent()
        .get(&EQUIPMENT_STORAGE)
        .unwrap_or(Map::new(env));
    if equipment_map.contains_key(id.clone()) {
        panic!("Equipment already registered");
    }
    let owner = env.current_contract_address();
    let equipment = Equipment {
        id: id.clone(),
        equipment_type,
        owner,
        rental_price_per_day,
        available: true,
        location,
        maintenance_status: MaintenanceStatus::Good,
    };
    equipment_map.set(id.clone(), equipment);
    env.storage().persistent().set(&EQUIPMENT_STORAGE, &equipment_map);
}

/// Change the availability status of equipment
pub fn update_availability(env: &Env, id: BytesN<32>, caller: Address, available: bool) -> Result<(), Error> {
    let mut equipment_map: Map<BytesN<32>, Equipment> = env
        .storage()
        .persistent()
        .get(&EQUIPMENT_STORAGE)
        .unwrap_or(Map::new(env));
    
    if !equipment_map.contains_key(id.clone()) {
        return Err(Error::from_contract_error(1006));
    }
    
    let mut equipment = equipment_map.get_unchecked(id.clone());
    if equipment.owner != caller {
        return Err(Error::from_contract_error(1007));
    }
    
    equipment.available = available;
    equipment_map.set(id.clone(), equipment);
    env.storage().persistent().set(&EQUIPMENT_STORAGE, &equipment_map);
    Ok(())
}

/// Update maintenance status for equipment
pub fn update_maintenance_status(env: &Env, id: BytesN<32>, caller: Address, status: MaintenanceStatus) -> Result<(), Error> {
    let mut equipment_map: Map<BytesN<32>, Equipment> = env
        .storage()
        .persistent()
        .get(&EQUIPMENT_STORAGE)
        .unwrap_or(Map::new(env));
    
    if !equipment_map.contains_key(id.clone()) {
        return Err(Error::from_contract_error(1006));
    }
    
    let mut equipment = equipment_map.get_unchecked(id.clone());
    if equipment.owner != caller {
        return Err(Error::from_contract_error(1007));
    }
    
    equipment.maintenance_status = status;
    equipment_map.set(id.clone(), equipment);
    env.storage().persistent().set(&EQUIPMENT_STORAGE, &equipment_map);
    Ok(())
}

/// List all equipment IDs, optionally filtering only available equipment
#[allow(dead_code)]
pub fn list_equipment(env: &Env, only_available: bool) -> Vec<BytesN<32>> {
    let equipment_map: Map<BytesN<32>, Equipment> = env
        .storage()
        .persistent()
        .get(&EQUIPMENT_STORAGE)
        .unwrap_or(Map::new(env));
    let mut result = Vec::new(env);
    for (id, equipment) in equipment_map.iter() {
        if !only_available || equipment.available {
            result.push_back(id.clone());
        }
    }
    result
}

/// Retrieve equipment details by ID
pub fn get_equipment(env: &Env, id: BytesN<32>) -> Option<Equipment> {
    let equipment_map: Map<BytesN<32>, Equipment> = env
        .storage()
        .persistent()
        .get(&EQUIPMENT_STORAGE)
        .unwrap_or(Map::new(env));
    if equipment_map.contains_key(id.clone()) {
        Some(equipment_map.get_unchecked(id.clone()))
    } else {
        None
    }
}
