//! Tests for compliance verification functionality
//!
//! This module tests compliance checks and verification, including:
//! - Compliance verification against standards
//! - Invalid compliance data handling
//! - Certificate integration validation
//! - Regulatory standard checks

#[cfg(test)]
mod tests {
    use crate::interfaces::{CarbonContract, VerificationContract};
    use crate::tests::utils::*;
    use crate::EnvironmentalContract;

    #[test]
    fn test_verify_valid_credit() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue credit with valid verification method
            EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            )
            .unwrap();

            // Verify the credit
            let is_verified =
                EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();

            assert!(is_verified);
        });
    }

    #[test]
    fn test_verify_nonexistent_credit() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Try to verify credit that doesn't exist
            let result = EnvironmentalContract::verify_credit(&test_env.env, credit_id);

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_verify_different_verification_standards() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        let standards = [
            ("Verified Carbon Standard", true),
            ("Gold Standard", true),
            ("American Carbon Registry", true),
            ("Climate Action Reserve", true),
            ("Verra VCS", true),
        ];

        test_env.env.as_contract(&test_env.contract_id, || {
            for (i, (standard, should_verify)) in standards.iter().enumerate() {
                let credit_id = create_credit_id(&test_env.env, (i + 1) as u8);

                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT,
                    custom_verification_method(&test_env.env, standard),
                )
                .unwrap();

                let is_verified =
                    EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();

                assert_eq!(is_verified, *should_verify);
            }
        });
    }

    #[test]
    fn test_compliance_check_after_issuance() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue credit
            EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            )
            .unwrap();

            // Immediate compliance check
            let is_compliant =
                EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();

            assert!(is_compliant);
        });
    }

    #[test]
    fn test_bulk_compliance_verification() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue 20 credits
            for i in 1..=20 {
                let credit_id = create_credit_id(&test_env.env, i);
                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id,
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT,
                    standard_verification_method(&test_env.env),
                )
                .unwrap();
            }

            // Verify all credits
            for i in 1..=20 {
                let credit_id = create_credit_id(&test_env.env, i);
                let is_verified =
                    EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();
                assert!(is_verified);
            }
        });
    }

    #[test]
    fn test_compliance_verification_different_projects() {
        let test_env = setup_test();

        test_env.env.as_contract(&test_env.contract_id, || {
            // Create credits across 5 different projects
            for project_num in 1..=5 {
                let project_id = create_project_id(&test_env.env, project_num);

                for credit_num in 1..=3 {
                    let credit_id = create_credit_id(
                        &test_env.env,
                        (project_num - 1) * 10 + credit_num,
                    );

                    EnvironmentalContract::issue_carbon_credit(
                        &test_env.env,
                        credit_id.clone(),
                        project_id.clone(),
                        STANDARD_CARBON_AMOUNT,
                        standard_verification_method(&test_env.env),
                    )
                    .unwrap();

                    // Verify each credit
                    let is_verified =
                        EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();
                    assert!(is_verified);
                }
            }
        });
    }

    #[test]
    fn test_verification_with_custom_standards() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        let custom_standards = [
            "ISO 14064",
            "PAS 2060",
            "GHG Protocol",
            "CDP",
            "SBTi",
        ];

        test_env.env.as_contract(&test_env.contract_id, || {
            for (i, standard) in custom_standards.iter().enumerate() {
                let credit_id = create_credit_id(&test_env.env, (i + 1) as u8);

                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT,
                    custom_verification_method(&test_env.env, standard),
                )
                .unwrap();

                let is_verified =
                    EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();

                // All non-empty verification methods should pass
                assert!(is_verified);
            }
        });
    }

    #[test]
    fn test_compliance_check_high_value_credits() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue credits with varying amounts
            let amounts = [
                STANDARD_CARBON_AMOUNT,
                LARGE_CARBON_AMOUNT,
                MAX_CARBON_AMOUNT / 2,
                MAX_CARBON_AMOUNT,
            ];

            for (i, amount) in amounts.iter().enumerate() {
                let credit_id = create_credit_id(&test_env.env, (i + 1) as u8);

                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    *amount,
                    standard_verification_method(&test_env.env),
                )
                .unwrap();

                let is_verified =
                    EnvironmentalContract::verify_credit(&test_env.env, credit_id).unwrap();

                assert!(is_verified);
            }
        });
    }

    #[test]
    fn test_verification_preserves_credit_data() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue credit
            EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id.clone(),
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            )
            .unwrap();

            // Get initial status
            let status_before =
                EnvironmentalContract::get_credit_status(&test_env.env, credit_id.clone())
                    .unwrap();

            // Perform verification
            EnvironmentalContract::verify_credit(&test_env.env, credit_id.clone()).unwrap();

            // Verify status unchanged
            let status_after =
                EnvironmentalContract::get_credit_status(&test_env.env, credit_id).unwrap();

            assert_eq!(status_before, status_after);
        });
    }
}
