#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN};

mod equipment;
mod rental;
mod pricing;
mod maintenance;


/// Main contract for equipment rental management
#[contract]
pub struct EquipmentRentalContract;

#[contractimpl]
impl EquipmentRentalContract {
    // Equipment management
    /// Register a new equipment item to the platform
    pub fn register_equipment(
        env: Env,
        id: BytesN<32>,
        equipment_type: String,
        owner: Address,
        rental_price_per_day: i128,
        location: String,
    ) {
        crate::equipment::register_equipment(&env, id, equipment_type, owner, rental_price_per_day, location);
    }
    /// Change the availability status of equipment
    pub fn update_availability(env: Env, id: BytesN<32>, available: bool) {
        crate::equipment::update_availability(&env, id, available);
    }
    /// Mark equipment status (Good, NeedsService, UnderMaintenance)
    pub fn update_maintenance_status(env: Env, id: BytesN<32>, status: crate::equipment::MaintenanceStatus) {
        crate::equipment::update_maintenance_status(&env, id, status);
    }
    /// Retrieve equipment details by ID
    pub fn get_equipment(env: Env, id: BytesN<32>) -> Option<crate::equipment::Equipment> {
        crate::equipment::get_equipment(&env, id)
    }

    // Rental lifecycle
    /// Initiate a rental request for a given date range
    pub fn create_rental(
        env: Env,
        equipment_id: BytesN<32>,
        renter: Address,
        start_date: u64,
        end_date: u64,
        total_price: i128,
    ) {
        crate::rental::create_rental(&env, equipment_id, renter, start_date, end_date, total_price);
    }
    /// Confirm and activate a rental
    pub fn confirm_rental(env: Env, equipment_id: BytesN<32>) {
        crate::rental::confirm_rental(&env, equipment_id);
    }
    /// Finalize rental and release equipment
    pub fn complete_rental(env: Env, equipment_id: BytesN<32>) {
        crate::rental::complete_rental(&env, equipment_id);
    }
    /// Cancel a rental agreement before start date
    pub fn cancel_rental(env: Env, equipment_id: BytesN<32>) {
        crate::rental::cancel_rental(&env, equipment_id);
    }
    /// Retrieve rental details by equipment ID
    pub fn get_rental(env: Env, equipment_id: BytesN<32>) -> Option<crate::rental::Rental> {
        crate::rental::get_rental(&env, equipment_id)
    }
    /// Retrieve all rental agreements for a given equipment
    pub fn get_rental_history_by_equipment(env: Env, equipment_id: BytesN<32>) -> Vec<crate::rental::Rental> {
        crate::rental::get_rental_history_by_equipment(&env, equipment_id)
    }
    /// Retrieve all rental agreements for a given renter address
    pub fn get_rental_history_by_user(env: Env, renter: Address) -> Vec<crate::rental::Rental> {
        crate::rental::get_rental_history_by_user(&env, renter)
    }

    // Pricing
    /// Compute total rental price for a date range
    pub fn compute_total_price(
        env: Env,
        equipment_id: BytesN<32>,
        start_date: u64,
        end_date: u64,
    ) -> i128 {
        let eq = crate::equipment::get_equipment(&env, equipment_id).expect("Equipment not found");
        crate::pricing::compute_total_price(&eq, start_date, end_date)
    }
    /// Validate proposed rental price for a date range
    pub fn validate_price(
        env: Env,
        equipment_id: BytesN<32>,
        start_date: u64,
        end_date: u64,
        proposed_price: i128,
    ) {
        let eq = crate::equipment::get_equipment(&env, equipment_id).expect("Equipment not found");
        crate::pricing::validate_price(&eq, start_date, end_date, proposed_price);
    }

    // Maintenance
    /// Log a maintenance event for equipment
    pub fn log_maintenance(
        env: Env,
        equipment_id: BytesN<32>,
        status: crate::equipment::MaintenanceStatus,
        timestamp: u64,
        notes: Option<String>,
    ) {
        crate::maintenance::log_maintenance(&env, equipment_id, status, timestamp, notes);
    }
    /// Retrieve maintenance history for all equipment
    pub fn get_maintenance_history(env: Env) -> Vec<crate::maintenance::MaintenanceRecord> {
        crate::maintenance::get_maintenance_history(&env)
    }
}
