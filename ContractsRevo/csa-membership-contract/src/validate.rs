use soroban_sdk::{BytesN, Env, String, IntoVal};
use crate::Error;

pub fn validate_season(env: &Env, farm_id: BytesN<32>, season: String, start_date: u64, end_date: u64) -> Result<(), Error> {
    let current_time = env.ledger().timestamp();
    env.logs().add("validate_season", &[start_date.into_val(env), current_time.into_val(env)]);

    let empty_farm_id = BytesN::from_array(env, &[0; 32]);
    if farm_id == empty_farm_id {
        return Err(Error::InvalidFarm);
    }

    if season.is_empty() {
        return Err(Error::InvalidSeason);
    }

    if start_date <= current_time {
        return Err(Error::InvalidDates);
    }
    if end_date <= start_date {
        return Err(Error::InvalidDates);
    }

    Ok(())
}