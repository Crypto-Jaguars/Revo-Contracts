#![cfg(test)]
use crate::{
    tests::utils::TestContext,
    CertStatus, CertificationError, RevokeError,
};

#[test]
fn test_successful_revocation() {
    let context = TestContext::setup();
    let client = context.client();

    let cert_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Verify initially valid
    let status = client.check_cert_status(&context.recipient1, &cert_id);
    assert_eq!(status, CertStatus::Valid);

    // Revoke certification
    context.env.mock_all_auths();
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert_id);

    // Verify status updated
    let status = client.check_cert_status(&context.recipient1, &cert_id);
    assert_eq!(status, CertStatus::Revoked);
}

#[test]
fn test_unauthorized_revocation() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue cert with issuer1
    let cert_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Try to revoke with issuer2 (unauthorized)
    context.env.mock_all_auths();
    let result = client.try_revoke_certification(&context.issuer2, &context.recipient1, &cert_id);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, RevokeError::Unauthorized);
    }

    // Verify cert is still valid
    let status = client.check_cert_status(&context.recipient1, &cert_id);
    assert_eq!(status, CertStatus::Valid);
}

#[test]
fn test_revoke_nonexistent_certification() {
    let context = TestContext::setup();
    let client = context.client();

    context.env.mock_all_auths();
    let result = client.try_revoke_certification(&context.issuer1, &context.recipient1, &999);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, RevokeError::NotFound);
    }
}

#[test]
fn test_revoke_already_revoked_certification() {
    let context = TestContext::setup();
    let client = context.client();

    let cert_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Revoke once
    context.env.mock_all_auths();
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert_id);

    // Try to revoke again
    let result = client.try_revoke_certification(&context.issuer1, &context.recipient1, &cert_id);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, RevokeError::AlreadyRevoked);
    }
}

#[test]
fn test_revoke_expired_certification() {
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

    // Advance time and expire
    context.advance_time(2000);
    client.expire_certification(&context.recipient1, &1);

    // Try to revoke expired cert
    let result = client.try_revoke_certification(&context.issuer1, &context.recipient1, &1);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, RevokeError::AlreadyRevoked);
    }
}

#[test]
fn test_certification_expiration() {
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

    // Verify initially valid
    let status = client.check_cert_status(&context.recipient1, &1);
    assert_eq!(status, CertStatus::Valid);

    // Advance time past expiration
    context.advance_time(2000);

    // Expire the certification
    client.expire_certification(&context.recipient1, &1);

    // Verify status updated
    let status = client.check_cert_status(&context.recipient1, &1);
    assert_eq!(status, CertStatus::Expired);
}

#[test]
fn test_expire_already_expired_certification() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 1000;
    let doc_hash = context.create_document_hash("Test document");

    // Issue and expire certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    context.advance_time(2000);
    client.expire_certification(&context.recipient1, &1);

    // Try to expire again
    let result = client.try_expire_certification(&context.recipient1, &1);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, CertificationError::AlreadyExpired);
    }
}

#[test]
fn test_expire_not_due_certification() {
    let context = TestContext::setup();
    let client = context.client();

    let cert_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Try to expire before expiration date
    context.env.mock_all_auths();
    let result = client.try_expire_certification(&context.recipient1, &cert_id);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, CertificationError::NotExpired);
    }
}

#[test]
fn test_authorization_required_for_revocation() {
    let context = TestContext::setup();
    let client = context.client();

    let cert_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);

    // Don't authorize - should fail
    context.env.set_auths(&[]);
    let result = client.try_revoke_certification(&context.issuer1, &context.recipient1, &cert_id);

    assert!(result.is_err());
}

#[test]
fn test_multiple_revocations_by_same_issuer() {
    let context = TestContext::setup();
    let client = context.client();

    // Issue multiple certs by same issuer
    let cert1_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "ORGANIC", 365);
    let cert2_id = context.issue_test_cert(&context.issuer1, &context.recipient1, "FAIRTRADE", 365);

    // Revoke both
    context.env.mock_all_auths();
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert1_id);
    client.revoke_certification(&context.issuer1, &context.recipient1, &cert2_id);

    // Verify both are revoked
    let status1 = client.check_cert_status(&context.recipient1, &cert1_id);
    let status2 = client.check_cert_status(&context.recipient1, &cert2_id);
    assert_eq!(status1, CertStatus::Revoked);
    assert_eq!(status2, CertStatus::Revoked);
}