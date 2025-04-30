#![cfg(test)]
use super::*;

use soroban_sdk::{
    testutils::{Address as _, Events, Ledger},
    Address, Env,
};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(CertificateManagement, ());
    let client = CertificateManagementClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    env.mock_all_auths();

    // Initialize the contract
    client.initialize(&admin);

    // Verify admin was set correctly
    assert_eq!(client.get_admin(), admin);
}

#[test]
fn test_issue_certificate() {
    // Set up the test environment
    let env = Env::default();
    let contract_id = env.register(CertificateManagement, ());
    let client = CertificateManagementClient::new(&env, &contract_id);

    // Set up the test users
    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Set up the test data
    let cert_type = Symbol::new(&env, "TestCertificate");
    let expiration_date = env.ledger().timestamp() + 86400; // 1 day from now
    let verification_hash = BytesN::from_array(&env, &[0; 32]);

    // Authorize the transaction
    env.mock_all_auths();

    client.issue_certification(
        &issuer,
        &recipient,
        &cert_type,
        &expiration_date,
        &verification_hash,
    );

    // Check if the event was emitted
    let events = env.events().all();
    assert_eq!(
        events.len(),
        1,
        "Should emit exactly one certification issuance event"
    );

    // Verify event details
    let event = events.get(0).unwrap();
    assert_eq!(event.0, contract_id, "Event should be from contract");
    assert!(!event.2.is_void(), "Event data should not be empty");

    let certification = client.get_cert(&recipient, &1);

    // assertions
    assert_eq!(certification.issuer, issuer);
    assert_eq!(certification.cert_type, cert_type);
    assert_eq!(certification.expiration_date, expiration_date);
}

#[test]
fn test_revoke_certification() {
    let env = Env::default();
    let contract_id = env.register(CertificateManagement, ());
    let client = CertificateManagementClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Issue a certificate first
    let cert_type = Symbol::new(&env, "TestCertificate");
    let expiration_date = env.ledger().timestamp() + 86400;
    let verification_hash = BytesN::from_array(&env, &[0; 32]);

    env.mock_all_auths();

    client.issue_certification(
        &issuer,
        &recipient,
        &cert_type,
        &expiration_date,
        &verification_hash,
    );

    // Revoke the certificate
    client.revoke_certification(&issuer, &recipient, &1);

    // Check certificate status
    let status = client.check_cert_status(&recipient, &1);
    assert_eq!(status, CertStatus::Revoked);
}

#[test]
fn test_expire_certification() {
    let env = Env::default();
    let contract_id = env.register(CertificateManagement, ());
    let client = CertificateManagementClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Issue a certificate
    let cert_type = Symbol::new(&env, "TestCertificate");
    let expiration_date = env.ledger().timestamp() + 100; // Short expiration
    let verification_hash = BytesN::from_array(&env, &[0; 32]);

    env.mock_all_auths();

    client.issue_certification(
        &issuer,
        &recipient,
        &cert_type,
        &expiration_date,
        &verification_hash,
    );

    // Advance time beyond expiration
    env.ledger().set_timestamp(expiration_date + 1);

    // Expire the certificate
    client.expire_certification(&recipient, &1);

    // Check certificate status
    let status = client.check_cert_status(&recipient, &1);
    assert_eq!(status, CertStatus::Expired);
}

#[test]
fn test_verify_document_hash() {
    let env = Env::default();
    let contract_id = env.register(CertificateManagement, ());
    let client = CertificateManagementClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Create verification hash
    let verification_hash = BytesN::from_array(&env, &[1; 32]);

    // Issue a certificate
    let cert_type = Symbol::new(&env, "TestCertificate");
    let expiration_date = env.ledger().timestamp() + 86400;

    env.mock_all_auths();

    client.issue_certification(
        &issuer,
        &recipient,
        &cert_type,
        &expiration_date,
        &verification_hash,
    );

    // Verify correct hash
    let result = client.try_verify_document_hash(&recipient, &1, &verification_hash);
    assert!(result.is_ok());

    // Verify incorrect hash
    let wrong_hash = BytesN::from_array(&env, &[2; 32]);
    let result = client.try_verify_document_hash(&recipient, &1, &wrong_hash);
    assert!(result.is_err());
}

#[test]
fn test_generate_cert_audit_report() {
    let env = Env::default();
    let contract_id = env.register(CertificateManagement, ());
    let client = CertificateManagementClient::new(&env, &contract_id);

    let issuer = Address::generate(&env);
    let recipient = Address::generate(&env);

    // Issue multiple certificates
    let cert_type = Symbol::new(&env, "TestCertificate");
    let expiration_date = env.ledger().timestamp() + 86400;
    let verification_hash = BytesN::from_array(&env, &[0; 32]);

    env.mock_all_auths();

    // Issue first certificate
    client.issue_certification(
        &issuer,
        &recipient,
        &cert_type,
        &expiration_date,
        &verification_hash,
    );

    // Issue second certificate
    client.issue_certification(
        &issuer,
        &recipient,
        &cert_type,
        &expiration_date,
        &verification_hash,
    );

    // Generate audit report
    let report = client.generate_cert_audit_report(&recipient, &Some(issuer.clone()), &None, &None);

    // Verify report contains both certificates
    assert_eq!(report.len(), 2);

    report.iter().for_each(|cert| {
        assert_eq!(&cert.issuer, &issuer);
        assert_eq!(cert.cert_type, cert_type);
        assert_eq!(cert.expiration_date, expiration_date);
    });
}