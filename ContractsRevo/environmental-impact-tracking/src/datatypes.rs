use soroban_sdk::{contracttype, Address, BytesN, String};

#[derive(Clone)]
#[contracttype]
pub struct CarbonCredit {
    pub project_id: BytesN<32>,
    pub carbon_amount: u32, // in kg
    pub verification_method: String,
    pub issuance_date: u64,
    pub retirement_status: RetirementStatus,
}

#[contracttype]
pub enum DataKey {
    Credit(BytesN<32>),
    ProjectCredits(BytesN<32>),
}

#[derive(Clone)]
#[contracttype]
pub enum RetirementStatus {
    Available,
    Retired(Address),
}
