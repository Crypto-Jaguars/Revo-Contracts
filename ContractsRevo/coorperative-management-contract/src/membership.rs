use crate::datatype::{CooperativeError, DataKey, Member};
use crate::interface::Membership;
use crate::CooperativeManagementContract;
use soroban_sdk::{Address, Env, String};

impl Membership for CooperativeManagementContract {
    fn register_member(env: Env, address: Address, name: String) {
        let member = Member {
            address: address.clone(),
            name,
            reputation: 0,
            contributions: 0,
            verified: false,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Member(address), &member);
    }

    fn verify_member(
        env: Env,
        admin: Address,
        address: Address,
    ) -> Result<(), CooperativeError> {
        admin.require_auth();
        let address_key = DataKey::Member(address);
        if let Some(mut member) = env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&address_key)
        {
            member.verified = true;
            env.storage().persistent().set(&address_key, &member);
            Ok(())
        } else {
            Err(CooperativeError::MemberNotFound)
        }
    }

    fn track_contribution(
        env: Env,
        address: Address,
        amount: u32,
    ) -> Result<(), CooperativeError> {
        let address_key = DataKey::Member(address);
        if let Some(mut member) = env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&address_key)
        {
            member.contributions += amount;
            env.storage().persistent().set(&address_key, &member);
            Ok(())
        } else {
            Err(CooperativeError::MemberNotFound)
        }
    }

    fn update_reputation(
        env: Env,
        address: Address,
        points: u32,
    ) -> Result<(), CooperativeError> {
        let address_key = DataKey::Member(address);
        if let Some(mut member) = env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&address_key)
        {
            member.reputation += points;
            env.storage().persistent().set(&address_key, &member);
            Ok(())
        } else {
            Err(CooperativeError::MemberNotFound)
        }
    }
}
