use crate::datatypes::{
    CertStatus, Certification, CertificationError, DataKey, Product, SupplyChainError, VerifyError,
    CERTIFICATE_MANAGEMENT_CONTRACT_KEY,
};
use crate::utils;
use soroban_sdk::{vec, Address, BytesN, Env, IntoVal, Symbol, Vec};

/// Validate product authenticity against recorded data and certifications
pub fn verify_authenticity(
    env: Env,
    farmer_id: Address,
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
    if let Some(cert_id) = product.certificate_id {
        return validate_certificate_hash(&env, &farmer_id, &cert_id, &verification_data);
    }

    Ok(is_authentic)
}

/// Associate a product with a certification from certificate-management-contract
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

    // Validate certificate exists
    if !verify_certificate_exists(&env, &product.farmer_id, &certificate_id)? {
        return Err(SupplyChainError::CertificateNotFound);
    }

    // Verify certificate status
    if !confirm_certificate_status_valid(&env, &product.farmer_id, &certificate_id)? {
        return Err(SupplyChainError::CertificateInvalid);
    }

    // Link certificate
    product.certificate_id = Some(certificate_id.clone());

    // Store updated product
    env.storage()
        .persistent()
        .set(&DataKey::Product(product_id.clone()), &product);

    env.events().publish(
        (Symbol::new(&env, "certificate_linked"), authority.clone()),
        (product_id.clone(), certificate_id.clone()),
    );

    Ok(())
}

/// Get linked certificate for a product
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
fn verify_stages_integrity(env: &Env, product: &Product, verification_data: &BytesN<32>) -> bool {
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

/// Verify certificate link with farmer (owner)
fn validate_certificate_hash(
    env: &Env,
    farmer_id: &Address,
    certificate_id: &BytesN<32>,
    verification_hash: &BytesN<32>,
) -> Result<bool, SupplyChainError> {
    // Verify certificate with
    if verify_certificate_exists(env, farmer_id, certificate_id)? {
        let cert_mgmt: Address = match env
            .storage()
            .instance()
            .get(&Symbol::new(env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY))
        {
            Some(addr) => addr,
            None => return Err(SupplyChainError::NotInitialized),
        };

        let args = vec![
            &env,
            farmer_id.into_val(env),
            certificate_id.into_val(env),
            verification_hash.into_val(env),
        ];
        let hash_status: Result<(), VerifyError> =
            env.invoke_contract(&cert_mgmt, &Symbol::new(env, "verify_document_hash"), args);

        match hash_status {
            Ok(_) => Ok(true),
            Err(_) => Err(SupplyChainError::VerificationHashInvalid),
        }
    } else {
        Err(SupplyChainError::CertificateNotFound)
    }
}

/// Confirm certificate status validity
fn confirm_certificate_status_valid(
    env: &Env,
    farmer_id: &Address,
    certificate_id: &BytesN<32>,
) -> Result<bool, SupplyChainError> {
    // Verify certificate exists
    let _cert_exists = match verify_certificate_exists(env, farmer_id, certificate_id) {
        Ok(_) => true,
        Err(_) => return Err(SupplyChainError::CertificateNotFound),
    };

    // Check certification status by interactinng with certification management contract
    let cert_mgmt: Address = match env
        .storage()
        .instance()
        .get(&Symbol::new(env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY))
    {
        Some(addr) => addr,
        None => return Err(SupplyChainError::NotInitialized),
    };

    let args = vec![&env, farmer_id.into_val(env), certificate_id.into_val(env)];
    let cert_status = env.invoke_contract(&cert_mgmt, &Symbol::new(env, "check_cert_status"), args);

    match cert_status {
        CertStatus::Valid => Ok(true),
        CertStatus::Expired => Err(SupplyChainError::CertificateInvalid),
        CertStatus::Revoked => Err(SupplyChainError::CertificateInvalid),
    }
}

/// Verify certificate existence validation
fn verify_certificate_exists(
    env: &Env,
    farmer_id: &Address,
    certificate_id: &BytesN<32>,
) -> Result<bool, SupplyChainError> {
    // Retrieve the certificate management contract address
    let cert_mgmt: Address = match env
        .storage()
        .instance()
        .get(&Symbol::new(env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY))
    {
        Some(addr) => addr,
        None => return Err(SupplyChainError::NotInitialized),
    };

    // Verify certificate existence by innvoking external contract
    match env.try_invoke_contract::<Certification, CertificationError>(
        &cert_mgmt,
        &Symbol::new(env, "get_cert"),
        Vec::from_array(env, [farmer_id.into_val(env), certificate_id.into_val(env)]),
    ) {
        Ok(_) => Ok(true),
        Err(_) => Err(SupplyChainError::CertificateNotFound),
    }
}
