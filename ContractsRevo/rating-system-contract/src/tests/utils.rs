#![cfg(test)]
use soroban_sdk::{testutils::Address as _, Address, Env};

use crate::{DataKey, RatingSystemContract};

pub struct TestCtx {
    pub env: Env,
    pub contract_id: Address,
}

impl TestCtx {
    pub fn new() -> Self {
        let env = Env::default();
        let contract_id = env.register(RatingSystemContract, ());
        Self { env, contract_id }
    }

    pub fn client(&self) -> crate::RatingSystemContractClient<'_> {
        crate::RatingSystemContractClient::new(&self.env, &self.contract_id)
    }

    pub fn gen_addr(&self) -> Address {
        Address::generate(&self.env)
    }

    pub fn seller_keys(&self, seller: &Address) -> (DataKey, DataKey, DataKey) {
        (
            DataKey::Rating(seller.clone()),
            DataKey::WeightedRating(seller.clone()),
            DataKey::ReputationHistory(seller.clone()),
        )
    }
}
