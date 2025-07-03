use crate::{CSAMembership, Error, ShareSize};
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

pub fn enroll_membership(
    env: Env,
    farm_id: BytesN<32>,
    season: String,
    share_size: ShareSize,
    pickup_location: String,
    start_date: u64,
    end_date: u64,
    member: Address,
) -> Result<BytesN<32>, Error> {
    env.logs().add("Starting enroll_membership", &[]);
    member.require_auth();
    env.logs().add("After require_auth", &[]);

    crate::validate::validate_season(&env, farm_id.clone(), season.clone(), start_date, end_date)?;
    env.logs().add("After validate_season", &[]);

    let token_id = BytesN::from_array(&env, &[0; 32]);
    env.logs().add("After generating token_id", &[]);

    let membership = CSAMembership {
        farm_id,
        season,
        share_size,
        pickup_location,
        start_date,
        end_date,
        member: member.clone(),
    };
    env.storage().persistent().set(&token_id, &membership);
    env.logs().add("After storage set", &[]);

    env.events().publish(
        (Symbol::new(&env, "membership_enrolled"), member.clone()),
        token_id.clone(),
    );
    env.logs().add("After event publish", &[]);

    Ok(token_id)
}
