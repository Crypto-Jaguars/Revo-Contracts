use soroban_sdk::{contracterror, contracttype, Address, BytesN, String, Vec};

/// Error types for supply chain tracking operations
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SupplyChainError {
    ProductNotFound = 1,
    StageNotFound = 2,
    InvalidStageTransition = 3,
    UnauthorizedAccess = 4,
    InvalidProductData = 5,
    CertificateNotFound = 6,
    DuplicateStage = 7,
    InvalidHash = 8,
    QRCodeNotFound = 9,
    InvalidInput = 10,
}

/// Storage keys for different data types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Product(BytesN<32>),                // Core: Product ID -> Product (with embedded stages)
    ProductRegistration(BytesN<32>),    // Product ID -> ProductRegistration details
    FarmerProducts(Address),              // Extended: Farmer -> Vec<BytesN<32>>
    TraceabilityIndex(String),            // Extended: Product Type -> Vec<BytesN<32>>
    StageValidation(u32),                 // Extended: Stage validation rules
    QRCodeMapping(String),                // Extended: QR Code -> BytesN<32>
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
