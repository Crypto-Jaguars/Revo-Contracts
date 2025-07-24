use crate::datatypes::{
    CertStatus, CertificateId, Certification, CertificationError, DataKey, Product,
    SupplyChainError, VerifyError, CERTIFICATE_MANAGEMENT_CONTRACT_KEY,
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
    if let CertificateId::Some(cert_id) = product.certificate_id {
        return validate_certificate_hash(&env, &farmer_id, &cert_id, &verification_data);
    }

    Ok(is_authentic)
}

/// Associate a product with a certification from certificate-management-contract
pub fn link_certificate(
    env: Env,
    product_id: BytesN<32>,
    certificate_id: CertificateId,
    authority: Address,
) -> Result<(), SupplyChainError> {
    authority.require_auth();

    // Get and update product
    let mut product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    let cert_bytes = match &certificate_id {
        CertificateId::Some(bytes) => bytes,
        CertificateId::None => return Err(SupplyChainError::CertificateInvalid),
    };

    // Validate certificate exists
    if !verify_certificate_exists(&env, &product.farmer_id, cert_bytes)? {
        return Err(SupplyChainError::CertificateNotFound);
    }

    // Verify certificate status
    if !confirm_certificate_status_valid(&env, &product.farmer_id, cert_bytes)? {
        return Err(SupplyChainError::CertificateInvalid);
    }

    // Link certificate
    product.certificate_id = CertificateId::Some(cert_bytes.clone());

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
) -> Result<CertificateId, SupplyChainError> {
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
    certificate_id_bytes: &BytesN<32>,
    verification_hash: &BytesN<32>,
) -> Result<bool, SupplyChainError> {
    let cert_mgmt: Address = match env
        .storage()
        .instance()
        .get(&Symbol::new(env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY))
    {
        Some(addr) => addr,
        None => return Err(SupplyChainError::NotInitialized),
    };

    // Convert BytesN<32> to u32 using deterministic hash-based approach
    let cert_id_u32 = utils::convert_bytes_to_u32(env, certificate_id_bytes);

    let args = vec![
        &env,
        farmer_id.into_val(env),
        cert_id_u32.into_val(env),
        verification_hash.into_val(env),
    ];

    // Use try_invoke_contract to properly handle the VerifyError
    match env.try_invoke_contract::<(), VerifyError>(
        &cert_mgmt,
        &Symbol::new(env, "verify_document_hash"),
        args,
    ) {
        Ok(_) => Ok(true),
        Err(Ok(verify_error)) => match verify_error {
            VerifyError::HashMismatch => Err(SupplyChainError::VerificationHashInvalid),
            VerifyError::NotFound => Err(SupplyChainError::CertificateNotFound),
            VerifyError::Expired => Err(SupplyChainError::CertificateInvalid),
            VerifyError::Revoked => Err(SupplyChainError::CertificateInvalid),
            VerifyError::ExpirationDue => Err(SupplyChainError::CertificateInvalid),
        },
        Err(Err(_)) => Err(SupplyChainError::CertificateNotFound), // InvokeError fallback
    }
}

/// Confirm certificate status validity
fn confirm_certificate_status_valid(
    env: &Env,
    farmer_id: &Address,
    cert_id_bytes: &BytesN<32>,
) -> Result<bool, SupplyChainError> {
    // Check certification status by interacting with certification management contract
    let cert_mgmt: Address = match env
        .storage()
        .instance()
        .get(&Symbol::new(env, CERTIFICATE_MANAGEMENT_CONTRACT_KEY))
    {
        Some(addr) => addr,
        None => return Err(SupplyChainError::NotInitialized),
    };

    // Convert BytesN<32> to u32 using deterministic hash-based approach
    let cert_id_u32 = utils::convert_bytes_to_u32(env, cert_id_bytes);

    let args = vec![&env, farmer_id.into_val(env), cert_id_u32.into_val(env)];
    
    // Invoke cetificate management contract and validate certificate status
    match env.invoke_contract::<CertStatus>(
        &cert_mgmt,
        &Symbol::new(env, "check_cert_status"),
        args,
    ) {
        cert_status => match cert_status {
            CertStatus::Valid => Ok(true),
            CertStatus::Expired => Err(SupplyChainError::CertificateInvalid),
            CertStatus::Revoked => Err(SupplyChainError::CertificateInvalid),
        },
    }
}

/// Verify certificate existence validation
fn verify_certificate_exists(
    env: &Env,
    farmer_id: &Address,
    certificate_id_bytes: &BytesN<32>,
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

    // Convert BytesN<32> to u32 using deterministic hash-based approach
    let cert_id_u32 = utils::convert_bytes_to_u32(env, certificate_id_bytes);

    // Verify certificate existence by invoking external contract
    match env.try_invoke_contract::<Certification, CertificationError>(
        &cert_mgmt,
        &Symbol::new(env, "get_cert"),
        Vec::from_array(env, [farmer_id.into_val(env), cert_id_u32.into_val(env)]),
    ) {
        Ok(_) => Ok(true),
        Err(_) => Err(SupplyChainError::CertificateNotFound),
    }
}
