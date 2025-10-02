//! Tests for credit retirement functionality
//!
//! This module tests credit retirement operations, including:
//! - Valid retirement flows
//! - Double retirement prevention
//! - Retirement status tracking
//! - Authorization checks

#[cfg(test)]
mod tests {
    use crate::datatypes::RetirementStatus;
    use crate::interfaces::{CarbonContract, RetirementContract};
    use crate::tests::utils::*;
    use crate::EnvironmentalContract;

    #[test]
    fn test_retire_valid_credit() {
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

            // Retire credit
            let result = EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id.clone(),
                test_env.user1.clone(),
            );

            assert!(result.is_ok());

            // Verify status changed to Retired
            let status =
                EnvironmentalContract::get_credit_status(&test_env.env, credit_id).unwrap();

            assert_eq!(status, RetirementStatus::Retired(test_env.user1.clone()));
        });
    }

    #[test]
    fn test_retire_nonexistent_credit() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let result = EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id,
                test_env.user1.clone(),
            );

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_double_retirement_prevention() {
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

            // First retirement should succeed
            EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id.clone(),
                test_env.user1.clone(),
            )
            .unwrap();

            // Second retirement should fail
            let result = EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id,
                test_env.user2.clone(),
            );

            assert!(result.is_err());
        });
    }

    #[test]
    fn test_retirement_by_different_users() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Create 3 credits and retire each by different user
            for i in 1..=3 {
                let credit_id = create_credit_id(&test_env.env, i);

                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT,
                    standard_verification_method(&test_env.env),
                )
                .unwrap();

                let retiree = match i {
                    1 => test_env.user1.clone(),
                    2 => test_env.user2.clone(),
                    _ => test_env.admin.clone(),
                };

                EnvironmentalContract::retire_credit(&test_env.env, credit_id.clone(), retiree.clone())
                    .unwrap();

                let status =
                    EnvironmentalContract::get_credit_status(&test_env.env, credit_id).unwrap();

                assert_eq!(status, RetirementStatus::Retired(retiree));
            }
        });
    }

    #[test]
    fn test_get_retirement_status() {
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

            // Check initial status
            let status_before =
                EnvironmentalContract::get_retirement_status(&test_env.env, credit_id.clone())
                    .unwrap();
            assert_eq!(status_before, RetirementStatus::Available);

            // Retire credit
            EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id.clone(),
                test_env.user1.clone(),
            )
            .unwrap();

            // Check final status
            let status_after =
                EnvironmentalContract::get_retirement_status(&test_env.env, credit_id).unwrap();
            assert_eq!(
                status_after,
                RetirementStatus::Retired(test_env.user1.clone())
            );
        });
    }

    #[test]
    fn test_set_retirement_status() {
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

            // Set status to Retired (admin function)
            EnvironmentalContract::set_retirement_status(
                &test_env.env,
                credit_id.clone(),
                RetirementStatus::Retired(test_env.admin.clone()),
            )
            .unwrap();

            let status =
                EnvironmentalContract::get_retirement_status(&test_env.env, credit_id).unwrap();

            assert_eq!(
                status,
                RetirementStatus::Retired(test_env.admin.clone())
            );
        });
    }

    #[test]
    fn test_bulk_retirement() {
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

            // Retire all credits
            for i in 1..=20 {
                let credit_id = create_credit_id(&test_env.env, i);
                let result = EnvironmentalContract::retire_credit(
                    &test_env.env,
                    credit_id.clone(),
                    test_env.user1.clone(),
                );
                assert!(result.is_ok());

                let status =
                    EnvironmentalContract::get_retirement_status(&test_env.env, credit_id)
                        .unwrap();
                assert_eq!(
                    status,
                    RetirementStatus::Retired(test_env.user1.clone())
                );
            }
        });
    }

    #[test]
    fn test_retirement_status_persistence() {
        let test_env = setup_test();
        let credit_id = create_credit_id(&test_env.env, 1);
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue and retire credit
            EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id,
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            )
            .unwrap();

            EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id.clone(),
                test_env.user1.clone(),
            )
            .unwrap();

            // Check status multiple times to ensure persistence
            for _ in 0..5 {
                let status =
                    EnvironmentalContract::get_retirement_status(&test_env.env, credit_id.clone())
                        .unwrap();
                assert_eq!(
                    status,
                    RetirementStatus::Retired(test_env.user1.clone())
                );
            }
        });
    }
}
