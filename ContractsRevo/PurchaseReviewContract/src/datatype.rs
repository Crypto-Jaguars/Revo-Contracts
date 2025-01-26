use soroban_sdk::{Address, contracterror, contracttype, String, Vec};

// Main categories for rating products/services
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Category {
   Quality,
   Shipping, 
   CustomerService,
}

// Star rating system (1-5 stars)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Rating {
   OneStar = 1,
   TwoStars = 2,
   ThreeStars = 3,
   FourStars = 4,
   FiveStars = 5,
}

// Types of attachments allowed in reviews
#[contracttype]
pub enum AttachmentType {
   Image,
   Video
}

// Structure for storing media attachments
#[contracttype]
pub struct Attachment {
   attachment_type: AttachmentType,
   hash: String,         // IPFS hash for content retrieval
   timestamp: u64
}

// Storage keys for contract data
#[derive(Clone)]
#[contracttype]
pub enum DataKeys {
   Rating(Address),
   CategoryRating(Address),
   RatingStats(Address),
   ProductRatings(u128),
   CategoryMapping(Address),
}

// Error types for contract operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum PurchaseReviewError {
   InvalidRating = 1,
   ReviewAlreadyExists = 2,
   ReviewNotFound = 3,
   InvalidCategory = 4,
   UnauthorizedAccess = 5,
   RatingOutOfRange = 6,
   RatingUpdateError = 7,
   PurchaseNotVerified = 8,
   InvalidAttachment = 9,
   ProductNotFound = 10
}

// Rating for a specific category with attachments
#[contracttype]
pub struct CategoryRating {
   pub category: Category,
   pub rating: Rating,
   pub timestamp: u64,
   pub attachments: Attachment,
   pub user: Address,
   pub weight: u32
}

// Collection of category ratings for a product
#[contracttype]
pub struct ProductRatings {
   pub ratings: Vec<CategoryRating>
}