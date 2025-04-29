use soroban_sdk::{Address, BytesN, Env, Vec};
use crate::datatypes::{CertificationData, CertificationError, CertificationType, DataKey, Status};

// Get all certifications for a specific holder
pub fn get_holder_certifications(
    env: &Env,
    holder: &Address,
) -> Result<Vec<CertificationData>, CertificationError> {
    let holder_certs = env.storage().persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::HolderCertifications(holder.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    let mut certifications = Vec::new(env);
    
    for cert_id in holder_certs.iter() {
        if let Some(cert) = env.storage().persistent().get::<_, CertificationData>(&DataKey::Certification(cert_id)) {
            certifications.push_back(cert);
        }
    }
    
    Ok(certifications)
}

// Get all certifications for a specific issuer
pub fn get_issuer_certifications(
    env: &Env,
    issuer: &Address,
) -> Result<Vec<CertificationData>, CertificationError> {
    let issuer_certs = env.storage().persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::IssuerCertifications(issuer.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    let mut certifications = Vec::new(env);
    
    for cert_id in issuer_certs.iter() {
        if let Some(cert) = env.storage().persistent().get::<_, CertificationData>(&DataKey::Certification(cert_id)) {
            certifications.push_back(cert);
        }
    }
    
    Ok(certifications)
}

// Get all certifications of a specific type
pub fn get_certifications_by_type(
    env: &Env,
    certification_type: &CertificationType,
) -> Result<Vec<CertificationData>, CertificationError> {
    let type_certs = env.storage().persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::CertificationsByType(certification_type.clone()))
        .unwrap_or_else(|| Vec::new(env));
    
    let mut certifications = Vec::new(env);
    
    for cert_id in type_certs.iter() {
        if let Some(cert) = env.storage().persistent().get::<_, CertificationData>(&DataKey::Certification(cert_id)) {
            certifications.push_back(cert);
        }
    }
    
    Ok(certifications)
}

// Generate a comprehensive audit report with optional filters
pub fn generate_audit_report(
    env: &Env,
    certification_type: Option<CertificationType>,
    issuer: Option<Address>,
    status: Option<Status>,
) -> Result<Vec<CertificationData>, CertificationError> {
    let mut certifications = Vec::new(env);
    
    // If specific issuer is provided, get only their certifications
    if let Some(specific_issuer) = issuer {
        certifications = get_issuer_certifications(env, &specific_issuer)?;
    } 
    // If specific type is provided, get only that type of certifications
    else if let Some(specific_type) = certification_type {
        certifications = get_certifications_by_type(env, &specific_type)?;
    } 
    // Otherwise, gather all certifications from all issuers and types
    else {
        // This would be more efficiently implemented with pagination in a real system
        // For test purposes, we'll use a simplified approach
        
        // Get verified issuers
        if let Some(verified_issuers) = env.storage().instance().get::<_, Vec<Address>>(&DataKey::VerifiedIssuers) {
            for issuer in verified_issuers.iter() {
                let issuer_certs = get_issuer_certifications(env, &issuer)?;
                for cert in issuer_certs.iter() {
                    certifications.push_back(cert);
                }
            }
        }
    }
    
    // Filter by status if provided
    if let Some(specific_status) = status {
        let mut filtered_certs = Vec::new(env);
        for cert in certifications.iter() {
            if cert.status == specific_status {
                filtered_certs.push_back(cert);
            }
        }
        certifications = filtered_certs;
    }
    
    Ok(certifications)
}

// Count certifications by type for statistical purposes
pub fn count_certifications_by_type(
    env: &Env,
) -> Result<Vec<(CertificationType, u32)>, CertificationError> {
    // In a real implementation, this would be more sophisticated
    // For now, let's use a simplified approach for testing
    
    // Example implementation
    let mut result = Vec::new(env);
    
    // Manual count for Organic certifications
    let organic_certs = get_certifications_by_type(env, &CertificationType::Organic)?;
    result.push_back((CertificationType::Organic, organic_certs.len() as u32));
    
    // Manual count for FairTrade certifications
    let fairtrade_certs = get_certifications_by_type(env, &CertificationType::FairTrade)?;
    result.push_back((CertificationType::FairTrade, fairtrade_certs.len() as u32));
    
    // And so on for other types...
    
    Ok(result)
} 