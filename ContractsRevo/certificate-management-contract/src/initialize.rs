use soroban_sdk::{Address, Env, Map, Symbol};

use crate::{AdminError, DataKey, UserCertCount, UserCertificates};

pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(AdminError::AlreadyInitialized);
    }

    admin.require_auth();

    let user_certificates: UserCertificates = Map::new(&env);
    let user_cert_count: UserCertCount = Map::new(&env);

    env.storage().instance().set(&DataKey::Admin, &admin);

    env.storage()
        .instance()
        .set(&DataKey::UserCertCount, &user_cert_count);
    
    env.storage()
        .instance()
        .set(&DataKey::UsersCertificates, &user_certificates);

    env.events().publish(
        (Symbol::new(&env, "contract_initialized"), admin.clone()),
        env.ledger().timestamp(),
    );

    Ok(())
}