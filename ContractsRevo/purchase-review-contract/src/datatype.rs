use soroban_sdk::{Address, contracterror, contracttype, String, Vec};

/// Main categories for rating different aspects of products/services
/// Used to organize and segment ratings into specific areas of evaluation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Category {
    Quality,         // Product/service quality assessment
    Shipping,        // Delivery and shipping experience
    CustomerService, // Customer support and service interaction
}

/// Star rating system allowing users to rate from 1 to 5 stars
/// Each variant represents a different level of satisfaction
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Rating {
    OneStar = 1,    // Poor/Unsatisfactory
    TwoStars = 2,   // Below Average
    ThreeStars = 3, // Average/Neutral
    FourStars = 4,  // Above Average
    FiveStars = 5,  // Excellent
}

/// Storage key definitions for organizing contract data in the ledger
/// Each variant represents a different type of data storage with its associated key structure
#[derive(Clone)]
#[contracttype]
pub enum DataKeys {
    Rating(Address),                // User's overall rating
    CategoryRating(Address),        // User's rating for specific categories
    RatingStats(Address),          // Statistical data about ratings
    ProductRatings(u128),          // All ratings for a specific product
    CategoryMapping(Address),       // Maps categories to products/users
    Review(u128, u32),             // Specific review identified by product_id and review_id
    PurchaseVerification(u128, Address),    // Verification status for a purchase
    ReviewReport(u128, u32),       // Report data for a specific review
    ReviewCount(u128),
    ReviewVote(u128, u32, Address), // (product_id, review_id, voter)
    AlreadyVoted(u128, u32, Address), // (product_id, review_id, voter)
    UserReviewReport(u128, u32, Address), // (product_id, review_id, reporter)
    VoteRateLimit(Address),
}

/// Error types that can occur during contract operations
/// Each error has a unique code and represents a specific failure condition
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PurchaseReviewError {
    InvalidRating = 1,          // Rating value is not within acceptable range
    ReviewAlreadyExists = 2,    // Attempt to create duplicate review
    ReviewNotFound = 3,         // Referenced review doesn't exist
    InvalidCategory = 4,        // Category specified is not valid
    UnauthorizedAccess = 5,     // User doesn't have permission for operation
    RatingOutOfRange = 6,       // Rating value exceeds allowed range
    RatingUpdateError = 7,      // Failed to update rating
    PurchaseNotVerified = 8,    // Review attempted without verified purchase
    InvalidAttachment = 9,      // Attached content is invalid
    ProductNotFound = 10,       // Referenced product doesn't exist
    AlreadyVerified = 11,       // Purchase already verified
    PurchaseNotFound = 12,      // Referenced purchase doesn't exist 
    EditWindowExpired = 13,     // Time window for editing has passed
    AlreadyReviewed = 14,       // User already submitted a review
    WeightedRatingOverflow = 15,
    AlreadyVoted = 16,
    InvalidReportReason = 17,
    AlreadyReported = 18,
    InvalidReviewText = 19,
    InvalidTimestamp = 20,
    RateLimitExceeded = 21,
}

/// Represents a rating for a specific category with additional metadata
#[contracttype]
pub struct CategoryRating {
    pub category: Category,     // The category being rated
    pub rating: Rating,         // The star rating given
    pub timestamp: u64,         // When the rating was submitted
    pub attachment: String,     // Additional comments or media
    pub user: Address,          // Address of the user who rated
    pub weight: u32            // Weight/importance of this rating
}

/// Collection of category-specific ratings for a product
#[contracttype]
pub struct ProductRatings {
    pub ratings: Vec<CategoryRating> // List of all category ratings
}

/// Detailed information about a product review
#[contracttype]
pub struct ReviewDetails {
    pub review_text: String,         // The actual review content
    pub reviewer: Address,           // Address of the reviewer
    pub timestamp: u64,              // When the review was submitted
    pub helpful_votes: u32,          // Number of helpful votes
    pub not_helpful_votes: u32,      // Number of not helpful votes
    pub verified_purchase: bool,      // Whether reviewer purchased the product
    pub responses: Vec<String>       // Responses to the review
}

/// Data structure for purchase verification
#[contracttype]
pub struct PurchaseVerificationData {
    pub user: Address,               // User who made the purchase
    pub product_id: u128,            // ID of the purchased product
    pub purchase_link: String,       // Link to purchase proof
    pub is_verified: bool,           // Verification status
    pub timestamp: u64,              // When purchase was made
    pub has_review: bool,            // Whether user has reviewed
}

/// Information about reported reviews
#[contracttype]
pub struct ReviewReportData {
    pub reporter: Address,           // User reporting the review
    pub product_id: u128,            // Product ID of reported review
    pub review_id: u32,              // ID of reported review
    pub reason: String,              // Reason for reporting
    pub timestamp: u64               // When report was submitted
}