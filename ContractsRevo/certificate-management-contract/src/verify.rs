use soroban_sdk::{Address, BytesN, Env, Symbol};

use crate::{DataKey, UsersCertificates, VerifyError};

pub fn verify_document_hash(
    env: Env,
    owner: Address,
    id: u32,
    submitted_hash: BytesN<32>,
) -> Result<(), VerifyError> {
    let users_certificates: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .ok_or(VerifyError::NotFound)?;

    let user_certificates = users_certificates
        .get(owner.clone())
        .ok_or(VerifyError::NotFound)?;

    let certification = user_certificates.get(id).ok_or(VerifyError::NotFound)?;

    certification.verify(submitted_hash, env.ledger().timestamp())?;

    env.events().publish(
        (Symbol::new(&env, "certification_verified"), owner.clone()),
        env.ledger().timestamp(),
    );
    
    Ok(())
}