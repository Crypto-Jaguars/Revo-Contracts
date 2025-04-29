#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Vec};

use crate::datatypes::{CertificationData, CertificationError, CertificationType, DataKey, Status};

mod datatypes;
mod issuance;
mod validation;
mod audit;

#[cfg(test)]
mod test;

#[contract]
pub struct CertificationContract;

#[contractimpl]
impl CertificationContract {
    // Initialize the contract with admin
    pub fn initialize(env: Env, admin: Address) -> Result<(), CertificationError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(CertificationError::AlreadyInitialized);
        }
        
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        Ok(())
    }
    
    // Issue a new certification
    pub fn issue_certification(
        env: Env,
        issuer: Address,
        holder: Address,
        certification_type: CertificationType,
        document_hash: BytesN<32>,
        metadata: Vec<Symbol>,
        valid_from: u64,
        valid_to: u64,
    ) -> Result<BytesN<32>, CertificationError> {
        issuance::issue_certification(
            &env,
            &issuer,
            &holder,
            certification_type,
            &document_hash,
            &metadata,
            valid_from,
            valid_to,
        )
    }
    
    // Verify a certification
    pub fn verify_certification(
        env: Env,
        certification_id: BytesN<32>,
        document_hash: BytesN<32>,
    ) -> Result<bool, CertificationError> {
        validation::verify_certification(&env, &certification_id, &document_hash)
    }
    
    // Revoke a certification
    pub fn revoke_certification(
        env: Env,
        issuer: Address,
        certification_id: BytesN<32>,
        reason: Symbol,
    ) -> Result<(), CertificationError> {
        validation::revoke_certification(&env, &issuer, &certification_id, &reason)
    }
    
    // Get certification details
    pub fn get_certification(
        env: Env,
        certification_id: BytesN<32>,
    ) -> Result<CertificationData, CertificationError> {
        let certification = env.storage().persistent().get(&DataKey::Certification(certification_id.clone()))
            .ok_or(CertificationError::CertificationNotFound)?;
        
        Ok(certification)
    }
    
    // Get all certifications for a holder
    pub fn get_holder_certifications(
        env: Env,
        holder: Address,
    ) -> Result<Vec<CertificationData>, CertificationError> {
        audit::get_holder_certifications(&env, &holder)
    }
    
    // Generate audit report
    pub fn generate_audit_report(
        env: Env,
        certification_type: Option<CertificationType>,
        issuer: Option<Address>,
        status: Option<Status>,
    ) -> Result<Vec<CertificationData>, CertificationError> {
        audit::generate_audit_report(&env, certification_type, issuer, status)
    }
} 