use soroban_sdk::Env;

use crate::VerifyError;

pub fn verify_document_hash(
    env: Env,
    cert_id: u32,
    submitted_hash: soroban_sdk::BytesN<32>,
) -> Result<(), VerifyError> {
    todo!()
}