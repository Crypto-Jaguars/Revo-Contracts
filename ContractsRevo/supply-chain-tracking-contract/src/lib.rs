#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol, Vec};

mod datatypes;
mod product;
mod tracking;
mod utils;
mod validation;

#[cfg(test)]
mod test;

pub use datatypes::*;

#[contract]
pub struct SupplyChainTrackingContract;

#[contractimpl]
impl SupplyChainTrackingContract {
    // ========== ADMIN FUNCTIONS ==========

    /// Initialize the contract with an admin and certificate management contract address
    pub fn initialize(
        env: Env,
        admin: Address,
        cert_management_contract: Address,
    ) -> Result<(), SupplyChainError> {
        admin.require_auth();

        // Check if already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(SupplyChainError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);

        // Store certificate management contract address for production cross-contract calls
        env.storage().instance().set(
            &Symbol::new(&env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY),
            &cert_management_contract,
        );

        // Emit initialization event
        env.events().publish(
            (Symbol::new(&env, "contract_initialized"), admin.clone()),
            cert_management_contract.clone(),
            // env.ledger().timestamp(),
        );

        Ok(())
    }

    /// Set or update the certificate management contract address (admin only)
    pub fn set_cert_mgmt_contract(
        env: Env,
        admin: Address,
        cert_management_contract: Address,
    ) -> Result<(), SupplyChainError> {
        admin.require_auth();

        // Verify admin
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SupplyChainError::NotInitialized)?;

        if admin != stored_admin {
            return Err(SupplyChainError::UnauthorizedAccess);
        }

        env.storage().instance().set(
            &Symbol::new(&env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY),
            &cert_management_contract,
        );

        // Emit configuration event
        env.events().publish(
            (Symbol::new(&env, "cert_contract_configured"), admin),
            cert_management_contract,
        );

        Ok(())
    }

    /// Get the certificate management contract address
    pub fn get_cert_mgmt_contract(env: Env) -> Result<Address, SupplyChainError> {
        env.storage()
            .instance()
            .get(&Symbol::new(&env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY))
            .ok_or(SupplyChainError::NotInitialized)
    }

    /// Get the contract admin
    pub fn get_admin(env: Env) -> Result<Address, SupplyChainError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(SupplyChainError::NotInitialized)
    }

    // ========== CORE FUNCTIONS ==========

    /// Register a new agricultural product with initial details
    pub fn register_product(
        env: Env,
        farmer_id: Address,
        product_type: String,
        batch_number: String,
        origin_location: String,
        metadata_hash: BytesN<32>,
    ) -> Result<BytesN<32>, SupplyChainError> {
        product::register_product(
            env,
            farmer_id,
            product_type,
            batch_number,
            origin_location,
            metadata_hash,
        )
    }

    /// Record a new stage in the product's lifecycle with tier validation
    pub fn add_stage(
        env: Env,
        product_id: BytesN<32>,
        stage_tier: StageTier,
        stage_name: String,
        location: String,
        handler: Address,
        data_hash: BytesN<32>,
    ) -> Result<u32, SupplyChainError> {
        tracking::add_stage(
            env, product_id, stage_tier, stage_name, location, handler, data_hash,
        )
    }

    /// Retrieve the full lifecycle of a product
    pub fn get_product_trace(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<(Product, Vec<Stage>), SupplyChainError> {
        tracking::get_product_trace(env, product_id)
    }

    /// Validate product authenticity against recorded data and certifications
    pub fn verify_authenticity(
        env: Env,
        farmer_id: Address,
        product_id: BytesN<32>,
        verification_data: BytesN<32>,
    ) -> Result<bool, SupplyChainError> {
        validation::verify_authenticity(env, farmer_id, product_id, verification_data)
    }

    /// Associate a product with a certification from certificate-management-contract
    pub fn link_certificate(
        env: Env,
        product_id: BytesN<32>,
        certificate_id: CertificateId,
        authority: Address,
    ) -> Result<(), SupplyChainError> {
        validation::link_certificate(env, product_id, certificate_id, authority)
    }

    // ========== ADDITIONAL FUNCTIONS ==========

    /// Get detailed information about a specific product
    pub fn get_product_details(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<Product, SupplyChainError> {
        product::get_product_details(env, product_id)
    }

    /// Get product registration details
    pub fn get_product_registration(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<ProductRegistration, SupplyChainError> {
        product::get_product_registration(env, product_id)
    }

    /// List all products for a specific farmer
    pub fn list_products_by_farmer(
        env: Env,
        farmer_id: Address,
    ) -> Result<Vec<BytesN<32>>, SupplyChainError> {
        product::list_products_by_farmer(env, farmer_id)
    }

    /// List products by product type for traceability
    pub fn list_products_by_type(
        env: Env,
        product_type: String,
    ) -> Result<Vec<BytesN<32>>, SupplyChainError> {
        product::list_products_by_type(env, product_type)
    }

    /// Validate stage transition logic
    pub fn validate_stage_transition(
        env: Env,
        product_id: BytesN<32>,
        from_stage: u32,
        to_stage: u32,
    ) -> Result<bool, SupplyChainError> {
        tracking::validate_stage_transition(env, product_id, from_stage, to_stage)
    }

    /// Get the current stage of a product
    pub fn get_current_stage(env: Env, product_id: BytesN<32>) -> Result<Stage, SupplyChainError> {
        tracking::get_current_stage(env, product_id)
    }

    /// Get complete stage history for a product
    pub fn get_stage_history(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<Vec<Stage>, SupplyChainError> {
        tracking::get_stage_history(env, product_id)
    }

    /// Get a specific stage by ID
    pub fn get_stage_by_id(
        env: Env,
        product_id: BytesN<32>,
        stage_id: u32,
    ) -> Result<Stage, SupplyChainError> {
        tracking::get_stage_by_id(env, product_id, stage_id)
    }

    /// Get the next expected tier for a product
    pub fn get_next_expected_tier(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<Option<StageTier>, SupplyChainError> {
        tracking::get_next_expected_tier(env, product_id)
    }

    /// Get the current tier for a product
    pub fn get_current_tier(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<Option<StageTier>, SupplyChainError> {
        tracking::get_current_tier(env, product_id)
    }

    /// Get product trace using QR code
    pub fn trace_by_qr_code(
        env: Env,
        qr_code: String,
    ) -> Result<(Product, Vec<Stage>), SupplyChainError> {
        let product_id = utils::resolve_qr_code(&env, &qr_code)?;
        tracking::get_product_trace(env, product_id)
    }

    /// Get linked certificate for a product
    pub fn get_linked_certificate(
        env: Env,
        product_id: BytesN<32>,
    ) -> Result<CertificateId, SupplyChainError> {
        validation::get_linked_certificate(env, product_id)
    }

    /// Verify the hash chain integrity of a product's supply chain
    pub fn verify_hash_chain(env: Env, product_id: BytesN<32>) -> Result<bool, SupplyChainError> {
        utils::verify_hash_chain(&env, &product_id)
    }

    /// Generate QR code data for consumer access to traceability
    pub fn generate_qr_code(env: Env, product_id: BytesN<32>) -> Result<String, SupplyChainError> {
        utils::generate_qr_code_data(&env, &product_id)
    }
}
