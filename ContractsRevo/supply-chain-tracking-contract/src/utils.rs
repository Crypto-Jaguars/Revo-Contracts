use crate::datatypes::{DataKey, Product, SupplyChainError};
use soroban_sdk::{xdr::ToXdr, Address, Bytes, BytesN, Env, String};

/// Generate unique product ID using hash of farmer, product type, batch, and timestamp
pub fn generate_product_id(
    env: &Env,
    farmer_id: &Address,
    product_type: &String,
    batch_number: &String,
) -> BytesN<32> {
    let mut data = Bytes::new(env);
    data.append(&farmer_id.to_xdr(env));

    // Convert Soroban String to Bytes using proper API
    data.append(&product_type.clone().to_xdr(env));
    data.append(&batch_number.clone().to_xdr(env));
    data.append(&Bytes::from_array(
        env,
        &env.ledger().timestamp().to_be_bytes(),
    ));

    env.crypto().sha256(&data).into()
}

/// Generate hash for stage data for off-chain verification
#[allow(dead_code)]
pub fn generate_stage_hash(
    env: &Env,
    stage_data: &String,
    timestamp: u64,
    handler: &Address,
) -> BytesN<32> {
    let mut data = Bytes::new(env);

    // Convert Soroban String to Bytes
    data.append(&stage_data.clone().to_xdr(env));
    data.append(&Bytes::from_array(env, &timestamp.to_be_bytes()));
    data.append(&handler.to_xdr(env));

    env.crypto().sha256(&data).into()
}

/// Calculate the complete supply chain hash for verification
pub fn calculate_supply_chain_hash(
    env: &Env,
    product_id: &BytesN<32>,
) -> Result<BytesN<32>, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    if product.stages.is_empty() {
        return Err(SupplyChainError::InvalidHash);
    }

    let mut combined_data = Bytes::new(env);
    combined_data.append(&Bytes::from_array(env, &product.product_id.to_array()));
    combined_data.append(&product.farmer_id.to_xdr(env));

    // Add all stage hashes to create a chain
    for stage in product.stages.iter() {
        combined_data.append(&Bytes::from_array(env, &stage.data_hash.to_array()));
        combined_data.append(&Bytes::from_array(env, &stage.timestamp.to_be_bytes()));
    }

    Ok(env.crypto().sha256(&combined_data).into())
}

/// Generate QR code data for consumer access
pub fn generate_qr_code_data(
    env: &Env,
    product_id: &BytesN<32>,
) -> Result<String, SupplyChainError> {
    // Verify product exists
    if !env
        .storage()
        .persistent()
        .has(&DataKey::Product(product_id.clone()))
    {
        return Err(SupplyChainError::ProductNotFound);
    }

    // Create QR code data with hex representation of product ID
    let hex_str = hex_encode(env, product_id.to_array());

    // Store QR mapping for resolution - use hex string as key
    env.storage()
        .persistent()
        .set(&DataKey::QRCodeMapping(hex_str.clone()), product_id);

    Ok(hex_str)
}

/// Resolve QR code to product ID
pub fn resolve_qr_code(env: &Env, qr_code: &String) -> Result<BytesN<32>, SupplyChainError> {
    env.storage()
        .persistent()
        .get(&DataKey::QRCodeMapping(qr_code.clone()))
        .ok_or(SupplyChainError::QRCodeNotFound)
}

/// Verify the hash chain integrity of a product's supply chain
pub fn verify_hash_chain(env: &Env, product_id: &BytesN<32>) -> Result<bool, SupplyChainError> {
    let product: Product = env
        .storage()
        .persistent()
        .get(&DataKey::Product(product_id.clone()))
        .ok_or(SupplyChainError::ProductNotFound)?;

    if product.stages.is_empty() {
        return Ok(false);
    }

    // Verify each stage hash is valid
    for stage in product.stages.iter() {
        // Basic validation: ensure hash is not zero
        if stage.data_hash.to_array().iter().all(|&x| x == 0) {
            return Ok(false);
        }
    }

    // Verify sequential stage IDs
    for (i, stage) in product.stages.iter().enumerate() {
        if stage.stage_id != (i as u32 + 1) {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Simple hex encoding helper
fn hex_encode(env: &Env, bytes: [u8; 32]) -> String {
    let hex_chars = b"0123456789abcdef";
    let mut result_bytes: [u8; 64] = [0; 64];

    for (i, byte) in bytes.iter().enumerate() {
        let high = (byte >> 4) as usize;
        let low = (byte & 0xF) as usize;
        result_bytes[i * 2] = hex_chars[high];
        result_bytes[i * 2 + 1] = hex_chars[low];
    }

    String::from_str(env, core::str::from_utf8(&result_bytes).unwrap())
}

/// Convert BytesN<32> certificate ID to u32 using deterministic hashing
pub fn convert_bytes_to_u32(env: &Env, certificate_id_bytes: &BytesN<32>) -> u32 {
    // Create a deterministic hash of the certificate ID
    let hash = env.crypto().sha256(&certificate_id_bytes.into());

    // Take the first 4 bytes of the hash and convert to u32
    let hash_bytes = hash.to_array();
    u32::from_be_bytes([hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3]])
}
