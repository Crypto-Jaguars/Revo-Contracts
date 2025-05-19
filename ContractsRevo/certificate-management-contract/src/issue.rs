use soroban_sdk::{Address, Env, Map, Symbol};

use crate::{Certification, DataKey, IssueError, UserCertCount, UsersCertificates};

pub fn issue_certification(
    env: Env,
    issuer: Address,
    recipient: Address,
    cert_type: Symbol,
    expiration_date: u64,
    verification_hash: soroban_sdk::BytesN<32>,
) -> Result<(), IssueError> {
    issuer.require_auth();

    let mut user_cert_count: UserCertCount = env
        .storage()
        .instance()
        .get(&DataKey::UserCertCount)
        .unwrap_or_else(|| Map::new(&env));

    let id = user_cert_count.get(recipient.clone()).unwrap_or(0) + 1;
    let issued_date = env.ledger().timestamp();

    if issued_date >= expiration_date {
        return Err(IssueError::InvalidExpirationDate);
    }

    let certification = Certification::new(
        id,
        cert_type,
        issuer,
        issued_date,
        expiration_date,
        verification_hash,
    );

    let mut users_certificates: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .unwrap_or_else(|| Map::new(&env));

    let mut user_certificates = users_certificates
        .get(recipient.clone())
        .unwrap_or_else(|| Map::new(&env));

    user_certificates.set(id, certification);
    users_certificates.set(recipient.clone(), user_certificates);
    user_cert_count.set(recipient.clone(), id);

    env.storage()
        .instance()
        .set(&DataKey::UserCertCount, &user_cert_count);
    env.storage()
        .instance()
        .set(&DataKey::UsersCertificates, &users_certificates);

    env.events().publish(
        (Symbol::new(&env, "certification_issued"), recipient.clone()),
        env.ledger().timestamp(),
    );

    Ok(())
}