#![cfg(test)]
use soroban_sdk::{
    symbol, Address, BytesN, Env, Symbol,
    testutils::{Address as _, Ledger, LedgerInfo},
};

use crate::{
    AuditError, CertificateManagementContract, CertificateManagementContractClient,
    CertStatus, Certification, CertificationError, IssueError, RevokeError, VerifyError,
};

struct TestContext {
    env: Env,
    contract_id: Address,
    admin: Address,
    issuer1: Address,
    issuer2: Address,
    recipient1: Address,
    recipient2: Address,
}

impl TestContext {
    fn setup() -> Self {
        let env = Env::default();
        let contract_id = env.register_contract(None, CertificateManagementContract);
        
        // Create test addresses
        let admin = Address::generate(&env);
        let issuer1 = Address::generate(&env);
        let issuer2 = Address::generate(&env);
        let recipient1 = Address::generate(&env);
        let recipient2 = Address::generate(&env);
        
        let client = CertificateManagementContractClient::new(&env, &contract_id);
        
        // Initialize contract
        env.mock_all_auths();
        client.initialize(&admin).expect("Failed to initialize contract");
        
        Self {
            env,
            contract_id,
            admin,
            issuer1,
            issuer2, 
            recipient1,
            recipient2,
        }
    }
    
    fn client(&self) -> CertificateManagementContractClient {
        CertificateManagementContractClient::new(&self.env, &self.contract_id)
    }
    
    fn create_document_hash(&self, content: &str) -> BytesN<32> {
        let bytes = soroban_sdk::Bytes::from_slice(&self.env, content.as_bytes());
        self.env.crypto().sha256(&bytes)
    }
    
    fn advance_time(&self, seconds: u64) {
        let current_ts = self.env.ledger().timestamp();
        
        // Convert network_id to expected format
        let network_id = self.env.ledger().network_id();
        let network_id_array: [u8; 32] = network_id.into();
        
        self.env.ledger().set(LedgerInfo {
            timestamp: current_ts + seconds,
            protocol_version: self.env.ledger().protocol_version(),
            sequence_number: self.env.ledger().sequence(),
            network_id: network_id_array,
            base_reserve: 0,
            min_temp_entry_ttl: 0,
            min_persistent_entry_ttl: 0,
            max_entry_ttl: 0
        });
    }
}

// Test initialization
#[test]
fn test_initialize() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Verify admin was set correctly
    let admin = client.get_admin().expect("Failed to get admin");
    assert_eq!(admin, context.admin);
}

// Test double initialization (should fail)
#[test]
fn test_double_initialization() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Try to initialize again
    context.env.mock_all_auths();
    let result = client.try_initialize(&context.admin);
    
    assert!(result.is_err());
}

// Test certification issuance
#[test]
fn test_certification_issuance() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Create test data
    let cert_type = symbol!("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000; // 1 year
    let doc_hash = context.create_document_hash("Organic certification document");
    
    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    ).expect("Failed to issue certification");
    
    // Verify certification exists and is valid
    let cert = client.get_cert(&context.recipient1, &0).expect("Failed to get certification");
    assert_eq!(cert.cert_type, cert_type);
    assert_eq!(cert.issuer, context.issuer1);
    assert_eq!(cert.status, CertStatus::Valid);
    assert_eq!(cert.expiration_date, expiration);
}

// Test document hash verification
#[test]
fn test_document_verification() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Create test data
    let cert_type = symbol!("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000; // 1 year
    let doc_hash = context.create_document_hash("Organic certification document");
    
    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    ).expect("Failed to issue certification");
    
    // Verify with correct hash
    let verify_result = client.verify_document_hash(
        &context.recipient1,
        &0,
        &doc_hash
    );
    assert!(verify_result.is_ok());
    
    // Verify with incorrect hash
    let wrong_hash = context.create_document_hash("Modified document");
    let verify_result = client.verify_document_hash(
        &context.recipient1,
        &0,
        &wrong_hash
    );
    assert!(verify_result.is_err());
    assert!(matches!(verify_result.unwrap_err(), VerifyError::HashMismatch));
}

// Test certification revocation
#[test]
fn test_certification_revocation() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Create test data
    let cert_type = symbol!("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000; // 1 year
    let doc_hash = context.create_document_hash("Organic certification document");
    
    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    ).expect("Failed to issue certification");
    
    // Verify initially valid
    let status = client.check_cert_status(&context.recipient1, &0)
        .expect("Failed to check certification status");
    assert_eq!(status, CertStatus::Valid);
    
    // Revoke certification
    context.env.mock_all_auths();
    client.revoke_certification(
        &context.issuer1,
        &context.recipient1,
        &0
    ).expect("Failed to revoke certification");
    
    // Verify status updated to Revoked
    let status = client.check_cert_status(&context.recipient1, &0)
        .expect("Failed to check certification status");
    assert_eq!(status, CertStatus::Revoked);
    
    // Verify a revoked certification cannot be verified
    let verify_result = client.verify_document_hash(
        &context.recipient1,
        &0,
        &doc_hash
    );
    assert!(verify_result.is_err());
    assert!(matches!(verify_result.unwrap_err(), VerifyError::Revoked));
}

// Test unauthorized revocation
#[test]
fn test_unauthorized_revocation() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Create test data
    let cert_type = symbol!("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000; // 1 year
    let doc_hash = context.create_document_hash("Organic certification document");
    
    // Issue certification by issuer1
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    ).expect("Failed to issue certification");
    
    // Attempt to revoke with issuer2 (unauthorized)
    context.env.mock_all_auths();
    let result = client.try_revoke_certification(
        &context.issuer2,
        &context.recipient1,
        &0
    );
    
    assert!(result.is_err());
    // Check for unauthorized error
    assert!(matches!(result.unwrap_err(), RevokeError::Unauthorized));
}

// Test certification expiration
#[test]
fn test_expiration_date_respected() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Create test data
    let cert_type = symbol!("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 1000; // Short expiration (1000 seconds)
    let doc_hash = context.create_document_hash("Organic certification document");
    
    // Issue certification
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    ).expect("Failed to issue certification");
    
    // Verify certification is valid
    let status = client.check_cert_status(&context.recipient1, &0)
        .expect("Failed to check certification status");
    assert_eq!(status, CertStatus::Valid);
    
    // Advance time past expiration
    context.advance_time(2000);
    
    // Expire the certification
    context.env.mock_all_auths();
    client.expire_certification(
        &context.recipient1,
        &0
    ).expect("Failed to expire certification");
    
    // Verify status updated to Expired
    let status = client.check_cert_status(&context.recipient1, &0)
        .expect("Failed to check certification status");
    assert_eq!(status, CertStatus::Expired);
    
    // Verify an expired certification cannot be verified
    let verify_result = client.verify_document_hash(
        &context.recipient1,
        &0,
        &doc_hash
    );
    assert!(verify_result.is_err());
    assert!(matches!(verify_result.unwrap_err(), VerifyError::Expired));
}

// Test audit report generation
#[test]
fn test_certification_audit() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Create multiple certifications
    let now = context.env.ledger().timestamp();
    
    // Organic certification for recipient1
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &symbol!("ORGANIC"),
        &(now + 31536000),
        &context.create_document_hash("Organic certification document")
    ).expect("Failed to issue organic certification");
    
    // Fair Trade certification for recipient1
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer2,
        &context.recipient1,
        &symbol!("FAIRTRADE"),
        &(now + 15768000),
        &context.create_document_hash("Fair Trade certification document")
    ).expect("Failed to issue fair trade certification");
    
    // Organic certification for recipient2
    context.env.mock_all_auths();
    client.issue_certification(
        &context.issuer1,
        &context.recipient2,
        &symbol!("ORGANIC"),
        &(now + 31536000),
        &context.create_document_hash("Organic certification document for recipient2")
    ).expect("Failed to issue second organic certification");
    
    // Revoke one certification
    context.env.mock_all_auths();
    client.revoke_certification(
        &context.issuer1,
        &context.recipient2,
        &0
    ).expect("Failed to revoke certification");
    
    // Test audit reports with different filters
    
    // All certifications for recipient1
    let recipient1_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &None,
        &None,
        &None
    ).expect("Failed to generate audit report");
    assert_eq!(recipient1_certs.len(), 2);
    
    // All certifications from issuer1
    let issuer1_certs = client.generate_cert_audit_report(
        &context.recipient1,
        &Some(context.issuer1.clone()),
        &None,
        &None
    ).expect("Failed to generate filtered audit report");
    assert_eq!(issuer1_certs.len(), 1);
    
    // All revoked certifications
    let revoked_certs = client.generate_cert_audit_report(
        &context.recipient2,
        &None,
        &Some(CertStatus::Revoked),
        &None
    ).expect("Failed to generate status-filtered audit report");
    assert_eq!(revoked_certs.len(), 1);
}

// Test verification of non-existent certification
#[test]
fn test_verify_nonexistent_certification() {
    let context = TestContext::setup();
    let client = context.client();
    
    let doc_hash = context.create_document_hash("Some document");
    
    let result = client.try_verify_document_hash(
        &context.recipient1,
        &999, // Non-existent ID
        &doc_hash
    );
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), VerifyError::CertNotFound));
}

// Test authorization is required
#[test]
fn test_authorization_required() {
    let context = TestContext::setup();
    let client = context.client();
    
    let cert_type = symbol!("ORGANIC");
    let now = context.env.ledger().timestamp();
    let expiration = now + 31536000;
    let doc_hash = context.create_document_hash("Organic certification document");
    
    // Don't authorize issuer1 - should fail
    context.env.set_auths(&[]);
    let auth_error = client.try_issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    );
    
    // Should fail due to missing authorization
    assert!(auth_error.is_err());
    
    // Now authorize properly and try again
    context.env.mock_all_auths();
    
    // Now it should work
    let result = client.issue_certification(
        &context.issuer1,
        &context.recipient1,
        &cert_type,
        &expiration,
        &doc_hash
    );
    
    assert!(result.is_ok());
} 