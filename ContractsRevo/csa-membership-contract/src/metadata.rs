use soroban_sdk::{BytesN, Env};
use crate::CSAMembership;

pub fn get_membership_metadata(env: Env, token_id: BytesN<32>) -> Option<CSAMembership> {
    env.storage().persistent().get(&token_id)
}