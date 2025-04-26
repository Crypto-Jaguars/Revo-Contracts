use soroban_sdk::Env;
use crate::equipment::Equipment;

#[derive(Debug, Eq, PartialEq)]
pub enum PriceValidationError {
    InvalidDate,
    PriceMismatch {
        proposed: i128,
        expected: i128,
        context: &'static str,
    },
}


/// Compute the total rental price for a given period
///
/// # Arguments
/// * `start_date` and `end_date` are days since Unix epoch (not timestamps in seconds)
///
/// Returns an error if the period is invalid or if arithmetic overflows.
pub fn compute_total_price(
    equipment: &Equipment,
    start_date: u64,
    end_date: u64,
) -> Result<i128, PriceValidationError> {
    let duration_days = end_date
        .checked_sub(start_date)
        .ok_or(PriceValidationError::InvalidDate)?;
    equipment
        .rental_price_per_day
        .checked_mul(duration_days.into())
        .ok_or(PriceValidationError::InvalidDate)
} 

/// Validate that the proposed price matches the expected price for the rental period
pub fn validate_price(
    equipment: &Equipment,
    start_date: u64,
    end_date: u64,
    proposed_price: i128,
    tolerance: i128,
) -> Result<(), PriceValidationError> {
    let expected = compute_total_price(equipment, start_date, end_date)?;
    let diff = if proposed_price > expected {
        proposed_price - expected
    } else {
        expected - proposed_price
    };
    if diff > tolerance {
        return Err(PriceValidationError::PriceMismatch {
            proposed: proposed_price,
            expected,
            context: "Proposed price does not match expected price within allowed tolerance",
        });
    }
    Ok(())
}
