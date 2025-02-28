use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec, vec};
use crate::datatypes::*;

// Helper function to verify mediator authorization
fn verify_mediator(env: &Env, mediator: &Address) -> Result<(), AgricQualityError> {
    let mediators: Vec<Address> = env.storage().instance()
        .get(&DataKey::Mediators)
        .unwrap_or_else(|| vec![env]);

    if !mediators.contains(mediator) {
        return Err(AgricQualityError::Unauthorized);
    }
    mediator.require_auth();
    Ok(())
}

// Helper function to calculate compensation based on resolution outcome
fn calculate_compensation_amount(
    env: &Env,
    certification: &CertificationData,
    dispute: &DisputeData,
    outcome: &ResolutionOutcome,
) -> u32 {
    match outcome {
        ResolutionOutcome::Upheld => 0, // No compensation
        ResolutionOutcome::Revoked => {
            // Full compensation based on certification score
            let base = 100_000; // Base compensation amount
            (base * certification.audit_score) / 100
        },
        ResolutionOutcome::Modified => {
            // Partial compensation
            let base = 50_000;
            (base * certification.audit_score) / 100
        },
        ResolutionOutcome::RequireReinspection => {
            // Compensation for reinspection costs
            25_000
        },
        ResolutionOutcome::Dismissed => 0,
        ResolutionOutcome::Pending => 0, // No compensation while pending
    }
}

pub fn resolve_dispute(
    env: &Env,
    mediator: &Address,
    dispute_id: &BytesN<32>,
    outcome: ResolutionOutcome,
    _notes: String, // Prefix with underscore since unused
) -> Result<(), AgricQualityError> {
    // Verify mediator authorization
    verify_mediator(env, mediator)?;

    // Get dispute data
    let mut dispute: DisputeData = env.storage().instance()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Verify mediator is assigned to this dispute
    if dispute.mediator != *mediator {
        return Err(AgricQualityError::Unauthorized);
    }

    // Ensure dispute is under review
    if dispute.status != DisputeStatus::UnderReview {
        return Err(AgricQualityError::InvalidStatus);
    }

    // Get certification data
    let mut certification: CertificationData = env.storage().instance()
        .get(&DataKey::Certification(dispute.certification.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Update certification status based on resolution
    match outcome {
        ResolutionOutcome::Upheld => {
            certification.status = CertificationStatus::Active;
        },
        ResolutionOutcome::Revoked => {
            certification.status = CertificationStatus::Revoked;
        },
        ResolutionOutcome::Modified => {
            certification.status = CertificationStatus::Active;
            // Adjust audit score based on findings
            certification.audit_score = (certification.audit_score * 90) / 100;
        },
        ResolutionOutcome::RequireReinspection => {
            certification.status = CertificationStatus::Pending;
        },
        ResolutionOutcome::Dismissed => {
            certification.status = CertificationStatus::Active;
        },
        ResolutionOutcome::Pending => {
            return Err(AgricQualityError::InvalidStatus);
        },
    }

    // Update dispute status
    dispute.status = DisputeStatus::Resolved;
    dispute.resolution = outcome;

    // Store updated data
    env.storage().instance().set(&DataKey::Certification(dispute.certification.clone()), &certification);
    env.storage().instance().set(&DataKey::Dispute(dispute_id.clone()), &dispute);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "dispute_resolved"),),
        (mediator, dispute_id.clone(), outcome),
    );

    Ok(())
}

pub fn process_appeal(
    env: &Env,
    appellant: &Address,
    dispute_id: &BytesN<32>,
    new_evidence: Vec<BytesN<32>>,
    _justification: String, 
) -> Result<(), AgricQualityError> {
    // Get dispute data
    let mut dispute: DisputeData = env.storage().instance()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Verify appellant is involved in dispute
    if dispute.complainant != *appellant && dispute.respondent != *appellant {
        return Err(AgricQualityError::Unauthorized);
    }

    // Check appeal deadline
    if dispute.appeal_deadline == 0 {
        return Err(AgricQualityError::NotEligible);
    }
    if env.ledger().timestamp() > dispute.appeal_deadline {
        return Err(AgricQualityError::DeadlinePassed);
    }

    // Ensure dispute is resolved (can only appeal resolved disputes)
    if dispute.status != DisputeStatus::Resolved {
        return Err(AgricQualityError::InvalidStatus);
    }

    // Update dispute status and evidence
    dispute.status = DisputeStatus::Appealed;
    for evidence in new_evidence.iter() {
        dispute.evidence.push_back(evidence.clone());
    }

    // Store updated dispute
    env.storage().instance().set(&DataKey::Dispute(dispute_id.clone()), &dispute);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "dispute_appealed"),),
        (appellant, dispute_id.clone()),
    );

    Ok(())
}

pub fn calculate_compensation(
    env: &Env,
    dispute_id: &BytesN<32>,
) -> Result<u32, AgricQualityError> {
    // Get dispute data
    let dispute: DisputeData = env.storage().instance()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Ensure dispute is resolved
    if dispute.status != DisputeStatus::Resolved {
        return Err(AgricQualityError::InvalidStatus);
    }

    // Get certification data
    let certification: CertificationData = env.storage().instance()
        .get(&DataKey::Certification(dispute.certification.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Calculate compensation based on resolution outcome
    let amount = calculate_compensation_amount(env, &certification, &dispute, &dispute.resolution);

    Ok(amount)
}

pub fn track_enforcement(
    env: &Env,
    authority: &Address,
    dispute_id: &BytesN<32>,
    enforced: bool,
    _notes: String, // Prefix with underscore since unused
) -> Result<(), AgricQualityError> {
    // Verify authority
    let authorities: Vec<Address> = env.storage().instance()
        .get(&DataKey::Authorities)
        .unwrap_or_else(|| vec![env]);

    if !authorities.contains(authority) {
        return Err(AgricQualityError::Unauthorized);
    }
    authority.require_auth();

    // Get dispute data
    let dispute: DisputeData = env.storage().instance()
        .get(&DataKey::Dispute(dispute_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Ensure dispute is resolved
    if dispute.status != DisputeStatus::Resolved {
        return Err(AgricQualityError::InvalidStatus);
    }

    // Emit enforcement tracking event
    env.events().publish(
        (Symbol::new(env, "resolution_enforced"),),
        (authority, dispute_id.clone(), enforced),
    );

    Ok(())
} 