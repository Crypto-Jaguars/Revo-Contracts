use soroban_sdk::{contracttype, contracterror, Address, BytesN, Env, Symbol, Vec};

// Define the certificate types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificationType {
    Organic,
    FairTrade,
    UTZ,
    RainforestAlliance,
    ISO9001,
    ISO14001,
    HACCP,
    Kosher,
    Halal,
    Demeter,
    Custom(Symbol),
}

// Status of certification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Status {
    Valid,
    Expired,
    Revoked,
}

// Data structure for certification
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificationData {
    pub certification_id: BytesN<32>,
    pub issuer: Address,
    pub holder: Address,
    pub certification_type: CertificationType,
    pub document_hash: BytesN<32>,
    pub metadata: Vec<Symbol>,
    pub status: Status,
    pub issue_date: u64,
    pub valid_from: u64,
    pub valid_to: u64,
    pub revocation_reason: Option<Symbol>,
    pub last_updated: u64,
}

// Event structure for certification events
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificationEvent {
    pub certification_id: BytesN<32>,
    pub event_type: Symbol,
    pub timestamp: u64,
    pub data: Vec<Symbol>,
}

// Storage keys
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Certification(BytesN<32>),  // certification_id -> CertificationData
    HolderCertifications(Address), // holder -> Vec<BytesN<32>>
    IssuerCertifications(Address), // issuer -> Vec<BytesN<32>>
    CertificationEvents(BytesN<32>), // certification_id -> Vec<CertificationEvent>
    CertificationsByType(CertificationType), // type -> Vec<BytesN<32>>
    VerifiedIssuers, // Vec<Address>
}

// Error types
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CertificationError {
    AlreadyInitialized = 1,
    UnauthorizedAccess = 2,
    InvalidIssuer = 3,
    InvalidValidity = 4,
    CertificationNotFound = 5,
    DocumentHashMismatch = 6,
    CertificationExpired = 7,
    CertificationRevoked = 8,
    InvalidStatus = 9,
    InvalidMetadata = 10,
}

// Issuance-related errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum IssuanceError {
    InvalidMetadata = 1,
    MetadataTooLarge = 2,
} 