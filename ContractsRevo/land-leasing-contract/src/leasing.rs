use soroban_sdk::{
    contracttype, Address, BytesN, Env, String, Vec, Symbol, symbol_short
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaseAgreement {
    pub lease_id: BytesN<32>,
    pub lessor_id: Address,
    pub lessee_id: Address,
    pub land_id: BytesN<32>,
    pub duration: u64, // Duration in months
    pub payment_amount: i128,
    pub status: String, // Active, Terminated, Disputed
    pub start_time: u64,
    pub next_payment_due: u64,
    pub payments_made: u32,
    pub total_payments_required: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Land {
    pub land_id: BytesN<32>,
    pub location: String,
    pub size: u32, // Size in hectares
    pub data_hash: BytesN<32>, // Hash of off-chain land details
    pub owner: Address,
    pub is_available: bool,
}

const LEASE_AGREEMENTS: Symbol = symbol_short!("LEASES");
const LAND_REGISTRY: Symbol = symbol_short!("LANDS");
const LEASE_COUNTER: Symbol = symbol_short!("COUNTER");
const USER_LEASES: Symbol = symbol_short!("USERLS");

pub fn create_lease_agreement(
    env: &Env,
    lessor: Address,
    lessee: Address,
    land_id: BytesN<32>,
    location: String,
    size: u32,
    duration: u64,
    payment_amount: i128,
    data_hash: BytesN<32>,
) -> BytesN<32> {
    // Verify lessor authorization
    lessor.require_auth();
    
    // Validate inputs
    assert!(duration > 0, "Duration must be greater than 0");
    assert!(payment_amount > 0, "Payment amount must be greater than 0");
    assert!(size > 0, "Land size must be greater than 0");
    assert!(lessor != lessee, "Lessor and lessee cannot be the same");
    
    // Generate unique lease ID
    let mut counter: u64 = env.storage().instance().get(&LEASE_COUNTER).unwrap_or(0);
    counter += 1;
    env.storage().instance().set(&LEASE_COUNTER, &counter);
    
    let lease_id = crate::utils::generate_id(env, counter);
    
    // Create or update land record
    let land = Land {
        land_id: land_id.clone(),
        location,
        size,
        data_hash,
        owner: lessor.clone(),
        is_available: false, // Mark as leased
    };
    env.storage().persistent().set(&(LAND_REGISTRY, land_id.clone()), &land);
    
    // Create lease agreement
    let current_time = env.ledger().timestamp();
    let one_month_seconds: u64 = 2629746; // Approximately 1 month in seconds
    
    let lease_agreement = LeaseAgreement {
        lease_id: lease_id.clone(),
        lessor_id: lessor.clone(),
        lessee_id: lessee.clone(),
        land_id,
        duration,
        payment_amount,
        status: String::from_str(env, "Active"),
        start_time: current_time,
        next_payment_due: current_time + one_month_seconds,
        payments_made: 0,
        total_payments_required: duration as u32,
    };
    
    // Store lease agreement
    env.storage().persistent().set(&(LEASE_AGREEMENTS, lease_id.clone()), &lease_agreement);
    
    // Track user leases
    add_user_lease(env, &lessee, &lease_id);
    add_user_lease(env, &lessor, &lease_id);
    
    // Emit event - Fixed symbol length
    env.events().publish(
        (symbol_short!("created"),),
        (lease_id.clone(), lessor, lessee)
    );
    
    lease_id
}

pub fn terminate_lease_agreement(
    env: &Env,
    lease_id: BytesN<32>,
    terminator: Address,
) -> bool {
    // Get lease agreement
    let mut lease: LeaseAgreement = env
        .storage()
        .persistent()
        .get(&(LEASE_AGREEMENTS, lease_id.clone()))
        .expect("Lease agreement not found");
    
    // Verify authorization
    assert!(
        terminator == lease.lessor_id || terminator == lease.lessee_id,
        "Unauthorized termination attempt"
    );
    terminator.require_auth();
    
    // Check if lease is active
    assert_eq!(lease.status, String::from_str(env, "Active"), "Lease is not active");
    
    // Update status
    lease.status = String::from_str(env, "Terminated");
    
    // Store updated lease
    env.storage().persistent().set(&(LEASE_AGREEMENTS, lease_id.clone()), &lease);
    
    // Mark land as available again
    if let Some(mut land) = get_land_info(env, lease.land_id.clone()) {
        land.is_available = true;
        env.storage().persistent().set(&(LAND_REGISTRY, lease.land_id.clone()), &land);
    }
    
    // Emit event - Fixed symbol length
    env.events().publish(
        (symbol_short!("ended"),),
        (lease_id, terminator)
    );
    
    true
}

pub fn extend_lease_duration(
    env: &Env,
    lease_id: BytesN<32>,
    requester: Address,
    additional_months: u64,
) -> bool {
    requester.require_auth();
    
    let mut lease: LeaseAgreement = env
        .storage()
        .persistent()
        .get(&(LEASE_AGREEMENTS, lease_id.clone()))
        .expect("Lease agreement not found");
    
    // Only lessor or lessee can extend
    assert!(
        requester == lease.lessor_id || requester == lease.lessee_id,
        "Unauthorized extension attempt"
    );
    
    // Check if lease is active
    assert_eq!(lease.status, String::from_str(env, "Active"), "Lease is not active");
    
    // Extend duration
    lease.duration += additional_months;
    lease.total_payments_required += additional_months as u32;
    
    // Store updated lease
    env.storage().persistent().set(&(LEASE_AGREEMENTS, lease_id.clone()), &lease);
    
    // Emit event
    env.events().publish(
        (symbol_short!("extended"),),
        (lease_id, requester, additional_months)
    );
    
    true
}

pub fn get_lease_agreement(env: &Env, lease_id: BytesN<32>) -> Option<LeaseAgreement> {
    env.storage().persistent().get(&(LEASE_AGREEMENTS, lease_id))
}

pub fn get_land_info(env: &Env, land_id: BytesN<32>) -> Option<Land> {
    env.storage().persistent().get(&(LAND_REGISTRY, land_id))
}

pub fn update_lease_status(env: &Env, lease_id: BytesN<32>, new_status: String) {
    let mut lease: LeaseAgreement = env
        .storage()
        .persistent()
        .get(&(LEASE_AGREEMENTS, lease_id.clone()))
        .expect("Lease agreement not found");
    
    lease.status = new_status;
    env.storage().persistent().set(&(LEASE_AGREEMENTS, lease_id), &lease);
}

pub fn update_next_payment_due(env: &Env, lease_id: BytesN<32>, next_due: u64) {
    let mut lease: LeaseAgreement = env
        .storage()
        .persistent()
        .get(&(LEASE_AGREEMENTS, lease_id.clone()))
        .expect("Lease agreement not found");
    
    lease.next_payment_due = next_due;
    env.storage().persistent().set(&(LEASE_AGREEMENTS, lease_id), &lease);
}

pub fn increment_payments_made(env: &Env, lease_id: BytesN<32>) {
    let mut lease: LeaseAgreement = env
        .storage()
        .persistent()
        .get(&(LEASE_AGREEMENTS, lease_id.clone()))
        .expect("Lease agreement not found");
    
    lease.payments_made += 1;
    env.storage().persistent().set(&(LEASE_AGREEMENTS, lease_id), &lease);
}

fn add_user_lease(env: &Env, user: &Address, lease_id: &BytesN<32>) {
    let mut user_leases: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&(USER_LEASES, user.clone()))
        .unwrap_or(Vec::new(env));
    
    user_leases.push_back(lease_id.clone());
    env.storage().persistent().set(&(USER_LEASES, user.clone()), &user_leases);
}

pub fn get_user_active_leases(env: &Env, user: Address) -> Vec<BytesN<32>> {
    let user_leases: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&(USER_LEASES, user))
        .unwrap_or(Vec::new(env));
    
    let mut active_leases = Vec::new(env);
    
    // Fix ownership issue by cloning lease_id
    for lease_id in user_leases.iter() {
        if let Some(lease) = get_lease_agreement(env, lease_id.clone()) {
            if lease.status == String::from_str(env, "Active") {
                active_leases.push_back(lease_id.clone());
            }
        }
    }
    
    active_leases
}
