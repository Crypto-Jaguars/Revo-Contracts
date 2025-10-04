#![cfg(test)]
use crate::{tests::utils::TestContext, CertStatus, VerifyError};

#[test]
fn test_successful_document_verification() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Organic certification document");

    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    // Verify with correct hash
    let result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);
    assert!(result.is_ok());
}

#[test]
fn test_hash_mismatch_verification() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Original document");
    let wrong_hash = context.create_document_hash("Modified document");

    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    // Verify with incorrect hash
    let result = client.try_verify_document_hash(&context.recipient1, &1, &wrong_hash);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, VerifyError::HashMismatch);
    }
}

#[test]
fn test_verify_nonexistent_certification() {
    let context = TestContext::setup();
    let client = context.client();

    let doc_hash = context.create_document_hash("Some document");

    let result = client.try_verify_document_hash(&context.recipient1, &999, &doc_hash);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, VerifyError::NotFound);
    }
}

#[test]
fn test_verify_revoked_certification() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Test document");

    // Issue and revoke certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    client.revoke_certification(&context.issuer1, &context.recipient1, &1);

    // Try to verify revoked certification
    let result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, VerifyError::Revoked);
    }
}

#[test]
fn test_verify_expired_certification() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 1000; // Short expiration
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

    // Advance time past expiration
    context.advance_time(2000);

    // Expire the certification
    client.expire_certification(&context.recipient1, &1);

    // Try to verify expired certification
    let result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, VerifyError::Expired);
    }
}

#[test]
fn test_verify_expiration_due() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 1000; // Short expiration
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

    // Advance time past expiration (but don't call expire)
    context.advance_time(2000);

    // Try to verify - should fail due to expiration date
    let result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, VerifyError::Expired);
    }
}

#[test]
fn test_multiple_verifications_same_cert() {
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

    // Verify multiple times - should all succeed
    for _ in 0..5 {
        let result = client.try_verify_document_hash(&context.recipient1, &1, &doc_hash);
        assert!(result.is_ok());
    }
}

#[test]
fn test_check_cert_status() {
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

    // Check initial status
    let status = client.check_cert_status(&context.recipient1, &1);
    assert_eq!(status, CertStatus::Valid);

    // Revoke and check status
    client.revoke_certification(&context.issuer1, &context.recipient1, &1);
    let status = client.check_cert_status(&context.recipient1, &1);
    assert_eq!(status, CertStatus::Revoked);
}
