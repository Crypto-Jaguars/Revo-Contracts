#![no_std]
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, String, Symbol, Vec};

mod claim;
mod datatypes;
mod fund;
mod repay;
mod request;

pub use claim::*;
pub use datatypes::*;
pub use fund::*;
pub use repay::*;
pub use request::*;

#[contract]
pub struct Microlending;

#[contractimpl]
impl Microlending {
    // Initialize the contract
    pub fn initialize(env: Env, token_address: Address) {
        // Check if already initialized
        if env.storage().persistent().has(&DataKey::AssetCode) {
            panic_with_error!(env, MicrolendingError::AlreadyInitialized);
        }

        // Store token address
        env.storage()
            .persistent()
            .set(&DataKey::AssetCode, &token_address);

        // Emit initialization event
        env.events()
            .publish((Symbol::new(&env, "initialized"),), (token_address,));
    }

    // Loan request functions
    pub fn create_loan_request(
        env: Env,
        borrower: Address,
        amount: i128,
        purpose: String,
        duration_days: u32,
        interest_rate: u32,
        collateral: CollateralInfo,
    ) -> u32 {
        request::create_loan_request(
            &env,
            borrower,
            amount,
            purpose,
            duration_days,
            interest_rate,
            collateral,
        )
    }

    pub fn get_loan_request(env: Env, loan_id: u32) -> LoanRequest {
        request::get_loan_request(&env, loan_id)
    }

    pub fn get_loan_history(env: &Env, loan_id: u32) -> LoanHistory {
        let loan = get_loan_request(env, loan_id);
        let fundings = get_loan_fundings(env, loan_id);
        let repayments = get_loan_repayments(env, loan_id);
        let total_due = calculate_total_repayment_due(&loan);
        let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
        let interest_earned = if total_repaid > loan.amount {
            total_repaid - loan.amount
        } else {
            0
        };

        LoanHistory {
            loan_request: loan.clone(),
            funding_contributions: fundings,
            repayments,
            total_due,
            total_repaid,
            interest_earned,
            status: if check_default_status(env, &loan) {
                LoanStatus::Defaulted
            } else {
                loan.status
            },
        }
    }

    pub fn get_borrower_loans(env: Env, borrower: Address) -> Vec<u32> {
        request::get_borrower_loans(&env, borrower)
    }

    pub fn cancel_loan_request(env: Env, borrower: Address, loan_id: u32) {
        request::cancel_loan_request(&env, borrower, loan_id)
    }

    pub fn update_loan_request(
        env: Env,
        borrower: Address,
        loan_id: u32,
        amount: i128,
        purpose: String,
        duration_days: u32,
        interest_rate: u32,
        collateral: CollateralInfo,
    ) {
        request::update_loan_request(
            &env,
            borrower,
            loan_id,
            amount,
            purpose,
            duration_days,
            interest_rate,
            collateral,
        )
    }

    // Funding functions
    pub fn fund_loan(env: Env, lender: Address, loan_id: u32, amount: i128) {
        fund::fund_loan(&env, lender, loan_id, amount)
    }

    pub fn get_loan_fundings(env: Env, loan_id: u32) -> Vec<FundingContribution> {
        fund::get_loan_fundings(&env, loan_id)
    }

    pub fn get_lender_loans(env: Env, lender: Address) -> Vec<u32> {
        fund::get_lender_loans(&env, lender)
    }

    pub fn calculate_lender_share(env: Env, lender: Address, loan_id: u32) -> i128 {
        fund::calculate_lender_share(&env, lender, loan_id)
    }

    pub fn calculate_lender_share_percent(env: Env, lender: Address, loan_id: u32) -> u32 {
        fund::calculate_lender_share_percentage(&env, lender, loan_id)
    }

    // Repayment functions
    pub fn repay_loan(env: Env, borrower: Address, loan_id: u32, amount: i128) {
        repay::repay_loan(&env, borrower, loan_id, amount)
    }

    pub fn get_loan_repayments(env: Env, loan_id: u32) -> Vec<Repayment> {
        repay::get_loan_repayments(&env, loan_id)
    }

    pub fn calculate_total_repayment_due(env: Env, loan_id: u32) -> i128 {
        let loan = request::get_loan_request(&env, loan_id);
        repay::calculate_total_repayment_due(&loan)
    }

    // Default claim functions
    pub fn claim_default(env: Env, lender: Address, loan_id: u32) {
        claim::claim_default(&env, lender, loan_id)
    }

    pub fn check_default_status(env: Env, loan_id: u32) -> bool {
        let loan = request::get_loan_request(&env, loan_id);
        claim::check_default_status(&env, &loan)
    }
}
