use crate::datatypes::*;
use crate::fund::{calculate_lender_share_percentage, get_loan_fundings};
use crate::request::get_loan_request;
use soroban_sdk::{panic_with_error, token, Address, Env, Symbol, Vec};

pub fn repay_loan(env: &Env, borrower: Address, loan_id: u32, amount: i128) {
    borrower.require_auth();

    // Validate inputs
    if amount <= 0 {
        panic_with_error!(env, MicrolendingError::InvalidAmount);
    }

    // Get loan request
    let mut loan = get_loan_request(env, loan_id);

    // Verify borrower is the loan creator
    if loan.borrower != borrower {
        panic_with_error!(env, MicrolendingError::Unauthorized);
    }

    // Verify loan is Funded or Repaying
    if loan.status != LoanStatus::Funded && loan.status != LoanStatus::Repaying {
        panic_with_error!(env, MicrolendingError::LoanNotRepayable);
    }

    // Get repayments
    let mut repayments: Vec<Repayment> = env
        .storage()
        .persistent()
        .get(&DataKey::Repayments(loan_id))
        .unwrap_or_else(|| Vec::new(env));

    // Validate against repayment schedule
    if loan.repayment_schedule.installments > 0 {
        // Check if all installments are paid
        if repayments.len() as u32 >= loan.repayment_schedule.installments {
            panic_with_error!(env, MicrolendingError::RepaymentExceedsDue);
        }
        // Validate amount
        if amount != loan.repayment_schedule.per_installment_amount {
            panic_with_error!(env, MicrolendingError::RepaymentScheduleViolation);
        }
        // Check timing
        let installment_index = repayments.len() as u64; // 0 for first payment, 1 for second, etc.
        let funded_timestamp = loan.funded_timestamp.unwrap_or(env.ledger().timestamp());
        let expected_due_time = funded_timestamp
            + (installment_index * loan.repayment_schedule.frequency_days as u64 * 24 * 60 * 60);
        let current_timestamp = env.ledger().timestamp();
        // Grace period: 3 days early, 7 days late
        let early_window = expected_due_time.saturating_sub(3 * 24 * 60 * 60);
        let late_window = expected_due_time + (7 * 24 * 60 * 60);
        if current_timestamp < early_window || current_timestamp > late_window {
            panic_with_error!(env, MicrolendingError::RepaymentScheduleViolation);
        }
    }

    // Calculate total repayment due
    let total_due = calculate_total_repayment_due(&loan);
    let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
    let remaining_due = total_due - total_repaid;

    // Verify repayment doesn't exceed remaining amount
    if amount > remaining_due {
        panic_with_error!(env, MicrolendingError::RepaymentExceedsDue);
    }

    // Transfer repayment to contract
    let token_id = env
        .storage()
        .persistent()
        .get(&DataKey::AssetCode)
        .unwrap_or_else(|| panic_with_error!(env, MicrolendingError::TokenNotConfigured));
    let token_client = token::Client::new(env, &token_id);

    // Check borrower balance
    if token_client.balance(&borrower) < amount {
        panic_with_error!(env, MicrolendingError::InsufficientBalance);
    }
    token_client.transfer(&borrower, &env.current_contract_address(), &amount);

    // Record repayment
    repayments.push_back(Repayment {
        amount,
        timestamp: env.ledger().timestamp(),
    });
    env.storage()
        .persistent()
        .set(&DataKey::Repayments(loan_id), &repayments);

    // Update loan status
    let is_first_repayment = loan.status == LoanStatus::Funded;
    if is_first_repayment {
        loan.status = LoanStatus::Repaying;
    }

    // Check contract balance for lender distributions
    if token_client.balance(&env.current_contract_address()) < amount {
        panic_with_error!(env, MicrolendingError::InsufficientBalance);
    }

    // Distribute repayment to lenders proportionally
    let mut contributions = get_loan_fundings(env, loan_id);
    for (i, mut contribution) in contributions.iter().enumerate() {
        let lender_share_percentage =
            calculate_lender_share_percentage(env, contribution.lender.clone(), loan_id);
        let lender_share = (amount as u128 * lender_share_percentage as u128 / 10000) as i128;
        if lender_share > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &contribution.lender,
                &lender_share,
            );
        }
    env.storage()
        .persistent()
        .set(&DataKey::Funding(loan_id), &contributions);

    // Check if loan is fully repaid
    let new_total_repaid = total_repaid + amount;
    let is_fully_repaid = new_total_repaid >= total_due;
    if is_fully_repaid {
        loan.status = LoanStatus::Completed;

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
        borrower_metrics.completed_loans += 1;
        env.storage().persistent().set(
            &DataKey::BorrowerMetrics(borrower.clone()),
            &borrower_metrics,
        );

        // Update system stats
        let total_loans_completed: u32 = env
            .storage()
            .persistent()
            .get(&DataKey::TotalLoansCompleted)
            .unwrap_or(0);
        env.storage()
            .persistent()
            .set(&DataKey::TotalLoansCompleted, &(total_loans_completed + 1));
    }

    // Update system stats for total repaid
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
    system_stats.total_repaid += amount;
    env.storage()
        .persistent()
        .set(&DataKey::SystemStats, &system_stats);

    // Store updated loan
    env.storage()
        .persistent()
        .set(&DataKey::Loan(loan_id), &loan);

    // Emit repayment event with installment number
    env.events().publish(
        (Symbol::new(env, "loan_repaid"),),
        (loan_id, borrower.clone(), amount, repayments.len() as u32),
    );

    // Emit completed event if applicable
    if is_fully_repaid {
        env.events().publish(
            (Symbol::new(env, "loan_completed"),),
            (loan_id, borrower.clone()),
        );
    }
}

pub fn get_loan_repayments(env: &Env, loan_id: u32) -> Vec<Repayment> {
    env.storage()
        .persistent()
        .get(&DataKey::Repayments(loan_id))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn calculate_total_repayment_due(loan: &LoanRequest) -> i128 {
    let principal = loan.amount;
    let interest = (principal as u128 * loan.interest_rate as u128 / 10000) as i128;
    principal + interest
}
