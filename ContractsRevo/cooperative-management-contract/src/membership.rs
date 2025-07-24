use crate::datatype::{CooperativeError, DataKey, Member};
use crate::interface::Membership;
use crate::{
    CooperativeManagementContract, CooperativeManagementContractArgs,
    CooperativeManagementContractClient,
};
use soroban_sdk::{contractimpl, Address, Env, String};

#[contractimpl]
impl Membership for CooperativeManagementContract {
    fn register_member(
        env: Env,
        address: Address,
        name: String,
        role: String,
    ) -> Result<(), CooperativeError> {
        let key = DataKey::Member(address.clone());

        // Check if the member is already registered
        if env.storage().persistent().has(&key) {
            return Err(CooperativeError::MemberAlreadyExists);
        }

        let member = Member {
            address: address.clone(),
            name,
            role,
            reputation: 0,
            contributions: 0,
            verified: false,
        };

        env.storage().persistent().set(&key, &member);

        Ok(())
    }

    fn verify_member(env: Env, admin: Address, address: Address) -> Result<(), CooperativeError> {
        admin.require_auth();
        let address_key = DataKey::Member(address.clone());
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

    fn track_contribution(env: Env, address: Address, amount: u32) -> Result<(), CooperativeError> {
        let address_key = DataKey::Member(address.clone());
        if let Some(mut member) = env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&address_key)
        {
            member.contributions += &amount;
            env.storage().persistent().set(&address_key, &member);
            Ok(())
        } else {
            Err(CooperativeError::MemberNotFound)
        }
    }

    fn update_reputation(
        env: Env,
        admin: Address,
        address: Address,
        points: u32,
    ) -> Result<(), CooperativeError> {
        // Ensure admin authorization
        admin.require_auth();

        let address_key = DataKey::Member(address.clone());
        if let Some(mut member) = env
            .storage()
            .persistent()
            .get::<DataKey, Member>(&address_key)
        {
            member.reputation += &points;
            env.storage().persistent().set(&address_key, &member);
            Ok(())
        } else {
            Err(CooperativeError::MemberNotFound)
        }
    }
}
