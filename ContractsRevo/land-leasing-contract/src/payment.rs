use crate::leasing::{
    get_lease_agreement, increment_payments_made, update_lease_status, update_next_payment_due,
};
use soroban_sdk::{contracttype, symbol_short, Address, BytesN, Env, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PaymentRecord {
    pub payment_id: BytesN<32>,
    pub lease_id: BytesN<32>,
    pub payer: Address,
    pub amount: i128,
    pub timestamp: u64,
    pub payment_type: String, // Regular, Late, Early, Penalty
}

const PAYMENT_HISTORY: Symbol = symbol_short!("PAYMENTS");
const PAYMENT_COUNTER: Symbol = symbol_short!("PAYCNT");

pub fn process_lease_payment(
    env: &Env,
    lease_id: BytesN<32>,
    payer: Address,
    amount: i128,
) -> bool {
    payer.require_auth();

    // Get lease agreement
    let lease = get_lease_agreement(env, lease_id.clone()).expect("Lease agreement not found");

    // Verify payer is the lessee
    assert_eq!(payer, lease.lessee_id, "Only lessee can make payments");

    // Check if lease is active
    assert_eq!(
        lease.status,
        String::from_str(env, "Active"),
        "Lease is not active"
    );

    // Validate payment amount
    assert!(amount > 0, "Payment amount must be greater than 0");
    assert!(
        amount >= lease.payment_amount,
        "Insufficient payment amount"
    );

    // Determine payment type
    let current_time = env.ledger().timestamp();
    let payment_type = if current_time > lease.next_payment_due {
        String::from_str(env, "Late")
    } else if current_time < lease.next_payment_due - 86400 {
        // 1 day early
        String::from_str(env, "Early")
    } else {
        String::from_str(env, "Regular")
    };

    // Generate payment ID
    let mut counter: u64 = env.storage().instance().get(&PAYMENT_COUNTER).unwrap_or(0);
    counter += 1;
    env.storage().instance().set(&PAYMENT_COUNTER, &counter);

    let payment_id = crate::utils::generate_id(env, counter);

    // Create payment record
    let payment_record = PaymentRecord {
        payment_id: payment_id.clone(),
        lease_id: lease_id.clone(),
        payer: payer.clone(),
        amount,
        timestamp: current_time,
        payment_type: payment_type.clone(),
    };

    // Store payment record
    store_payment_record(env, &lease_id, &payment_record);

    // Update lease payment tracking
    increment_payments_made(env, lease_id.clone());

    // Calculate next payment due (1 month = 2629746 seconds)
    let next_due = lease.next_payment_due + 2629746;
    update_next_payment_due(env, lease_id.clone(), next_due);

    // Check if lease is fully paid
    let updated_lease = get_lease_agreement(env, lease_id.clone()).unwrap();
    if updated_lease.payments_made >= updated_lease.total_payments_required {
        update_lease_status(env, lease_id.clone(), String::from_str(env, "Completed"));
    }

    // Emit payment event
    env.events().publish(
        (symbol_short!("payment"),),
        (payment_id, lease_id, payer, amount),
    );

    true
}

pub fn get_payment_history(env: &Env, lease_id: BytesN<32>) -> Vec<PaymentRecord> {
    env.storage()
        .persistent()
        .get(&(PAYMENT_HISTORY, lease_id))
        .unwrap_or(Vec::new(env))
}

pub fn get_total_payments_made(env: &Env, lease_id: BytesN<32>) -> i128 {
    let payment_history = get_payment_history(env, lease_id);
    let mut total = 0i128;

    for payment in payment_history.iter() {
        if payment.payment_type != String::from_str(env, "Penalty") {
            total += payment.amount;
        }
    }

    total
}

pub fn get_outstanding_balance(env: &Env, lease_id: BytesN<32>) -> i128 {
    let lease = get_lease_agreement(env, lease_id.clone()).expect("Lease agreement not found");

    let total_required = lease.payment_amount * lease.total_payments_required as i128;
    let total_paid = get_total_payments_made(env, lease_id);

    if total_required > total_paid {
        total_required - total_paid
    } else {
        0
    }
}

fn store_payment_record(env: &Env, lease_id: &BytesN<32>, payment_record: &PaymentRecord) {
    let mut payment_history: Vec<PaymentRecord> = env
        .storage()
        .persistent()
        .get(&(PAYMENT_HISTORY, lease_id.clone()))
        .unwrap_or(Vec::new(env));

    payment_history.push_back(payment_record.clone());
    env.storage()
        .persistent()
        .set(&(PAYMENT_HISTORY, lease_id.clone()), &payment_history);
}
