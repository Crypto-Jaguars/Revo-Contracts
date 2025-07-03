use crate::datatypes::*;
use crate::request::get_loan_request;
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol, Vec};

pub fn fund_loan(env: &Env, lender: Address, loan_id: u32, amount: i128) {
    lender.require_auth();

    // Validate inputs
    if amount <= 0 {
        panic_with_error!(env, MicrolendingError::InvalidAmount);
    }

    // Get loan request
    let mut loan = get_loan_request(env, loan_id);

    // Verify loan is not fully funded yet (allow funding if still needs money)
    if loan.funded_amount >= loan.amount {
        panic_with_error!(env, MicrolendingError::LoanFullyFunded);
    }

    // Verify loan is in a fundable state (Pending or Funded but not fully funded)
    if loan.status != LoanStatus::Pending && loan.status != LoanStatus::Funded {
        panic_with_error!(env, MicrolendingError::InvalidLoanStatus);
    }

    // Verify lender is not the borrower
    if loan.borrower == lender {
        panic_with_error!(env, MicrolendingError::Unauthorized);
    }

    // Calculate remaining amount needed
    let remaining_amount = loan.amount - loan.funded_amount;
    if remaining_amount <= 0 {
        panic_with_error!(env, MicrolendingError::LoanFullyFunded);
    }

    // Accept up to the remaining amount
    let funding_amount = amount.min(remaining_amount);

    // Transfer tokens to contract
    let token_id = env
        .storage()
        .persistent()
        .get(&DataKey::AssetCode)
        .unwrap_or_else(|| panic_with_error!(env, MicrolendingError::TokenNotConfigured));
    let token_client = token::Client::new(env, &token_id);

    // Check lender balance
    if token_client.balance(&lender) < funding_amount {
        panic_with_error!(env, MicrolendingError::InsufficientBalance);
    }
    token_client.transfer(&lender, &env.current_contract_address(), &funding_amount);

    // Update funded amount
    loan.funded_amount += funding_amount;
    let is_fully_funded = loan.funded_amount >= loan.amount;

    // Record funding contribution
    let mut contributions: Vec<FundingContribution> = env
        .storage()
        .persistent()
        .get(&DataKey::Funding(loan_id))
        .unwrap_or_else(|| Vec::new(env));
    contributions.push_back(FundingContribution {
        lender: lender.clone(),
        amount: funding_amount,
        timestamp: env.ledger().timestamp(),
        claimed: false,
    });
    env.storage()
        .persistent()
        .set(&DataKey::Funding(loan_id), &contributions);

    // Update lender loans
    let mut lender_loans: Vec<u32> = env
        .storage()
        .persistent()
        .get(&DataKey::LenderLoans(lender.clone()))
        .unwrap_or_else(|| Vec::new(env));
    if !lender_loans.contains(&loan_id) {
        lender_loans.push_back(loan_id);
        env.storage()
            .persistent()
            .set(&DataKey::LenderLoans(lender.clone()), &lender_loans);
    }

    // If fully funded, update loan status, timestamps, and disburse to borrower
    let mut total_loans_funded: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalLoansFunded)
        .unwrap_or(0);

    if is_fully_funded {
        loan.status = LoanStatus::Funded;
        loan.funded_timestamp = Some(env.ledger().timestamp());
        // Set final due date based on schedule or duration
        let due_timestamp = if loan.repayment_schedule.installments > 0 {
            env.ledger().timestamp()
                + (loan.repayment_schedule.installments * loan.repayment_schedule.frequency_days)
                    as u64
                    * 24
                    * 60
                    * 60
        } else {
            env.ledger().timestamp() + (loan.duration_days as u64) * 24 * 60 * 60
        };
        loan.repayment_due_timestamp = Some(due_timestamp);

        // Check contract balance
        if token_client.balance(&env.current_contract_address()) < loan.funded_amount {
            panic_with_error!(env, MicrolendingError::InsufficientBalance);
        }
        token_client.transfer(
            &env.current_contract_address(),
            &loan.borrower,
            &loan.funded_amount,
        );

        total_loans_funded += 1;
        env.storage()
            .persistent()
            .set(&DataKey::TotalLoansFunded, &total_loans_funded);
    }

    // Update system stats for total funded amount
    let mut system_stats: SystemStats = env
        .storage()
        .persistent()
        .get(&DataKey::SystemStats)
        .unwrap_or_else(|| SystemStats {
            total_loans: env
                .storage()
                .persistent()
                .get(&DataKey::TotalLoansCreated)
                .unwrap_or(0),
            total_funded: 0,
            total_repaid: 0,
            default_rate: 0,
        });
    system_stats.total_funded += funding_amount;
    env.storage()
        .persistent()
        .set(&DataKey::SystemStats, &system_stats);

    // Store updated loan
    env.storage()
        .persistent()
        .set(&DataKey::Loan(loan_id), &loan);

    // Emit funding event
    env.events().publish(
        (Symbol::new(env, "loan_funded"),),
        (loan_id, lender.clone(), funding_amount),
    );

    // Emit fully funded event if applicable
    if is_fully_funded {
        env.events().publish(
            (Symbol::new(env, "loan_fully_funded"),),
            (loan_id, loan.borrower.clone()),
        );
    }
}

pub fn get_loan_fundings(env: &Env, loan_id: u32) -> Vec<FundingContribution> {
    env.storage()
        .persistent()
        .get(&DataKey::Funding(loan_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn get_lender_loans(env: &Env, lender: Address) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&DataKey::LenderLoans(lender))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn calculate_lender_share(env: &Env, lender: Address, loan_id: u32) -> i128 {
    let fundings = get_loan_fundings(env, loan_id);
    let mut lender_total: i128 = 0;
    for funding in fundings.iter() {
        if funding.lender == lender {
            lender_total += funding.amount;
        }
    }
    lender_total
}

pub fn calculate_lender_share_percentage(env: &Env, lender: Address, loan_id: u32) -> u32 {
    let loan = get_loan_request(env, loan_id);
    let lender_share = calculate_lender_share(env, lender, loan_id);
    if loan.amount == 0 {
        return 0;
    }
    ((lender_share as u128 * 10000) / loan.amount as u128) as u32
}
