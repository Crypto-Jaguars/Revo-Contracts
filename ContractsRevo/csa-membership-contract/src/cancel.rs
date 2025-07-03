use crate::{CSAMembership, Error};
use soroban_sdk::{Address, BytesN, Env, Symbol};

pub fn cancel_membership(env: Env, token_id: BytesN<32>, member: Address) -> Result<(), Error> {
    env.logs().add("Starting cancel_membership", &[]);
    member.require_auth();
    env.logs().add("After require_auth", &[]);

    let membership: CSAMembership = env
        .storage()
        .persistent()
        .get(&token_id)
        .ok_or(Error::NotFound)?;
    env.logs().add("After getting membership", &[]);

    if membership.member != member {
        return Err(Error::NotAuthorized);
    }
    env.logs().add("After member check", &[]);

    env.storage().persistent().remove(&token_id);
    env.logs().add("After removing membership", &[]);

    // Evento modificado con Symbol
    env.events().publish(
        (Symbol::new(&env, "membership_cancelled"), member.clone()),
        token_id,
    );
    env.logs().add("After event publish", &[]);

    Ok(())
}
