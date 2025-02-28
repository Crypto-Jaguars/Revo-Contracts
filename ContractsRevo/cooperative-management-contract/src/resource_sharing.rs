use crate::datatype::{CooperativeError, DataKey, Resource};
use crate::interface::ResourceSharing;
use crate::CooperativeManagementContract;
use soroban_sdk::{Address, Env, String, Vec};

impl ResourceSharing for CooperativeManagementContract {
    fn register_resource(env: Env, owner: Address, description: String) {
        let resource = Resource {
            owner: owner.clone(),
            description,
            available: true,
            borrower: None,
            schedule: Vec::new(&env),
        };
        env.storage()
            .persistent()
            .set(&DataKey::Resource(owner), &resource);
    }

    fn borrow_resource(
        env: Env,
        borrower: Address,
        owner: Address,
    ) -> Result<(), CooperativeError> {
        let owner_key = DataKey::Resource(owner);
        if let Some(mut resource) = env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&owner_key)
        {
            if resource.available {
                resource.available = false;
                resource.borrower = Some(borrower);
                env.storage().persistent().set(&owner_key, &resource);
                Ok(())
            } else {
                Err(CooperativeError::ResourceNotAvailable)
            }
        } else {
            Err(CooperativeError::ResourceNotAvailable)
        }
    }

    fn return_resource(env: Env, owner: Address) -> Result<(), CooperativeError> {
        let owner_key = DataKey::Resource(owner);
        if let Some(mut resource) = env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&owner_key)
        {
            resource.available = true;
            resource.borrower = None;
            env.storage().persistent().set(&owner_key, &resource);
            Ok(())
        } else {
            Err(CooperativeError::ResourceNotAvailable)
        }
    }

    fn schedule_resource(
        env: Env,
        owner: Address,
        borrower: Address,
        time_slot: String,
    ) -> Result<(), CooperativeError> {
        let owner_key = DataKey::Resource(owner);
        if let Some(mut resource) = env
            .storage()
            .persistent()
            .get::<DataKey, Resource>(&owner_key)
        {
            if resource.available {
                resource.borrower = Some(borrower);
                resource.schedule.push_back(time_slot);
                env.storage().persistent().set(&owner_key, &resource);
                Ok(())
            } else {
                Err(CooperativeError::ResourceNotAvailable)
            }
        } else {
            Err(CooperativeError::ResourceNotAvailable)
        }
    }

    fn track_maintenance(
        env: Env,
        owner: Address,
        details: String,
    ) -> Result<(), CooperativeError> {
        let maintenance_log_key = DataKey::MaintenanceLog(owner);
        let mut logs = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<String>>(&maintenance_log_key)
            .unwrap_or(Vec::new(&env));
        logs.push_back(details);
        env.storage().persistent().set(&maintenance_log_key, &logs);
        Ok(())
    }
}
