use soroban_sdk::{Address, BytesN, Env, Symbol, Vec, vec};
use crate::datatypes::{CertificationData, CertificationError, CertificationEvent, DataKey, Status};

// Verify if a certification is valid based on its status and date
fn is_certification_valid(env: &Env, certification: &CertificationData) -> bool {
    let current_time = env.ledger().timestamp();
    
    match certification.status {
        Status::Valid => {
            // Check if not expired
            current_time >= certification.valid_from && current_time <= certification.valid_to
        },
        Status::Expired | Status::Revoked => false,
    }
}

// Update certification status based on time
fn update_status_if_needed(env: &Env, certification_id: &BytesN<32>) -> Result<CertificationData, CertificationError> {
    let cert_key = DataKey::Certification(certification_id.clone());
    let mut certification = env.storage().persistent().get::<_, CertificationData>(&cert_key)
        .ok_or(CertificationError::CertificationNotFound)?;
    
    let current_time = env.ledger().timestamp();
    
    // If valid but expired, update status
    if certification.status == Status::Valid && current_time > certification.valid_to {
        certification.status = Status::Expired;
        certification.last_updated = current_time;
        
        env.storage().persistent().set(&cert_key, &certification);
        
        // Record expiration event
        let expiration_event = CertificationEvent {
            certification_id: certification_id.clone(),
            event_type: Symbol::new(env, "expired"),
            timestamp: current_time,
            data: vec![env, Symbol::new(env, "expired")],
        };
        
        let mut events = env.storage().persistent()
            .get::<_, Vec<CertificationEvent>>(&DataKey::CertificationEvents(certification_id.clone()))
            .unwrap_or_else(|| Vec::new(env));
        events.push_back(expiration_event);
        env.storage().persistent().set(&DataKey::CertificationEvents(certification_id.clone()), &events);
        
        // Publish event
        env.events().publish(
            (Symbol::new(env, "certification_expired"), certification.issuer.clone()),
            certification_id.clone(),
        );
    }
    
    Ok(certification)
}

// Verify a certification by checking document hash and validity
pub fn verify_certification(
    env: &Env,
    certification_id: &BytesN<32>,
    document_hash: &BytesN<32>,
) -> Result<bool, CertificationError> {
    let certification = update_status_if_needed(env, certification_id)?;
    
    // First check document hash
    if certification.document_hash != *document_hash {
        return Ok(false);
    }
    
    // Then check validity
    Ok(is_certification_valid(env, &certification))
}

// Revoke a certification
pub fn revoke_certification(
    env: &Env,
    issuer: &Address,
    certification_id: &BytesN<32>,
    reason: &Symbol,
) -> Result<(), CertificationError> {
    issuer.require_auth();
    
    let cert_key = DataKey::Certification(certification_id.clone());
    let mut certification = env.storage().persistent().get::<_, CertificationData>(&cert_key)
        .ok_or(CertificationError::CertificationNotFound)?;
    
    // Verify issuer has authority to revoke
    if certification.issuer != *issuer {
        return Err(CertificationError::UnauthorizedAccess);
    }
    
    // Check if already revoked
    if certification.status == Status::Revoked {
        return Err(CertificationError::InvalidStatus);
    }
    
    let current_time = env.ledger().timestamp();
    
    // Update certification status
    certification.status = Status::Revoked;
    certification.revocation_reason = Some(reason.clone());
    certification.last_updated = current_time;
    
    env.storage().persistent().set(&cert_key, &certification);
    
    // Record revocation event
    let revocation_event = CertificationEvent {
        certification_id: certification_id.clone(),
        event_type: Symbol::new(env, "revoked"),
        timestamp: current_time,
        data: vec![env, Symbol::new(env, "revoked"), reason.clone()],
    };
    
    let mut events = env.storage().persistent()
        .get::<_, Vec<CertificationEvent>>(&DataKey::CertificationEvents(certification_id.clone()))
        .unwrap_or_else(|| Vec::new(env));
    events.push_back(revocation_event);
    env.storage().persistent().set(&DataKey::CertificationEvents(certification_id.clone()), &events);
    
    // Publish event
    env.events().publish(
        (Symbol::new(env, "certification_revoked"), 
         issuer.clone(), 
         certification.holder.clone()),
        (certification_id.clone(), reason.clone()),
    );
    
    Ok(())
}

// Get all events for a certification (audit trail)
pub fn get_certification_events(
    env: &Env, 
    certification_id: &BytesN<32>
) -> Result<Vec<CertificationEvent>, CertificationError> {
    // Check if certification exists
    if !env.storage().persistent().has(&DataKey::Certification(certification_id.clone())) {
        return Err(CertificationError::CertificationNotFound);
    }
    
    let events = env.storage().persistent()
        .get::<_, Vec<CertificationEvent>>(&DataKey::CertificationEvents(certification_id.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    Ok(events)
} 