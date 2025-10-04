#![cfg(test)]
use crate::{tests::utils::TestContext, AuditError, CertStatus};

#[test]
fn test_audit_report_all_certifications() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue multiple certifications
    let _cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);
    let _cert2 = context.issue_test_cert(&context.issuer2, &context.recipient1, "FAIRTRADE", 180);

    // Get all certifications
    let certs = client.generate_cert_audit_report(&context.recipient1, &None, &None, &None);
    assert_eq!(certs.len(), 2);
}

#[test]
fn test_audit_report_by_issuer() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue certifications from different issuers
    let _cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);
    let _cert2 = context.issue_test_cert(&context.issuer2, &context.recipient1, "FAIRTRADE", 180);
    let _cert3 = context.issue_test_cert(&context.issuer1, &context.recipient1, "NON_GMO", 365);

    // Filter by issuer1
    let issuer1_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &Some(context.issuer1.clone()),
        &None,
        &None,
    );
    assert_eq!(issuer1_certs.len(), 2);

    // Filter by issuer2
    let issuer2_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &Some(context.issuer2.clone()),
        &None,
        &None,
    );
    assert_eq!(issuer2_certs.len(), 1);
}

#[test]
fn test_audit_report_by_status() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue certifications
    let cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);
    let _cert2 = context.issue_test_cert(&context.issuer1, &context.recipient1, "FAIRTRADE", 365);

    // Revoke one certification
    context.env.mock_all_auths();
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert1);

    // Filter by status - valid certs
    let valid_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &Some(CertStatus::Valid),
        &None,
    );
    assert_eq!(valid_certs.len(), 1);

    // Filter by status - revoked certs
    let revoked_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &Some(CertStatus::Revoked),
        &None,
    );
    assert_eq!(revoked_certs.len(), 1);
}

#[test]
fn test_audit_report_by_timestamp() {
    let context = TestContext::setup();
    let client = context.client();

    let initial_time = context.env.ledger().timestamp();

    // Issue first certification
    let _cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Advance time
    context.advance_time(86400); // 1 day
    let mid_time = context.env.ledger().timestamp();

    // Issue second certification
    let _cert2 = context.issue_test_cert(&context.issuer1, &context.recipient1, "FAIRTRADE", 365);

    // Get certs after initial time (should get both)
    let all_certs =
        client.generate_cert_audit_report(&context.recipient1, &None, &None, &Some(initial_time));
    assert_eq!(all_certs.len(), 2);

    // Get certs after mid time (should get only second)
    let recent_certs =
        client.generate_cert_audit_report(&context.recipient1, &None, &None, &Some(mid_time));
    assert_eq!(recent_certs.len(), 1);
}

#[test]
fn test_audit_report_combined_filters() {
    let context = TestContext::setup();
    let client = context.client();

    let _initial_time = context.env.ledger().timestamp();

    // Issue multiple certifications
    let cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);
    let _cert2 = context.issue_test_cert(&context.issuer2, &context.recipient1, "FAIRTRADE", 365);

    context.advance_time(86400); // 1 day
    let mid_time = context.env.ledger().timestamp();

    let _cert3 = context.issue_test_cert(&context.issuer1, &context.recipient1, "NON_GMO", 365);

    // Revoke first cert
    context.env.mock_all_auths();
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert1);

    // Filter: issuer1, valid certs, after mid_time
    let filtered_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &Some(context.issuer1.clone()),
        &Some(CertStatus::Valid),
        &Some(mid_time),
    );
    assert_eq!(filtered_certs.len(), 1); // Only cert3 matches all criteria
}

#[test]
fn test_audit_report_empty_results() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue one certification
    let _cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Filter by non-existent issuer
    let empty_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &Some(context.issuer2.clone()),
        &None,
        &None,
    );
    assert_eq!(empty_certs.len(), 0);

    // Filter by revoked status when no certs are revoked
    let empty_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &Some(CertStatus::Revoked),
        &None,
    );
    assert_eq!(empty_certs.len(), 0);
}

#[test]
fn test_audit_report_nonexistent_user() {
    let context = TestContext::setup();
    let client = context.client();

    let result = client.try_generate_cert_audit_report(
        &context.recipient2, // User with no certs
        &None,
        &None,
        &None,
    );

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, AuditError::NotFound);
    }
}

#[test]
fn test_scalability_high_volume_operations() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue many certifications
    context.env.mock_all_auths();
    for i in 0..50 {
        // Create unique document content
        let doc_content = if i % 2 == 0 {
            "Document type A content"
        } else {
            "Document type B content"
        };

        client.issue_certification(
            &context.issuer1,
            &context.recipient1,
            &context.symbol("ORGANIC"),
            &(context.env.ledger().timestamp() + 31536000),
            &context.create_document_hash(doc_content),
        );
    }

    // Generate audit report for all certs
    let all_certs = client.generate_cert_audit_report(&context.recipient1, &None, &None, &None);
    assert_eq!(all_certs.len(), 50);

    // Revoke half of them
    for i in 1..26 {
        client.revoke_certification(&context.issuer1, &context.recipient1, &i);
    }

    // Check revoked count
    let revoked_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &Some(CertStatus::Revoked),
        &None,
    );
    assert_eq!(revoked_certs.len(), 25);

    // Check valid count
    let valid_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &Some(CertStatus::Valid),
        &None,
    );
    assert_eq!(valid_certs.len(), 25);
}

#[test]
fn test_get_cert_details() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Test document");

    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    // Get certification details
    let cert = client.get_cert(&context.recipient1, &1);

    assert_eq!(cert.id, 1);
    assert_eq!(cert.cert_type, context.symbol("ORGANIC"));
    assert_eq!(cert.issuer, context.issuer1);
    assert_eq!(cert.expiration_date, expiration);
    assert_eq!(cert.verification_hash, doc_hash);
    assert_eq!(cert.status, CertStatus::Valid);
    assert!(cert.issued_date >= now);
}

#[test]
fn test_integration_cert_lifecycle() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 86400; // 1 day
    let doc_hash = context.create_document_hash("Integration test document");

    // 1. Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    // 2. Verify certification is valid
    let status = client.check_cert_status(&context.recipient1, &1);
    assert_eq!(status, CertStatus::Valid);

    // 3. Verify document hash
    let verify_result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);
    assert!(verify_result.is_ok());

    // 4. Generate audit report
    let audit = client.generate_cert_audit_report(&context.recipient1, &None, &None, &None);
    assert_eq!(audit.len(), 1);

    // 5. Revoke certification
    client.revoke_certification(&context.issuer1, &context.recipient1, &1);

    // 6. Verify status changed
    let status = client.check_cert_status(&context.recipient1, &1);
    assert_eq!(status, CertStatus::Revoked);

    // 7. Verify document hash fails
    let verify_result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);
    assert!(verify_result.is_err());

    // 8. Audit shows revoked status
    let revoked_audit = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &Some(CertStatus::Revoked),
        &None,
    );
    assert_eq!(revoked_audit.len(), 1);
}
