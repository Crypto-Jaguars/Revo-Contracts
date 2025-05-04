use crate::datatypes::*;
use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

pub fn create_loan_request(
    env: &Env,
    borrower: Address,
    amount: i128,
    purpose: String,
    duration_days: u32,
    interest_rate: u32,
    collateral: CollateralInfo,
) -> u32 {
    borrower.require_auth();

    // Validate inputs
    validate_loan_inputs(env, amount, duration_days, interest_rate, &collateral);

    // Get next loan ID
    let loan_id = next_loan_id(env);

    // Create loan request
    let loan_request = LoanRequest {
        id: loan_id,
        borrower: borrower.clone(),
        amount,
        purpose,
        duration_days,
        interest_rate,
        collateral,
        status: LoanStatus::Pending,
        funded_amount: 0,
        creation_timestamp: env.ledger().timestamp(),
        funded_timestamp: None,
        repayment_due_timestamp: None,
    };

    // Store loan request
    env.storage()
        .persistent()
        .set(&DataKey::Loan(loan_id), &loan_request);

    // Initialize funding contributions
    let contributions: Vec<FundingContribution> = Vec::new(env);
    env.storage()
        .persistent()
        .set(&DataKey::Funding(loan_id), &contributions);

    // Initialize repayments
    let repayments: Vec<Repayment> = Vec::new(env);
    env.storage()
        .persistent()
        .set(&DataKey::Repayments(loan_id), &repayments);

    // Update borrower loans
    let mut borrower_loans: Vec<u32> = env
        .storage()
        .persistent()
        .get(&DataKey::BorrowerLoans(borrower.clone()))
        .unwrap_or_else(|| Vec::new(env));
    borrower_loans.push_back(loan_id);
    env.storage()
        .persistent()
        .set(&DataKey::BorrowerLoans(borrower.clone()), &borrower_loans);

    // Update borrower metrics
    let mut borrower_metrics: BorrowerMetrics = env
        .storage()
        .persistent()
        .get(&DataKey::BorrowerMetrics(borrower.clone()))
        .unwrap_or_else(|| BorrowerMetrics {
            total_loans: 0,
            completed_loans: 0,
            defaulted_loans: 0,
        });
    borrower_metrics.total_loans += 1;
    env.storage().persistent().set(
        &DataKey::BorrowerMetrics(borrower.clone()),
        &borrower_metrics,
    );

    // Update system stats
    let total_loans: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalLoansCreated)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::TotalLoansCreated, &(total_loans + 1));

    // Emit loan created event
    env.events().publish(
        (Symbol::new(&env, "loan_created"),),
        (loan_id, borrower.clone(), amount),
    );

    loan_id
}

pub fn get_loan_request(env: &Env, loan_id: u32) -> LoanRequest {
    env.storage()
        .persistent()
        .get(&DataKey::Loan(loan_id))
        .unwrap_or_else(|| panic!("Loan not found: {}", loan_id))
}

pub fn get_borrower_loans(env: &Env, borrower: Address) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::BorrowerLoans(borrower))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn cancel_loan_request(env: &Env, borrower: Address, loan_id: u32) {
    borrower.require_auth();

    // Get loan request
    let mut loan = get_loan_request(env, loan_id);

    // Verify borrower is the loan creator
    if loan.borrower != borrower {
        panic!("Only the loan creator can cancel this loan request");
    }

    // Verify loan is still in pending status
    if loan.status != LoanStatus::Pending {
        panic!("Only pending loans can be cancelled");
    }

    // Update loan status
    loan.status = LoanStatus::Cancelled;
    env.storage()
        .persistent()
        .set(&DataKey::Loan(loan_id), &loan);

    // Emit loan cancelled event
    env.events().publish(
        (Symbol::new(env, "loan_cancelled"),),
        (loan_id, borrower.clone()),
    );
}

pub fn update_loan_request(
    env: &Env,
    borrower: Address,
    loan_id: u32,
    amount: i128,
    purpose: String,
    duration_days: u32,
    interest_rate: u32,
    collateral: CollateralInfo,
) {
    borrower.require_auth();

    // Get loan request
    let mut loan = get_loan_request(env, loan_id);

    // Verify borrower is the loan creator
    if loan.borrower != borrower {
        panic!("Only the loan creator can update this loan request");
    }

    // Verify loan is still in pending status and has no funding
    if loan.status != LoanStatus::Pending || loan.funded_amount > 0 {
        panic!("Only unfunded pending loans can be updated");
    }

    // Validate inputs
    validate_loan_inputs(env, amount, duration_days, interest_rate, &collateral);

    // Update loan fields
    loan.amount = amount;
    loan.purpose = purpose;
    loan.duration_days = duration_days;
    loan.interest_rate = interest_rate;
    loan.collateral = collateral;

    // Store updated loan
    env.storage()
        .persistent()
        .set(&DataKey::Loan(loan_id), &loan);

    // Emit loan updated event
    env.events().publish(
        (Symbol::new(env, "loan_updated"),),
        (loan_id, borrower.clone()),
    );
}

fn validate_loan_inputs(
    env: &Env,
    amount: i128,
    duration_days: u32,
    interest_rate: u32,
    collateral: &CollateralInfo,
) {
    if amount <= 0 {
        panic!("Loan amount must be positive");
    }
    if duration_days < 1 || duration_days > 1095 {
        panic!("Loan duration must be between 1 day and 3 years");
    }
    if interest_rate == 0 || interest_rate > 10000 {
        panic!("Interest rate must be between 0.01% and 100%");
    }
    let collateral_info = collateral;
    if collateral_info.estimated_value <= 0 {
        panic!("Collateral value must be positive");
    }
    if collateral_info.asset_type.is_empty() {
        panic!("Collateral asset type must not be empty");
    }
}

fn next_loan_id(env: &Env) -> u32 {
    let loan_id: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::NextLoanId)
        .unwrap_or(1u32);
    env.storage()
        .persistent()
        .set(&DataKey::NextLoanId, &(loan_id + 1));
    loan_id
}
