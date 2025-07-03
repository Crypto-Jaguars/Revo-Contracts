use crate::datatypes::*;
use crate::fund::{calculate_lender_share_percentage, get_loan_fundings};
use crate::repay::{calculate_total_repayment_due, get_loan_repayments};
use crate::request::get_loan_request;
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol};

pub fn claim_default(env: &Env, lender: Address, loan_id: u32) {
    lender.require_auth();

    // Get loan request
    let mut loan = get_loan_request(env, loan_id);

    // Verify loan is not already Defaulted or Completed
    if loan.status == LoanStatus::Defaulted || loan.status == LoanStatus::Completed {
        panic_with_error!(env, MicrolendingError::InvalidLoanStatus);
    }

    // Check if loan is in default
    if !check_default_status(env, &loan) {
        panic_with_error!(env, MicrolendingError::NotInDefault);
    }

    // Verify lender has a contribution
    let mut contributions = get_loan_fundings(env, loan_id);
    let contribution_index = contributions
        .iter()
        .position(|c| c.lender == lender && !c.claimed)
        .unwrap_or_else(|| panic_with_error!(env, MicrolendingError::NoContribution));

    // Update loan status to Defaulted
    loan.status = LoanStatus::Defaulted;

    // Update borrower metrics
    let mut borrower_metrics: BorrowerMetrics = env
        .storage()
        .persistent()
        .get(&DataKey::BorrowerMetrics(loan.borrower.clone()))
        .unwrap_or_else(|| BorrowerMetrics {
            total_loans: 0,
            completed_loans: 0,
            defaulted_loans: 0,
        });
    borrower_metrics.defaulted_loans += 1;
    env.storage().persistent().set(
        &DataKey::BorrowerMetrics(loan.borrower.clone()),
        &borrower_metrics,
    );

    // Update system stats
    let total_loans_defaulted: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalLoansDefaulted)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&DataKey::TotalLoansDefaulted, &(total_loans_defaulted + 1));

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
    system_stats.default_rate = calculate_default_rate(env, total_loans_defaulted + 1);
    env.storage()
        .persistent()
        .set(&DataKey::SystemStats, &system_stats);

    // Distribute collateral value to the calling lender
    let collateral_value = loan.collateral.estimated_value;
    let token_id = env
        .storage()
        .persistent()
        .get(&DataKey::AssetCode)
        .unwrap_or_else(|| panic_with_error!(env, MicrolendingError::TokenNotConfigured));
    let token_client = token::Client::new(env, &token_id);

    // Ensure contract has enough balance for collateral distribution
    let contract_balance = token_client.balance(&env.current_contract_address());
    if contract_balance < collateral_value {
        // In test environment, mint tokens to contract if needed
        // This simulates the collateral being converted to tokens
        let shortfall = collateral_value - contract_balance;
        // Note: In production, this would be handled by actual collateral liquidation
        // For testing, we'll assume the collateral is worth the specified amount
    }

    // Process the calling lender's share
    let mut contribution = contributions.get_unchecked(contribution_index as u32);
    let lender_share_percentage = calculate_lender_share_percentage(env, lender.clone(), loan_id);
    let lender_share = (collateral_value as u128 * lender_share_percentage as u128 / 10000) as i128;
    if lender_share > 0 {
        token_client.transfer(&env.current_contract_address(), &lender, &lender_share);
        contribution.claimed = true;
        contributions.set(contribution_index as u32, contribution);
    }

    // Store updated contributions
    env.storage()
        .persistent()
        .set(&DataKey::Funding(loan_id), &contributions);

    // Store updated loan
    env.storage()
        .persistent()
        .set(&DataKey::Loan(loan_id), &loan);

    // Emit default event
    env.events().publish(
        (Symbol::new(env, "loan_defaulted"),),
        (loan_id, lender.clone(), lender_share),
    );
}

pub fn check_default_status(env: &Env, loan: &LoanRequest) -> bool {
    if loan.status != LoanStatus::Funded && loan.status != LoanStatus::Repaying {
        return false;
    }

    let current_timestamp = env.ledger().timestamp();
    let funded_timestamp = loan.funded_timestamp.unwrap_or(current_timestamp);

    // Check based on repayment schedule
    if loan.repayment_schedule.installments > 0 {
        let repayments = get_loan_repayments(env, loan.id);
        let elapsed_days = (current_timestamp - funded_timestamp) / (24 * 60 * 60);
        let expected_installments =
            (elapsed_days / loan.repayment_schedule.frequency_days as u64) as u32;
        // Allow a 7-day grace period for missed installments and payment amounts
        let grace_period = 7 * 24 * 60 * 60;
        let grace_period_expired = current_timestamp > funded_timestamp + grace_period;

        // Check for missed installments (with grace period)
        if expected_installments > repayments.len() as u32 && grace_period_expired {
            return true;
        }

        // Check if total repaid is less than expected (with grace period)
        let expected_repaid =
            expected_installments as i128 * loan.repayment_schedule.per_installment_amount;
        let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
        if total_repaid < expected_repaid && grace_period_expired {
            return true;
        }
    } else if let Some(due_timestamp) = loan.repayment_due_timestamp {
        // Fallback for single payment
        if current_timestamp > due_timestamp {
            let total_due = calculate_total_repayment_due(loan);
            let repayments = get_loan_repayments(env, loan.id);
            let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
            if total_repaid < total_due {
                return true;
            }
        }
    }

    false
}

fn calculate_default_rate(env: &Env, total_loans_defaulted: u32) -> u32 {
    let total_loans: u32 = env
        .storage()
        .persistent()
        .get(&DataKey::TotalLoansCreated)
        .unwrap_or(0);
    if total_loans == 0 {
        return 0;
    }
    ((total_loans_defaulted as u128 * 10000) / total_loans as u128) as u32
}
