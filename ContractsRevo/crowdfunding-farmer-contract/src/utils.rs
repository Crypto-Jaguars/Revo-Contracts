use soroban_sdk::{token, Address, BytesN, Env, Vec};

use crate::{campaign::Campaign, contribution::Contribution};

pub fn save_campaign(env: &Env, campaign_id: &BytesN<32>, campaign: &Campaign) {
    env.storage().persistent().set(campaign_id, campaign);
}

pub fn read_campaign(env: &Env, campaign_id: &BytesN<32>) -> Option<Campaign> {
    env.storage().persistent().get(campaign_id)
}

pub fn save_contributions(env: &Env, campaign_id: &BytesN<32>, contributions: &Vec<Contribution>) {
    env.storage().persistent().set(campaign_id, contributions);
}

pub fn read_contributions(env: &Env, campaign_id: &BytesN<32>) -> Option<Vec<Contribution>> {
    env.storage().persistent().get(campaign_id)
}

pub fn validate_amount(amount: i128) {
    if amount <= 0 {
        panic!("Amount must be positive");
    }
}

pub fn validate_deadline(current_time: u64, deadline: u64) {
    if deadline <= current_time {
        panic!("Deadline must be in the future");
    }
}

pub fn transfer_tokens(env: &Env, token_address: &Address, from: &Address, to: &Address, amount: i128) {
    let client = token::Client::new(env, token_address);
    client.transfer(from, to, &amount);
}
