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

/// Retrieve maintenance history, optionally filtered by equipment ID
pub fn get_maintenance_history(env: &Env, equipment_id: Option<BytesN<32>>) -> Vec<MaintenanceRecord> {
    let all_records = env.storage()
        .persistent()
        .get(&MAINTENANCE_HISTORY_STORAGE)
        .unwrap_or(Vec::new(env));
    if let Some(id) = equipment_id {
        let filtered: Vec<MaintenanceRecord> = all_records
            .iter()
            .filter(|r| r.equipment_id == id)
            .cloned()
            .collect(env);
        return filtered;
    }
    all_records
}

/// Retrieve maintenance history with optional filtering and pagination
pub fn get_maintenance_history_paginated(
    env: &Env,
    equipment_id: Option<BytesN<32>>,
    offset: u32,
    limit: u32
) -> Vec<MaintenanceRecord> {
    let all_records = env.storage()
        .persistent()
        .get(&MAINTENANCE_HISTORY_STORAGE)
        .unwrap_or(Vec::new(env));
    let filtered = if let Some(id) = equipment_id {
        all_records
            .iter()
            .filter(|r| r.equipment_id == id)
            .collect::<Vec<_>>(env)
    } else {
        all_records.iter().collect::<Vec<_>>(env)
    };
    let start = offset as usize;
    let end = core::cmp::min(start + limit as usize, filtered.len());
    if start >= filtered.len() {
        return Vec::new(env);
    }
    filtered[start..end].iter().cloned().collect(env)
}
