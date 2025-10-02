use crate::{
    AgriculturalAuctionContract, AgriculturalProduct, FreshnessRating, QualityGrade,
    SeasonalStatus, StorageCondition,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol, Vec};

pub struct TestEnv {
    pub env: Env,
    pub contract_id: Address,
    pub admin: Address,
    pub farmer: Address,
    pub bidder1: Address,
    pub bidder2: Address,
    pub bidder3: Address,
}

pub fn setup_test() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(AgriculturalAuctionContract, ());
    let admin = Address::generate(&env);
    let farmer = Address::generate(&env);
    let bidder1 = Address::generate(&env);
    let bidder2 = Address::generate(&env);
    let bidder3 = Address::generate(&env);

    TestEnv {
        env,
        contract_id,
        admin,
        farmer,
        bidder1,
        bidder2,
        bidder3,
    }
}

pub fn create_standard_product(env: &Env, farmer: Address, product_id: u64) -> AgriculturalProduct {
    let current_time = env.ledger().timestamp();

    AgriculturalProduct {
        id: product_id,
        farmer: farmer.clone(),
        name: Symbol::new(env, "Tomatoes"),
        description: String::from_str(env, "Fresh organic tomatoes"),
        base_price: 100,
        current_price: 100,
        weight_kg: 50,
        quantity: 100,
        harvest_date: current_time,
        expiry_date: current_time + 86400 * 7, // 7 days from now
        images: {
            let mut vec = Vec::new(env);
            vec.push_back(String::from_str(env, "image1.jpg"));
            vec
        },
        freshness_rating: FreshnessRating::Premium,
        quality_grade: QualityGrade::GradeA,
        verified: true,
        certifications: {
            let mut vec = Vec::new(env);
            vec.push_back(Symbol::new(env, "Organic"));
            vec
        },
        storage_condition: StorageCondition::Refrigerated,
        product_type: Symbol::new(env, "Tomato"),
        region: Symbol::new(env, "North"),
        seasonal_status: SeasonalStatus::InSeason,
    }
}

pub const STANDARD_RESERVE_PRICE: u64 = 100;
pub const STANDARD_MIN_QUANTITY: u32 = 10;
pub const STANDARD_BULK_THRESHOLD: u32 = 50;
pub const STANDARD_BULK_DISCOUNT: u32 = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test() {
        let test_env = setup_test();
        assert!(test_env.env.ledger().timestamp() >= 0);
    }

    #[test]
    fn test_create_standard_product() {
        let test_env = setup_test();
        let product = create_standard_product(&test_env.env, test_env.farmer.clone(), 1);
        assert_eq!(product.id, 1);
        assert_eq!(product.quantity, 100);
        assert_eq!(product.quality_grade, QualityGrade::GradeA);
    }
}
