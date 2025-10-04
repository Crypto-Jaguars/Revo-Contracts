use soroban_sdk::{xdr::ToXdr, BytesN, Env, IntoVal, Val};

/// Contains shared utility functions for the contract.
pub mod utils {
    use super::*;

    /// Generates a unique 32-byte ID by hashing a tuple of input values.
    /// This provides a deterministic way to create IDs for programs and certificates.
    pub fn generate_id(env: &Env, inputs: impl IntoVal<Env, Val>) -> BytesN<32> {
        let val = inputs.into_val(env);
        let bytes = val.to_xdr(env);
        env.crypto().sha256(&bytes).into()
    }
}
