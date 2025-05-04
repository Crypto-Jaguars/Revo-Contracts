#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Vec};

mod datatype;
mod rental;
mod maintenance;
mod test;

use datatype::*;

#[contract]
pub struct EquipmentRentalContract;

#[contractimpl]
impl EquipmentRentalContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn register_equipment(
        env: Env,
        owner: Address,
        metadata: EquipmentMetadata,
    ) -> Result<u64, Error> {
        owner.require_auth();
        
        if metadata.rental_price == 0 {
            return Err(Error::InvalidRentalPrice);
        }

        let equipment_id = env.storage().instance().get(&DataKey::NextEquipmentId)
            .unwrap_or(0);

        let equipment = Equipment {
            id: equipment_id,
            owner: owner.clone(),
            metadata,
            status: EquipmentStatus::Good,
            availability: true,
            maintenance_history: Vec::new(&env),
            rental_history: Vec::new(&env),
        };

        env.storage().instance().set(&DataKey::Equipment(equipment_id), &equipment);
        env.storage().instance().set(&DataKey::NextEquipmentId, &(equipment_id + 1));

        Ok(equipment_id)
    }

    pub fn get_equipment(&self, env: Env, equipment_id: u64) -> Result<Equipment, Error> {
        env.storage()
            .instance()
            .get(&DataKey::Equipment(equipment_id))
            .ok_or(Error::EquipmentNotFound)
    }

    pub fn update_equipment_status(
        env: Env,
        equipment_id: u64,
        new_status: EquipmentStatus,
        notes: String,
    ) -> Result<(), Error> {
        let mut equipment = Self::get_equipment(&Self {}, env.clone(), equipment_id)?;
        
        if new_status == EquipmentStatus::UnderMaintenance {
            equipment.availability = false;
        }

        let maintenance_record = MaintenanceRecord {
            timestamp: env.ledger().timestamp(),
            old_status: equipment.status,
            new_status: new_status.clone(),
            notes,
        };

        equipment.maintenance_history.push_back(maintenance_record);
        equipment.status = new_status;

        env.storage().instance().set(&DataKey::Equipment(equipment_id), &equipment);
        Ok(())
    }

    pub fn create_rental(
        env: Env,
        renter: Address,
        equipment_id: u64,
        start_date: u64,
        end_date: u64,
    ) -> Result<u64, Error> {
        renter.require_auth();

        let equipment = Self::get_equipment(&Self {}, env.clone(), equipment_id)?;
        
        if !equipment.availability {
            return Err(Error::EquipmentNotAvailable);
        }

        if equipment.status != EquipmentStatus::Good {
            return Err(Error::EquipmentNotInGoodCondition);
        }

        if start_date >= end_date {
            return Err(Error::InvalidDateRange);
        }

        let rental_id = env.storage().instance().get(&DataKey::NextRentalId)
            .unwrap_or(0);

        let rental = RentalAgreement {
            id: rental_id,
            equipment_id,
            renter: renter.clone(),
            start_date,
            end_date,
            status: RentalStatus::Pending,
            total_price: calculate_rental_price(&equipment.metadata.rental_price, start_date, end_date),
        };

        env.storage().instance().set(&DataKey::Rental(rental_id), &rental);
        env.storage().instance().set(&DataKey::NextRentalId, &(rental_id + 1));

        // Update equipment rental history
        let mut updated_equipment = equipment;
        updated_equipment.rental_history.push_back(rental_id);
        env.storage().instance().set(&DataKey::Equipment(equipment_id), &updated_equipment);

        Ok(rental_id)
    }

    pub fn activate_rental(env: Env, rental_id: u64) -> Result<(), Error> {
        let mut rental = env.storage()
            .instance()
            .get(&DataKey::Rental(rental_id))
            .ok_or(Error::RentalNotFound)?;

        if rental.status != RentalStatus::Pending {
            return Err(Error::InvalidRentalStatus);
        }

        let equipment = Self::get_equipment(&Self {}, env.clone(), rental.equipment_id)?;
        
        if !equipment.availability || equipment.status != EquipmentStatus::Good {
            return Err(Error::EquipmentNotAvailable);
        }

        rental.status = RentalStatus::Active;
        
        // Update equipment availability
        let mut updated_equipment = equipment;
        updated_equipment.availability = false;
        
        env.storage().instance().set(&DataKey::Rental(rental_id), &rental);
        env.storage().instance().set(&DataKey::Equipment(rental.equipment_id), &updated_equipment);
        
        Ok(())
    }

    pub fn complete_rental(env: Env, rental_id: u64) -> Result<(), Error> {
        let mut rental = env.storage()
            .instance()
            .get(&DataKey::Rental(rental_id))
            .ok_or(Error::RentalNotFound)?;

        if rental.status != RentalStatus::Active {
            return Err(Error::InvalidRentalStatus);
        }

        rental.status = RentalStatus::Completed;
        
        // Make equipment available again
        let mut equipment = Self::get_equipment(&Self {}, env.clone(), rental.equipment_id)?;
        equipment.availability = true;
        
        env.storage().instance().set(&DataKey::Rental(rental_id), &rental);
        env.storage().instance().set(&DataKey::Equipment(rental.equipment_id), &equipment);
        
        Ok(())
    }

    pub fn cancel_rental(env: Env, rental_id: u64) -> Result<(), Error> {
        let mut rental = env.storage()
            .instance()
            .get(&DataKey::Rental(rental_id))
            .ok_or(Error::RentalNotFound)?;

        if rental.status != RentalStatus::Pending {
            return Err(Error::InvalidRentalStatus);
        }

        if env.ledger().timestamp() >= rental.start_date {
            return Err(Error::CancellationPeriodExpired);
        }

        rental.status = RentalStatus::Cancelled;
        
        env.storage().instance().set(&DataKey::Rental(rental_id), &rental);
        Ok(())
    }
}

fn calculate_rental_price(price_per_day: u64, start_date: u64, end_date: u64) -> u64 {
    let duration = end_date.saturating_sub(start_date);
    let days = (duration + 86399) / 86400; // Round up to nearest day (86400 seconds = 1 day)
    price_per_day.saturating_mul(days)
}
