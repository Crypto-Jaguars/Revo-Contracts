use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{log, vec, Address, Bytes, BytesN, Env, String, Symbol, Vec};

use crate::datatypes::*;

// Helper function to generate a unique certification ID
fn generate_certification_id(
    env: &Env,
    holder: &Address,
    standard: &QualityStandard,
    timestamp: u64,
) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.append(&holder.to_xdr(env));
    data.append(&Bytes::from_array(env, &[standard.to_u8()]));
    data.append(&Bytes::from_array(env, &timestamp.to_be_bytes()));
    data.append(&env.current_contract_address().to_xdr(env));

    env.crypto().sha256(&data).into()
}

// Helper function to verify inspector authorization
fn verify_inspector(env: &Env, inspector: &Address) -> Result<(), AgricQualityError> {
    let inspectors: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Inspectors)
        .unwrap_or_else(|| vec![env]);

    if !inspectors.contains(inspector) {
        return Err(AgricQualityError::Unauthorized);
    }
    inspector.require_auth();

    Ok(())
}

// Helper function to verify issuer authorization
fn verify_issuer(env: &Env, issuer: &Address) -> Result<(), AgricQualityError> {
    let authorities: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Authorities)
        .unwrap_or_else(|| vec![env]);

    if !authorities.contains(issuer) {
        return Err(AgricQualityError::Unauthorized);
    }
    issuer.require_auth();
    Ok(())
}

pub fn submit_for_certification(
    env: &Env,
    holder: &Address,
    standard: QualityStandard,
    conditions: Vec<String>,
) -> Result<BytesN<32>, AgricQualityError> {
    // Require holder authorization
    holder.require_auth();

    // Generate certification ID
    let certification_id =
        generate_certification_id(env, holder, &standard, env.ledger().timestamp());

    let meta_data_len = conditions.len();
    if meta_data_len < 1 || meta_data_len > 8 {
        return Err(AgricQualityError::InvalidInput);
    }

    // Check if certification already exists
    if env
        .storage()
        .persistent()
        .has(&DataKey::Certification(certification_id.clone()))
    {
        return Err(AgricQualityError::AlreadyExists);
    }

    // Create certification data
    let certification = CertificationData {
        holder: holder.clone(),
        standard: standard.clone(),
        status: CertificationStatus::Pending,
        issue_date: env.ledger().timestamp(),
        expiry_date: 0,
        issuer: Address::from_str(
            &env,
            "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF",
        ),
        audit_score: 0,
        conditions,
    };

    env.storage().persistent().set(
        &DataKey::Certification(certification_id.clone()),
        &certification,
    );

    let mut holder_certs: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::HolderCertifications(holder.clone()))
        .unwrap_or_else(|| vec![env]);
    holder_certs.push_back(certification_id.clone());
    env.storage().persistent().set(
        &DataKey::HolderCertifications(holder.clone()),
        &holder_certs,
    );

    // Emit event
    env.events().publish(
        (Symbol::new(env, "certification_submitted"),),
        (holder, certification_id.clone()),
    );

    Ok(certification_id)
}

pub fn record_inspection(
    env: &Env,
    inspector: &Address,
    certification_id: &BytesN<32>,
    metrics: Vec<(Symbol, u32)>,
    findings: Vec<String>,
    recommendations: Vec<String>,
) -> Result<(), AgricQualityError> {
    // Verify inspector authorization

    verify_inspector(env, inspector)?;

    let certification: CertificationData = env
        .storage()
        .persistent()
        .get(&DataKey::Certification(certification_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Ensure certification is pending
    if certification.status != CertificationStatus::Pending {
        return Err(AgricQualityError::InvalidStatus);
    }

    // Calculate overall score
    let total_score: u32 = metrics.iter().map(|(_, score)| score).sum();
    let overall_score = if !metrics.is_empty() {
        total_score / (metrics.len() as u32)
    } else {
        0
    };

    let report = InspectionReport {
        inspector: inspector.clone(),
        timestamp: env.ledger().timestamp(),
        metrics,
        overall_score,
        findings,
        recommendations,
    };

    // Store inspection report
    env.storage()
        .persistent()
        .set(&DataKey::Inspection(certification_id.clone()), &report);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "inspection_recorded"),),
        (inspector, certification_id.clone(), overall_score),
    );

    Ok(())
}

pub fn process_certification(
    env: &Env,
    issuer: &Address,
    certification_id: &BytesN<32>,
    approved: bool,
    validity_period: u64,
) -> Result<(), AgricQualityError> {
    // Verify issuer authorization
    verify_issuer(env, issuer)?;

    let mut certification: CertificationData = env
        .storage()
        .persistent()
        .get(&DataKey::Certification(certification_id.clone()))
        .ok_or_else(|| AgricQualityError::NotFound)?;

    if certification.status != CertificationStatus::Pending {
        return Err(AgricQualityError::InvalidStatus);
    }

    let inspection: InspectionReport = env
        .storage()
        .persistent()
        .get(&DataKey::Inspection(certification_id.clone()))
        .ok_or_else(|| AgricQualityError::NotFound)?;

    // Update certification status and details
    certification.status = if approved {
        CertificationStatus::Active
    } else {
        CertificationStatus::Revoked
    };
    certification.issuer = issuer.clone();
    certification.audit_score = inspection.overall_score;

    if approved {
        certification.expiry_date = env.ledger().timestamp() + validity_period;
    }

    // Store updated certification
    env.storage().persistent().set(
        &DataKey::Certification(certification_id.clone()),
        &certification,
    );

    // Update issuer's certifications list
    let mut issuer_certs: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::IssuerCertifications(issuer.clone()))
        .unwrap_or_else(|| vec![env]);
    issuer_certs.push_back(certification_id.clone());
    env.storage().persistent().set(
        &DataKey::IssuerCertifications(issuer.clone()),
        &issuer_certs,
    );

    // Emit event
    env.events().publish(
        (Symbol::new(env, "certification_processed"),),
        (issuer, certification_id.clone(), approved),
    );

    Ok(())
}

pub fn get_certification_history(
    env: &Env,
    holder: &Address,
) -> Result<Vec<CertificationData>, AgricQualityError> {
    let cert_ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::HolderCertifications(holder.clone()))
        .unwrap_or_else(|| vec![env]);

    let mut certifications = vec![env];
    for id in cert_ids.iter() {
        if let Some(cert) = env
            .storage()
            .persistent()
            .get(&DataKey::Certification(id.clone()))
        {
            certifications.push_back(cert);
        }
    }

    Ok(certifications)
}
