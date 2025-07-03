#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, IntoVal, String, Symbol,
};

// Helper to mint tokens using the token contract's interface
fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    env.invoke_contract::<()>(token, &Symbol::short("mint"), (to, &amount).into_val(env));
}

// Helper to set up the environment and contract client
fn setup_test<'a>() -> (
    Env,
    Address,
    MicrolendingClient<'a>,
    Address,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    let _admin = Address::generate(&env);
    let borrower = Address::generate(&env);
    let lender1 = Address::generate(&env);
    let lender2 = Address::generate(&env);

    // Deploy a mock token contract and mint tokens
    let token_admin = Address::generate(&env);
    let token_address = env.register_stellar_asset_contract(token_admin.clone());
    mint_tokens(&env, &token_address, &borrower, 100_000);
    mint_tokens(&env, &token_address, &lender1, 100_000);
    mint_tokens(&env, &token_address, &lender2, 100_000);

    // Mint tokens to contract address for collateral claims
    let contract_id = env.register(Microlending, ());
    mint_tokens(&env, &token_address, &contract_id, 50_000);

    // Register and initialize your contract with the mock token address
    let client = MicrolendingClient::new(&env, &contract_id);
    client.initialize(&token_address);

    (env, contract_id, client, borrower, lender1, lender2)
}

#[test]
fn test_create_loan_request_success() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Buy seeds"),
        &90u32,
        &500u32, // 5% interest
        &collateral,
    );
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.borrower, borrower);
    assert_eq!(loan.amount, 1000);
    assert_eq!(loan.purpose, String::from_str(&env, "Buy seeds"));
    assert_eq!(loan.status, LoanStatus::Pending);
}

#[test]
fn test_create_loan_request_without_collateral() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, ""),
        estimated_value: 0,
        verification_data: BytesN::from_array(&env, &[0u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "No collateral"),
        &30u32,
        &500u32,
        &collateral,
    );
    // Should error with InvalidCollateral
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidCollateral.into() => (),
        _ => panic!("Expected InvalidCollateral error, got: {:?}", result),
    }
}

#[test]
fn test_funding_mechanism_multiple_lenders() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &2000,
        &String::from_str(&env, "Buy fertilizer"),
        &60u32,
        &400u32,
        &collateral,
    );
    // Fund part of the loan with lender1
    client.fund_loan(&lender1, &loan_id, &1200);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.funded_amount, 1200);
    assert_eq!(loan.status, LoanStatus::Pending);
    // Fund the rest with lender2
    client.fund_loan(&lender2, &loan_id, &800);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.funded_amount, 2000);
    assert_eq!(loan.status, LoanStatus::Funded);
    // Attempt to overfund (should only accept up to remaining amount)
    let result = client.try_fund_loan(&lender1, &loan_id, &100);
    match result {
        Err(Ok(e)) if e == MicrolendingError::LoanFullyFunded.into() => (),
        _ => panic!("Expected LoanFullyFunded error, got: {:?}", result),
    }
}

#[test]
fn test_repayment_flow_and_completion() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Harvest"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[2u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Repayment test"),
        &60u32,
        &1000u32, // 10% interest
        &collateral,
    );
    client.fund_loan(&lender1, &loan_id, &600);
    client.fund_loan(&lender2, &loan_id, &400);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Funded);
    // Simulate time and repayments
    let total_due = client.calculate_total_repayment_due(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;
    // First repayment (simulate time forward)
    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Repaying);
    // Second repayment (simulate time forward)
    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Completed);
    // Check repayments
    let repayments = client.get_loan_repayments(&loan_id);
    let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
    assert_eq!(total_repaid, total_due);
}

#[test]
fn test_default_and_collateral_claim() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Land"),
        estimated_value: 2000,
        verification_data: BytesN::from_array(&env, &[3u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Default test"),
        &30u32,
        &1000u32,
        &collateral,
    );
    client.fund_loan(&lender1, &loan_id, &1000);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Funded);
    // Simulate time past due date (plus grace period)
    env.ledger()
        .with_mut(|li| li.timestamp += 40 * 24 * 60 * 60);
    // Should now be in default
    let is_default = client.check_default_status(&loan_id);
    assert!(is_default);
    // Lender claims collateral
    client.claim_default(&lender1, &loan_id);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Defaulted);
}

#[test]
fn test_loan_history_and_tracking() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[4u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1500,
        &String::from_str(&env, "History test"),
        &60u32,
        &500u32,
        &collateral,
    );
    client.fund_loan(&lender1, &loan_id, &1000);
    client.fund_loan(&lender2, &loan_id, &500);
    // Repay fully
    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;
    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);
    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);
    // If still not completed, repay the remaining due
    let loan = client.get_loan_request(&loan_id);
    if loan.status == LoanStatus::Repaying {
        let total_due = client.calculate_total_repayment_due(&loan_id);
        let repayments = client.get_loan_repayments(&loan_id);
        let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
        let remaining_due = total_due - total_repaid;
        if remaining_due > 0 {
            client.repay_loan(&borrower, &loan_id, &remaining_due);
        }
    }
    let _loan = client.get_loan_request(&loan_id);
    // Check borrower loans
    let borrower_loans = client.get_borrower_loans(&borrower);
    assert!(borrower_loans.contains(&loan_id));
    // Check lender loans
    let lender1_loans = client.get_lender_loans(&lender1);
    assert!(lender1_loans.contains(&loan_id));
    let lender2_loans = client.get_lender_loans(&lender2);
    assert!(lender2_loans.contains(&loan_id));
    // Check loan history
    let history = client.get_loan_history(&loan_id);
    assert_eq!(history.loan_request.id, loan_id);
    assert_eq!(history.status, LoanStatus::Completed);
}

#[test]
fn test_verification_data_integrity() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let verification_hash = BytesN::from_array(&env, &[5u8; 32]);
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: verification_hash.clone(),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Verification test"),
        &30u32,
        &500u32,
        &collateral,
    );
    let loan = client.get_loan_request(&loan_id);
    // Verify the hash is stored correctly
    assert_eq!(loan.collateral.verification_data, verification_hash);
}

#[test]
fn test_attempt_overfund_completed_loan() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[6u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Overfund test"),
        &60u32,
        &500u32,
        &collateral,
    );
    // Fund the loan completely
    client.fund_loan(&lender1, &loan_id, &1000);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Funded);
    // Attempt to overfund a completed loan
    let result = client.try_fund_loan(&lender2, &loan_id, &100);
    match result {
        Err(Ok(e)) if e == MicrolendingError::LoanFullyFunded.into() => (),
        _ => panic!("Expected LoanFullyFunded error, got: {:?}", result),
    }
}
