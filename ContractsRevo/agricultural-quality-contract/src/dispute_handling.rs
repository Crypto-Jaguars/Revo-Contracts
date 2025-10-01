use crate::datatypes::*;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{vec, Address, Bytes, BytesN, Env, String, Symbol, Vec};

// Helper function to generate a unique dispute ID
fn generate_dispute_id(
    env: &Env,
    complainant: &Address,
    certification_id: &BytesN<32>,
    timestamp: u64,
) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.append(&complainant.to_xdr(env));
    data.append(&Bytes::from_array(env, &certification_id.to_array()));
    data.append(&Bytes::from_array(env, &timestamp.to_be_bytes()));
    data.append(&env.current_contract_address().to_xdr(env));

    env.crypto().sha256(&data).into()
}

// Helper function to verify mediator authorization
fn verify_mediator(env: &Env, mediator: &Address) -> Result<(), AgricQualityError> {
    let mediators: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Mediators)
        .unwrap_or_else(|| vec![env]);

    if !mediators.contains(mediator) {
        return Err(AgricQualityError::Unauthorized);
    }
    mediator.require_auth();
    Ok(())
}

// Helper function to validate evidence format
fn validate_evidence(_env: &Env, evidence: &Vec<BytesN<32>>) -> Result<(), AgricQualityError> {
    if evidence.is_empty() {
        return Err(AgricQualityError::InvalidEvidence);
    }

    // Check if all evidence hashes are valid (non-zero)
    for hash in evidence.iter() {
        if hash.as_ref().iter().all(|b| b == 0) {
            return Err(AgricQualityError::InvalidEvidence);
        }
    }

    Ok(())
}

pub fn file_dispute(
    env: &Env,
    complainant: &Address,
    certification_id: &BytesN<32>,
    description: String,
    evidence: Vec<BytesN<32>>,
) -> Result<BytesN<32>, AgricQualityError> {
    // Require complainant authorization
    complainant.require_auth();

    // Validate evidence
    validate_evidence(env, &evidence)?;

    // Get certification data
    let certification: CertificationData = env
        .storage()
        .persistent()
        .get(&DataKey::Certification(certification_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Generate dispute ID
    let dispute_id =
        generate_dispute_id(env, complainant, certification_id, env.ledger().timestamp());

    // Create dispute data with default/empty values for non-Option fields
    let dispute = DisputeData {
        id: dispute_id.clone(),
        certification: certification_id.clone(),
        description: description.clone(),
        complainant: complainant.clone(),
        respondent: certification.holder.clone(),
        timestamp: env.ledger().timestamp(),
        status: DisputeStatus::Filed,
        evidence,
        mediator: env.current_contract_address(), // Use contract address as default
        resolution: ResolutionOutcome::Pending,
        appeal_deadline: 0, // Use 0 as default/none value
    };

    // Store dispute data
    env.storage()
        .persistent()
        .set(&DataKey::Dispute(dispute_id.clone()), &dispute);

    // Update disputes by holder
    let mut holder_disputes: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::DisputesByHolder(
            certification.holder.clone().into(),
        ))
        .unwrap_or_else(|| vec![env]);
    holder_disputes.push_back(dispute_id.clone());
    env.storage().persistent().set(
        &DataKey::DisputesByHolder(certification.holder),
        &holder_disputes,
    );

    // Update disputes by standard
    let mut standard_disputes: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&DataKey::DisputesByStandard(certification.standard.clone()))
        .unwrap_or_else(|| vec![env]);
    standard_disputes.push_back(dispute_id.clone());
    env.storage().persistent().set(
        &DataKey::DisputesByStandard(certification.standard),
        &standard_disputes,
    );

    // Emit event
    env.events().publish(
        (Symbol::new(env, "dispute_filed"),),
        (complainant, dispute_id.clone()),
    );

    Ok(dispute_id)
}

pub fn submit_evidence(
    env: &Env,
    handler: &Address,
    dispute_id: &BytesN<32>,
    description: String,
    data_type: Symbol,
    metadata: Vec<(Symbol, String)>,
) -> Result<BytesN<32>, AgricQualityError> {
    // Require handler authorization
    handler.require_auth();

    // Get dispute data
    let mut dispute: DisputeData = env
        .storage()
        .persistent()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Verify handler is involved in dispute
    if dispute.complainant != *handler && dispute.respondent != *handler {
        return Err(AgricQualityError::Unauthorized);
    }

    // Generate evidence hash
    let mut data = Bytes::new(env);
    data.append(&handler.to_xdr(env));
    data.append(&Bytes::from_array(env, &dispute_id.to_array()));
    data.append(&Bytes::from_array(
        env,
        &env.ledger().timestamp().to_be_bytes(),
    ));
    let evidence_hash = env.crypto().sha256(&data);

    // Create evidence record
    let evidence = Evidence {
        hash: evidence_hash.clone().into(),
        handler: handler.clone(),
        timestamp: env.ledger().timestamp(),
        description,
        data_type,
        metadata,
    };

    // Store evidence
    env.storage()
        .persistent()
        .set(&DataKey::Evidence(evidence_hash.clone().into()), &evidence);

    // Update dispute evidence list
    dispute.evidence.push_back(evidence_hash.clone().into());
    env.storage()
        .persistent()
        .set(&DataKey::Dispute(dispute_id.clone()), &dispute);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "evidence_submitted"),),
        (
            handler,
            dispute_id.clone(),
            BytesN::from(evidence_hash.clone()),
        ),
    );

    Ok(BytesN::from(evidence_hash))
}

pub fn assign_mediator(
    env: &Env,
    authority: &Address,
    dispute_id: &BytesN<32>,
    mediator: &Address,
) -> Result<(), AgricQualityError> {
    // Verify authority
    let authorities: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Authorities)
        .unwrap_or_else(|| vec![env]);

    if !authorities.contains(authority) {
        return Err(AgricQualityError::Unauthorized);
    }
    authority.require_auth();

    // Verify mediator is registered
    verify_mediator(env, mediator)?;

    // Get dispute data
    let mut dispute: DisputeData = env
        .storage()
        .instance()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Ensure dispute is in correct status
    if dispute.status != DisputeStatus::Filed {
        return Err(AgricQualityError::InvalidStatus);
    }

    // Update dispute
    dispute.status = DisputeStatus::UnderReview;
    dispute.mediator = mediator.clone();
    dispute.appeal_deadline = env.ledger().timestamp() + 7 * 24 * 60 * 60; // 7 days for appeal

    // Store updated dispute
    env.storage()
        .instance()
        .set(&DataKey::Dispute(dispute_id.clone()), &dispute);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "mediator_assigned"),),
        (authority, dispute_id.clone(), mediator),
    );

    Ok(())
}

pub fn get_dispute_details(
    env: &Env,
    dispute_id: &BytesN<32>,
) -> Result<DisputeData, AgricQualityError> {
    env.storage()
        .persistent()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)
}
