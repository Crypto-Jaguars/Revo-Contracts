#![cfg(test)]

use super::*;
use crate::{rating::Rating, reputation::ReputationRecord};
use soroban_sdk::{testutils::Address as TestAddress, Env, Vec};

// #[test]
// #[should_panic(expected = "Buyer and seller cannot be the same address")]
// fn draft_test_rate_seller_panic_same_buyer_seller() {
//     let env = Env::default();
//     let contract_id = env.register(RatingSystemContract, ());
//     let client = RatingSystemContractClient::new(&env, &contract_id);
//     let seller_address = <Address>::generate(&env);
//     let rating = 3u32;
//     let weight = 2u32;

//     client.rate_seller(&seller_address, &seller_address, &rating, &weight, &None);
// }

// #[test]
// #[should_panic(expected = "Rating must be between 1 and 5")]
// fn draft_test_rate_seller_panic_rating_lower_bound() {
//     let env = Env::default();
//     let contract_id = env.register(RatingSystemContract, ());
//     let client = RatingSystemContractClient::new(&env, &contract_id);
//     let seller_address = <Address>::generate(&env);
//     let buyer_address = <Address>::generate(&env);
//     let weight = 2u32;

//     client.rate_seller(&seller_address, &buyer_address, &0, &weight, &None);
// }

// #[test]
// #[should_panic(expected = "Rating must be between 1 and 5")]
// fn draft_test_rate_seller_panic_rating_upper_bound() {
//     let env = Env::default();
//     let contract_id = env.register(RatingSystemContract, ());
//     let client = RatingSystemContractClient::new(&env, &contract_id);
//     let seller_address = <Address>::generate(&env);
//     let buyer_address = <Address>::generate(&env);
//     let weight = 2u32;

//     client.rate_seller(&seller_address, &buyer_address, &10, &weight, &None);
// }

// #[test]
// fn draft_test_rate_seller() {
//     let env = Env::default();
//     let contract_id = env.register(RatingSystemContract, ());
//     let client = RatingSystemContractClient::new(&env, &contract_id);
//     let seller_address = <Address>::generate(&env);
//     let buyer_address = <Address>::generate(&env);
//     let rating = 3u32;
//     let weight = 2u32;
//     let weighted_key = DataKey::WeightedRating(seller_address.clone());
//     let rating_key = DataKey::Rating(seller_address.clone());

//     // Helper function to verify storage
//     fn assert_storage(
//         env: &Env,
//         contract_id: &Address,
//         weighted_key: &DataKey,
//         rating_key: &DataKey,
//         expected_ratings: u32,
//         expected_total_weighted_rating: u32,
//         expected_total_weight: u32,
//     ) {
//         env.as_contract(contract_id, || {
//             let (total_weighted_rating, total_weight): (u32, u32) = env
//                 .storage()
//                 .instance()
//                 .get(weighted_key)
//                 .expect("Weighted rating key not found");

//             let ratings: Vec<Rating> = env
//                 .storage()
//                 .instance()
//                 .get(rating_key)
//                 .expect("Rating key not found");

//             assert_eq!(ratings.len(), expected_ratings);
//             assert_eq!(total_weighted_rating, expected_total_weighted_rating);
//             assert_eq!(total_weight, expected_total_weight);
//         });
//     }

//     // First rating submission
//     client.rate_seller(&seller_address, &buyer_address, &rating, &weight, &None);
//     assert_storage(
//         &env,
//         &contract_id,
//         &weighted_key,
//         &rating_key,
//         1,
//         rating * weight,
//         weight,
//     );

//     // Second and third ratings by the same buyer
//     client.rate_seller(&seller_address, &buyer_address, &rating, &weight, &None);
//     client.rate_seller(&seller_address, &buyer_address, &rating, &weight, &None);
//     assert_storage(
//         &env,
//         &contract_id,
//         &weighted_key,
//         &rating_key,
//         3,
//         (rating * weight) * 3,
//         weight * 3,
//     );

//     // Edge case: Rating with zero weight
//     client.rate_seller(&seller_address, &buyer_address, &rating, &0, &None);
//     assert_storage(
//         &env,
//         &contract_id,
//         &weighted_key,
//         &rating_key,
//         4,
//         (rating * weight) * 3,
//         weight * 3,
//     );
// }

// #[test]
// #[should_panic(expected = "No rating available")]
// fn test_seller_reputation_score_panic_rating_no_available() {
//     let env = Env::default();
//     let contract_id = env.register(RatingSystemContract, ());
//     let client = RatingSystemContractClient::new(&env, &contract_id);
//     let seller_address = <Address>::generate(&env);

//     client.seller_reputation_score(&seller_address);
// }

#[test]
fn test_seller_reputation_score() {
    let env = Env::default();
    let contract_id = env.register(RatingSystemContract, ());
    let client = RatingSystemContractClient::new(&env, &contract_id);
    let seller_address = <Address>::generate(&env);
    let buyer_address = <Address>::generate(&env);
    let reputation_history_key = DataKey::ReputationHistory(seller_address.clone());
    let weight = 2u32;

    // First rating submission
    client.rate_seller(&seller_address, &buyer_address, &5, &weight, &None);

    // Helper function to verify storage
    fn assert_storage(
        env: &Env,
        contract_id: &Address,
        reputation_history_key: &DataKey,
        expected_reputation_records: u32,
    ) {
        env.as_contract(contract_id, || {
            let reputation_records: Vec<ReputationRecord> = env
                .storage()
                .instance()
                .get(reputation_history_key)
                .expect("Reputation history key rating key not found");

            assert_eq!(reputation_records.len(), expected_reputation_records);
        });
    }

    let reputation_score = client.seller_reputation_score(&seller_address);
    assert_eq!(reputation_score, 5);
    assert_storage(&env, &contract_id, &reputation_history_key, 1);

    // Second and Third rating submission
    client.rate_seller(&seller_address, &buyer_address, &3, &weight, &None);
    client.rate_seller(&seller_address, &buyer_address, &2, &weight, &None);

    let reputation_score = client.seller_reputation_score(&seller_address);
    assert_eq!(reputation_score, 3);
    assert_storage(&env, &contract_id, &reputation_history_key, 2);
}
