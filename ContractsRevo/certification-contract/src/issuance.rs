use soroban_sdk::{Address, BytesN, Bytes, Env, Symbol, Vec, vec, IntoVal};
use crate::datatypes::{CertificationData, CertificationError, CertificationType, CertificationEvent, DataKey, Status};

// Helper function to generate a unique certification ID
fn generate_certification_id(
    env: &Env,
    issuer: &Address,
    holder: &Address,
    document_hash: &BytesN<32>,
    timestamp: u64,
) -> BytesN<32> {
    let mut data = Bytes::new(env);
    
    // Create unique input by combining data
    data.append(&Bytes::from_array(env, &document_hash.to_array()));
    
    // Use a simpler approach - just convert timestamp to bytes
    let timestamp_bytes = timestamp.to_be_bytes();
    let timestamp_data = Bytes::from_slice(env, &timestamp_bytes);
    data.append(&timestamp_data);
    
    env.crypto().sha256(&data).into()
}

// Function to check if issuer is verified
fn is_verified_issuer(env: &Env, issuer: &Address) -> bool {
    if let Some(verified_issuers) = env.storage().instance().get::<_, Vec<Address>>(&DataKey::VerifiedIssuers) {
        return verified_issuers.contains(issuer);
    }
    false
}

// Verify metadata is valid
fn validate_metadata(metadata: &Vec<Symbol>) -> bool {
    !metadata.is_empty() && metadata.len() < 100
}

// Issue a new certification
pub fn issue_certification(
    env: &Env,
    issuer: &Address,
    holder: &Address,
    certification_type: CertificationType,
    document_hash: &BytesN<32>,
    metadata: &Vec<Symbol>,
    valid_from: u64,
    valid_to: u64,
) -> Result<BytesN<32>, CertificationError> {
    // Require authorization from issuer
    issuer.require_auth();
    
    // Validate issuer is verified
    if !is_verified_issuer(env, issuer) {
        return Err(CertificationError::InvalidIssuer);
    }
    
    // Ensure validity dates are logical
    if valid_from >= valid_to || valid_from < env.ledger().timestamp() {
        return Err(CertificationError::InvalidValidity);
    }
    
    // Validate metadata
    if !validate_metadata(metadata) {
        return Err(CertificationError::InvalidMetadata);
    }
    
    // Generate certification ID
    let certification_id = generate_certification_id(
        env,
        issuer,
        holder,
        document_hash,
        env.ledger().timestamp(),
    );
    
    // Check if certification already exists
    if env.storage().persistent().has(&DataKey::Certification(certification_id.clone())) {
        return Err(CertificationError::AlreadyInitialized);
    }
    
    let current_time = env.ledger().timestamp();
    
    // Create certification data
    let certification = CertificationData {
        certification_id: certification_id.clone(),
        issuer: issuer.clone(),
        holder: holder.clone(),
        certification_type: certification_type.clone(),
        document_hash: document_hash.clone(),
        metadata: metadata.clone(),
        status: Status::Valid,
        issue_date: current_time,
        valid_from,
        valid_to,
        revocation_reason: None,
        last_updated: current_time,
    };
    
    // Store certification
    env.storage().persistent().set(&DataKey::Certification(certification_id.clone()), &certification);
    
    // Add to holder's certifications
    let mut holder_certs = env.storage().persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::HolderCertifications(holder.clone()))
        .unwrap_or_else(|| Vec::new(env));
    holder_certs.push_back(certification_id.clone());
    env.storage().persistent().set(&DataKey::HolderCertifications(holder.clone()), &holder_certs);
    
    // Add to issuer's certifications
    let mut issuer_certs = env.storage().persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::IssuerCertifications(issuer.clone()))
        .unwrap_or_else(|| Vec::new(env));
    issuer_certs.push_back(certification_id.clone());
    env.storage().persistent().set(&DataKey::IssuerCertifications(issuer.clone()), &issuer_certs);
    
    // Add to certifications by type
    let mut type_certs = env.storage().persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::CertificationsByType(certification_type.clone()))
        .unwrap_or_else(|| Vec::new(env));
    type_certs.push_back(certification_id.clone());
    env.storage().persistent().set(&DataKey::CertificationsByType(certification_type.clone()), &type_certs);
    
    // Record issuance event
    let issuance_event = CertificationEvent {
        certification_id: certification_id.clone(),
        event_type: Symbol::new(env, "issued"),
        timestamp: current_time,
        data: vec![env, Symbol::new(env, "issued")],
    };
    
    let mut events = env.storage().persistent()
        .get::<_, Vec<CertificationEvent>>(&DataKey::CertificationEvents(certification_id.clone()))
        .unwrap_or_else(|| Vec::new(env));
    events.push_back(issuance_event);
    env.storage().persistent().set(&DataKey::CertificationEvents(certification_id.clone()), &events);
    
    // Publish event
    env.events().publish(
        (Symbol::new(env, "certification_issued"), 
         issuer.clone(), 
         holder.clone()),
        certification_id.clone(),
    );
    
    Ok(certification_id)
}

// Add an issuer to the verified list
pub fn add_verified_issuer(env: &Env, admin: &Address, issuer: &Address) -> Result<(), CertificationError> {
    // Ensure admin authorization
    let stored_admin: Address = env.storage().instance().get(&DataKey::Admin)
        .ok_or(CertificationError::UnauthorizedAccess)?;
        
    if stored_admin != *admin {
        return Err(CertificationError::UnauthorizedAccess);
    }
    
    admin.require_auth();
    
    let mut verified_issuers = env.storage().instance()
        .get::<_, Vec<Address>>(&DataKey::VerifiedIssuers)
        .unwrap_or_else(|| Vec::new(env));
        
    if !verified_issuers.contains(issuer) {
        verified_issuers.push_back(issuer.clone());
        env.storage().instance().set(&DataKey::VerifiedIssuers, &verified_issuers);
        
        env.events().publish(
            (Symbol::new(env, "issuer_verified"), admin.clone()),
            issuer.clone(),
        );
    }
    
    Ok(())
} 