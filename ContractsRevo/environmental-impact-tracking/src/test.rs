#![cfg(test)]
use core::result;

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String, Vec
};

use crate::{
    datatypes::{CarbonCredit, DataKey, RetirementStatus},
    EnvironmentalContract,
};
use crate::interfaces::{CarbonContract, RetirementContract, ReportingContract};

// Helper function to set up test environment
fn setup_test() -> (Env, Address, Address) {
    let env = Env::default();
    let contract_id = env.register(EnvironmentalContract, ());
    let admin = Address::generate(&env);

    (env, contract_id ,admin)
}

// Helper function to create a valid credit ID
fn create_credit_id(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[1u8; 32])
}

// Helper function to create a valid project ID
fn create_project_id(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[2u8; 32])
}

#[test]
fn test_issue_carbon_credit() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1000u32;
    let verification_method = String::from_str(&env, "Verified Carbon Standard");

    env.as_contract(&contract_id, || {
        let _ = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id.clone(),
            project_id.clone(),
            carbon_amount,
            verification_method.clone(),
        );

        // Verify credit was stored correctly
        let stored_credit: CarbonCredit = env
            .storage()
            .persistent()
            .get(&DataKey::Credit(credit_id.clone()))
            .unwrap();

        assert_eq!(stored_credit.project_id, project_id);
        assert_eq!(stored_credit.carbon_amount, carbon_amount);
        assert_eq!(stored_credit.verification_method, verification_method);
        assert_eq!(stored_credit.retirement_status, RetirementStatus::Available);

        // Verify credit was added to project's list
        let project_credits: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&DataKey::ProjectCredits(project_id))
            .unwrap();

        assert_eq!(project_credits.len(), 1);
        assert_eq!(project_credits.get(0).unwrap(), credit_id);
    });
}

#[test]
fn test_issue_carbon_credit_zero_amount() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 0u32;
    let verification_method = String::from_str(&env, "Verified Carbon Standard");

    env.as_contract(&contract_id, || {
        let result = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id,
            project_id,
            carbon_amount,
            verification_method,
        );
        assert!(result.is_err());
    });
}

#[test]
fn test_issue_carbon_credit_invalid_amount() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1_000_000_001u32; // Exceeds MAX_AMOUNT
    let verification_method = String::from_str(&env, "Verified Carbon Standard");

    env.as_contract(&contract_id, || {
        let result = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id,
            project_id,
            carbon_amount,
            verification_method,
        );
        assert!(result.is_err());
    });
}

#[test]
fn test_issue_carbon_credit_empty_verification() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1000u32;
    let verification_method = String::from_str(&env, "");

    env.as_contract(&contract_id, || {
        let result = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id,
            project_id,
            carbon_amount,
            verification_method,
        );
        assert!(result.is_err());
    });
}

#[test]
fn test_issue_carbon_credit_duplicate() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1000u32;
    let verification_method = String::from_str(&env, "Verified Carbon Standard");

    env.as_contract(&contract_id, || {
        // First issuance
        let _ = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id.clone(),
            project_id.clone(),
            carbon_amount,
            verification_method.clone(),
        );

        // Second issuance with same credit_id
        let result = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id,
            project_id,
            carbon_amount,
            verification_method,
        );
        assert!(result.is_err());
    });
}

#[test]
fn test_retire_credit() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1000u32;
    let verification_method = String::from_str(&env, "Verified Carbon Standard");
    let retiree = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Issue credit first
        let _ = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id.clone(),
            project_id,
            carbon_amount,
            verification_method,
        );

        // Retire the credit
        let _ = EnvironmentalContract::retire_credit(&env, credit_id.clone(), retiree.clone());

        // Verify retirement status
        let stored_credit: CarbonCredit = env
            .storage()
            .persistent()
            .get(&DataKey::Credit(credit_id))
            .unwrap();

        assert_eq!(
            stored_credit.retirement_status,
            RetirementStatus::Retired(retiree)
        );
    });
}

#[test]
fn test_retire_credit_twice() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1000u32;
    let verification_method = String::from_str(&env, "Verified Carbon Standard");
    let retiree = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Issue credit
        let _ = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id.clone(),
            project_id,
            carbon_amount,
            verification_method,
        );

        // First retirement
        let _ = EnvironmentalContract::retire_credit(&env, credit_id.clone(), retiree.clone());

        // Second retirement attempt
        let result = EnvironmentalContract::retire_credit(&env, credit_id, retiree);
        assert!(result.is_err());
    });
}

#[test]
fn test_retire_nonexistent_credit() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let retiree = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let result = EnvironmentalContract::retire_credit(&env, credit_id, retiree);
        assert!(result.is_err());
    });
}

#[test]
fn test_generate_impact_report() {
    let (env, contract_id, _) = setup_test();
    let project_id = create_project_id(&env);
    let verification_method = String::from_str(&env, "Verified Carbon Standard");
    let retiree = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Issue and retire multiple credits
        for i in 0..3 {
            let credit_id = BytesN::from_array(&env, &[i as u8 + 1; 32]);
            let carbon_amount = 1000u32 * (i + 1);

            let _ = EnvironmentalContract::issue_carbon_credit(
                &env,
                credit_id.clone(),
                project_id.clone(),
                carbon_amount,
                verification_method.clone(),
            );
            let _ = EnvironmentalContract::retire_credit(&env, credit_id, retiree.clone());
        }

        // Generate impact report
        let total_offset = EnvironmentalContract::generate_impact_report(&env, project_id);

        // Verify total offset (1000 + 2000 + 3000 = 6000)
        assert_eq!(total_offset, 6000);
    });
}

#[test]
fn test_generate_impact_report_empty_project() {
    let (env, contract_id, _) = setup_test();
    let project_id = create_project_id(&env);

    env.as_contract(&contract_id, || {
        // Generate impact report for empty project
        let total_offset = EnvironmentalContract::generate_impact_report(&env, project_id);

        assert_eq!(total_offset, 0);
    });
}

#[test]
fn test_generate_impact_report_mixed_status() {
    let (env, contract_id, _) = setup_test();
    let project_id = create_project_id(&env);
    let verification_method = String::from_str(&env, "Verified Carbon Standard");
    let retiree = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Issue 3 credits
        for i in 0..3 {
            let credit_id = BytesN::from_array(&env, &[i as u8 + 1; 32]);
            let carbon_amount = 1000u32 * (i + 1);

            let _ = EnvironmentalContract::issue_carbon_credit(
                &env,
                credit_id.clone(),
                project_id.clone(),
                carbon_amount,
                verification_method.clone(),
            );
        }

        // Retire only the first two credits
        for i in 0..2 {
            let credit_id = BytesN::from_array(&env, &[i as u8 + 1; 32]);
            let _ = EnvironmentalContract::retire_credit(&env, credit_id, retiree.clone());
        }

        // Generate impact report
        let total_offset = EnvironmentalContract::generate_impact_report(&env, project_id);

        // Verify total offset (1000 + 2000 = 3000)
        assert_eq!(total_offset, 3000);
    });
}

#[test]
fn test_get_credit_status() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);
    let project_id = create_project_id(&env);
    let carbon_amount = 1000u32;
    let verification_method = String::from_str(&env, "Verified Carbon Standard");
    let retiree = Address::generate(&env);

    env.as_contract(&contract_id, || {
        // Issue credit
        let _ = EnvironmentalContract::issue_carbon_credit(
            &env,
            credit_id.clone(),
            project_id,
            carbon_amount,
            verification_method,
        );

        // Check initial status
        let initial_status = EnvironmentalContract::get_credit_status(&env, credit_id.clone()).unwrap();
        assert_eq!(initial_status, RetirementStatus::Available);

        // Retire credit
        let _ = EnvironmentalContract::retire_credit(&env, credit_id.clone(), retiree.clone());

        // Check final status
        let final_status = EnvironmentalContract::get_credit_status(&env, credit_id).unwrap();
        assert_eq!(final_status, RetirementStatus::Retired(retiree));
    });
}

#[test]
fn test_get_credit_status_nonexistent() {
    let (env, contract_id, _) = setup_test();
    let credit_id = create_credit_id(&env);

    env.as_contract(&contract_id, || {
        let result = EnvironmentalContract::get_credit_status(&env, credit_id);
        assert!(result.is_err());
    });
}

#[test]
fn test_list_credits_by_project() {
    let (env, contract_id, _) = setup_test();
    let project_id = create_project_id(&env);
    let verification_method = String::from_str(&env, "Verified Carbon Standard");

    env.as_contract(&contract_id, || {
        // Issue multiple credits
        for i in 0..3 {
            let credit_id = BytesN::from_array(&env, &[i as u8 + 1; 32]);
            let carbon_amount = 1000u32 * (i + 1);

            let _ = EnvironmentalContract::issue_carbon_credit(
                &env,
                credit_id.clone(),
                project_id.clone(),
                carbon_amount,
                verification_method.clone(),
            );
        }

        // List credits
        let credits = EnvironmentalContract::list_credits_by_project(&env, project_id).unwrap();

        // Verify all credits are listed
        assert_eq!(credits.len(), 3);
        for i in 0..3 {
            let expected_id = BytesN::from_array(&env, &[i as u8 + 1; 32]);
            assert_eq!(credits.get(i).unwrap(), expected_id);
        }
    });
}

#[test]
fn test_list_credits_by_project_empty() {
    let (env, contract_id, _) = setup_test();
    let project_id = create_project_id(&env);

    env.as_contract(&contract_id, || {
        // List credits for empty project
        let credits = EnvironmentalContract::list_credits_by_project(&env, project_id).unwrap();

        assert_eq!(credits.len(), 0);
    });
}
