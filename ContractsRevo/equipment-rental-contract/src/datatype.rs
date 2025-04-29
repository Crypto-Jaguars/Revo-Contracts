use soroban_sdk::{Address, Env, String, Vec, contracterror};

#[derive(Clone)]
#[contracterror]
pub enum Error {
    AlreadyInitialized = 1,
    UnauthorizedAccess = 2,
    EquipmentNotFound = 3,
    RentalNotFound = 4,
    InvalidRentalPrice = 5,
    EquipmentNotAvailable = 6,
    EquipmentNotInGoodCondition = 7,
    InvalidDateRange = 8,
    InvalidRentalStatus = 9,
    CancellationPeriodExpired = 10,
}

#[derive(Clone)]
pub enum DataKey {
    Admin,
    NextEquipmentId,
    NextRentalId,
    Equipment(u64),
    Rental(u64),
}

#[derive(Clone, PartialEq)]
pub enum EquipmentStatus {
    Good,
    NeedsService,
    UnderMaintenance,
}

#[derive(Clone)]
pub struct Location {
    pub latitude: i64,  // Stored as fixed-point: actual value * 1_000_000
    pub longitude: i64, // Stored as fixed-point: actual value * 1_000_000
}

#[derive(Clone)]
pub struct EquipmentMetadata {
    pub name: String,
    pub description: String,
    pub category: String,
    pub rental_price: u64,  // Price per day in smallest currency unit
    pub location: Location,
}

#[derive(Clone)]
pub struct MaintenanceRecord {
    pub timestamp: u64,
    pub old_status: EquipmentStatus,
    pub new_status: EquipmentStatus,
    pub notes: String,
}

#[derive(Clone)]
pub struct Equipment {
    pub id: u64,
    pub owner: Address,
    pub metadata: EquipmentMetadata,
    pub status: EquipmentStatus,
    pub availability: bool,
    pub maintenance_history: Vec<MaintenanceRecord>,
    pub rental_history: Vec<u64>,
}

#[derive(Clone, PartialEq)]
pub enum RentalStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[derive(Clone)]
pub struct RentalAgreement {
    pub id: u64,
    pub equipment_id: u64,
    pub renter: Address,
    pub start_date: u64,
    pub end_date: u64,
    pub status: RentalStatus,
    pub total_price: u64,
}
