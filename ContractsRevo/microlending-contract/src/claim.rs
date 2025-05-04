use crate::datatypes::*;
use crate::fund::{calculate_lender_share_percentage, get_loan_fundings};
use crate::repay::{calculate_total_repayment_due, get_loan_repayments};
use crate::request::get_loan_request;
use soroban_sdk::{token, Address, Env, Symbol, Vec};

pub fn claim_default(env: &Env, lender: Address, loan_id: u32) {
    lender.require_auth();

    // Get loan request
    let mut loan = get_loan_request(env, loan_id);

    // Verify loan is not already Defaulted or Completed
    if loan.status == LoanStatus::Defaulted || loan.status == LoanStatus::Completed {
        panic!("Loan is already defaulted or completed");
    }

    // Check if loan is in default
    if !check_default_status(env, &loan) {
        panic!("Loan is not in default");
    }

    // Verify lender has a contribution
    let mut contributions = get_loan_fundings(env, loan_id);
    let has_contribution = contributions
        .iter()
        .any(|c| c.lender == lender && !c.claimed);
    if !has_contribution {
        panic!("Lender has no unclaimed contribution to this loan");
    }

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
            total_loans: total_loans_defaulted,
            total_funded: 0,
            total_repaid: 0,
            default_rate: 0,
        });
    system_stats.default_rate =
        calculate_default_rate(env, &system_stats, total_loans_defaulted + 1);
    env.storage()
        .persistent()
        .set(&DataKey::SystemStats, &system_stats);

    // Distribute collateral value to lenders
    let collateral_value = loan.collateral.estimated_value;
    let token_id = env
        .storage()
        .persistent()
        .get(&DataKey::AssetCode)
        .unwrap_or_else(|| panic!("Token contract not configured"));
    let token_client = token::Client::new(env, &token_id);

    for mut contribution in contributions.iter() {
        if !contribution.claimed {
            let lender_share_percentage =
                calculate_lender_share_percentage(env, contribution.lender.clone(), loan_id);
            let lender_share =
                (collateral_value as u128 * lender_share_percentage as u128 / 10000) as i128;
            if lender_share > 0 {
                token_client.transfer(
                    &env.current_contract_address(),
                    &contribution.lender,
                    &lender_share,
                );
                contribution.claimed = true;
            }
        }
    }
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
        (loan_id, lender.clone(), collateral_value),
    );
}

pub fn check_default_status(env: &Env, loan: &LoanRequest) -> bool {
    if loan.status != LoanStatus::Funded && loan.status != LoanStatus::Repaying {
        return false;
    }

    let current_timestamp = env.ledger().timestamp();
    if let Some(due_timestamp) = loan.repayment_due_timestamp {
        if current_timestamp > due_timestamp {
            // Check if loan is fully repaid
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

fn calculate_default_rate(
    env: &Env,
    system_stats: &SystemStats,
    total_loans_defaulted: u32,
) -> u32 {
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
