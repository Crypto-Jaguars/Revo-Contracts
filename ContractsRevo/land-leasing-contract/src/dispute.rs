use crate::leasing::{get_lease_agreement, update_lease_status};
use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, String, Symbol};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub dispute_id: BytesN<32>,
    pub lease_id: BytesN<32>,
    pub complainant: Address,
    pub defendant: Address,
    pub reason: String,
    pub status: String, // Open, InProgress, Resolved, Rejected
    pub resolution: String,
    pub resolver: Option<Address>,
    pub created_at: u64,
    pub resolved_at: Option<u64>,
}

const DISPUTES: Symbol = symbol_short!("DISPUTES");
const DISPUTE_COUNTER: Symbol = symbol_short!("DISPCNT");

pub fn raise_dispute(
    env: &Env,
    lease_id: BytesN<32>,
    complainant: Address,
    reason: String,
) -> bool {
    complainant.require_auth();

    // Get lease agreement
    let lease = get_lease_agreement(env, lease_id.clone()).expect("Lease agreement not found");

    // Verify complainant is involved in the lease
    assert!(
        complainant == lease.lessor_id || complainant == lease.lessee_id,
        "Only lease parties can raise disputes"
    );

    // Check if lease is active
    assert_eq!(
        lease.status,
        String::from_str(env, "Active"),
        "Lease is not active"
    );

    // Validate reason
    assert!(!reason.is_empty(), "Dispute reason cannot be empty");

    // Generate dispute ID
    let mut counter: u64 = env.storage().instance().get(&DISPUTE_COUNTER).unwrap_or(0);
    counter += 1;
    env.storage().instance().set(&DISPUTE_COUNTER, &counter);

    let dispute_id = crate::utils::generate_id(env, counter);

    // Determine defendant
    let defendant = if complainant == lease.lessor_id {
        lease.lessee_id.clone()
    } else {
        lease.lessor_id.clone()
    };

    // Create dispute
    let dispute = Dispute {
        dispute_id: dispute_id.clone(),
        lease_id: lease_id.clone(),
        complainant: complainant.clone(),
        defendant,
        reason: reason.clone(),
        status: String::from_str(env, "Open"),
        resolution: String::from_str(env, ""),
        resolver: None,
        created_at: env.ledger().timestamp(),
        resolved_at: None,
    };

    // Store dispute
    env.storage()
        .persistent()
        .set(&(DISPUTES, dispute_id.clone()), &dispute);

    // Update lease status to disputed
    update_lease_status(env, lease_id.clone(), String::from_str(env, "Disputed"));

    // Emit dispute event
    env.events().publish(
        (symbol_short!("dispute"),),
        (dispute_id, lease_id, complainant),
    );

    true
}

pub fn resolve_lease_dispute(
    env: &Env,
    lease_id: BytesN<32>,
    resolver: Address,
    resolution: String,
) -> bool {
    resolver.require_auth();

    // Check if resolver is authorized (admin)
    assert!(
        crate::utils::is_admin(env, &resolver),
        "Unauthorized resolver"
    );

    // Find open dispute for this lease
    let dispute_id =
        find_open_dispute_for_lease(env, &lease_id).expect("No open dispute found for this lease");

    let mut dispute: Dispute = env
        .storage()
        .persistent()
        .get(&(DISPUTES, dispute_id.clone()))
        .expect("Dispute not found");

    // Check if dispute is open
    assert_eq!(
        dispute.status,
        String::from_str(env, "Open"),
        "Dispute is not open"
    );

    // Validate resolution
    assert!(!resolution.is_empty(), "Resolution cannot be empty");

    // Update dispute
    dispute.status = String::from_str(env, "Resolved");
    dispute.resolution = resolution.clone();
    dispute.resolver = Some(resolver.clone());
    dispute.resolved_at = Some(env.ledger().timestamp());

    // Store updated dispute
    env.storage()
        .persistent()
        .set(&(DISPUTES, dispute_id.clone()), &dispute);

    // Update lease status back to active
    update_lease_status(env, lease_id.clone(), String::from_str(env, "Active"));

    // Emit resolution event
    env.events().publish(
        (symbol_short!("resolved"),),
        (dispute_id, lease_id, resolver),
    );

    true
}

pub fn get_dispute_details(env: &Env, dispute_id: BytesN<32>) -> Option<Dispute> {
    env.storage().persistent().get(&(DISPUTES, dispute_id))
}

fn find_open_dispute_for_lease(env: &Env, lease_id: &BytesN<32>) -> Option<BytesN<32>> {
    let counter: u64 = env.storage().instance().get(&DISPUTE_COUNTER).unwrap_or(0);

    for i in 1..=counter {
        let dispute_id = crate::utils::generate_id(env, i);
        if let Some(dispute) = get_dispute_details(env, dispute_id.clone()) {
            if dispute.lease_id == *lease_id && dispute.status == String::from_str(env, "Open") {
                return Some(dispute_id);
            }
        }
    }

    None
}
