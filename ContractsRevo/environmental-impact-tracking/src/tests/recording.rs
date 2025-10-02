//! Tests for environmental impact data recording functionality
//!
//! This module tests the recording of carbon credits, including:
//! - Valid credit issuance
//! - Invalid source validation
//! - Data integrity checks
//! - Authorization and access control
//! - Edge cases and boundary conditions

#[cfg(test)]
mod tests {
    use soroban_sdk::{BytesN, String};

    use crate::datatypes::{CarbonCredit, DataKey, RetirementStatus};
    use crate::interfaces::CarbonContract;
    use crate::tests::utils::*;
    use crate::EnvironmentalContract;

    /// Test successful recording of a carbon credit
    #[test]
    fn test_record_valid_carbon_credit() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id.clone(),
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            );

            assert!(result.is_ok());

            // Verify credit was stored with correct data
            let stored_credit: CarbonCredit = test_env
                .env
                .storage()
                .persistent()
                .get(&DataKey::Credit(credit_id.clone()))
                .unwrap();

            assert_eq!(stored_credit.project_id, project_id);
            assert_eq!(stored_credit.carbon_amount, STANDARD_CARBON_AMOUNT);
            assert_eq!(
                stored_credit.retirement_status,
                RetirementStatus::Available
            );
        });
    }

    /// Test recording with zero carbon amount (should fail)
    #[test]
    fn test_record_zero_carbon_amount() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id,
                project_id,
                0,
                standard_verification_method(&test_env.env),
            );

            assert!(result.is_err());
        });
    }

    /// Test recording with excessive carbon amount (should fail)
    #[test]
    fn test_record_excessive_carbon_amount() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id,
                project_id,
                MAX_CARBON_AMOUNT + 1,
                standard_verification_method(&test_env.env),
            );

            assert!(result.is_err());
        });
    }

    /// Test recording with maximum allowed carbon amount (should succeed)
    #[test]
    fn test_record_maximum_carbon_amount() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id,
                MAX_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            );

            assert!(result.is_ok());

            let stored_credit: CarbonCredit = test_env
                .env
                .storage()
                .persistent()
                .get(&DataKey::Credit(credit_id))
                .unwrap();

            assert_eq!(stored_credit.carbon_amount, MAX_CARBON_AMOUNT);
        });
    }

    /// Test recording with empty verification method (should fail)
    #[test]
    fn test_record_empty_verification_method() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id,
                project_id,
                STANDARD_CARBON_AMOUNT,
                String::from_str(&test_env.env, ""),
            );

            assert!(result.is_err());
        });
    }

    /// Test recording with invalid (zero) credit ID (should fail)
    #[test]
    fn test_record_invalid_credit_id() {
        let test_env = setup_test();
        let invalid_credit_id = BytesN::from_array(&test_env.env, &[0u8; 32]);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                invalid_credit_id,
                project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            );

            assert!(result.is_err());
        });
    }

    /// Test recording with invalid (zero) project ID (should fail)
    #[test]
    fn test_record_invalid_project_id() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let invalid_project_id = BytesN::from_array(&test_env.env, &[0u8; 32]);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id,
                invalid_project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            );

            assert!(result.is_err());
        });
    }

    /// Test duplicate credit ID (should fail)
    #[test]
    fn test_record_duplicate_credit_id() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // First issuance should succeed
            let result1 = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id.clone(),
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            );
            assert!(result1.is_ok());

            // Second issuance with same credit_id should fail
            let result2 = EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id,
                project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            );
            assert!(result2.is_err());
        });
    }

    /// Test recording multiple credits for same project
    #[test]
    fn test_record_multiple_credits_same_project() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            for i in 1..=5 {
                let credit_id = create_credit_id(&test_env.env, i);
                let result = EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id,
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT * i as u32,
                    standard_verification_method(&test_env.env),
                );
                assert!(result.is_ok());
            }

            // Verify project has all credits
            let project_credits = test_env
                .env
                .storage()
                .persistent()
                .get::<_, soroban_sdk::Vec<BytesN<32>>>(&DataKey::ProjectCredits(
                    project_id,
                ))
                .unwrap();

            assert_eq!(project_credits.len(), 5);
        });
    }

    /// Test recording with different verification methods
    #[test]
    fn test_record_different_verification_methods() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        let verification_methods = [
            "Verified Carbon Standard",
            "Gold Standard",
            "American Carbon Registry",
            "Climate Action Reserve",
        ];

        test_env.env.as_contract(&test_env.contract_id, || {
            for (i, method) in verification_methods.iter().enumerate() {
                let credit_id = create_credit_id(&test_env.env, (i + 1) as u8);
                let result = EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT,
                    custom_verification_method(&test_env.env, method),
                );

                assert!(result.is_ok());

                // Verify verification method was stored correctly
                let stored_credit: CarbonCredit = test_env
                    .env
                    .storage()
                    .persistent()
                    .get(&DataKey::Credit(credit_id))
                    .unwrap();

                assert_eq!(
                    stored_credit.verification_method,
                    custom_verification_method(&test_env.env, method)
                );
            }
        });
    }

    /// Test that issuance date is correctly recorded
    #[test]
    fn test_issuance_date_recording() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let timestamp_before = test_env.env.ledger().timestamp();

            EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            )
            .unwrap();

            let stored_credit: CarbonCredit = test_env
                .env
                .storage()
                .persistent()
                .get(&DataKey::Credit(credit_id))
                .unwrap();

            let timestamp_after = test_env.env.ledger().timestamp();

            assert!(stored_credit.issuance_date >= timestamp_before);
            assert!(stored_credit.issuance_date <= timestamp_after);
        });
    }

    /// Test high volume credit recording (scalability test)
    #[test]
    fn test_high_volume_credit_recording() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Record 50 credits
            for i in 1..=50 {
                let credit_id = create_credit_id(&test_env.env, i);
                let result = EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id,
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT,
                    standard_verification_method(&test_env.env),
                );
                assert!(result.is_ok());
            }

            // Verify all credits were stored
            let project_credits = test_env
                .env
                .storage()
                .persistent()
                .get::<_, soroban_sdk::Vec<BytesN<32>>>(&DataKey::ProjectCredits(
                    project_id,
                ))
                .unwrap();

            assert_eq!(project_credits.len(), 50);
        });
    }
}
