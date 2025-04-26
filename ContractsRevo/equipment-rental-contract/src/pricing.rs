use soroban_sdk::Env;
use crate::equipment::Equipment;

#[derive(Debug, Eq, PartialEq)]
pub enum PriceValidationError {
    InvalidDate,
    PriceMismatch { proposed: i128, expected: i128 },
}


/// Compute the total rental price for a given period
pub fn compute_total_price(
    equipment: &Equipment,
    start_date: u64,
    end_date: u64,
) -> Result<i128, PriceValidationError> {
    if end_date <= start_date {
        return Err(PriceValidationError::InvalidDate);
    }
    let duration_days = end_date - start_date;
    Ok(equipment.rental_price_per_day * (duration_days as i128))
} 

/// Validate that the proposed price matches the expected price for the rental period
pub fn validate_price(
    equipment: &Equipment,
    start_date: u64,
    end_date: u64,
    proposed_price: i128,
) -> Result<(), PriceValidationError> {
    let expected = compute_total_price(equipment, start_date, end_date)?;
    if proposed_price != expected {
        return Err(PriceValidationError::PriceMismatch {
            proposed: proposed_price,
            expected,
        });
    }
    Ok(())
}
