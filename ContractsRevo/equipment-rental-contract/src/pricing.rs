use soroban_sdk::Env;
use crate::equipment::Equipment;

/// Compute the total rental price for a given period
pub fn compute_total_price(equipment: &Equipment, start_date: u64, end_date: u64) -> i128 {
    if end_date <= start_date {
        panic!("End date must be after start date");
    }
    let duration_days = end_date - start_date;
    equipment.rental_price_per_day * (duration_days as i128)
}

/// Validate that the proposed price matches the expected price for the rental period
pub fn validate_price(equipment: &Equipment, start_date: u64, end_date: u64, proposed_price: i128) {
    let expected = compute_total_price(equipment, start_date, end_date);
    if proposed_price != expected {
        panic!("Proposed price does not match expected price");
    }
}
