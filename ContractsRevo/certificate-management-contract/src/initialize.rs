use soroban_sdk::{Address, Env, Symbol};

use crate::{AdminError, DataKey};

pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(AdminError::AlreadyInitialized);
    }

    admin.require_auth();
    env.storage().instance().set(&DataKey::Admin, &admin);

    env.events().publish(
        (Symbol::new(&env, "contract_initialized"), admin.clone()),
        env.ledger().timestamp(),
    );

    Ok(())
}