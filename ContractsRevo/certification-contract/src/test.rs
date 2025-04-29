#![cfg(test)]
use super::*;
use crate::datatypes::{CertificationData, CertificationError, CertificationType, Status};
use soroban_sdk::{symbol_short, vec, Bytes, BytesN};
use soroban_sdk::testutils::{Ledger, LedgerInfo, Address as _};

struct TestContext {
    env: Env,
    contract_id: Address,
    admin: Address,
    issuer1: Address,
    issuer2: Address,
    holder1: Address,
    holder2: Address,
}

impl TestContext {
    fn setup() -> Self {
        let env = Env::default();
        let contract_id = env.register_contract(None, CertificationContract);
        
        // Use the test utils to create addresses
        let admin = Address::generate(&env);
        let issuer1 = Address::generate(&env);
        let issuer2 = Address::generate(&env);
        let holder1 = Address::generate(&env);
        let holder2 = Address::generate(&env);
        
        let client = CertificationContractClient::new(&env, &contract_id);
        
        // Initialize contract
        env.mock_all_auths();
        let _ = client.initialize(&admin);
        
        Self {
            env,
            contract_id,
            admin,
            issuer1,
            issuer2, 
            holder1,
            holder2,
        }
    }
    
    fn client(&self) -> CertificationContractClient {
        CertificationContractClient::new(&self.env, &self.contract_id)
    }
    
    fn create_document_hash(&self, content: &str) -> BytesN<32> {
        let bytes = Bytes::from_slice(&self.env, content.as_bytes());
        self.env.crypto().sha256(&bytes).into()
    }
    
    fn add_verified_issuer(&self, issuer: &Address) {
        use super::issuance::add_verified_issuer;
        self.env.mock_all_auths();
        
        // Execute in contract context and ensure it succeeds
        let result = self.env.as_contract(&self.contract_id, || {
            add_verified_issuer(&self.env, &self.admin, issuer)
        });
        
        // Make sure the issuer was added successfully
        assert!(result.is_ok(), "Failed to add verified issuer in test setup");
    }
    
    fn advance_time(&self, seconds: u64) {
        let current_ts = self.env.ledger().timestamp();
        
        // Convert network_id to the expected format
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

// Test certification issuance with correct metadata
#[test]
fn test_certification_issuance() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x"), symbol_short!("acres"), symbol_short!("50")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now + 10;
    let valid_to = now + 31536000; // 1 year
    
    // Authorize issuer1
    context.env.mock_all_auths();
    
    // Issue certification
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Verify cert was issued correctly
    let cert = client.get_certification(&cert_id);
    assert_eq!(cert.issuer, context.issuer1);
    assert_eq!(cert.holder, context.holder1);
    assert_eq!(cert.certification_type, CertificationType::Organic);
    assert_eq!(cert.document_hash, document_hash);
    assert_eq!(cert.status, Status::Valid);
    assert_eq!(cert.valid_from, valid_from);
    assert_eq!(cert.valid_to, valid_to);
}

// Test certification issuance with invalid issuer
#[test]
fn test_certification_issuance_invalid_issuer() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Do not add issuer2 to verified list
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now + 10;
    let valid_to = now + 31536000; // 1 year
    
    // Authorize issuer2
    context.env.mock_all_auths();
    
    // Issue certification should fail
    let result = client.try_issue_certification(
        &context.issuer2,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::InvalidIssuer),
        Err(_) => panic!("Expected Ok(CertificationError::InvalidIssuer)"),
    }
}

// Test hash-based document verification
#[test]
fn test_document_hash_verification() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now;
    let valid_to = now + 31536000; // 1 year
    
    // Authorize issuer1
    context.env.mock_all_auths();
    
    // Issue certification
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Verify with correct hash
    let is_valid = client.verify_certification(&cert_id, &document_hash);
    assert!(is_valid);
    
    // Verify with incorrect hash
    let wrong_hash = context.create_document_hash("Modified document");
    let is_valid = client.verify_certification(&cert_id, &wrong_hash);
    assert!(!is_valid);
}

// Test expiration dates are respected
#[test]
fn test_expiration_date_respected() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now;
    let valid_to = now + 100; // Short validity of 100 seconds
    
    // Authorize issuer1
    context.env.mock_all_auths();
    
    // Issue certification
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Verify certificate is valid
    let is_valid = client.verify_certification(&cert_id, &document_hash);
    assert!(is_valid);
    
    // Advance time past expiration
    context.advance_time(200);
    
    // Verify certificate is now invalid due to expiration
    let is_valid = client.verify_certification(&cert_id, &document_hash);
    assert!(!is_valid);
    
    // Check if status was updated to Expired
    let cert = client.get_certification(&cert_id);
    assert_eq!(cert.status, Status::Expired);
}

// Test certification revocation
#[test]
fn test_certification_revocation() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now;
    let valid_to = now + 31536000; // 1 year
    
    // Authorize issuer1
    context.env.mock_all_auths();
    
    // Issue certification
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Verify initially valid
    let is_valid = client.verify_certification(&cert_id, &document_hash);
    assert!(is_valid);
    
    // Revoke certification
    let _ = client.revoke_certification(
        &context.issuer1,
        &cert_id,
        &symbol_short!("violation")
    );
    
    // Check if status was updated to Revoked
    let cert = client.get_certification(&cert_id);
    assert_eq!(cert.status, Status::Revoked);
    assert_eq!(cert.revocation_reason, Some(symbol_short!("violation")));
    
    // Verify certificate is now invalid due to revocation
    let is_valid = client.verify_certification(&cert_id, &document_hash);
    assert!(!is_valid);
}

// Test that unauthorized users cannot revoke certifications
#[test]
fn test_unauthorized_revocation() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer1 to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now;
    let valid_to = now + 31536000; // 1 year
    
    // Authorize issuer1
    context.env.mock_all_auths();
    
    // Issue certification by issuer1
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Add issuer2 to verified list too
    context.add_verified_issuer(&context.issuer2);
    
    // Attempt to revoke with issuer2 (unauthorized)
    let result = client.try_revoke_certification(
        &context.issuer2,
        &cert_id,
        &symbol_short!("fraud")
    );
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::UnauthorizedAccess),
        Err(_) => panic!("Expected Ok(CertificationError::UnauthorizedAccess)"),
    }
    
    // Certificate should still be valid
    let cert = client.get_certification(&cert_id);
    assert_eq!(cert.status, Status::Valid);
}

// Test attempting to revoke already revoked certification
#[test]
fn test_revoke_already_revoked() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now;
    let valid_to = now + 31536000; // 1 year
    
    // Authorize issuer1
    context.env.mock_all_auths();
    
    // Issue certification
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Revoke certification
    let _ = client.revoke_certification(
        &context.issuer1,
        &cert_id,
        &symbol_short!("violation")
    );
    
    // Try to revoke again
    let result = client.try_revoke_certification(
        &context.issuer1,
        &cert_id,
        &symbol_short!("fraud")
    );
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::InvalidStatus),
        Err(_) => panic!("Expected Ok(CertificationError::InvalidStatus)"),
    }
}

// Test auditing and verification
#[test]
fn test_certification_audit() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add both issuers to verified list
    context.add_verified_issuer(&context.issuer1);
    context.add_verified_issuer(&context.issuer2);
    
    // Create multiple certifications
    context.env.mock_all_auths();
    
    // Create an Organic certification
    let doc_hash1 = context.create_document_hash("Organic certification document");
    let metadata1 = vec![&context.env, symbol_short!("type"), symbol_short!("organic")];
    let now = context.env.ledger().timestamp();
    let valid_from1 = now;
    let valid_to1 = now + 31536000; // 1 year
    
    let cert_id1 = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &doc_hash1,
        &metadata1,
        &valid_from1,
        &valid_to1,
    );
    
    // Create a FairTrade certification
    let doc_hash2 = context.create_document_hash("Fair Trade certification document");
    let metadata2 = vec![&context.env, symbol_short!("type"), symbol_short!("fairtrade")];
    let valid_from2 = now;
    let valid_to2 = now + 15768000; // 6 months
    
    let _cert_id2 = client.issue_certification(
        &context.issuer2,
        &context.holder1,
        &CertificationType::FairTrade,
        &doc_hash2,
        &metadata2,
        &valid_from2,
        &valid_to2,
    );
    
    // Create another FairTrade certification for a different holder
    let doc_hash3 = context.create_document_hash("Fair Trade certification document 2");
    let metadata3 = vec![&context.env, symbol_short!("type"), symbol_short!("fairtrade")];
    let valid_from3 = now;
    let valid_to3 = now + 15768000; // 6 months
    
    let cert_id3 = client.issue_certification(
        &context.issuer2,
        &context.holder2,
        &CertificationType::FairTrade,
        &doc_hash3,
        &metadata3,
        &valid_from3,
        &valid_to3,
    );
    
    // Revoke one certification
    let _ = client.revoke_certification(
        &context.issuer2,
        &cert_id3,
        &symbol_short!("violation")
    );
    
    // Now test various audit report filters
    
    // Get all certifications for holder1
    let holder1_certs = client.get_holder_certifications(&context.holder1);
    assert_eq!(holder1_certs.len(), 2);
    
    // Generate audit report for Organic certifications
    let organic_certs = client.generate_audit_report(
        &Some(CertificationType::Organic),
        &None,
        &None
    );
    assert_eq!(organic_certs.len(), 1);
    assert_eq!(organic_certs.get(0).unwrap().certification_id, cert_id1);
    
    // Generate audit report for FairTrade certifications
    let fairtrade_certs = client.generate_audit_report(
        &Some(CertificationType::FairTrade),
        &None,
        &None
    );
    assert_eq!(fairtrade_certs.len(), 2);
    
    // Generate audit report for issuer2
    let issuer2 = context.issuer2.clone();
    let issuer2_certs = client.generate_audit_report(
        &None,
        &Some(issuer2),
        &None
    );
    assert_eq!(issuer2_certs.len(), 2);
    
    // Generate audit report for revoked certifications
    let revoked_certs = client.generate_audit_report(
        &None,
        &None,
        &Some(Status::Revoked)
    );
    assert_eq!(revoked_certs.len(), 1);
    assert_eq!(revoked_certs.get(0).unwrap().certification_id, cert_id3);
    
    // Generate audit report for valid certifications
    let valid_certs = client.generate_audit_report(
        &None,
        &None,
        &Some(Status::Valid)
    );
    assert_eq!(valid_certs.len(), 2);
}

// Test that invalid inputs are handled properly
#[test]
fn test_invalid_inputs() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    
    context.env.mock_all_auths();
    
    // Test invalid validity period (valid_to before valid_from)
    let valid_from = now + 200;
    let valid_to = now + 100;
    let result = client.try_issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::InvalidValidity),
        Err(_) => panic!("Expected Ok(CertificationError::InvalidValidity)"),
    }
    
    // Test empty metadata
    let empty_metadata = vec![&context.env];
    let valid_from = now;
    let valid_to = now + 31536000;
    let result = client.try_issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &empty_metadata,
        &valid_from,
        &valid_to,
    );
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::InvalidMetadata),
        Err(_) => panic!("Expected Ok(CertificationError::InvalidMetadata)"),
    }
}

// Test verification of non-existent certification
#[test]
fn test_verify_nonexistent_certification() {
    let context = TestContext::setup();
    let client = context.client();
    
    let fake_cert_id = BytesN::from_array(&context.env, &[0u8; 32]);
    let document_hash = context.create_document_hash("Some document");
    
    let result = client.try_verify_certification(&fake_cert_id, &document_hash);
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::CertificationNotFound),
        Err(_) => panic!("Expected Ok(CertificationError::CertificationNotFound)"),
    }
}

// Test initializing contract multiple times fails
#[test]
fn test_double_initialization() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Try to initialize again
    context.env.mock_all_auths();
    let result = client.try_initialize(&context.admin);
    
    assert!(result.is_err());
    match result.err().unwrap() {
        Ok(e) => assert_eq!(e, CertificationError::AlreadyInitialized),
        Err(_) => panic!("Expected Ok(CertificationError::AlreadyInitialized)"),
    }
}

// Test authorization is required
#[test]
fn test_authorization_required() {
    let context = TestContext::setup();
    let client = context.client();
    
    // Add issuer to verified list
    context.add_verified_issuer(&context.issuer1);
    
    let document_hash = context.create_document_hash("Organic certification document for farm X");
    let metadata = vec![&context.env, symbol_short!("location"), symbol_short!("farm_x")];
    
    let now = context.env.ledger().timestamp();
    let valid_from = now;
    let valid_to = now + 31536000; // 1 year
    
    // Don't authorize issuer1 - should fail
    context.env.set_auths(&[]);
    let auth_error = client.try_issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    // Should fail due to missing authorization
    assert!(auth_error.is_err());
    
    // Now authorize properly and try again
    context.env.mock_all_auths();
    
    // Now it should work
    let cert_id = client.issue_certification(
        &context.issuer1,
        &context.holder1,
        &CertificationType::Organic,
        &document_hash,
        &metadata,
        &valid_from,
        &valid_to,
    );
    
    assert!(!cert_id.to_array().is_empty());
} 