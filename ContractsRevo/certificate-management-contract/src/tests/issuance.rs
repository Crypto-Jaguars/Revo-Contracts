#![cfg(test)]
use crate::{
    tests::utils::TestContext,
    CertStatus, AdminError, IssueError,
};

#[test]
fn test_contract_initialization() {
    let context = TestContext::setup();
    let client = context.client();

    // Verify admin was set correctly
    let admin = client.get_admin();
    assert_eq!(admin, context.admin);
}

#[test]
fn test_double_initialization_fails() {
    let context = TestContext::setup();
    let client = context.client();

    // Try to initialize again
    context.env.mock_all_auths();
    let result = client.try_initialize(&context.admin);

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, AdminError::AlreadyInitialized);
    }
}

#[test]
fn test_basic_certification_issuance() {
    let context = TestContext::setup();
    let client = context.client();

    let cert_type = context.symbol("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000; // 1 year
    let doc_hash = context.create_document_hash("Organic certification document");

    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash,
    );

    let cert = client.get_cert(&context.recipient1, &1);
    assert_eq!(cert.id, 1);
    assert_eq!(cert.cert_type, cert_type);
    assert_eq!(cert.issuer, context.issuer1);
    assert_eq!(cert.status, CertStatus::Valid);
    assert_eq!(cert.verification_hash, doc_hash);
}

#[test]
fn test_multiple_certifications_for_same_user() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;

    // Issue first certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &context.create_document_hash("Organic cert"),
    );

    // Issue second certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("FAIRTRADE"),
        &expiration,
        &context.create_document_hash("Fair trade cert"),
    );

    // Verify both certifications exist
    let cert1 = client.get_cert(&context.recipient1, &1);
    let cert2 = client.get_cert(&context.recipient1, &2);

    assert_eq!(cert1.cert_type, context.symbol("ORGANIC"));
    assert_eq!(cert2.cert_type, context.symbol("FAIRTRADE"));
    assert_eq!(cert1.id, 1);
    assert_eq!(cert2.id, 2);
}

#[test]
fn test_invalid_expiration_date() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let past_expiration = if now > 3600 { now - 3600 } else { 0 }; // 1 hour ago or 0 if would underflow
    let doc_hash = context.create_document_hash("Test document");

    context.env.mock_all_auths();
    let result = client.try_issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &past_expiration,
        &doc_hash,
    );

    assert!(result.is_err());
    if let Err(Ok(e)) = result {
        assert_eq!(e, IssueError::InvalidExpirationDate);
    }
}

#[test]
fn test_certification_uniqueness_across_users() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;

    // Issue certification to recipient1
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &context.create_document_hash("Organic cert 1"),
    );

    // Issue certification to recipient2
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient2,
        &context.symbol("ORGANIC"),
        &expiration,
        &context.create_document_hash("Organic cert 2"),
    );

    // Both should have ID 1 (unique per user)
    let cert1 = client.get_cert(&context.recipient1, &1);
    let cert2 = client.get_cert(&context.recipient2, &1);

    assert_eq!(cert1.id, 1);
    assert_eq!(cert2.id, 1);
    assert_ne!(cert1.verification_hash, cert2.verification_hash);
}

#[test]
fn test_authorization_required_for_issuance() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Test document");

    // Don't authorize - should fail
    context.env.set_auths(&[]);
    let result = client.try_issue_certification(
        &context.issuer1,
        &context.recipient1,
        &context.symbol("ORGANIC"),
        &expiration,
        &doc_hash,
    );

    assert!(result.is_err());
}

#[test]
fn test_different_cert_types() {
    let context = TestContext::setup();
    let client = context.client();

    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let _cert_types = ["ORGANIC", "FAIRTRADE", "NON_GMO", "RAINFOREST_ALLIANCE"];

    context.env.mock_all_auths();
    
    let cert_types = ["ORGANIC", "FAIRTRADE", "NON_GMO", "RAINFOREST_ALLIANCE"];
    let cert_docs = [
        "Organic certification",
        "Fair trade certification", 
        "Non-GMO certification",
        "Rainforest Alliance certification"
    ];

    for (index, (cert_type, doc_content)) in cert_types.iter().zip(cert_docs.iter()).enumerate() {
        client.issue_certification(
            &context.issuer1,
            &context.recipient1,
            &context.symbol(cert_type),
            &expiration,
            &context.create_document_hash(doc_content),
        );

        let cert = client.get_cert(&context.recipient1, &((index + 1) as u32));
        assert_eq!(cert.cert_type, context.symbol(cert_type));
    }
}