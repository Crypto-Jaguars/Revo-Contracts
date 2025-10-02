use crate::datatype::DataKey;
use crate::CooperativeManagementContract;
use soroban_sdk::{testutils::Address as _, Address, Env, String};

pub struct TestEnv {
    pub env: Env,
    pub contract_id: Address,
    pub admin: Address,
    pub member1: Address,
    pub member2: Address,
    pub member3: Address,
}

pub fn setup_test() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let member1 = Address::generate(&env);
    let member2 = Address::generate(&env);
    let member3 = Address::generate(&env);

    let contract_id = env.register(CooperativeManagementContract, ());

    // Store admin address in contract storage
    env.as_contract(&contract_id, || {
        let admin_key = DataKey::Admin;
        env.storage().persistent().set(&admin_key, &admin);
    });

    TestEnv {
        env,
        contract_id,
        admin,
        member1,
        member2,
        member3,
    }
}

pub fn standard_member_name(env: &Env) -> String {
    String::from_str(env, "John Doe")
}

pub fn standard_farmer_role(env: &Env) -> String {
    String::from_str(env, "Farmer")
}

pub fn standard_manager_role(env: &Env) -> String {
    String::from_str(env, "Manager")
}

pub fn standard_resource_description(env: &Env) -> String {
    String::from_str(env, "Tractor")
}

pub fn standard_proposal_description(env: &Env) -> String {
    String::from_str(env, "Expand cooperative to neighboring region")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test() {
        let test_env = setup_test();
        assert!(test_env.env.ledger().timestamp() >= 0);
    }

    #[test]
    fn test_standard_member_name() {
        let test_env = setup_test();
        let name = standard_member_name(&test_env.env);
        assert_eq!(name, String::from_str(&test_env.env, "John Doe"));
    }

    #[test]
    fn test_standard_farmer_role() {
        let test_env = setup_test();
        let role = standard_farmer_role(&test_env.env);
        assert_eq!(role, String::from_str(&test_env.env, "Farmer"));
    }

    #[test]
    fn test_admin_setup() {
        let test_env = setup_test();
        // Verify admin is stored
        let admin_key = DataKey::Admin;
        let stored_admin = test_env.env.as_contract(&test_env.contract_id, || {
            test_env
                .env
                .storage()
                .persistent()
                .get::<DataKey, Address>(&admin_key)
        });
        assert_eq!(stored_admin, Some(test_env.admin));
    }
}
