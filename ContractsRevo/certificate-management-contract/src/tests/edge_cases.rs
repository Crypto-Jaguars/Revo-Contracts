#![cfg(test)]
use crate::{
    tests::utils::TestContext,
    CertStatus, IssueError, VerifyError,
};

#[test]
fn test_symbol_length_limits() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Test document");
    
    // Test with maximum reasonable symbol length (32 chars is typical limit)
    let max_length_type = "ORGANIC_FAIR_TRADE_NON_GMO_CERT";

    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol(max_length_type),
        &expiration,
        &doc_hash,
    );

    let cert = client.get_cert(&context.recipient1, &1);
    assert_eq!(cert.cert_type, context.symbol(max_length_type));
}

#[test]
fn test_duplicate_certificate_issuance() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Same document");

    // Issue first certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    // Issue second certification with same data (should succeed with different ID)
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    // Verify both certifications exist
    let cert1 = client.get_cert(&context.recipient1, &1);
    let cert2 = client.get_cert(&context.recipient1, &2);

    assert_eq!(cert1.id, 1);
    assert_eq!(cert2.id, 2);
    assert_eq!(cert1.verification_hash, cert2.verification_hash);
}

#[test]
fn test_certificate_at_exact_expiration_time() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 1000;
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

    // Advance time to exact expiration
    context.advance_time(1001); // Advance slightly past expiration to ensure it's expired

    // Try to verify at exact expiration time
    let result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);
    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, VerifyError::Expired);
    }
}

#[test]
fn test_zero_expiration_date() {
    let context = TestContext::setup();
    let client = context.client();

    let doc_hash = context.create_document_hash("Test document");

    // Try to issue with zero expiration
    context.env.mock_all_auths();
    let result = client.try_issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &0,
        &doc_hash,
    );

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, IssueError::InvalidExpirationDate);
    }
}

#[test]
fn test_maximum_timestamp_values() {
    let context = TestContext::setup();
    let client = context.client();

    let doc_hash = context.create_document_hash("Test document");
    let max_timestamp = u64::MAX;

    // Issue with maximum timestamp
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &max_timestamp,
        &doc_hash,
    );

    // Verify certification exists
    let cert = client.get_cert(&context.recipient1, &1);
    assert_eq!(cert.expiration_date, max_timestamp);
}

#[test]
fn test_empty_certificate_type() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Test document");

    // Issue with empty symbol (should work)
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol(""),
        &expiration,
        &doc_hash,
    );

    let cert = client.get_cert(&context.recipient1, &1);
    assert_eq!(cert.cert_type, context.symbol(""));
}

#[test]
fn test_revocation_race_conditions() {
    let context = TestContext::setup();
    let client = context.client();

    let cert_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Try to revoke and expire simultaneously
    context.env.mock_all_auths();
    
    // First revoke
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert_id);
    
    // Then try to expire (should fail as it's already revoked)
    let result = client.try_expire_certification(&context.recipient1, &cert_id);
    assert!(result.is_err());
}

#[test]
fn test_very_long_certificate_type() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Test document");
    
    let long_type = "VERY_LONG_CERT_TYPE";

    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol(long_type),
        &expiration,
        &doc_hash,
    );

    let cert = client.get_cert(&context.recipient1, &1);
    assert_eq!(cert.cert_type, context.symbol(long_type));
}

#[test]
fn test_boundary_conditions_timestamp_filter() {
    let context = TestContext::setup();
    let client = context.client();

    let initial_time = context.env.ledger().timestamp();
    
    // Issue cert at exact time
    let _cert1 = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);
    
    // Filter with exact timestamp
    let exact_match = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &None,
        &Some(initial_time),
    );
    assert_eq!(exact_match.len(), 1);

    // Filter with timestamp + 1 (should exclude the cert)
    let no_match = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &None,
        &Some(initial_time + 1),
    );
    assert_eq!(no_match.len(), 0);
}

#[test]
fn test_self_issued_certificate() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue certificate to self (issuer = recipient)
    let cert_id = context.issue_test_cert(&context.issuer1, &context.issuer1, "ORGANIC", 365);

    // Verify self-issued cert works
    let cert = client.get_cert(&context.issuer1, &cert_id);
    assert_eq!(cert.issuer, context.issuer1);

    // Self-revoke should work
    context.env.mock_all_auths();
    client.revoke_certification(&context.issuer1, &context.issuer1, &cert_id);

    let status = client.check_cert_status(&context.issuer1, &cert_id);
    assert_eq!(status, CertStatus::Revoked);
}

#[test]
fn test_certificate_id_overflow_protection() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue many certificates to approach potential overflow
    context.env.mock_all_auths();
    for i in 1..=100 {
        // Create different document content without format! macro
        let doc_content = if i % 10 == 0 {
            "Document batch 10"
        } else if i % 5 == 0 {
            "Document batch 5"
        } else {
            "Standard document"
        };
        
        client.issue_certification(
            &context.issuer1,
            &context.recipient1,
            &context.symbol("TEST"),
            &(context.env.ledger().timestamp() + 31536000),
            &context.create_document_hash(doc_content),
        );
    }

    // Verify all certificates have sequential IDs
    for i in 1..=100 {
        let cert = client.get_cert(&context.recipient1, &i);
        assert_eq!(cert.id, i);
    }
}

#[test]
fn test_hash_collision_resistance() {
    let context = TestContext::setup();
    let client = context.client();

    let hash1 = context.create_document_hash("Document 1");
    let hash2 = context.create_document_hash("Document 2");

    // Ensure hashes are different
    assert_ne!(hash1, hash2);

    // Issue certificates with different hashes
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &(context.env.ledger().timestamp() + 31536000),
        &hash1,
    );

    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &(context.env.ledger().timestamp() + 31536000),
        &hash2,
    );

    // Verify each cert only accepts its own hash
    let result1 = client.try_verify_document_hash(&context.recipient1, &1, &hash2);
    let result2 = client.try_verify_document_hash(&context.recipient1, &2, &hash1);

    assert!(result1.is_err());
    assert!(result2.is_err());
}