#[cfg(test)]
mod test {
    // use crate::tests::utils::{setup_test};
    use crate::CertificationStatus;
    use crate::DisputeStatus;
    use crate::QualityStandard;
    use crate::ResolutionOutcome;

    use crate::tests::utils::setup_test;
    use crate::{AgricQualityContract, AgricQualityContractClient};
    use soroban_sdk::{symbol_short, testutils::Address as _, vec, Address, Env, String};

    // Test initialization
    #[test]
    #[should_panic]
    fn test_initialize_contract() {
        let (env, _, client, admin, _, _, _) = setup_test();

        // Check if admin is set correctly using the non-try method
        assert_eq!(client.get_admin(), admin.clone());

        // Attempt to initialize again (should fail)
        client.initialize(&admin);
    }

    // Test registering a quality metric
    #[test]
    fn test_register_quality_metric() {
        let (env, _, client, admin, _, _, authority) = setup_test();

        // Add authority
        client.add_authority(&admin, &authority);

        // Define quality metric parameters
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("pestici");
        let min_score = 80u32;
        let weight = 50u32;

        // Register quality metric
        let result =
            client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Verify metric storage
        let metrics = client.get_standard_metrics(&standard);
        assert_eq!(metrics.len(), 1, "One metric should be registered");

        let metric = metrics.get(0).unwrap();
        assert_eq!(metric.name, metric_name, "Metric name should match");
        assert_eq!(metric.standard, standard, "Standard should match");
        assert_eq!(metric.min_score, min_score, "Min score should match");
        assert_eq!(metric.weight, weight, "Weight should match");
        assert_eq!(metric.authority, authority, "Authority should match");
    }

    // Test updating a quality metric
    #[test]
    fn test_update_quality_metric() {
        let (env, _, client, admin, _, _, authority) = setup_test();

        // Add authority
        client.add_authority(&admin, &authority);

        // Register initial quality metric
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("pestic");
        let min_score = 80u32;
        let weight = 50u32;
        client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Update metric
        let new_min_score = 85u32;
        let new_weight = 60u32;
        let result = client.update_metric(
            &authority,
            &standard,
            &metric_name,
            &new_min_score,
            &new_weight,
        );

        // Verify updated metric
        let metrics = client.get_standard_metrics(&standard);
        assert_eq!(metrics.len(), 1, "One metric should exist");

        let metric = metrics.get(0).unwrap();
        assert_eq!(metric.name, metric_name, "Metric name should match");
        assert_eq!(
            metric.min_score, new_min_score,
            "Min score should be updated"
        );
        assert_eq!(metric.weight, new_weight, "Weight should be updated");
    }

    // Test unauthorized attempt to register a quality metric
    #[test]
    #[should_panic]
    fn test_unauthorized_register_metric() {
        let (env, _, client, _, _, _, _) = setup_test();
        let unauthorized = Address::generate(&env);

        // Attempt to register metric without authority
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("pestic");
        let min_score = 80u32;
        let weight = 50u32;
        client.register_metric(&unauthorized, &standard, &metric_name, &min_score, &weight);
    }

    #[test]
    fn test_check_compliance() {
        let (env, _, client, admin, farmer, inspector, authority) = setup_test();

        // Add authority and inspector
        client.add_authority(&admin, &authority);
        client.add_inspector(&admin, &inspector);

        // Register quality metric with a short symbol name (<= 9 characters)
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("pesticide"); // Shortened to 9 characters
        let min_score = 85u32;
        let weight = 50u32;
        client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Submit certification
        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = client.submit_for_certification(&farmer, &standard, &conditions);

        // Record inspection with passing score
        let metrics = vec![&env, (metric_name.clone(), 90u32)]; // Above min_score
        let findings = vec![
            &env,
            String::from_str(&env, "No pesticide residue detected"),
        ];
        let recommendations = vec![&env, String::from_str(&env, "Maintain current practices")];
        client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

        // // Check compliance
        let report = client.check_compliance(&cert_id, &inspector);
        assert_eq!(report.inspector, inspector, "Inspector should match");
        assert_eq!(report.metrics.len(), 1, "One metric should be recorded");
        assert_eq!(
            report.metrics.get(0).unwrap().0,
            metric_name,
            "Metric name should match"
        );
        assert_eq!(
            report.metrics.get(0).unwrap().1,
            90u32,
            "Metric score should match"
        );
        assert_eq!(
            report.overall_score, 90u32,
            "Overall score should match recorded score"
        );
        assert_eq!(report.findings.len(), 0, "No findings for passing score");
        assert_eq!(
            report.recommendations.len(),
            0,
            "No recommendations for passing score"
        );
    }

    // Test compliance failure due to low score
    #[test]
    fn test_check_compliance_failure() {
        let (env, _, client, admin, farmer, inspector, authority) = setup_test();

        // Add authority and inspector
        client.add_authority(&admin, &authority);
        client.add_inspector(&admin, &inspector);

        // Register quality metric with a short symbol name (<= 9 characters)
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("pesticide"); // Shortened to 9 characters
        let min_score = 85u32;
        let weight = 50u32;
        client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Submit certification
        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = client.submit_for_certification(&farmer, &standard, &conditions);

        // Record inspection with failing score
        let metrics = vec![&env, (metric_name.clone(), 70u32)]; // Below min_score
        let findings = vec![&env, String::from_str(&env, "Pesticide residue detected")];
        let recommendations = vec![&env, String::from_str(&env, "Reduce pesticide use")];
        client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

        // Check compliance
        let report = client.check_compliance(&cert_id, &inspector);
        assert_eq!(report.inspector, inspector, "Inspector should match");
        assert_eq!(report.metrics.len(), 1, "One metric should be recorded");
        assert_eq!(
            report.metrics.get(0).unwrap().0,
            metric_name,
            "Metric name should match"
        );
        assert_eq!(
            report.metrics.get(0).unwrap().1,
            0u32,
            "Metric score should be 0 due to Organic standard adjustment"
        );
        assert_eq!(
            report.overall_score, 0u32,
            "Overall score should be 0 due to failing score"
        );
        assert_eq!(
            report.findings.get(0).unwrap(),
            String::from_str(&env, "Score below minimum required threshold"),
            "Findings should indicate score failure"
        );
        assert_eq!(
            report.recommendations.get(0).unwrap(),
            String::from_str(&env, "Improve metric score to meet minimum requirements"),
            "Recommendations should suggest improvement"
        );
    }
}
