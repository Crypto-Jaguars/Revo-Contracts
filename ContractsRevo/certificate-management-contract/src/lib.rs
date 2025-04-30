#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol};

mod audit;
mod certification;
mod datatype;
mod error;
mod initialize;
mod issue;
mod revoke;
mod verify;

pub use datatype::*;
pub use error::{AdminError, AuditError, CertificationError, IssueError, RevokeError, VerifyError};

#[contract]
pub struct CertificateManagement;

#[contractimpl]
impl CertificateManagement {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
        initialize::initialize(env, admin)
    }

    pub fn issue_certification(
        env: Env,
        cert_type: Symbol,
        recipient: soroban_sdk::Address,
        expiration_date: u64,
        verification_hash: soroban_sdk::BytesN<32>,
    ) -> Result<(), IssueError> {
        issue::issue_certification(
            env,
            cert_type,
            recipient,
            expiration_date,
            verification_hash,
        )
    }

    pub fn revoke_certification(env: Env, cert_id: u32) -> Result<(), RevokeError> {
        revoke::revoke_certification(env, cert_id)
    }

    pub fn check_cert_status(env: Env, cert_id: u32) -> Result<(), CertificationError> {
        certification::check_cert_status(env, cert_id)
    }

    pub fn verify_document_hash(
        env: Env,
        cert_id: u32,
        submitted_hash: soroban_sdk::BytesN<32>,
    ) -> Result<(), VerifyError> {
        verify::verify_document_hash(env, cert_id, submitted_hash)
    }

    pub fn generate_cert_audit_report(
        env: Env,
        issuer: soroban_sdk::Address,
    ) -> Result<(), AuditError> {
        audit::generate_cert_audit_report(env, issuer)
    }

    // GETTERS
    pub fn get_admin(env: Env) -> Result<Address, AdminError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AdminError::UnauthorizedAccess)
    }
}