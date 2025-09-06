use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger, LedgerInfo},
    vec, Address, Bytes, BytesN, Env, String, Symbol, TryFromVal, symbol_short,
};
use crate::{AgricQualityContract, AgricQualityContractClient, CertificationData, CertificationStatus, QualityStandard};
use certificate_management_contract::{CertificateManagementContract, CertificateManagementContractClient, CertStatus, Certification, DataKey};


pub fn setup_test<'a>() -> (
    Env,
    Address, // Contract ID
    AgricQualityContractClient<'a>,
    Address, // Admin
    Address, // Farmer 1
    Address, // Inspector
    Address, // Authority
) {
    let env = Env::default();
    env.mock_all_auths(); // Automatically approve all auth calls for convenience

    // Generate identities
    let admin = Address::generate(&env);
    let farmer1 = Address::generate(&env);
    let inspector = Address::generate(&env);
    let authority = Address::generate(&env);

    // Register the contract
    // Use register_contract for contracts, not register() which is for custom types
    let contract_id = env.register(AgricQualityContract, ());
    let client = AgricQualityContractClient::new(&env, &contract_id);

    // Initialize the contract
    client.initialize(&admin);

    (
        env,
        contract_id, // Return the contract Address (ID)
        client,
        admin,
        farmer1,
        inspector,
        authority,
    )
}


   
// Helper function to setup both contracts
pub fn setup_integration_test<'a>() -> (
    Env,
    Address, // AgricQualityContract ID
    AgricQualityContractClient<'a>,
    Address, // CertificationContract ID
    CertificateManagementContractClient<'a>,
    Address, // Admin
    Address, // Farmer
    Address, // Inspector
    Address, // Authority
) {
    let env = Env::default();
    env.mock_all_auths(); // Automatically approve all auth calls

    // Generate identities
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let inspector = Address::generate(&env);
    let authority = Address::generate(&env);

    // Register AgricQualityContract
    let agric_contract_id = env.register(AgricQualityContract, ());
    let agric_client = AgricQualityContractClient::new(&env, &agric_contract_id);
    agric_client.initialize(&admin);
    agric_client.add_authority(&admin, &authority);
    agric_client.add_inspector(&admin, &inspector);

    // Register CertificationContract
    let cert_contract_id = env.register(CertificateManagementContract, ());
    let cert_client = CertificateManagementContractClient::new(&env, &cert_contract_id);
    cert_client.initialize(&admin);

    (
        env,
        agric_contract_id,
        agric_client,
        cert_contract_id,
        cert_client,
        admin,
        farmer,
        inspector,
        authority,
    )
}

pub fn create_document_hash(env: &Env, content: &str) -> BytesN<32> {
    let bytes = soroban_sdk::Bytes::from_slice(&env, content.as_bytes());
    env.crypto().sha256(&bytes).into()
}

pub fn advance_time(env: &Env, seconds: u64) {
    let current_ts = env.ledger().timestamp();

    // Convert network_id to expected format
    let network_id = env.ledger().network_id();
    let network_id_array: [u8; 32] = network_id.into();

    env.ledger().set(LedgerInfo {
        timestamp: current_ts + seconds,
        protocol_version: env.ledger().protocol_version(),
        sequence_number: env.ledger().sequence(),
        network_id: network_id_array,
        base_reserve: 0,
        min_temp_entry_ttl: 0,
        min_persistent_entry_ttl: 0,
        max_entry_ttl: 0,
    });
}