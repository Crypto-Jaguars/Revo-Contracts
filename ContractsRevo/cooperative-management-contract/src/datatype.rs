use soroban_sdk::{contracttype, Address, String, Vec};

#[derive(Debug)]
#[contracttype]
pub enum CooperativeError {
    MemberNotFound,
    ResourceNotAvailable,
    Unauthorized,
    ProposalNotFound,
    ProposalRejected,
    InsufficientFunds,
    InvalidInput,
}

#[derive(Debug)]
#[contracttype]
pub enum DataKey {
    Admin,
    Member(Address),
    Resource(Address),
    MaintenanceLog(Address),
    Investment(Address),
    Balance(Address),
    Expense(Address),
    Proposal(Address),
    ProposalVotes(Address),
    Emergency,
    Reputation(Address),
}

#[contracttype]
pub struct Member {
    pub address: Address,
    pub name: String,
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
