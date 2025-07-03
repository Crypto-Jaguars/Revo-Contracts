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

    // Distribute repayment to lenders proportionally with remainder handling
    let mut contributions = get_loan_fundings(env, loan_id);
    let mut total_distributed: i128 = 0;
    let mut eligible_lenders: Vec<(u32, Address, u32)> = Vec::new(env); // (index, lender, percentage)
    
    // First pass: calculate initial shares and identify eligible lenders
    for (i, contribution) in contributions.iter().enumerate() {
        if !contribution.claimed {
            let lender_share_percentage =
                calculate_lender_share_percentage(env, contribution.lender.clone(), loan_id);
            if lender_share_percentage > 0 {
                eligible_lenders.push_back((i as u32, contribution.lender.clone(), lender_share_percentage));
            }
        }
    }
    
    // Calculate initial distribution amounts
    let mut distribution_amounts: Vec<i128> = Vec::new(env);
    for i in 0..eligible_lenders.len() {
        let (_, _, percentage) = eligible_lenders.get_unchecked(i as u32);
        let initial_share = (amount as u128 * percentage as u128 / 10000) as i128;
        distribution_amounts.push_back(initial_share);
        total_distributed += initial_share;
    }
    
    // Calculate remainder
    let remainder = amount - total_distributed;
    
    // Distribute remainder proportionally among eligible lenders
    if remainder > 0 && !eligible_lenders.is_empty() {
        let mut total_percentage: u32 = 0;
        for i in 0..eligible_lenders.len() {
            let (_, _, percentage) = eligible_lenders.get_unchecked(i as u32);
            total_percentage += percentage;
        }
        
        let mut remainder_distributed: i128 = 0;
        
        for i in 0..eligible_lenders.len() {
            let (_, _, percentage) = eligible_lenders.get_unchecked(i as u32);
            if total_percentage > 0 {
                let remainder_share = (remainder as u128 * percentage as u128 / total_percentage as u128) as i128;
                let current_amount = distribution_amounts.get_unchecked(i as u32);
                distribution_amounts.set(i as u32, current_amount + remainder_share);
                remainder_distributed += remainder_share;
            }
        }
        
        // Handle any final rounding by adding to the first eligible lender
        let final_remainder = remainder - remainder_distributed;
        if final_remainder > 0 && !distribution_amounts.is_empty() {
            let first_amount = distribution_amounts.get_unchecked(0);
            distribution_amounts.set(0, first_amount + final_remainder);
        }
    }
    
    // Execute transfers and update contributions
    for i in 0..eligible_lenders.len() {
        let (contribution_index, lender, _) = eligible_lenders.get_unchecked(i as u32);
        let distribution_amount = distribution_amounts.get_unchecked(i as u32);
        if distribution_amount > 0 {
            token_client.transfer(
                &env.current_contract_address(),
                &lender,
                &distribution_amount,
            );
            
            // Mark contribution as claimed
            let mut contribution = contributions.get_unchecked(contribution_index);
            contribution.claimed = true;
            contributions.set(contribution_index, contribution);
        }
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
