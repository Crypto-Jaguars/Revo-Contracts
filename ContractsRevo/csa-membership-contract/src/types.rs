use soroban_sdk::contracttype;
use soroban_sdk::BytesN;
use soroban_sdk::String;
use soroban_sdk::Address;

#[contracttype]
#[derive(Clone, Debug)]
pub struct CSAMembership {
    pub farm_id: BytesN<32>,
    pub season: String,
    pub share_size: ShareSize,
    pub pickup_location: String,
    pub start_date: u64,
    pub end_date: u64,
    pub member: Address,
}

#[contracttype]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ShareSize {
    Small,
    Medium,
    Large,
}