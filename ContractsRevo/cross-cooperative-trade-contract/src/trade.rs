use crate::{utils::generate_id, DataKey, TradeError, TradeOffer};
use soroban_sdk::{Address, BytesN, Env, String, Vec};

pub fn create_trade_offer(
    env: Env,
    cooperative_id: Address,
    offered_product: BytesN<32>,
    requested_product: BytesN<32>,
) -> Result<BytesN<32>, TradeError> {
    // Verify the caller is the cooperative
    cooperative_id.require_auth();

    // Basic validation: offered and requested products should be different
    if offered_product == requested_product {
        return Err(TradeError::InvalidQuantity);
    }

    // Generate unique offer ID
    let offer_id = generate_id(&env);

    // Create the simplified trade offer
    let trade_offer = TradeOffer {
        offer_id: offer_id.clone(),
        cooperative_id: cooperative_id.clone(),
        offered_product,
        requested_product,
        status: String::from_str(&env, "Pending"),
    };

    // Store the trade offer
    env.storage()
        .persistent()
        .set(&DataKey::TradeOffer(offer_id.clone()), &trade_offer);

    // Add to active offers list
    let mut active_offers: Vec<BytesN<32>> = env
        .storage()
        .instance()
        .get(&DataKey::ActiveOffers)
        .unwrap_or(Vec::new(&env));
    active_offers.push_back(offer_id.clone());
    env.storage()
        .instance()
        .set(&DataKey::ActiveOffers, &active_offers);

    Ok(offer_id)
}

pub fn accept_trade(
    env: Env,
    offer_id: BytesN<32>,
    accepting_cooperative: Address,
) -> Result<BytesN<32>, TradeError> {
    // Verify the caller is the accepting cooperative
    accepting_cooperative.require_auth();

    // Get the trade offer
    let mut trade_offer: TradeOffer = env
        .storage()
        .persistent()
        .get(&DataKey::TradeOffer(offer_id.clone()))
        .ok_or(TradeError::TradeOfferNotFound)?;

    // Validate trade offer
    if trade_offer.cooperative_id == accepting_cooperative {
        return Err(TradeError::CannotAcceptOwnOffer);
    }

    if trade_offer.status != String::from_str(&env, "Pending") {
        return Err(TradeError::InvalidTradeStatus);
    }

    // Update trade offer status
    trade_offer.status = String::from_str(&env, "Accepted");
    env.storage()
        .persistent()
        .set(&DataKey::TradeOffer(offer_id.clone()), &trade_offer);

    // Create barter agreement
    let agreement_id = crate::barter::create_barter_agreement(
        env.clone(),
        offer_id.clone(),
        trade_offer.cooperative_id,
        accepting_cooperative,
    );

    // Remove from active offers
    let active_offers: Vec<BytesN<32>> = env
        .storage()
        .instance()
        .get(&DataKey::ActiveOffers)
        .unwrap_or(Vec::new(&env));

    let mut new_active_offers = Vec::new(&env);
    for i in 0..active_offers.len() {
        if active_offers.get(i).unwrap() != offer_id.clone() {
            new_active_offers.push_back(active_offers.get(i).unwrap());
        }
    }
    env.storage()
        .instance()
        .set(&DataKey::ActiveOffers, &new_active_offers);

    Ok(agreement_id)
}

pub fn complete_trade(env: Env, offer_id: BytesN<32>, caller: Address) -> Result<(), TradeError> {
    // Verify caller authorization
    caller.require_auth();

    // Get the trade offer
    let mut trade_offer: TradeOffer = env
        .storage()
        .persistent()
        .get(&DataKey::TradeOffer(offer_id.clone()))
        .ok_or(TradeError::TradeOfferNotFound)?;

    // Validate that caller is involved in the trade
    if trade_offer.cooperative_id != caller {
        return Err(TradeError::UnauthorizedAccess);
    }

    if trade_offer.status != String::from_str(&env, "Accepted") {
        return Err(TradeError::InvalidTradeStatus);
    }

    // Update trade offer status
    trade_offer.status = String::from_str(&env, "Completed");
    env.storage()
        .persistent()
        .set(&DataKey::TradeOffer(offer_id.clone()), &trade_offer);

    // Update reputations for both cooperatives
    crate::reputation::update_reputation_after_trade(
        &env,
        &trade_offer.cooperative_id,
        true,
    )?;

    Ok(())
}

pub fn get_trade_details(env: Env, offer_id: BytesN<32>) -> Result<TradeOffer, TradeError> {
    env.storage()
        .persistent()
        .get(&DataKey::TradeOffer(offer_id))
        .ok_or(TradeError::TradeOfferNotFound)
}

pub fn list_active_offers(env: Env) -> Result<Vec<BytesN<32>>, TradeError> {
    let active_offers = env
        .storage()
        .instance()
        .get(&DataKey::ActiveOffers)
        .unwrap_or(Vec::new(&env));

    Ok(active_offers)
}
