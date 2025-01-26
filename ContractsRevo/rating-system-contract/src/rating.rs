use soroban_sdk::{contracttype, Address, Env, String, Symbol, Vec};
// use crate::datatypes::{Error};

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Rating {
    pub buyer: Address,
    pub rating: u32,
    pub weight: u32,
    pub feedback: Option<String>,
}

pub const MIN_RATING: u32 = 1;
pub const MAX_RATING: u32 = 5;

// Add a rating for the seller
pub fn rate_seller_system(
    env: Env,
    seller: Address,
    buyer: Address,
    rating: u32,
    weight: u32,
    feedback: Option<String>,
) -> bool {
    // Validate rating range
    if rating < MIN_RATING || rating > MAX_RATING {
        panic!("rating value must be in rang");
    }

    // Create a new rating record
    let seller_rating = Rating {
        buyer,
        rating,
        weight,
        feedback,
    };

    // fetch existing ratings or initialize new vector
    let mut ratings: Vec<Rating> = match env.storage().instance().get(&seller) {
        Some(x) => x,
        None => Vec::new(&env),
    };

    // Add new rating data in vector
    ratings.push_back(seller_rating);

    // Update seller ratings in storage
    env.storage().instance().set(&seller, &ratings);

    env.events().publish(
        (Symbol::new(&env, "buyer_rate_the_seller"), seller.clone()),
        rating,
    );

    true
}

// update the seller weighted rating
pub fn update_weighted_rating(env: Env, seller: Address, rating: u32, weight: u32) {
    // Fetch existing weighted rating and total weight or initialize to zero
    let (mut total_weighted_rating, mut total_weight): (u32, u32) =
        match env.storage().instance().get(&seller) {
            Some((x, y)) => (x, y),
            None => (0, 0),
        };

    // Update total weighted rating and weight
    total_weighted_rating += rating * weight;
    total_weight += weight;

    // save updated values in storage
    env.storage()
        .instance()
        .set(&seller, &(total_weighted_rating, total_weight));

    env.events().publish(
        (Symbol::new(&env, "updated_weighted_rating"), seller.clone()),
        weight,
    );
}

// calculates the seller's weighted rating
pub fn calculate_weighted_rating(env: Env, seller: Address) -> f32 {
    // Fetch existing total weighted rating and total weight or initialize to zero
    let (total_weighted_rating, total_weight): (u32, u32) =
        match env.storage().instance().get(&seller) {
            Some((x, y)) => (x, y),
            None => (0, 0),
        };

    // ensure there are ratings to calculate
    if total_weight == 0 {
        panic!("No rating available");
    }

    // Compute weighted rating
    let weighted_rating = (total_weighted_rating / total_weight) as f32;

    env.events().publish(
        (
            Symbol::new(&env, "calculated_weighted_rating"),
            seller.clone(),
        ),
        weighted_rating as u32,
    );

    weighted_rating
}
