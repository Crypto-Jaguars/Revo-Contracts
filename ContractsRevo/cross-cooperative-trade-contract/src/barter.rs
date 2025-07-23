use crate::{utils::generate_id, BarterAgreement, DataKey, TradeError};
use soroban_sdk::{Address, BytesN, Env, String};

pub fn create_barter_agreement(
    env: Env,
    trade_offer_id: BytesN<32>,
    offering_cooperative: Address,
    accepting_cooperative: Address,
) -> BytesN<32> {
    let agreement_id = generate_id(&env);

    let barter_agreement = BarterAgreement {
        agreement_id: agreement_id.clone(),
        trade_offer_id,
        offering_cooperative,
        accepting_cooperative,
        status: String::from_str(&env, "Active"),
    };

    env.storage().persistent().set(
        &DataKey::BarterAgreement(agreement_id.clone()),
        &barter_agreement,
    );

    agreement_id
}

pub fn get_barter_agreement(env: Env, agreement_id: BytesN<32>) -> Result<BarterAgreement, TradeError> {
    env.storage()
        .persistent()
        .get(&DataKey::BarterAgreement(agreement_id))
        .ok_or(TradeError::TradeOfferNotFound) // Reusing existing error variant for consistency
}
