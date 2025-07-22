use soroban_sdk::{Address, BytesN, Env, Symbol};

use crate::datatypes::{DataKey, Product, SupplyChainError};
use crate::utils;

/// Validate product authenticity against recorded data and certifications
/// MANDATORY from roadmap.md
pub fn verify_authenticity(
    env: Env,
    product_id: BytesN<32>,
    verification_data: BytesN<32>,
) -> Result<bool, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    // Basic verification against stages data
    let is_authentic = verify_stages_integrity(&env, &product, &verification_data);

    // If certificate is linked, verify with certificate-management-contract
    // (ONLY certificate-management borrowing here)
    if let Some(cert_id) = product.certificate_id {
        return verify_certificate_link(&env, &cert_id, &product_id);
    }

    Ok(is_authentic)
}

/// Associate a product with a certification from certificate-management-contract
/// MANDATORY from roadmap.md - BORROWS from certificate-management
pub fn link_certificate(
    env: Env,
    product_id: BytesN<32>,
    certificate_id: BytesN<32>,
    authority: Address,
) -> Result<(), SupplyChainError> {
    authority.require_auth();

    // Get and update product
    let mut product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    // Validate certificate exists (following certificate-management pattern)
    if !validate_certificate_exists(&env, &certificate_id) {
        return Err(SupplyChainError::CertificateNotFound);
    }

    // Link certificate (following certificate-management pattern)
    product.certificate_id = Some(certificate_id.clone());

    // Store updated product
    env.storage()
        .persistent()
        .set(&DataKey::Product(product_id.clone()), &product);

    // Emit event (following certificate-management pattern)
    env.events().publish(
        (Symbol::new(&env, "certificate_linked"), authority),
        (product_id, certificate_id),
    );

    Ok(())
}

/// Validate certificate integrity with certificate-management-contract
/// EXTENDED functionality - Certificate management integration
pub fn validate_certificate_integrity(
    env: Env,
    certificate_id: BytesN<32>,
) -> Result<bool, SupplyChainError> {
    // This would call into certificate-management-contract to validate
    // For now, we'll do basic validation
    validate_certificate_exists(&env, &certificate_id)
        .then_some(true)
        .ok_or(SupplyChainError::CertificateNotFound)
}

/// Get linked certificate for a product
/// EXTENDED functionality
pub fn get_linked_certificate(
    env: Env,
    product_id: BytesN<32>,
) -> Result<Option<BytesN<32>>, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id))
        .ok_or(SupplyChainError::ProductNotFound)?;

    Ok(product.certificate_id)
}

/// Verify the integrity of all stages in a product's supply chain
/// Internal helper function
fn verify_stages_integrity(
    env: &Env,
    product: &Product,
    verification_data: &BytesN<32>,
) -> bool {
    if product.stages.is_empty() {
        return false;
    }

    // Calculate hash chain of all stages
    let calculated_hash = utils::calculate_supply_chain_hash(env, &product.product_id);
    
    match calculated_hash {
        Ok(hash) => hash == *verification_data,
        Err(_) => false,
    }
}

/// Verify certificate link with certificate-management-contract
/// Internal helper function - BORROWS from certificate-management
fn verify_certificate_link(
    env: &Env,
    certificate_id: &BytesN<32>,
    _product_id: &BytesN<32>,
) -> Result<bool, SupplyChainError> {
    // This would make a cross-contract call to certificate-management-contract
    // For now, we'll do basic validation
    if validate_certificate_exists(env, certificate_id) {
        Ok(true)
    } else {
        Err(SupplyChainError::CertificateNotFound)
    }
}

/// Basic certificate existence validation
/// Internal helper function - simulates certificate-management integration
fn validate_certificate_exists(_env: &Env, certificate_id: &BytesN<32>) -> bool {
    // In a real implementation, this would call the certificate-management-contract
    // For now, we'll assume any non-zero certificate ID is valid
    !certificate_id.to_array().iter().all(|&x| x == 0)
}