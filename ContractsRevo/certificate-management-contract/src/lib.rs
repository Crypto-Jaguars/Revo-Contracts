#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Vec};

mod audit;
mod certification;
mod datatypes;
mod error;
mod initialize;
mod issue;
mod revoke;
mod verify;

pub use datatypes::*;
pub use error::{AdminError, AuditError, CertificationError, IssueError, RevokeError, VerifyError};

#[contract]
pub struct CertificateManagementContract;

#[contractimpl]
impl CertificateManagementContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
        initialize::initialize(env, admin)
    }

    pub fn issue_certification(
        env: Env,
        issuer: Address,
        recipient: Address,
        cert_type: Symbol,
        expiration_date: u64,
        verification_hash: soroban_sdk::BytesN<32>,
    ) -> Result<(), IssueError> {
        issue::issue_certification(
            env,
            issuer,
            recipient,
            cert_type,
            expiration_date,
            verification_hash,
        )
    }

    pub fn revoke_certification(
        env: Env,
        issuer: Address,
        owner: Address,
        id: u32,
    ) -> Result<(), RevokeError> {
        revoke::revoke_certification(env, issuer, owner, id)
    }

    pub fn expire_certification(
        env: Env,
        owner: Address,
        id: u32,
    ) -> Result<(), CertificationError> {
        certification::expire(env, owner, id)
    }

    pub fn verify_document_hash(
        env: Env,
        owner: Address,
        id: u32,
        submitted_hash: BytesN<32>,
    ) -> Result<(), VerifyError> {
        verify::verify_document_hash(env, owner, id, submitted_hash)
    }

    // GETTERS
    pub fn get_admin(env: Env) -> Result<Address, AdminError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AdminError::Uninitialized)
    }

    pub fn check_cert_status(
        env: Env,
        owner: Address,
        id: u32,
    ) -> Result<CertStatus, CertificationError> {
        certification::check_cert_status(env, owner, id)
    }

    pub fn get_cert(
        env: Env,
        owner: Address,
        id: u32,
    ) -> Result<Certification, CertificationError> {
        certification::get_cert(env, owner, id)
    }

    pub fn generate_cert_audit_report(
        env: Env,
        owner: Address,
        issuer: Option<Address>,
        status_filter: Option<CertStatus>,
        after_timestamp: Option<u64>,
    ) -> Result<Vec<Certification>, AuditError> {
        audit::generate_cert_audit_report(env, owner, issuer, status_filter, after_timestamp)
    }
}