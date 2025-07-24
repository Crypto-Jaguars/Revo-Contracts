use soroban_sdk::{contracterror, contracttype, Address, BytesN, String, Symbol, Vec};

pub const CERTIFICATE_MANAGEMENT_CONTRACT_KEY: &str = "cert_mgmt_contract";
pub const MAX_PRODUCTS_PER_FARMER: u32 = 1000;
pub const MAX_PRODUCTS_PER_TYPE: u32 = 5000;

/// Storage keys for different data types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Product(BytesN<32>), // Product ID -> Product (with embedded stages)
    ProductRegistration(BytesN<32>), // Product ID -> ProductRegistration details
    FarmerProducts(Address), // Farmer -> Vec<BytesN<32>>
    ProductTypeIndex(String), // Product Type -> Vec<BytesN<32>>
    StageValidation(u32), // Stage validation rules
    QRCodeMapping(String), // QR Code -> BytesN<32>
}

/// Product structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Product {
    pub product_id: BytesN<32>,
    pub farmer_id: Address,
    pub stages: Vec<Stage>,
    pub certificate_id: CertificateId,
}

/// Custom Option type for BytesN<32> to use with #[contracttype]
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateId {
    None,
    Some(BytesN<32>),
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
    pub tier: StageTier,
    pub name: String,
    pub timestamp: u64,
    pub location: String,
    pub data_hash: BytesN<32>, // Hash of off-chain data
}

/// Stage tiers in the agricultural supply chain process
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StageTier {
    Planting = 1,       // Seeds/seedlings planted
    Cultivation = 2,    // Growing, watering, care
    Harvesting = 3,     // Crop collection
    Processing = 4,     // Cleaning, initial processing
    Packaging = 5,      // Packaging for distribution
    Storage = 6,        // Warehousing/storage
    Transportation = 7, // Shipping/logistics
    Distribution = 8,   // Wholesale distribution
    Retail = 9,         // Retail/market
    Consumer = 10,      // Final consumer delivery
}

impl StageTier {
    /// Get the numeric value of the tier
    pub fn value(&self) -> u32 {
        match self {
            StageTier::Planting => 1,
            StageTier::Cultivation => 2,
            StageTier::Harvesting => 3,
            StageTier::Processing => 4,
            StageTier::Packaging => 5,
            StageTier::Storage => 6,
            StageTier::Transportation => 7,
            StageTier::Distribution => 8,
            StageTier::Retail => 9,
            StageTier::Consumer => 10,
        }
    }

    /// Get the next tier in the sequence
    pub fn next(&self) -> Option<StageTier> {
        match self {
            StageTier::Planting => Some(StageTier::Cultivation),
            StageTier::Cultivation => Some(StageTier::Harvesting),
            StageTier::Harvesting => Some(StageTier::Processing),
            StageTier::Processing => Some(StageTier::Packaging),
            StageTier::Packaging => Some(StageTier::Storage),
            StageTier::Storage => Some(StageTier::Transportation),
            StageTier::Transportation => Some(StageTier::Distribution),
            StageTier::Distribution => Some(StageTier::Retail),
            StageTier::Retail => Some(StageTier::Consumer),
            StageTier::Consumer => None, // Final stage
        }
    }

    /// Create StageTier from u32 value
    pub fn from_value(value: u32) -> Option<StageTier> {
        match value {
            1 => Some(StageTier::Planting),
            2 => Some(StageTier::Cultivation),
            3 => Some(StageTier::Harvesting),
            4 => Some(StageTier::Processing),
            5 => Some(StageTier::Packaging),
            6 => Some(StageTier::Storage),
            7 => Some(StageTier::Transportation),
            8 => Some(StageTier::Distribution),
            9 => Some(StageTier::Retail),
            10 => Some(StageTier::Consumer),
            _ => None,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StageValidation {
    pub required_fields: Vec<String>,
    pub allowed_transitions: Vec<u32>,
    pub minimum_duration: u64,
}

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
    ProductAlreadyExists = 9,
    InvalidStageTransition = 10,
    DuplicateStage = 11,
    QRCodeNotFound = 12,
    CertificateInvalid = 13,
    VerificationHashInvalid = 14,
    InvalidStageTier = 15,
    DuplicateStageTier = 16,
    InvalidTierProgression = 17,
    ProductLimitExceeded = 18,
}

// Certificate datatypes
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Certification {
    pub id: u32,
    pub cert_type: Symbol, // "Organic", "Fair Trade", etc.
    pub issuer: Address,   // Certifying authority
    pub issued_date: u64,
    pub expiration_date: u64,
    verification_hash: BytesN<32>, // Hash of certificate documents
    pub status: CertStatus,
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
    NotFound = 19,
    AlreadyExpired = 20,
    NotExpired = 21,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    NotFound = 22,
    HashMismatch = 23,
    Expired = 24,
    Revoked = 25,
    ExpirationDue = 26,
}
