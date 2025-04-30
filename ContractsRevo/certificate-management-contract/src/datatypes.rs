use soroban_sdk::{contracttype, Address, BytesN, Map, Symbol};

use crate::{CertificationError, RevokeError, VerifyError};

#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    UserCertCount,
    UsersCertificates,
}

pub type UsersCertificates = Map<Address, UserCertificates>; // User -> UserCertificates
pub type UserCertificates = Map<u32, Certification>; // Certification Id -> Certification
pub type UserCertCount = Map<Address, u32>; // User -> Number of certifications

#[derive(Clone)]
#[contracttype]
pub struct Certification {
    pub id: u32,
    pub cert_type: Symbol, // "Organic", "Fair Trade", etc.
    pub issuer: Address,   // Certifying authority
    pub issued_date: u64,
    pub expiration_date: u64,
    verification_hash: BytesN<32>, // Hash of certification documents
    pub status: CertStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum CertStatus {
    Valid,
    Expired,
    Revoked,
}

impl Certification {
    pub fn new(
        id: u32,
        cert_type: Symbol,
        issuer: Address,
        issued_date: u64,
        expiration_date: u64,
        verification_hash: BytesN<32>,
    ) -> Self {
        Self {
            id,
            cert_type,
            issuer,
            issued_date,
            expiration_date,
            verification_hash,
            status: CertStatus::Valid,
        }
    }

    pub fn revoke(&mut self) -> Result<(), RevokeError> {
        if self.status == CertStatus::Revoked || self.status == CertStatus::Expired {
            return Err(RevokeError::AlreadyRevoked);
        }
        self.status = CertStatus::Revoked;

        Ok(())
    }

    pub fn verify(&self, submitted_hash: BytesN<32>, current_time: u64) -> Result<(), VerifyError> {
        if self.is_expiration_due(current_time) || self.is_expired() {
            return Err(VerifyError::Expired);
        }
        if self.status == CertStatus::Revoked {
            return Err(VerifyError::Revoked);
        }
        if self.verification_hash != submitted_hash {
            return Err(VerifyError::HashMismatch);
        }

        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        match self.status {
            CertStatus::Expired => true,
            _ => false,
        }
    }

    pub fn is_expiration_due(&self, current_time: u64) -> bool {
        self.expiration_date < current_time
    }

    pub fn expire(&mut self, current_time: u64) -> Result<(), CertificationError> {
        if self.is_expired() {
            return Err(CertificationError::AlreadyExpired);
        }
        if !self.is_expiration_due(current_time) {
            return Err(CertificationError::NotExpired);
        }

        self.status = CertStatus::Expired;

        Ok(())
    }
}