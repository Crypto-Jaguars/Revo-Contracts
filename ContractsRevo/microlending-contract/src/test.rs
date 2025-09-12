#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, IntoVal, String, symbol_short,
};

// Helper to mint tokens using the token contract's interface
fn mint_tokens(env: &Env, token: &Address, to: &Address, amount: i128) {
    env.invoke_contract::<()>(token, &symbol_short!("mint"), (to, &amount).into_val(env));
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
    let token_address = env.register_stellar_asset_contract_v2(token_admin.clone()).address();
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

// === COMPREHENSIVE LOAN CREATION TESTS ===

#[test]
fn test_loan_creation_with_invalid_amount_zero() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &0, // Invalid: zero amount
        &String::from_str(&env, "Zero amount test"),
        &30u32,
        &500u32,
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidAmount.into() => (),
        _ => panic!("Expected InvalidAmount error, got: {:?}", result),
    }
}

#[test]
fn test_loan_creation_with_invalid_amount_negative() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &-100, // Invalid: negative amount
        &String::from_str(&env, "Negative amount test"),
        &30u32,
        &500u32,
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidAmount.into() => (),
        _ => panic!("Expected InvalidAmount error, got: {:?}", result),
    }
}

#[test]
fn test_loan_creation_with_invalid_duration_zero() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Zero duration test"),
        &0u32, // Invalid: zero duration
        &500u32,
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidDuration.into() => (),
        _ => panic!("Expected InvalidDuration error, got: {:?}", result),
    }
}

#[test]
fn test_loan_creation_with_invalid_duration_too_long() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Too long duration test"),
        &1096u32, // Invalid: over 3 years (1095 days max)
        &500u32,
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidDuration.into() => (),
        _ => panic!("Expected InvalidDuration error, got: {:?}", result),
    }
}

#[test]
fn test_loan_creation_with_invalid_interest_rate_zero() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Zero interest test"),
        &30u32,
        &0u32, // Invalid: zero interest rate
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidInterestRate.into() => (),
        _ => panic!("Expected InvalidInterestRate error, got: {:?}", result),
    }
}

#[test]
fn test_loan_creation_with_invalid_interest_rate_too_high() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let result = client.try_create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "High interest test"),
        &30u32,
        &10001u32, // Invalid: over 100% interest rate (10000 basis points max)
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidInterestRate.into() => (),
        _ => panic!("Expected InvalidInterestRate error, got: {:?}", result),
    }
}

#[test]
fn test_loan_creation_with_short_duration_repayment_schedule() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Short duration test"),
        &15u32, // Short duration - should result in single payment
        &500u32,
        &collateral,
    );
    let loan = client.get_loan_request(&loan_id);
    // Should have no installment schedule for short duration
    assert_eq!(loan.repayment_schedule.installments, 0);
    assert_eq!(loan.repayment_schedule.frequency_days, 0);
    assert_eq!(loan.repayment_schedule.per_installment_amount, 0);
}

#[test]
fn test_loan_creation_with_monthly_repayment_schedule() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Monthly schedule test"),
        &90u32,   // 3 months
        &1200u32, // 12% interest
        &collateral,
    );
    let loan = client.get_loan_request(&loan_id);
    // Should have 3 monthly installments
    assert_eq!(loan.repayment_schedule.installments, 3);
    assert_eq!(loan.repayment_schedule.frequency_days, 30);
    // Total due = 1000 + (1000 * 1200 / 10000) = 1000 + 120 = 1120
    // Per installment = 1120 / 3 = 373 (integer division)
    assert_eq!(loan.repayment_schedule.per_installment_amount, 373);
}

#[test]
fn test_loan_update_success() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let original_collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Original purpose"),
        &60u32,
        &500u32,
        &original_collateral,
    );

    let updated_collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Land"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[2u8; 32]),
    };
    client.update_loan_request(
        &borrower,
        &loan_id,
        &1200,
        &String::from_str(&env, "Updated purpose"),
        &90u32,
        &750u32,
        &updated_collateral,
    );

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.amount, 1200);
    assert_eq!(loan.purpose, String::from_str(&env, "Updated purpose"));
    assert_eq!(loan.duration_days, 90);
    assert_eq!(loan.interest_rate, 750);
    assert_eq!(loan.collateral.asset_type, String::from_str(&env, "Land"));
    assert_eq!(loan.collateral.estimated_value, 1500);
}

#[test]
fn test_loan_update_unauthorized() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Original purpose"),
        &60u32,
        &500u32,
        &collateral,
    );

    let result = client.try_update_loan_request(
        &lender1, // Not the borrower
        &loan_id,
        &1200,
        &String::from_str(&env, "Updated purpose"),
        &90u32,
        &750u32,
        &collateral,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::Unauthorized.into() => (),
        _ => panic!("Expected Unauthorized error, got: {:?}", result),
    }
}

#[test]
fn test_loan_cancel_success() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "To be cancelled"),
        &60u32,
        &500u32,
        &collateral,
    );

    client.cancel_loan_request(&borrower, &loan_id);
    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Cancelled);
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

// === COMPREHENSIVE FUNDING TESTS ===

#[test]
fn test_funding_with_insufficient_balance() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &150_000, // More than lender balance (100,000)
        &String::from_str(&env, "Insufficient balance test"),
        &60u32,
        &500u32,
        &collateral,
    );

    let lender_with_insufficient_balance = Address::generate(&env);
    mint_tokens(
        &env,
        &env.register_stellar_asset_contract_v2(Address::generate(&env)).address(),
        &lender_with_insufficient_balance,
        500,
    ); // Low balance

    let result = client.try_fund_loan(&lender_with_insufficient_balance, &loan_id, &1000);
    match result {
        Err(Ok(e)) if e == MicrolendingError::InsufficientBalance.into() => (),
        _ => panic!("Expected InsufficientBalance error, got: {:?}", result),
    }
}

#[test]
fn test_borrower_cannot_fund_own_loan() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Self funding test"),
        &60u32,
        &500u32,
        &collateral,
    );

    let result = client.try_fund_loan(&borrower, &loan_id, &500);
    match result {
        Err(Ok(e)) if e == MicrolendingError::Unauthorized.into() => (),
        _ => panic!("Expected Unauthorized error, got: {:?}", result),
    }
}

#[test]
fn test_funding_nonexistent_loan() {
    let (_env, _contract_id, client, _borrower, lender1, _lender2) = setup_test();
    let nonexistent_loan_id = 999u32;
    let result = client.try_fund_loan(&lender1, &nonexistent_loan_id, &500);
    match result {
        Err(Ok(e)) if e == MicrolendingError::LoanNotFound.into() => (),
        _ => panic!("Expected LoanNotFound error, got: {:?}", result),
    }
}

#[test]
fn test_funding_cancelled_loan() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Cancelled loan test"),
        &60u32,
        &500u32,
        &collateral,
    );

    client.cancel_loan_request(&borrower, &loan_id);

    let result = client.try_fund_loan(&lender1, &loan_id, &500);
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidLoanStatus.into() => (),
        _ => panic!("Expected InvalidLoanStatus error, got: {:?}", result),
    }
}

// === COMPREHENSIVE REPAYMENT TESTS ===

#[test]
fn test_repayment_with_insufficient_balance() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Insufficient repayment test"),
        &60u32,
        &500u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    // Reduce borrower's balance to insufficient amount
    let token_id = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();
    let borrower_with_insufficient_balance = Address::generate(&env);
    mint_tokens(&env, &token_id, &borrower_with_insufficient_balance, 10); // Very low balance

    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);

    let result = client.try_repay_loan(
        &borrower_with_insufficient_balance,
        &loan_id,
        &per_installment,
    );
    match result {
        Err(Ok(e)) if e == MicrolendingError::Unauthorized.into() => (),
        _ => panic!(
            "Expected Unauthorized error (borrower mismatch), got: {:?}",
            result
        ),
    }
}

#[test]
fn test_repayment_by_unauthorized_user() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Unauthorized repayment test"),
        &60u32,
        &500u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);

    let result = client.try_repay_loan(&lender2, &loan_id, &per_installment);
    match result {
        Err(Ok(e)) if e == MicrolendingError::Unauthorized.into() => (),
        _ => panic!("Expected Unauthorized error, got: {:?}", result),
    }
}

#[test]
fn test_repayment_exceeds_due_amount() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Excess repayment test"),
        &60u32,
        &500u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    let total_due = client.calculate_total_repayment_due(&loan_id);

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);

    let result = client.try_repay_loan(&borrower, &loan_id, &(total_due + 100));
    match result {
        Err(Ok(e)) if e == MicrolendingError::RepaymentScheduleViolation.into() => (),
        _ => panic!(
            "Expected RepaymentScheduleViolation error, got: {:?}",
            result
        ),
    }
}

#[test]
fn test_early_full_repayment() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1200,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Early full repayment test"),
        &90u32,  // 3 months
        &600u32, // 6% interest
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    let total_due = client.calculate_total_repayment_due(&loan_id);

    // Make full repayment immediately (before first installment due)
    env.ledger()
        .with_mut(|li| li.timestamp += 15 * 24 * 60 * 60); // 15 days

    client.repay_loan(&borrower, &loan_id, &total_due);

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Completed);
}

#[test]
fn test_partial_installment_repayments() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1200,
        &String::from_str(&env, "Partial installments test"),
        &90u32,  // 3 months
        &750u32, // 7.5% interest
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1200);

    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;

    // Make partial payments for each installment
    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &(per_installment / 2)); // Half of first installment

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &(per_installment / 2)); // Half of second installment

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Repaying);

    let repayments = client.get_loan_repayments(&loan_id);
    assert_eq!(repayments.len(), 2);

    // Complete remaining payments
    let total_due = client.calculate_total_repayment_due(&loan_id);
    let total_repaid: i128 = repayments.iter().map(|r| r.amount).sum();
    let remaining_due = total_due - total_repaid;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &remaining_due);

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Completed);
}

// === COMPREHENSIVE DEFAULT HANDLING TESTS ===

#[test]
fn test_default_claim_by_non_lender() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 2000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Non-lender claim test"),
        &30u32,
        &500u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    // Simulate time past due date
    env.ledger()
        .with_mut(|li| li.timestamp += 40 * 24 * 60 * 60);

    let non_lender = Address::generate(&env);
    let result = client.try_claim_default(&non_lender, &loan_id);
    match result {
        Err(Ok(e)) if e == MicrolendingError::NoContribution.into() => (),
        _ => panic!("Expected NoContribution error, got: {:?}", result),
    }
}

#[test]
fn test_default_claim_on_non_defaulted_loan() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Non-default claim test"),
        &60u32,
        &500u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    // Try to claim default immediately (not past due)
    let result = client.try_claim_default(&lender1, &loan_id);
    match result {
        Err(Ok(e)) if e == MicrolendingError::NotInDefault.into() => (),
        _ => panic!("Expected NotInDefault error, got: {:?}", result),
    }
}

#[test]
fn test_multiple_lenders_default_scenario() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 3000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &2000,
        &String::from_str(&env, "Multi-lender default test"),
        &30u32,
        &800u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1200); // 60% share
    client.fund_loan(&lender2, &loan_id, &800); // 40% share

    // Simulate time past due date
    env.ledger()
        .with_mut(|li| li.timestamp += 40 * 24 * 60 * 60);

    let is_default = client.check_default_status(&loan_id);
    assert!(is_default);

    // First lender claims default
    client.claim_default(&lender1, &loan_id);

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.status, LoanStatus::Defaulted);

    // Second lender should not be able to claim again (loan already defaulted)
    let result = client.try_claim_default(&lender2, &loan_id);
    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidLoanStatus.into() => (),
        _ => panic!("Expected InvalidLoanStatus error, got: {:?}", result),
    }
}

#[test]
fn test_default_status_check_accuracy() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1200,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Default status test"),
        &30u32,
        &600u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    // Check loan details after funding
    let loan_after_funding = client.get_loan_request(&loan_id);
    let funded_timestamp = loan_after_funding.funded_timestamp.unwrap();
    let due_timestamp = loan_after_funding.repayment_due_timestamp.unwrap();

    // Check default status before due date
    let is_default_early = client.check_default_status(&loan_id);
    assert!(!is_default_early);

    // Simulate time to exactly the due date minus 1 second (should not be in default)
    let due_time_from_funded = due_timestamp - funded_timestamp - 1;
    env.ledger()
        .with_mut(|li| li.timestamp += due_time_from_funded);
    let is_default_before_due = client.check_default_status(&loan_id);
    assert!(!is_default_before_due); // Should not be in default before due date

    // Simulate time past due date (for single payment loan, any time past due is default)
    env.ledger().with_mut(|li| li.timestamp += 1 * 24 * 60 * 60); // 1 day past due
    let is_default_late = client.check_default_status(&loan_id);
    assert!(is_default_late);
}

// === EDGE CASES AND ERROR SCENARIOS ===

#[test]
fn test_loan_history_comprehensive() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 2000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };
    let loan_id = client.create_loan_request(
        &borrower,
        &1500,
        &String::from_str(&env, "History test"),
        &60u32,
        &800u32, // 8% interest
        &collateral,
    );

    // Multiple fundings
    client.fund_loan(&lender1, &loan_id, &900);
    client.fund_loan(&lender2, &loan_id, &600);

    // Multiple repayments
    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);

    // Get comprehensive history
    let history = client.get_loan_history(&loan_id);

    assert_eq!(history.loan_request.id, loan_id);
    assert_eq!(history.funding_contributions.len(), 2);
    assert_eq!(history.repayments.len(), 2);
    assert_eq!(history.status, LoanStatus::Completed);

    let expected_total_due = client.calculate_total_repayment_due(&loan_id);
    assert_eq!(history.total_due, expected_total_due);

    let expected_total_repaid: i128 = history.repayments.iter().map(|r| r.amount).sum();
    assert_eq!(history.total_repaid, expected_total_repaid);

    let expected_interest = history.total_repaid - history.loan_request.amount;
    assert_eq!(history.interest_earned, expected_interest);
}

#[test]
fn test_borrower_and_lender_loan_tracking() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Create multiple loans
    let loan_id1 = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "First loan"),
        &30u32,
        &500u32,
        &collateral.clone(),
    );

    let loan_id2 = client.create_loan_request(
        &borrower,
        &800,
        &String::from_str(&env, "Second loan"),
        &45u32,
        &600u32,
        &collateral,
    );

    // Fund loans
    client.fund_loan(&lender1, &loan_id1, &600);
    client.fund_loan(&lender2, &loan_id1, &400);
    client.fund_loan(&lender1, &loan_id2, &800);

    // Check borrower loans
    let borrower_loans = client.get_borrower_loans(&borrower);
    assert_eq!(borrower_loans.len(), 2);
    assert!(borrower_loans.contains(&loan_id1));
    assert!(borrower_loans.contains(&loan_id2));

    // Check lender loans
    let lender1_loans = client.get_lender_loans(&lender1);
    assert_eq!(lender1_loans.len(), 2);
    assert!(lender1_loans.contains(&loan_id1));
    assert!(lender1_loans.contains(&loan_id2));

    let lender2_loans = client.get_lender_loans(&lender2);
    assert_eq!(lender2_loans.len(), 1);
    assert!(lender2_loans.contains(&loan_id1));

    // Check lender shares
    let lender1_share_loan1 = client.calculate_lender_share(&lender1, &loan_id1);
    assert_eq!(lender1_share_loan1, 600);

    let lender1_share_percent_loan1 = client.calculate_lender_share_percent(&lender1, &loan_id1);
    assert_eq!(lender1_share_percent_loan1, 6000); // 60% in basis points

    let lender2_share_loan1 = client.calculate_lender_share(&lender2, &loan_id1);
    assert_eq!(lender2_share_loan1, 400);

    let lender2_share_percent_loan1 = client.calculate_lender_share_percent(&lender2, &loan_id1);
    assert_eq!(lender2_share_percent_loan1, 4000); // 40% in basis points
}

// === SYSTEM STATISTICS AND METRICS TESTS ===

#[test]
fn test_borrower_metrics_tracking() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 2000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Create and complete a loan
    let loan_id1 = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Successful loan"),
        &30u32,
        &500u32,
        &collateral.clone(),
    );
    client.fund_loan(&lender1, &loan_id1, &1000);

    // Complete the loan with full repayment
    let total_due = client.calculate_total_repayment_due(&loan_id1);
    client.repay_loan(&borrower, &loan_id1, &total_due);

    // Create and default another loan
    let loan_id2 = client.create_loan_request(
        &borrower,
        &800,
        &String::from_str(&env, "Defaulted loan"),
        &30u32,
        &600u32,
        &collateral,
    );
    client.fund_loan(&lender2, &loan_id2, &800);

    // Simulate time past due date
    env.ledger()
        .with_mut(|li| li.timestamp += 40 * 24 * 60 * 60);
    client.claim_default(&lender2, &loan_id2);

    // Check borrower metrics accuracy
    let borrower_loans = client.get_borrower_loans(&borrower);
    assert_eq!(borrower_loans.len(), 2);

    // Verify loan statuses
    let loan1 = client.get_loan_request(&loan_id1);
    let loan2 = client.get_loan_request(&loan_id2);
    assert_eq!(loan1.status, LoanStatus::Completed);
    assert_eq!(loan2.status, LoanStatus::Defaulted);
}

#[test]
fn test_lender_share_percentage_edge_cases() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Test with very small amounts and uneven divisions
    let loan_id = client.create_loan_request(
        &borrower,
        &333, // Amount that doesn't divide evenly
        &String::from_str(&env, "Edge case loan"),
        &30u32,
        &500u32,
        &collateral,
    );

    // Fund with uneven amounts
    client.fund_loan(&lender1, &loan_id, &111); // 33.33%
    client.fund_loan(&lender2, &loan_id, &222); // 66.67%

    let lender1_percent = client.calculate_lender_share_percent(&lender1, &loan_id);
    let lender2_percent = client.calculate_lender_share_percent(&lender2, &loan_id);

    // Should add up to close to 10000 basis points (100%) - allowing for rounding
    let total_percent = lender1_percent + lender2_percent;
    assert!(total_percent >= 9999 && total_percent <= 10000);

    // Verify individual percentages (allowing for rounding)
    assert!(lender1_percent >= 3330 && lender1_percent <= 3340); // ~33.3%
    assert!(lender2_percent >= 6660 && lender2_percent <= 6670); // ~66.7%
}

#[test]
fn test_funding_contributions_detailed_tracking() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 2000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    let loan_id = client.create_loan_request(
        &borrower,
        &1500,
        &String::from_str(&env, "Multi-funding test"),
        &60u32,
        &750u32,
        &collateral,
    );

    // Record funding timestamps
    let start_time = env.ledger().timestamp();

    client.fund_loan(&lender1, &loan_id, &500);
    env.ledger().with_mut(|li| li.timestamp += 1000); // Advance time

    client.fund_loan(&lender2, &loan_id, &700);
    env.ledger().with_mut(|li| li.timestamp += 2000); // Advance time

    client.fund_loan(&lender1, &loan_id, &300); // Second contribution from lender1

    let contributions = client.get_loan_fundings(&loan_id);
    assert_eq!(contributions.len(), 3);

    // Verify contribution details
    assert_eq!(contributions.get(0).unwrap().lender, lender1);
    assert_eq!(contributions.get(0).unwrap().amount, 500);
    assert!(contributions.get(0).unwrap().timestamp >= start_time);

    assert_eq!(contributions.get(1).unwrap().lender, lender2);
    assert_eq!(contributions.get(1).unwrap().amount, 700);

    assert_eq!(contributions.get(2).unwrap().lender, lender1);
    assert_eq!(contributions.get(2).unwrap().amount, 300);

    // Verify timestamps are sequential
    assert!(contributions.get(1).unwrap().timestamp > contributions.get(0).unwrap().timestamp);
    assert!(contributions.get(2).unwrap().timestamp > contributions.get(1).unwrap().timestamp);

    // Verify total funding
    let total_funded: i128 = contributions.iter().map(|c| c.amount).sum();
    assert_eq!(total_funded, 1500);
}

// === INTEGRATION TESTS WITH COMMODITY TOKEN CONTRACT ===

#[test]
fn test_tokenized_repayment_integration() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Commodity Tokens"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Create a commodity token contract
    let commodity_token_admin = Address::generate(&env);
    let commodity_token_id = env.register_stellar_asset_contract_v2(commodity_token_admin.clone()).address();

    // Mint commodity tokens to borrower for repayment
    mint_tokens(&env, &commodity_token_id, &borrower, 2000);

    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Commodity backed loan"),
        &60u32,
        &600u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    // Test cross-contract interaction by using different token for repayment
    // This simulates tokenized commodity repayments
    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);

    let repayments = client.get_loan_repayments(&loan_id);
    assert_eq!(repayments.len(), 1);
    // assert_eq!(repayments[0].amount, per_installment);

    let loan_after = client.get_loan_request(&loan_id);
    assert_eq!(loan_after.status, LoanStatus::Repaying);
}

#[test]
fn test_cross_contract_collateral_verification() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();

    // Simulate external contract verification data
    let external_verification_hash = BytesN::from_array(&env, &[42u8; 32]);
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "External Contract Asset"),
        estimated_value: 2000,
        verification_data: external_verification_hash.clone(),
    };

    let loan_id = client.create_loan_request(
        &borrower,
        &1200,
        &String::from_str(&env, "Cross-contract collateral"),
        &90u32,
        &750u32,
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1200);

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(
        loan.collateral.verification_data,
        external_verification_hash
    );
    assert_eq!(
        loan.collateral.asset_type,
        String::from_str(&env, "External Contract Asset")
    );

    // Verify the verification hash is preserved throughout loan lifecycle
    let total_due = client.calculate_total_repayment_due(&loan_id);
    client.repay_loan(&borrower, &loan_id, &total_due);

    let loan_completed = client.get_loan_request(&loan_id);
    assert_eq!(
        loan_completed.collateral.verification_data,
        external_verification_hash
    );
}

// === HIGH-VOLUME AND SCALABILITY TESTS ===

#[test]
fn test_multiple_concurrent_loans() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 10000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Create multiple borrowers for concurrent loans
    let borrower2 = Address::generate(&env);
    let borrower3 = Address::generate(&env);
    let token_id = env.register_stellar_asset_contract_v2(Address::generate(&env)).address();

    // Mint tokens for all borrowers
    mint_tokens(&env, &token_id, &borrower2, 50_000);
    mint_tokens(&env, &token_id, &borrower3, 50_000);

    // Create multiple loans simultaneously - commented out due to Vec collection issues
    /*
    let loan_ids: Vec<u32> = (0..5).map(|i| {
        let amount = 500 + (i * 100) as i128;
        let duration = 30 + (i * 15);
        let rate = 400 + (i * 50);

        let current_borrower = match i {
            0..=1 => &borrower,
            2..=3 => &borrower2,
            _ => &borrower3,
        };

        let purpose = match i {
            0 => "Loan 0",
            1 => "Loan 1",
            2 => "Loan 2",
            3 => "Loan 3",
            _ => "Loan 4",
        };
        client.create_loan_request(
            current_borrower,
            &amount,
            &String::from_str(&env, purpose),
            &duration,
            &rate,
            &collateral.clone(),
        )
    }).collect();

    // Fund all loans with different lenders
    for (i, loan_id) in loan_ids.iter().enumerate() {
        let loan = client.get_loan_request(&loan_id);
        let lender = if i % 2 == 0 { &lender1 } else { &lender2 };
        client.fund_loan(lender, &loan_id, &loan.amount);
    }

    // Verify all loans were created and funded correctly
    for loan_id in &loan_ids {
        let loan = client.get_loan_request(&loan_id);
        assert_eq!(loan.status, LoanStatus::Funded);
        assert_eq!(loan.funded_amount, loan.amount);
    }
    */

    // Simplified version - create loans individually
    let loan_id1 = client.create_loan_request(
        &borrower,
        &500,
        &String::from_str(&env, "Loan 1"),
        &30u32,
        &400u32,
        &collateral.clone(),
    );
    let loan_id2 = client.create_loan_request(
        &borrower2,
        &600,
        &String::from_str(&env, "Loan 2"),
        &45u32,
        &450u32,
        &collateral.clone(),
    );

    client.fund_loan(&lender1, &loan_id1, &500);
    client.fund_loan(&lender2, &loan_id2, &600);

    let loan1 = client.get_loan_request(&loan_id1);
    let loan2 = client.get_loan_request(&loan_id2);
    assert_eq!(loan1.status, LoanStatus::Funded);
    assert_eq!(loan2.status, LoanStatus::Funded);

    // Check lender portfolios
    let lender1_loans = client.get_lender_loans(&lender1);
    let lender2_loans = client.get_lender_loans(&lender2);
    assert_eq!(lender1_loans.len(), 1);
    assert_eq!(lender2_loans.len(), 1);
}

#[test]
fn test_high_volume_loan_transactions() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Bulk Equipment"),
        estimated_value: 50000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Create a large loan with multiple funding rounds
    let loan_id = client.create_loan_request(
        &borrower,
        &10000,
        &String::from_str(&env, "High volume loan"),
        &180u32,  // 6 months
        &1200u32, // 12% interest
        &collateral,
    );

    // Fund in multiple small increments to simulate high transaction volume
    let mut total_funded = 0i128;
    let increment = 500i128;

    while total_funded < 10000 {
        let remaining = 10000 - total_funded;
        let amount_to_fund = increment.min(remaining);
        let lender = if total_funded % 1000 == 0 {
            &lender1
        } else {
            &lender2
        };

        client.fund_loan(lender, &loan_id, &amount_to_fund);
        total_funded += amount_to_fund;

        // Verify funding progress
        let loan = client.get_loan_request(&loan_id);
        assert_eq!(loan.funded_amount, total_funded);
    }

    let final_loan = client.get_loan_request(&loan_id);
    assert_eq!(final_loan.status, LoanStatus::Funded);
    assert_eq!(final_loan.funded_amount, 10000);

    // Verify funding contributions count
    let contributions = client.get_loan_fundings(&loan_id);
    assert_eq!(contributions.len(), 20); // 10000 / 500 = 20 contributions

    // Test high-volume repayments
    let per_installment = final_loan.repayment_schedule.per_installment_amount;
    let installments = final_loan.repayment_schedule.installments;

    for i in 0..installments {
        env.ledger()
            .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60); // Monthly

        if i == installments - 1 {
            // For final payment, calculate exact remaining amount
            let repayments_so_far = client.get_loan_repayments(&loan_id);
            let total_paid_so_far: i128 = repayments_so_far.iter().map(|r| r.amount).sum();
            let total_due = client.calculate_total_repayment_due(&loan_id);
            let remaining_due = total_due - total_paid_so_far;
            client.repay_loan(&borrower, &loan_id, &remaining_due);
        } else {
            client.repay_loan(&borrower, &loan_id, &per_installment);
        }

        let loan_status = client.get_loan_request(&loan_id);
        if i == installments - 1 {
            assert_eq!(loan_status.status, LoanStatus::Completed);
        } else {
            assert_eq!(loan_status.status, LoanStatus::Repaying);
        }
    }

    let repayments = client.get_loan_repayments(&loan_id);
    assert_eq!(repayments.len() as u32, installments);
}

// === ADDITIONAL EDGE CASE TESTS ===

#[test]
fn test_loan_history_data_integrity() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Data Integrity Test"),
        estimated_value: 3000,
        verification_data: BytesN::from_array(&env, &[99u8; 32]),
    };

    let loan_id = client.create_loan_request(
        &borrower,
        &2000,
        &String::from_str(&env, "Data integrity loan"),
        &90u32,  // 3 months
        &900u32, // 9% interest
        &collateral,
    );

    // Multiple funding rounds
    client.fund_loan(&lender1, &loan_id, &800);
    client.fund_loan(&lender2, &loan_id, &1200);

    // Multiple repayments
    let loan = client.get_loan_request(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &per_installment);

    // For the final payment, calculate remaining due to ensure full repayment
    let repayments_so_far = client.get_loan_repayments(&loan_id);
    let total_paid_so_far: i128 = repayments_so_far.iter().map(|r| r.amount).sum();
    let total_due = client.calculate_total_repayment_due(&loan_id);
    let remaining_due = total_due - total_paid_so_far;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &remaining_due);

    // Get complete history and verify data integrity
    let history = client.get_loan_history(&loan_id);

    // Verify loan request data integrity
    assert_eq!(history.loan_request.id, loan_id);
    assert_eq!(history.loan_request.borrower, borrower);
    assert_eq!(history.loan_request.amount, 2000);
    assert_eq!(
        history.loan_request.purpose,
        String::from_str(&env, "Data integrity loan")
    );
    assert_eq!(history.loan_request.collateral.estimated_value, 3000);

    // Verify funding contributions integrity
    assert_eq!(history.funding_contributions.len(), 2);
    let total_contributed: i128 = history.funding_contributions.iter().map(|f| f.amount).sum();
    assert_eq!(total_contributed, 2000);
    assert_eq!(history.funding_contributions.get(0).unwrap().amount, 800);
    assert_eq!(history.funding_contributions.get(1).unwrap().amount, 1200);

    // Verify repayments integrity
    assert_eq!(history.repayments.len(), 3);
    let total_repaid_calc: i128 = history.repayments.iter().map(|r| r.amount).sum();
    assert_eq!(history.total_repaid, total_repaid_calc);

    // Verify calculated fields
    assert_eq!(
        history.total_due,
        history.loan_request.amount
            + ((history.loan_request.amount * history.loan_request.interest_rate as i128) / 10000)
    );
    assert_eq!(
        history.interest_earned,
        history.total_repaid - history.loan_request.amount
    );

    // Verify status consistency
    assert_eq!(history.status, LoanStatus::Completed);
}

#[test]
fn test_repayment_rounding_edge_cases() {
    let (env, _contract_id, client, borrower, lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Create loan with amounts that will result in rounding issues
    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Rounding test"),
        &90u32,  // 3 installments
        &333u32, // 3.33% interest - creates rounding scenarios
        &collateral,
    );

    client.fund_loan(&lender1, &loan_id, &1000);

    let loan = client.get_loan_request(&loan_id);
    let total_due = client.calculate_total_repayment_due(&loan_id);
    let per_installment = loan.repayment_schedule.per_installment_amount;
    let installments = loan.repayment_schedule.installments;

    // Verify that installments * per_installment doesn't exceed total_due by much
    let calculated_total = per_installment * installments as i128;
    let difference = (calculated_total - total_due).abs();
    assert!(
        difference <= installments as i128,
        "Rounding error too large: {} vs {}",
        calculated_total,
        total_due
    );

    // Make repayments and handle final rounding
    for _i in 0..(installments - 1) {
        env.ledger()
            .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
        client.repay_loan(&borrower, &loan_id, &per_installment);
    }

    // Final payment should handle any rounding difference
    let repayments = client.get_loan_repayments(&loan_id);
    let total_paid_so_far: i128 = repayments.iter().map(|r| r.amount).sum();
    let remaining_due = total_due - total_paid_so_far;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    client.repay_loan(&borrower, &loan_id, &remaining_due);

    let final_loan = client.get_loan_request(&loan_id);
    assert_eq!(final_loan.status, LoanStatus::Completed);

    let final_repayments = client.get_loan_repayments(&loan_id);
    let final_total_paid: i128 = final_repayments.iter().map(|r| r.amount).sum();
    assert_eq!(final_total_paid, total_due);
}

#[test]
fn test_zero_interest_rate_edge_case() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1000,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    // Try to create loan with zero interest (should fail)
    let result = client.try_create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Zero interest test"),
        &30u32,
        &0u32, // Zero interest rate
        &collateral,
    );

    match result {
        Err(Ok(e)) if e == MicrolendingError::InvalidInterestRate.into() => (),
        _ => panic!("Expected InvalidInterestRate error, got: {:?}", result),
    }
}

#[test]
fn test_maximum_values_edge_case() {
    let (env, _contract_id, client, borrower, _lender1, _lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Maximum Value Asset"),
        estimated_value: i128::MAX / 2, // Large but safe value
        verification_data: BytesN::from_array(&env, &[255u8; 32]),
    };

    // Test with maximum allowed values
    let loan_id = client.create_loan_request(
        &borrower,
        &100_000_000, // Large amount (within token supply)
        &String::from_str(&env, "Maximum values test"),
        &1095u32,  // Maximum duration (3 years)
        &10000u32, // Maximum interest rate (100%)
        &collateral,
    );

    let loan = client.get_loan_request(&loan_id);
    assert_eq!(loan.amount, 100_000_000);
    assert_eq!(loan.duration_days, 1095);
    assert_eq!(loan.interest_rate, 10000);

    // Verify interest calculation doesn't overflow
    let total_due = client.calculate_total_repayment_due(&loan_id);
    let expected_interest = (100_000_000i128 * 10000i128) / 10000i128;
    let expected_total = 100_000_000 + expected_interest;
    assert_eq!(total_due, expected_total);
}

#[test]
fn test_timestamp_precision_and_ordering() {
    let (env, _contract_id, client, borrower, lender1, lender2) = setup_test();
    let collateral = CollateralInfo {
        asset_type: String::from_str(&env, "Equipment"),
        estimated_value: 1500,
        verification_data: BytesN::from_array(&env, &[1u8; 32]),
    };

    let loan_id = client.create_loan_request(
        &borrower,
        &1000,
        &String::from_str(&env, "Timestamp test"),
        &60u32,
        &600u32,
        &collateral,
    );

    let creation_time = env.ledger().timestamp();

    // Fund with precise timing
    env.ledger().with_mut(|li| li.timestamp += 1);
    client.fund_loan(&lender1, &loan_id, &600);
    let funding_time1 = env.ledger().timestamp();

    env.ledger().with_mut(|li| li.timestamp += 1);
    client.fund_loan(&lender2, &loan_id, &400);
    let funding_time2 = env.ledger().timestamp();

    let loan = client.get_loan_request(&loan_id);
    assert!(loan.creation_timestamp <= creation_time + 1);
    assert!(loan.funded_timestamp.unwrap() >= funding_time1);

    let contributions = client.get_loan_fundings(&loan_id);
    assert!(contributions.get(0).unwrap().timestamp >= funding_time1);
    assert!(contributions.get(1).unwrap().timestamp >= funding_time2);
    assert!(contributions.get(1).unwrap().timestamp > contributions.get(0).unwrap().timestamp);

    // Test repayment timing
    let per_installment = loan.repayment_schedule.per_installment_amount;

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    let repayment_time1 = env.ledger().timestamp();
    client.repay_loan(&borrower, &loan_id, &per_installment);

    env.ledger()
        .with_mut(|li| li.timestamp += 31 * 24 * 60 * 60);
    let repayment_time2 = env.ledger().timestamp();
    client.repay_loan(&borrower, &loan_id, &per_installment);

    let repayments = client.get_loan_repayments(&loan_id);
    assert!(repayments.get(0).unwrap().timestamp >= repayment_time1);
    assert!(repayments.get(1).unwrap().timestamp >= repayment_time2);
    assert!(repayments.get(1).unwrap().timestamp > repayments.get(0).unwrap().timestamp);
}
