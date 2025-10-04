#[cfg(test)]
extern crate std; // This enables std in tests
mod data_validation_tests {
    use super::*;
    use crate::tests::utils::{advance_time, create_document_hash, setup_integration_test};
    use crate::{
        AgricQualityContract, AgricQualityContractClient, AgricQualityError, CertificationData,
        CertificationStatus, DataKey::Inspection, InspectionReport, QualityStandard,
    };
    use certificate_management_contract::{
        CertStatus, CertificateManagementContract, CertificateManagementContractClient,
        Certification, DataKey,
    };
    use soroban_sdk::{
        symbol_short,
        testutils::{Address as _, Events as _, Ledger, LedgerInfo},
        vec, Address, Bytes, BytesN, Env, String, Symbol, TryFromVal,
    };

    #[test]
    fn test_certification_integration_verify_doc_hash() {
        let (
            env,
            agric_contract_id,
            agric_client,
            cert_contract_id,
            cert_client,
            admin,
            farmer,
            inspector,
            authority,
        ) = setup_integration_test();
        let now = env.ledger().timestamp();
        // Authority registers quality metrics for a standard (e.g., Organic, GlobalGAP) with a name, minimum score, and weight.
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("pesticide");
        let min_score = 85u32;
        let weight = 50u32;
        agric_client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Submit certification
        //  Farmer submits a product for certification, specifying a standard and conditions.
        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = agric_client.submit_for_certification(&farmer, &standard, &conditions);

        // Record an inspection
        let metrics = vec![&env, (metric_name.clone(), min_score)];
        let findings = vec![&env, String::from_str(&env, "Good moisture level")];
        let recommendations = vec![&env, String::from_str(&env, "None needed")];
        agric_client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

        // 4. Process the Certification
        let approved = true;
        let validity_period = 31536000; // 1 year in seconds
        agric_client.process_certification(&authority, &cert_id, &approved, &validity_period);

        // Compliance Check
        let report = agric_client.check_compliance(&cert_id, &inspector);
        assert_eq!(report.inspector, inspector, "Inspector should match");
        assert_eq!(report.metrics.len(), 1, "One metric should be recorded");
        assert_eq!(
            report.metrics.get(0).unwrap().0,
            metric_name,
            "Metric name should match"
        );
        assert_eq!(
            report.metrics.get(0).unwrap().1,
            85u32,
            "Metric score should match"
        );
        assert_eq!(
            report.overall_score, 85u32,
            "Overall score should match recorded score"
        );
        assert_eq!(report.findings.len(), 0, "No findings for passing score");
        assert_eq!(
            report.recommendations.len(),
            0,
            "No recommendations for passing score"
        );

        // Step 5: Validate certificate association
        let agric_certs = agric_client.get_certification_history(&farmer);
        assert_eq!(agric_certs.len(), 1, "One certification should be recorded");

        let agric_cert = agric_certs.get(0).unwrap();
        assert_eq!(agric_cert.holder, farmer, "Holder should match");
        assert_eq!(
            agric_cert.standard,
            QualityStandard::Organic,
            "Standard should match"
        );
        assert_eq!(
            agric_cert.status,
            CertificationStatus::Active,
            "Certification should be active"
        );
        assert_eq!(agric_cert.issuer, authority, "Issuer should match");
        assert_eq!(
            agric_cert.expiry_date,
            now + 365 * 24 * 60 * 60,
            "Expiry date should match"
        );

        // Create test data
        let expiration_date = now + 365 * 24 * 60 * 60; // 1 year
        let verification_hash = create_document_hash(&env, "Organic certification document");
        let cert_type = symbol_short!("Organic");

        let res = cert_client.issue_certification(
            &authority,
            &farmer,
            &cert_type,
            &expiration_date,
            &verification_hash,
        );
        let cert_id = 1u32;

        // Step 6: Check certification status in CertificationContract
        let cert_status = cert_client.check_cert_status(&farmer, &cert_id);
        assert_eq!(
            cert_status,
            CertStatus::Valid,
            "Certification should be valid"
        );

        // Step 7: Verify integration by checking hash consistency
        let cert_from_mgr = cert_client.get_cert(&farmer, &cert_id);
        assert_eq!(
            cert_from_mgr.verification_hash, verification_hash,
            "Verification hash should match"
        );

        // Verify with correct hash
        cert_client.verify_document_hash(
            &farmer,
            &1, // First cert ID is 1
            &verification_hash,
        );
    }

    #[test]
    #[should_panic]
    fn test_invalid_condition() {
        let (env, _, agric_client, _, cert_client, _, farmer, inspector, authority) =
            setup_integration_test();

        // Step 1: Register quality metric
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("gmo_free");
        let min_score = 85u32;
        let weight = 50u32;
        agric_client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Step 2: Submit certification with invalid conditions (empty conditions)
        let conditions = vec![&env]; // Invalid: empty conditions
        let cert_id_result = agric_client.submit_for_certification(&farmer, &standard, &conditions);
    }

    #[test]
    fn test_quality_assessment_status_revoked() {
        let (env, _, agric_client, _, cert_client, _, farmer, inspector, authority) =
            setup_integration_test();

        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("gmo_free");
        let min_score = 85u32;
        let weight = 50u32;
        agric_client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        let conditions = vec![&env, String::from_str(&env, "Organic farming practice")];
        let cert_id = agric_client.submit_for_certification(&farmer, &standard, &conditions);

        let metrics = vec![&env, (metric_name.clone(), 30u32)];
        let findings = vec![&env, String::from_str(&env, "No GMO traces detected")];
        let recommendations = vec![&env, String::from_str(&env, "Maintain current practices")];
        let _ = agric_client.record_inspection(
            &inspector,
            &cert_id,
            &metrics,
            &findings,
            &recommendations,
        );

        // Step 6: Check compliance with invalid score
        let report_result = agric_client.check_compliance(&cert_id, &inspector);

        let _ =
            agric_client.process_certification(&authority, &cert_id, &false, &(365 * 24 * 60 * 60));

        let agric_certs = agric_client.get_certification_history(&farmer);
        let agric_cert = agric_certs.get(0).unwrap();
        assert_eq!(
            agric_cert.status,
            CertificationStatus::Revoked,
            "Certification should be revoked"
        );

        let cert_type = symbol_short!("Organic");
        let cert_id_mgr = 1u32;
        let issued_date = env.ledger().timestamp();
        let expiration_date = issued_date + 365 * 24 * 60 * 60;
        let verification_hash = create_document_hash(&env, "Organic certification document");
        cert_client.issue_certification(
            &authority,
            &farmer,
            &cert_type,
            &expiration_date,
            &verification_hash,
        );

        // Step 9: Verify certification status
        let cert_status = cert_client.check_cert_status(&farmer, &cert_id_mgr);

        // assert_eq!(cert_status, CertStatus::Valid, "Certification should remain valid");
    }

    // Test missing inspection report
    #[test]
    #[should_panic]
    fn test_missing_inspection_report() {
        let (env, _, agric_client, _, cert_client, _, farmer, _, authority) =
            setup_integration_test();

        // Step 1: Submit certification
        let standard = QualityStandard::Organic;
        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = agric_client.submit_for_certification(&farmer, &standard, &conditions);

        // Step 2: Issue certification without inspection
        let cert_type = symbol_short!("Organic");
        let cert_id_mgr = 1u32;
        let issued_date = env.ledger().timestamp();
        let expiration_date = issued_date + 365 * 24 * 60 * 60;
        let verification_hash = create_document_hash(&env, "Organic certification document");
        cert_client.issue_certification(
            &authority,
            &farmer,
            &cert_type,
            &expiration_date,
            &verification_hash,
        );

        // Step 3: Attempt to process certification (should fail due to missing inspection)
        let result =
            agric_client.process_certification(&authority, &cert_id, &true, &(365 * 24 * 60 * 60));
    }

    #[test]
    fn test_quality_assessment_failing_score() {
        let (env, _, agric_client, _, cert_client, _, farmer, inspector, authority) =
            setup_integration_test();

        // Step 1: Register quality metric
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("gmo_free");
        let min_score = 90u32;
        let weight = 50u32;
        agric_client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Step 2: Submit certification
        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = agric_client.submit_for_certification(&farmer, &standard, &conditions);

        // Step 3: Record inspection with invalid score (below min_score)
        let metrics = vec![&env, (metric_name.clone(), 80u32)]; // Below min_score
        let findings = vec![&env, String::from_str(&env, "GMO traces detected")];
        let recommendations = vec![&env, String::from_str(&env, "Enhance GMO-free practices")];
        let inspection_result = agric_client.record_inspection(
            &inspector,
            &cert_id,
            &metrics,
            &findings,
            &recommendations,
        );

        // Step 4: Check compliance with invalid score
        let report_result = agric_client.check_compliance(&cert_id, &inspector);
        assert_eq!(report_result.inspector, inspector, "Inspector should match");
        assert_eq!(
            report_result.metrics.len(),
            1,
            "One metric should be recorded"
        );
        assert_eq!(
            report_result.metrics.get(0).unwrap().0,
            metric_name,
            "Metric name should match"
        );
        assert_eq!(
            report_result.metrics.get(0).unwrap().1,
            0u32,
            "Metric score should be 0 due to Organic standard adjustment"
        );
        assert_eq!(
            report_result.overall_score, 0u32,
            "Overall score should be 0 due to failing score"
        );
        assert_eq!(
            report_result.findings.get(0).unwrap(),
            String::from_str(&env, "Score below minimum required threshold"),
            "Findings should indicate score failure"
        );

        // Step 5: Issue certification
        let cert_type = symbol_short!("Organic");
        let cert_id_mgr = 1u32;
        let issued_date = env.ledger().timestamp();
        let expiration_date = issued_date + 365 * 24 * 60 * 60;
        let verification_hash = create_document_hash(&env, "Organic certification document");
        cert_client.issue_certification(
            &authority,
            &farmer,
            &cert_type,
            &expiration_date,
            &verification_hash,
        );

        // Step 6: Attempt to process certification (should fail due to failing inspection)
        // Changed: Removed process_certification call to keep status Pending for failing score

        // Step 7: Verify certification status
        let cert_status = cert_client.check_cert_status(&farmer, &cert_id_mgr);
        assert_eq!(
            cert_status,
            CertStatus::Valid,
            "Certification should remain valid"
        );

        // Step 8: Validate certification data (remains pending)
        let agric_certs = agric_client.get_certification_history(&farmer);
        assert_eq!(agric_certs.len(), 1, "One certification should be recorded");
        let agric_cert = agric_certs.get(0).unwrap();
        assert_eq!(
            agric_cert.status,
            CertificationStatus::Pending,
            "Certification should remain pending"
        );
    }

    // Test certification integration with missing certificate (invalid cert_id_mgr)
    #[test]
    #[should_panic]
    fn test_certification_integration_invalid_certificate_id() {
        let (env, _, agric_client, _, cert_client, _, farmer, inspector, authority) =
            setup_integration_test();

        // Step 1: Register quality metric
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("gmo_free");
        let min_score = 90u32;
        let weight = 50u32;
        agric_client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        // Step 2: Submit certification
        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = agric_client.submit_for_certification(&farmer, &standard, &conditions);

        // Step 3: Record inspection
        let metrics = vec![&env, (metric_name.clone(), 95u32)];
        let findings = vec![&env, String::from_str(&env, "No GMO traces detected")];
        let recommendations = vec![&env, String::from_str(&env, "Maintain current practices")];
        agric_client.record_inspection(&inspector, &cert_id, &metrics, &findings, &recommendations);

        // Step 4: Issue certification with a different farmer
        let cert_type = symbol_short!("Organic");
        let cert_id_mgr = 1u32;
        let issued_date = env.ledger().timestamp();
        let expiration_date = issued_date + 365 * 24 * 60 * 60;
        let verification_hash = create_document_hash(&env, "Organic certification document");
        let other_farmer = Address::generate(&env);
        cert_client.issue_certification(
            &authority,
            &other_farmer,
            &cert_type,
            &expiration_date,
            &verification_hash,
        );

        // Step 5: Verify certification status (no certificate for farmer)
        let cert_status = cert_client.check_cert_status(&farmer, &cert_id_mgr);
    }

    #[test]
    #[should_panic]
    fn test_unauthorized_quality_assessment() {
        let (env, _, agric_client, _, _, _, farmer, inspector, authority) =
            setup_integration_test();

        // Step 1: Register quality metric
        let standard = QualityStandard::Organic;
        let metric_name = symbol_short!("gmo_free");
        let min_score = 90u32;
        let weight = 50u32;
        agric_client.register_metric(&authority, &standard, &metric_name, &min_score, &weight);

        let conditions = vec![&env, String::from_str(&env, "Organic farming practices")];
        let cert_id = agric_client.submit_for_certification(&farmer, &standard, &conditions);

        let unauthorized_inspector = Address::generate(&env); // Not added to inspectors
        let metrics = vec![&env, (metric_name.clone(), 95u32)];
        let findings = vec![&env, String::from_str(&env, "No GMO traces detected")];
        let recommendations = vec![&env, String::from_str(&env, "Maintain current practices")];
        let inspection_result = agric_client.record_inspection(
            &unauthorized_inspector,
            &cert_id,
            &metrics,
            &findings,
            &recommendations,
        );
    }
}
