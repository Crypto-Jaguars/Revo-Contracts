use soroban_sdk::{contracterror, contracttype, Address, String, Vec};

#[derive(Debug)]
#[contracterror]
pub enum CooperativeError {
    MemberNotFound = 1,
    MemberAlreadyExists = 2,
    ResourceNotAvailable = 3,
    ResourceNotFound = 4,
    TimeSlotConflict = 5,
    Unauthorized = 6,
    NotAMember = 7,
    ProposalNotFound = 8,
    ProposalAlreadyExecuted = 9,
    ProposalRejected = 10,
    InsufficientFunds = 11,
    InvalidInput = 12,
}

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Admin,
    Member(Address),
    Resource(Address, u32),
    ResourceCounter,
    OwnerResources(Address),
    MaintenanceLog(Address),
    Investment(Address),
    Balance(Address),
    Expense(Address),
    Proposal(Address),
    Emergency,
    Reputation(Address),
}

#[contracttype]
pub struct Member {
    pub address: Address,
    pub name: String,
    pub role: String,
    pub reputation: u32,
    pub contributions: u32,
    pub verified: bool,
}

#[contracttype]
pub struct Resource {
    pub owner: Address,
    pub description: String,
    pub available: bool,
    pub borrower: Option<Address>,
    pub schedule: Vec<String>,
}

#[contracttype]
pub enum RecordType {
    Expense,
    Investment,
    Profit,
}

#[contracttype]
pub struct FinancialRecord {
    pub member: Address,
    pub amount: u64,
    pub record_type: RecordType,
}

#[contracttype]
pub struct Proposal {
    pub proposer: Address,
    pub description: String,
    pub votes_for: u32,
    pub votes_against: u32,
    pub executed: bool,
}
