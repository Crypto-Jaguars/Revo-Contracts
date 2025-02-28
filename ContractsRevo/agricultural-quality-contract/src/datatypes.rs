use soroban_sdk::{contracttype, contracterror, Address, BytesN, String, Symbol, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityStandard {
    GlobalGAP,           // Global Good Agricultural Practices
    Organic,             // Organic Certification
    Fairtrade,          // Fairtrade Certification
    UTZ,                // UTZ Certification
    NonGMO,             // Non-GMO Project Verified
    PDO,                // Protected Designation of Origin
    PGI,                // Protected Geographical Indication
    Kosher,             // Kosher Certification
    GOTS,               // Global Organic Textile Standard
    Demeter,            // Demeter Biodynamic Certification
    Custom(Symbol),      // Custom certification
}

impl QualityStandard {
    pub fn to_u8(&self) -> u8 {
        match self {
            QualityStandard::GlobalGAP => 0,
            QualityStandard::Organic => 1,
            QualityStandard::Fairtrade => 2,
            QualityStandard::UTZ => 3,
            QualityStandard::NonGMO => 4,
            QualityStandard::PDO => 5,
            QualityStandard::PGI => 6,
            QualityStandard::Kosher => 7,
            QualityStandard::GOTS => 8,
            QualityStandard::Demeter => 9,
            QualityStandard::Custom(_) => 10,
        }
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificationStatus {
    Pending,
    Active,
    Suspended,
    Revoked,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeStatus {
    Filed,              // Initial filing
    UnderReview,        // Being reviewed
    InMediation,        // In mediation process
    Resolved,           // Resolution reached
    Appealed,           // Under appeal
    Closed,             // Final closure
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionOutcome {
    Upheld,             // Original certification stands
    Revoked,            // Certification revoked
    Modified,           // Certification modified
    RequireReinspection, // New inspection needed
    Dismissed,          // Dispute rejected
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificationData {
    pub holder: Address,
    pub standard: QualityStandard,
    pub status: CertificationStatus,
    pub issue_date: u64,
    pub expiry_date: u64,
    pub issuer: Address,
    pub audit_score: u32,
    pub conditions: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QualityMetric {
    pub name: Symbol,
    pub standard: QualityStandard,
    pub min_score: u32,
    pub weight: u32,
    pub version: u32,
    pub authority: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectionReport {
    pub inspector: Address,
    pub timestamp: u64,
    pub metrics: Vec<(Symbol, u32)>,
    pub overall_score: u32,
    pub findings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DisputeData {
    pub id: BytesN<32>,
    pub certification: BytesN<32>,
    pub complainant: Address,
    pub respondent: Address,
    pub timestamp: u64,
    pub status: DisputeStatus,
    pub evidence: Vec<BytesN<32>>,
    pub mediator: Option<Address>,
    pub resolution: Option<ResolutionOutcome>,
    pub appeal_deadline: Option<u64>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evidence {
    pub hash: BytesN<32>,
    pub handler: Address,
    pub timestamp: u64,
    pub description: String,
    pub data_type: Symbol,
    pub metadata: Vec<(Symbol, String)>,
}

#[contracttype]
pub enum DataKey {
    // Instance storage (small, frequently accessed data)
    Authorities,                            // -> Vec<Address>
    Inspectors,                            // -> Vec<Address>
    Mediators,                             // -> Vec<Address>
    StandardMetrics(QualityStandard),      // Standard -> Vec<Symbol>
    
    // Persistent storage (long-term data)
    Certification(BytesN<32>),              // Certification ID -> CertificationData
    Metric(QualityStandard, Symbol),        // (Standard, Name) -> QualityMetric
    Inspection(BytesN<32>),                 // Inspection ID -> InspectionReport
    Dispute(BytesN<32>),                    // Dispute ID -> DisputeData
    Evidence(BytesN<32>),                   // Evidence hash -> Evidence
    HolderCertifications(Address),          // Address -> Vec<BytesN<32>>
    IssuerCertifications(Address),          // Address -> Vec<BytesN<32>>
    DisputesByHolder(Address),              // Address -> Vec<BytesN<32>>
    DisputesByStandard(QualityStandard),   // Standard -> Vec<BytesN<32>>
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AgricQualityError {
    AlreadyExists = 1,
    NotFound = 2,
    Unauthorized = 3,
    InvalidInput = 4,
    Expired = 5,
    InsufficientScore = 6,
    InvalidStatus = 7,
    InvalidEvidence = 8,
    DeadlinePassed = 9,
    NotEligible = 10,
    CapacityExceeded = 11,
    StandardMismatch = 12,
    InsufficientAuthority = 13,
    InvalidTimestamp = 14,
    DuplicateSubmission = 15,
}

