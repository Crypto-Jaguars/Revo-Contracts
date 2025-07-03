use crate::equipment::MaintenanceStatus;
use soroban_sdk::{contracttype, symbol_short, BytesN, Env, String, Symbol, Vec};

/// Record of a maintenance event for equipment
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
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

const MAINTENANCE_HISTORY_STORAGE: Symbol = symbol_short!("maint");

/// Log a maintenance event for equipment
pub fn log_maintenance(
    env: &Env,
    equipment_id: BytesN<32>,
    status: MaintenanceStatus,
    timestamp: u64,
    notes: Option<String>,
) {
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
    env.storage()
        .persistent()
        .set(&MAINTENANCE_HISTORY_STORAGE, &history);
}

/// Retrieve maintenance history, optionally filtered by equipment ID
pub fn get_maintenance_history(
    env: &Env,
    equipment_id: Option<BytesN<32>>,
) -> Vec<MaintenanceRecord> {
    let all_records: Vec<MaintenanceRecord> = env
        .storage()
        .persistent()
        .get(&MAINTENANCE_HISTORY_STORAGE)
        .unwrap_or(Vec::new(env));
    if let Some(id) = equipment_id {
        let mut filtered = Vec::new(env);
        for r in all_records.iter() {
            if r.equipment_id == id {
                filtered.push_back(r.clone());
            }
        }
        return filtered;
    }
    all_records
}

/// Retrieve maintenance history with optional filtering and pagination
#[allow(dead_code)]
pub fn get_maintenance_history_paginated(
    env: &Env,
    equipment_id: Option<BytesN<32>>,
    offset: u32,
    limit: u32,
) -> Vec<MaintenanceRecord> {
    let all_records: Vec<MaintenanceRecord> = env
        .storage()
        .persistent()
        .get(&MAINTENANCE_HISTORY_STORAGE)
        .unwrap_or(Vec::new(env));
    let filtered = if let Some(id) = equipment_id {
        let mut filtered = Vec::new(env);
        for r in all_records.iter() {
            if r.equipment_id == id {
                filtered.push_back(r.clone());
            }
        }
        filtered
    } else {
        all_records
    };
    let start = offset as usize;
    let end = core::cmp::min(start + limit as usize, filtered.len() as usize);
    if start >= filtered.len() as usize {
        return Vec::new(env);
    }
    let mut result = Vec::new(env);
    let start_usize = start;
    let end_usize = end;
    for i in start_usize..end_usize {
        if let Some(r) = filtered.get(i as u32) {
            result.push_back(r.clone());
        }
    }
    result
}
