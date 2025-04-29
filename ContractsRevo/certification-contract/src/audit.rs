use soroban_sdk::{Address, BytesN, Env, Vec, vec};
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
        if let Some(cert) = env.storage().persistent().get::<_, CertificationData>(&DataKey::Certification(cert_id.clone())) {
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
        if let Some(cert) = env.storage().persistent().get::<_, CertificationData>(&DataKey::Certification(cert_id.clone())) {
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
        if let Some(cert) = env.storage().persistent().get::<_, CertificationData>(&DataKey::Certification(cert_id.clone())) {
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
    
    // Start with the broadest set (all certifications from verified issuers)
    if let Some(verified_issuers) = env.storage().instance().get::<_, Vec<Address>>(&DataKey::VerifiedIssuers) {
        // If specific issuer is provided, filter to just that issuer
        let issuers_to_check = if let Some(specific_issuer) = issuer {
            if verified_issuers.contains(&specific_issuer) {
                let mut issuers = Vec::new(env);
                issuers.push_back(specific_issuer);
                issuers
            } else {
                // If issuer filter specified but not found in verified issuers, return empty result
                return Ok(Vec::new(env));
            }
        } else {
            // Otherwise use all verified issuers
            verified_issuers
        };
        
        // Get all certifications for the filtered issuers
        for current_issuer in issuers_to_check.iter() {
            let issuer_certs = get_issuer_certifications(env, &current_issuer)?;
            
            // Apply both certification type and status filters in a single pass
            for cert in issuer_certs.iter() {
                // Skip if it doesn't match the certification type filter
                if let Some(ref cert_type) = certification_type {
                    if cert.certification_type != *cert_type {
                        continue;
                    }
                }
                
                // Skip if it doesn't match the status filter
                if let Some(ref specific_status) = status {
                    if cert.status != *specific_status {
                        continue;
                    }
                }
                
                // If we get here, the certification passed all filters
                certifications.push_back(cert);
            }
        }
    }
    
    Ok(certifications)
}

// Count certifications by type for statistical purposes
pub fn count_certifications_by_type(
    env: &Env,
) -> Result<Vec<(CertificationType, u32)>, CertificationError> {
    let mut result = Vec::new(env);
    
    // Define all standard certification types to count
    let mut types = Vec::new(env);
    types.push_back(CertificationType::Organic);
    types.push_back(CertificationType::FairTrade);
    types.push_back(CertificationType::UTZ);
    types.push_back(CertificationType::RainforestAlliance);
    types.push_back(CertificationType::ISO9001);
    types.push_back(CertificationType::ISO14001);
    types.push_back(CertificationType::HACCP);
    types.push_back(CertificationType::Kosher);
    types.push_back(CertificationType::Halal);
    types.push_back(CertificationType::Demeter);
    
    // Count each certification type
    for cert_type in types.iter() {
        let certs = get_certifications_by_type(env, &cert_type)?;
        result.push_back((cert_type.clone(), certs.len() as u32));
    }
    
    // Note: Custom types would need a different approach
    // This implementation only counts the standard types
    
    Ok(result)
} 