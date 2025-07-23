use soroban_sdk::{contracterror, contracttype, Address, BytesN, String, Symbol, Vec};

pub const CERTIFICATE_MANAGEMENT_CONTRACT_KEY: &str = "cert_mgmt_contract";

/// Error types for supply chain tracking operations
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainError {
    UnauthorizedAccess = 1,
    NotInitialized = 2,
    AlreadyInitialized = 3,
    CertificateNotFound = 4,
    ProductNotFound = 5,
    StageNotFound = 6,
    InvalidInput = 7,
    InvalidHash = 8,
    InvalidProductData = 9,
    InvalidStageTransition = 10,
    DuplicateStage = 11,
    QRCodeNotFound = 12,
    CertificateInvalid = 13,
    VerificationHashInvalid = 14,
}

/// Storage keys for different data types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Product(BytesN<32>), // Core: Product ID -> Product (with embedded stages)
    ProductRegistration(BytesN<32>), // Product ID -> ProductRegistration details
    FarmerProducts(Address), // Extended: Farmer -> Vec<BytesN<32>>
    TraceabilityIndex(String), // Extended: Product Type -> Vec<BytesN<32>>
    StageValidation(u32), // Extended: Stage validation rules
    QRCodeMapping(String), // Extended: QR Code -> BytesN<32>
}

/// Product structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Product {
    pub product_id: BytesN<32>,
    pub farmer_id: Address,
    pub stages: Vec<Stage>,
    pub certificate_id: Option<BytesN<32>>,
}

/// Product registration details
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProductRegistration {
    pub product_type: String,
    pub batch_number: String,
    pub origin_location: String,
    pub metadata_hash: BytesN<32>,
}

/// Stage structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stage {
    pub stage_id: u32,
    pub name: String,
    pub timestamp: u64,
    pub location: String,
    pub data_hash: BytesN<32>, // Hash of off-chain data
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageValidation {
    pub required_fields: Vec<String>,
    pub allowed_transitions: Vec<u32>,
    pub minimum_duration: u64,
}

// Certificate datatypes
#[derive(Clone)]
#[contracttype]
pub struct Certification {
    pub id: u32,
    pub cert_type: Symbol, // "Organic", "Fair Trade", etc.
    pub issuer: Address,   // Certifying authority
    pub issued_date: u64,
    pub expiration_date: u64,
    verification_hash: BytesN<32>, // Hash of certificate documents
    pub status: CertStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum CertStatus {
    Valid,
    Expired,
    Revoked,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificationError {
    NotFound = 1,
    AlreadyExpired = 2,
    NotExpired = 3,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    NotFound = 1,
    HashMismatch = 2,
    Expired = 3,
    Revoked = 4,
    ExpirationDue = 5,
}
