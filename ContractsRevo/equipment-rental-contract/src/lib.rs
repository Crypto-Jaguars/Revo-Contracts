#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Error, String, Vec};

mod equipment;
mod maintenance;
mod pricing;
mod rental;

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
        rental_price_per_day: i128,
        location: String,
    ) {
        equipment::register_equipment(&env, id, equipment_type, rental_price_per_day, location)
    }
    /// Change the availability status of equipment
    pub fn update_availability(env: Env, id: BytesN<32>, available: bool) -> Result<(), Error> {
        // Get equipment and verify caller is the owner
        let equipment = crate::equipment::get_equipment(&env, id.clone())
            .ok_or(Error::from_contract_error(1006))?;
        // Require authentication from the equipment owner
        equipment.owner.require_auth();
        crate::equipment::update_availability(&env, id, equipment.owner, available)
            .map_err(|_| Error::from_contract_error(1004))
    }
    /// Mark equipment status (Good, NeedsService, UnderMaintenance)
    pub fn update_maintenance_status(
        env: Env,
        id: BytesN<32>,
        status: crate::equipment::MaintenanceStatus,
    ) -> Result<(), Error> {
        // Get equipment and verify caller is the owner
        let equipment = crate::equipment::get_equipment(&env, id.clone())
            .ok_or(Error::from_contract_error(1006))?;
        // Require authentication from the equipment owner
        equipment.owner.require_auth();
        crate::equipment::update_maintenance_status(&env, id, equipment.owner, status)
            .map_err(|_| Error::from_contract_error(1005))
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
        crate::rental::create_rental(
            &env,
            equipment_id,
            renter,
            start_date,
            end_date,
            total_price,
        );
    }
    /// Confirm and activate a rental
    pub fn confirm_rental(env: Env, equipment_id: BytesN<32>) {
        // Get equipment owner and verify auth
        let equipment = crate::equipment::get_equipment(&env, equipment_id.clone())
            .expect("Equipment not found");
        equipment.owner.require_auth();
        crate::rental::confirm_rental(&env, equipment_id.clone());
    }
    /// Finalize rental and release equipment
    pub fn complete_rental(env: Env, equipment_id: BytesN<32>) {
        // Get equipment owner and verify auth
        let equipment = crate::equipment::get_equipment(&env, equipment_id.clone())
            .expect("Equipment not found");
        equipment.owner.require_auth();
        crate::rental::complete_rental(&env, equipment_id.clone());
    }
    /// Cancel a rental agreement before start date
    pub fn cancel_rental(env: Env, equipment_id: BytesN<32>) {
        // Get rental details
        let rental =
            crate::rental::get_rental(&env, equipment_id.clone()).expect("Rental not found");
        // Either the renter or equipment owner can cancel
        let caller = env.current_contract_address();
        let equipment = crate::equipment::get_equipment(&env, equipment_id.clone())
            .expect("Equipment not found");
        if caller == rental.renter {
            // Renter is cancelling
            rental.renter.require_auth();
        } else if caller == equipment.owner {
            // Owner is cancelling
            equipment.owner.require_auth();
        } else {
            panic!("Only the renter or equipment owner can cancel a rental");
        }
        crate::rental::cancel_rental(&env, equipment_id.clone());
    }
    /// Retrieve rental details by equipment ID
    pub fn get_rental(env: Env, equipment_id: BytesN<32>) -> Option<crate::rental::Rental> {
        crate::rental::get_rental(&env, equipment_id)
    }
    /// Retrieve all rental agreements for a given equipment
    pub fn get_rental_history_by_equipment(
        env: Env,
        equipment_id: BytesN<32>,
    ) -> Vec<crate::rental::Rental> {
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
    ) -> Result<i128, Error> {
        let eq = crate::equipment::get_equipment(&env, equipment_id)
            .ok_or(Error::from_contract_error(1001))?;
        crate::pricing::compute_total_price(&eq, start_date, end_date)
            .map_err(|_| Error::from_contract_error(1002))
    }
    /// Validate proposed rental price for a date range
    pub fn validate_price(
        env: Env,
        equipment_id: BytesN<32>,
        start_date: u64,
        end_date: u64,
        proposed_price: i128,
        tolerance: i128,
    ) -> Result<(), Error> {
        let equipment =
            equipment::get_equipment(&env, equipment_id).ok_or(Error::from_contract_error(1001))?;
        pricing::validate_price(&equipment, start_date, end_date, proposed_price, tolerance)
            .map_err(|_| Error::from_contract_error(1003))
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
        // Get equipment and verify caller is the owner
        let equipment = crate::equipment::get_equipment(&env, equipment_id.clone())
            .expect("Equipment not found");
        // Require authentication from the equipment owner
        equipment.owner.require_auth();
        crate::maintenance::log_maintenance(&env, equipment_id, status, timestamp, notes);
    }
    /// Retrieve maintenance history for all equipment
    pub fn get_maintenance_history(
        env: Env,
        equipment_id: Option<BytesN<32>>,
    ) -> Vec<crate::maintenance::MaintenanceRecord> {
        crate::maintenance::get_maintenance_history(&env, equipment_id)
    }
}
