use soroban_sdk::{Address, Env, Symbol};

use crate::{DataKey, RevokeError, UsersCertificates};

pub fn revoke_certification(
    env: Env,
    issuer: Address,
    owner: Address,
    id: u32,
) -> Result<(), RevokeError> {
    issuer.require_auth();

    let mut users_certificates: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .ok_or(RevokeError::NotFound)?;

    let mut user_certificates = users_certificates
        .get(owner.clone())
        .ok_or(RevokeError::NotFound)?;

    let mut certification = user_certificates.get(id).ok_or(RevokeError::NotFound)?;

    if certification.issuer != issuer {
        return Err(RevokeError::Unauthorized);
    }

    certification.revoke()?;

    user_certificates.set(id, certification);
    users_certificates.set(owner.clone(), user_certificates);

    env.storage()
        .instance()
        .set(&DataKey::UsersCertificates, &users_certificates);

    env.events().publish(
        (Symbol::new(&env, "certification_revoked"), owner.clone()),
        env.ledger().timestamp(),
    );

    Ok(())
}
