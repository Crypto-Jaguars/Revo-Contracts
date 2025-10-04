#![cfg(test)]
use soroban_sdk::{
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, BytesN, Env, Symbol,
};

use crate::{CertificateManagementContract, CertificateManagementContractClient};

pub struct TestContext {
    pub env: Env,
    pub contract_id: Address,
    pub admin: Address,
    pub issuer1: Address,
    pub issuer2: Address,
    pub recipient1: Address,
    pub recipient2: Address,
}

impl TestContext {
    pub fn setup() -> Self {
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
        client.initialize(&admin);

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

    pub fn client(&'_ self) -> CertificateManagementContractClient<'_> {
        CertificateManagementContractClient::new(&self.env, &self.contract_id)
    }

    pub fn create_document_hash(&self, content: &str) -> BytesN<32> {
        let bytes = soroban_sdk::Bytes::from_slice(&self.env, content.as_bytes());
        self.env.crypto().sha256(&bytes).into()
    }

    pub fn symbol(&self, s: &str) -> Symbol {
        Symbol::new(&self.env, s)
    }

    pub fn advance_time(&self, seconds: u64) {
        let current_ts = self.env.ledger().timestamp();
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
            max_entry_ttl: 0,
        });
    }

    pub fn issue_test_cert(
        &self,
        issuer: &Address,
        recipient: &Address,
        cert_type: &str,
        days_valid: u64,
    ) -> u32 {
        let client = self.client();
        let now = self.env.ledger().timestamp();
        let expiration = now + (days_valid * 86400); // Convert days to seconds

        // Create a simple document content
        let doc_content = match cert_type {
            "ORGANIC" => "Organic certification document",
            "FAIRTRADE" => "Fair trade certification document",
            "NON_GMO" => "Non-GMO certification document",
            _ => "Generic certification document",
        };
        let doc_hash = self.create_document_hash(doc_content);

        self.env.mock_all_auths();
        client.issue_certification(
            issuer,
            recipient,
            &self.symbol(cert_type),
            &expiration,
            &doc_hash,
        );

        // Get the cert count to determine the ID
        let count = client
            .generate_cert_audit_report(recipient, &None, &None, &None)
            .len();
        count as u32
    }
}
