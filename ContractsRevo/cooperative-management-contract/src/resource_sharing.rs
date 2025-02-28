use crate::datatype::{CooperativeError, DataKey, Resource};
use crate::interface::ResourceSharing;
use crate::CooperativeManagementContract;
use soroban_sdk::{Address, Env, String, Vec};

impl ResourceSharing for CooperativeManagementContract {
    // fn register_resource(env: Env, owner: Address, description: String) {
    //     let resource = Resource {
    //         owner: owner.clone(),
    //         description,
    //         available: true,
    //         borrower: None,
    //         schedule: Vec::new(&env),
    //     };
    //     env.storage()
    //         .persistent()
    //         .set(&DataKey::Resource(owner), &resource);
    // }

    fn register_resource(env: Env, owner: Address, description: String) -> Result<(), CooperativeError> {
        let counter_key = DataKey::ResourceCounter;
        let mut counter = env.storage().persistent().get::<DataKey, u32>(&counter_key).unwrap_or(0);
        counter += 1; // Increment counter for unique ID
    
        let resource_key = DataKey::Resource(owner.clone(), counter); // Unique resource key
    
        let resource = Resource {
            owner: owner.clone(),
            description,
            available: true,
            borrower: None,
            schedule: Vec::new(&env),
        };
    
        env.storage().persistent().set(&resource_key, &resource);
        env.storage().persistent().set(&counter_key, &counter); // Update counter
    
        // Store the resource ID under the owner's entry
        let owner_key = DataKey::OwnerResources(owner.clone());
        let mut owned_resources = env.storage().persistent().get::<DataKey, Vec<u32>>(&owner_key).unwrap_or(Vec::new(&env));
        
        owned_resources.push_back(counter);
        env.storage().persistent().set(&owner_key, &owned_resources);
    
        Ok(())
    }
    
    fn get_resources_by_owner(env: Env, owner: Address) -> Vec<u32> {
        let owner_key = DataKey::OwnerResources(owner);
        env.storage().persistent().get::<DataKey, Vec<u32>>(&owner_key).unwrap_or(Vec::new(&env))
    }
    

    fn borrow_resource(
        env: Env,
        borrower: Address,
        owner: Address,
        counter: u32,
    ) -> Result<(), CooperativeError> {
        let owner_key = DataKey::Resource(owner, counter);
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

    fn return_resource(env: Env, owner: Address, counter: u32) -> Result<(), CooperativeError> {
        let owner_key = DataKey::Resource(owner, counter);
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
        counter: u32,
        borrower: Address,
        time_slot: String,
    ) -> Result<(), CooperativeError> {
        let owner_key = DataKey::Resource(owner, counter);
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
