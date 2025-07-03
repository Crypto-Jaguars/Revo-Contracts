use soroban_sdk::{Address, Env, Symbol};

use crate::{CertStatus, CertificationError, DataKey, UsersCertificates};

pub fn check_cert_status(
    env: Env,
    owner: Address,
    id: u32,
) -> Result<CertStatus, CertificationError> {
    let users_certificates: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .ok_or(CertificationError::NotFound)?;

    let user_certificates = users_certificates
        .get(owner.clone())
        .ok_or(CertificationError::NotFound)?;

    let certification = user_certificates
        .get(id)
        .ok_or(CertificationError::NotFound)?;

    Ok(certification.status)
}

pub fn expire(env: Env, owner: Address, id: u32) -> Result<(), CertificationError> {
    let mut certs: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .ok_or(CertificationError::NotFound)?;

    let mut user_certificates = certs
        .get(owner.clone())
        .ok_or(CertificationError::NotFound)?;

    let mut certification = user_certificates
        .get(id)
        .ok_or(CertificationError::NotFound)?;

    certification.expire(env.ledger().timestamp())?;

    user_certificates.set(id, certification);
    certs.set(owner.clone(), user_certificates);

    env.storage()
        .instance()
        .set(&DataKey::UsersCertificates, &certs);

    env.events().publish(
        (Symbol::new(&env, "certification_expired"), owner.clone()),
        env.ledger().timestamp(),
    );

    Ok(())
}

pub fn get_cert(
    env: Env,
    owner: Address,
    id: u32,
) -> Result<crate::Certification, CertificationError> {
    let users_certificates: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .ok_or(CertificationError::NotFound)?;

    let user_certificates = users_certificates
        .get(owner.clone())
        .ok_or(CertificationError::NotFound)?;

    let certification = user_certificates
        .get(id)
        .ok_or(CertificationError::NotFound)?;

    Ok(certification)
}
