use soroban_sdk::{contracterror, contracttype, Address, String, Symbol, Vec};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminError {
    AlreadyInitialized = 1,
    UnauthorizedAccess = 2,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuctionError {
    BidTooLow = 1,
    AuctionEnded = 2,
    AuctionAlreadyExists = 3,
    InvalidBidder = 4,
    AuctionNotFound = 5,
    TooLateToExtend = 6,
    InvalidAuctionEndTime = 7,
    AuctionNotYetEnded = 8,
    NoBidsPlaced = 9,
    ProductNotFound = 10,
    OutOfStock = 11,
    ProductExpired = 12,
    BulkPurchaseUnavailable = 13,
    QuantityUnavailable = 14,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProductError {
    InvalidDescription = 1,
    InvalidPrice = 2,
    InvalidWeight = 3,
    OutOfStock = 4,
    InvalidImageCount = 5,
    ProductNotFound = 6,
    Unauthorized = 7,
    InvalidHarvestDate = 8,
    FreshnessNotVerified = 9,
    OutOfSeason = 10,
    InvalidCertification = 11,
    SeasonalDataNotAvailable = 12,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityError {
    InvalidGrade = 1,
    UnverifiedProduct = 2,
    DisputeAlreadyExists = 3,
    DisputeNotFound = 4,
    CertificationInvalid = 5,
    StorageConditionsNotMet = 6,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OracleError {
    PriceDataNotAvailable = 1,
    RegionNotSupported = 2,
    InvalidPriceData = 3,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TimeError {
    ProductExpired = 1,
    HarvestDateInFuture = 2,
    InvalidTimeframe = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FreshnessRating {
    Premium,   // Just harvested (1-2 days)
    Excellent, // Very fresh (3-5 days)
    Good,      // Good quality (6-10 days)
    Fair,      // Acceptable (11-15 days)
    Poor,      // Close to expiry (16+ days)
    Expired,   // Past recommended use
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QualityGrade {
    Premium,     // Top quality
    GradeA,      // High quality
    GradeB,      // Good quality
    GradeC,      // Average quality
    Substandard, // Below average
    Rejected,    // Not suitable for sale
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeasonalStatus {
    InSeason,    // Product is in high season
    EarlySeason, // Beginning of season
    LateSeason,  // End of season
    OutOfSeason, // Not in season
    YearRound,   // Available all year
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageCondition {
    Refrigerated,
    FreezerStorage,
    RoomTemperature,
    ControlledAtmosphere,
    Humidity,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AgriculturalProduct {
    pub id: u64,
    pub farmer: Address,
    pub name: Symbol,
    pub description: String,
    pub base_price: u64,
    pub current_price: u64,
    pub weight_kg: u64,
    pub quantity: u32,
    pub harvest_date: u64,
    pub expiry_date: u64,
    pub images: Vec<String>,
    pub freshness_rating: FreshnessRating,
    pub quality_grade: QualityGrade,
    pub verified: bool,
    pub certifications: Vec<Symbol>,
    pub storage_condition: StorageCondition,
    pub product_type: Symbol,
    pub region: Symbol,
    pub seasonal_status: SeasonalStatus,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Auction {
    pub product_id: u64,
    pub highest_bid: u64,
    pub highest_bidder: Option<Address>,
    pub reserve_price: u64,
    pub auction_end_time: u64,
    pub farmer: Address,
    pub quantity_available: u32,
    pub min_quantity: u32,
    pub bulk_discount_threshold: u32,
    pub bulk_discount_percentage: u32,
    pub dynamic_pricing: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct MarketPrice {
    pub product_type: Symbol,
    pub region: Symbol,
    pub price: u64,
    pub timestamp: u64,
    pub trend: i32, // Positive for rising, negative for falling
    pub volume: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct QualityDispute {
    pub buyer: Address,
    pub farmer: Address,
    pub product_id: u64,
    pub reason: String,
    pub reported_quality: QualityGrade,
    // pub resolution: Option<DisputeResolution>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisputeResolution {
    Rejected,
    PartialRefund,
    FullRefund,
    Replacement,
    Pending,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Auction(Address, u64),                 // Farmer's auction for a product
    ProductList(Address),                  // Farmer's product list
    Product(Address, u64),                 // Specific product details
    MarketPrice(Symbol, Symbol),           // Market price by product type and region
    QualityDispute(Address, Address, u64), // Dispute between buyer and farmer for a product
    CertificationVerification(Symbol),     // Verification for a certification type
    SeasonalStatus(Symbol, Symbol),        // Seasonal status for product type in a region
    PriceHistory(Symbol, Symbol, u64),     // Historical price data with timestamp
    StorageConditionMonitor(Address, u64), // Storage condition monitoring for a product
}
