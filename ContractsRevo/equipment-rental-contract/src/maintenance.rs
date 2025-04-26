use soroban_sdk::{Env, BytesN, Symbol, Vec, contracttype};
use crate::equipment::MaintenanceStatus;

/// Record of a maintenance event for equipment
#[derive(Clone, Debug, Eq, PartialEq, contracttype)]
pub struct MaintenanceRecord {
    /// Equipment item
    pub equipment_id: BytesN<32>,
    /// Maintenance status at this event
    pub status: MaintenanceStatus,
    /// UNIX timestamp of the event
    pub timestamp: u64,
    /// Optional notes or description
    pub notes: Option<String>,
}

const MAINTENANCE_HISTORY_STORAGE: Symbol = symbol_short!("maint_hist");

/// Log a maintenance event for equipment
pub fn log_maintenance(env: &Env, equipment_id: BytesN<32>, status: MaintenanceStatus, timestamp: u64, notes: Option<String>) {
    let mut history: Vec<MaintenanceRecord> = env
        .storage()
        .persistent()
        .get(&MAINTENANCE_HISTORY_STORAGE)
        .unwrap_or(Vec::new(env));
    let record = MaintenanceRecord {
        equipment_id,
        status,
        timestamp,
        notes,
    };
    history.push_back(record);
    env.storage().persistent().set(&MAINTENANCE_HISTORY_STORAGE, &history);
}

/// Retrieve maintenance history for all equipment
pub fn get_maintenance_history(env: &Env) -> Vec<MaintenanceRecord> {
    env.storage()
        .persistent()
        .get(&MAINTENANCE_HISTORY_STORAGE)
        .unwrap_or(Vec::new(env))
}
