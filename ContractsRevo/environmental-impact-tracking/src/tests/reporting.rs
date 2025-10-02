//! Tests for impact report generation functionality
//!
//! This module tests the generation of environmental impact reports, including:
//! - Accurate report calculation
//! - Report accessibility
//! - Missing data handling
//! - Multiple project reporting
//! - Report data integrity

#[cfg(test)]
mod tests {
    use crate::interfaces::{CarbonContract, ReportingContract, RetirementContract};
    use crate::tests::utils::*;
    use crate::EnvironmentalContract;

    #[test]
    fn test_generate_report_single_retired_credit() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);
        let credit_id = create_credit_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue and retire credit
            EnvironmentalContract::issue_carbon_credit(
                &test_env.env,
                credit_id.clone(),
                project_id.clone(),
                STANDARD_CARBON_AMOUNT,
                standard_verification_method(&test_env.env),
            )
            .unwrap();

            EnvironmentalContract::retire_credit(
                &test_env.env,
                credit_id,
                test_env.user1.clone(),
            )
            .unwrap();

            // Generate report
            let total_offset =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

            assert_eq!(total_offset, STANDARD_CARBON_AMOUNT);
        });
    }

    #[test]
    fn test_generate_report_multiple_retired_credits() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let mut expected_total = 0u32;

            // Issue and retire 5 credits
            for i in 1..=5 {
                let credit_id = create_credit_id(&test_env.env, i);
                let amount = STANDARD_CARBON_AMOUNT * i as u32;
                expected_total += amount;

                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    amount,
                    standard_verification_method(&test_env.env),
                )
                .unwrap();

                EnvironmentalContract::retire_credit(
                    &test_env.env,
                    credit_id,
                    test_env.user1.clone(),
                )
                .unwrap();
            }

            let total_offset =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

            assert_eq!(total_offset, expected_total);
        });
    }

    #[test]
    fn test_generate_report_empty_project() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            let total_offset =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

            assert_eq!(total_offset, 0);
        });
    }

    #[test]
    fn test_generate_report_only_available_credits() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue credits but don't retire them
            for i in 1..=3 {
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

            // Report should be 0 since no credits are retired
            let total_offset =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

            assert_eq!(total_offset, 0);
        });
    }

    #[test]
    fn test_generate_report_mixed_status_credits() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue 5 credits
            for i in 1..=5 {
                let credit_id = create_credit_id(&test_env.env, i);
                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id,
                    project_id.clone(),
                    STANDARD_CARBON_AMOUNT * i as u32,
                    standard_verification_method(&test_env.env),
                )
                .unwrap();
            }

            // Retire only first 3 credits
            for i in 1..=3 {
                let credit_id = create_credit_id(&test_env.env, i);
                EnvironmentalContract::retire_credit(
                    &test_env.env,
                    credit_id,
                    test_env.user1.clone(),
                )
                .unwrap();
            }

            let total_offset =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

            // Should only count retired credits: 1000 + 2000 + 3000 = 6000
            let expected = STANDARD_CARBON_AMOUNT + (STANDARD_CARBON_AMOUNT * 2) + (STANDARD_CARBON_AMOUNT * 3);
            assert_eq!(total_offset, expected);
        });
    }

    #[test]
    fn test_generate_report_multiple_projects() {
        let test_env = setup_test();

        test_env.env.as_contract(&test_env.contract_id, || {
            // Create 3 projects with different credit counts
            for project_num in 1..=3 {
                let project_id = create_project_id(&test_env.env, project_num);

                for credit_num in 1..=project_num {
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

                    EnvironmentalContract::retire_credit(
                        &test_env.env,
                        credit_id,
                        test_env.user1.clone(),
                    )
                    .unwrap();
                }

                let total_offset =
                    EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

                assert_eq!(total_offset, STANDARD_CARBON_AMOUNT * project_num as u32);
            }
        });
    }

    #[test]
    fn test_generate_report_large_volume() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue and retire 30 credits
            for i in 1..=30 {
                let credit_id = create_credit_id(&test_env.env, i);
                EnvironmentalContract::issue_carbon_credit(
                    &test_env.env,
                    credit_id.clone(),
                    project_id.clone(),
                    LARGE_CARBON_AMOUNT,
                    standard_verification_method(&test_env.env),
                )
                .unwrap();

                EnvironmentalContract::retire_credit(
                    &test_env.env,
                    credit_id,
                    test_env.user1.clone(),
                )
                .unwrap();
            }

            let total_offset =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);

            assert_eq!(total_offset, LARGE_CARBON_AMOUNT * 30);
        });
    }

    #[test]
    fn test_generate_report_after_additional_retirements() {
        let test_env = setup_test();
        let project_id = create_project_id(&test_env.env, 1);

        test_env.env.as_contract(&test_env.contract_id, || {
            // Issue 5 credits
            for i in 1..=5 {
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

            // Retire 2 credits and check report
            for i in 1..=2 {
                let credit_id = create_credit_id(&test_env.env, i);
                EnvironmentalContract::retire_credit(
                    &test_env.env,
                    credit_id,
                    test_env.user1.clone(),
                )
                .unwrap();
            }

            let offset1 =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id.clone());
            assert_eq!(offset1, STANDARD_CARBON_AMOUNT * 2);

            // Retire 2 more credits and check updated report
            for i in 3..=4 {
                let credit_id = create_credit_id(&test_env.env, i);
                EnvironmentalContract::retire_credit(
                    &test_env.env,
                    credit_id,
                    test_env.user1.clone(),
                )
                .unwrap();
            }

            let offset2 =
                EnvironmentalContract::generate_impact_report(&test_env.env, project_id);
            assert_eq!(offset2, STANDARD_CARBON_AMOUNT * 4);
        });
    }
}
