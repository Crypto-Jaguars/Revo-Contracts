#[cfg(test)]
mod test {
    use super::*;
    use crate::tests::utils::{
        advance_time, create_document_hash, setup_certification_test, setup_integration_test,
    };

    use soroban_sdk::{
        symbol_short,
        testutils::{Address as _, Events as _, Ledger, LedgerInfo},
        vec, Address, Bytes, BytesN, Env, String, Symbol, TryFromVal,
    };

    use crate::{
        AgricQualityContract, AgricQualityContractClient, CertificationData, CertificationStatus,
        QualityStandard,
    };
    use certificate_management_contract::{
        CertStatus, CertificateManagementContract, CertificateManagementContractClient,
        Certification, DataKey,
    };

    // Test integration: certificate association and status validation
    #[test]
    fn test_certification_integration() {
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

        let (cert_id, metric_name, validity_period) =
            setup_certification_test(&env, &agric_client, &farmer, &inspector, &authority);

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
            agric_cert.expiry_date, validity_period,
            "Expiry date should match"
        );

        // Create test data
        let verification_hash = create_document_hash(&env, "Organic certification document");
        let cert_type = symbol_short!("ORGANIC");

        let res = cert_client.issue_certification(
            &authority,
            &farmer,
            &cert_type,
            &validity_period,
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
    }

    // Test integration: handling expired certification
    #[test]
    fn test_expired_certification_integration() {
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

        let (cert_id, metric_name, validity_period) =
            setup_certification_test(&env, &agric_client, &farmer, &inspector, &authority);

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

        let now = env.ledger().timestamp();

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
        assert_eq!(agric_cert.issuer, authority, "Authority should match");
        assert_eq!(
            agric_cert.expiry_date,
            now + validity_period,
            "Expiry date should match"
        );

        // Create test data
        let verification_hash = create_document_hash(&env, "Organic certification document");
        let cert_type = symbol_short!("ORGANIC");

        let res = cert_client.issue_certification(
            &authority,
            &farmer,
            &cert_type,
            &validity_period,
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

        // Advance time past expiration
        advance_time(&env, validity_period + 1);

        // Expire the certification
        cert_client.expire_certification(
            &farmer, &1, // First cert ID is 1
        );

        // Verify status updated to Expired
        let status = cert_client.check_cert_status(&farmer, &1); // First cert ID is 1
        assert_eq!(status, CertStatus::Expired);
    }
}
