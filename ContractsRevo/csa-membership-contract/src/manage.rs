use crate::{CSAMembership, Error};
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

pub fn update_pickup_location(
    env: Env,
    token_id: BytesN<32>,
    new_location: String,
    member: Address,
) -> Result<(), Error> {
    env.logs().add("Starting update_pickup_location", &[]);
    member.require_auth();
    env.logs().add("After require_auth", &[]);

    let mut membership: CSAMembership = env
        .storage()
        .persistent()
        .get(&token_id)
        .ok_or(Error::NotFound)?;
    env.logs().add("After getting membership", &[]);

    if membership.member != member {
        return Err(Error::NotAuthorized);
    }
    env.logs().add("After member check", &[]);

    membership.pickup_location = new_location.clone();
    env.storage().persistent().set(&token_id, &membership);
    env.logs().add("After updating membership", &[]);

    env.events().publish(
        (Symbol::new(&env, "pickup_location_updated"), member.clone()),
        (token_id, new_location),
    );
    env.logs().add("After event publish", &[]);

    Ok(())
}
