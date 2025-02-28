use crate::CooperativeManagementContract;
use crate::datatype::{CooperativeError, DataKey};
use crate::interface::ProfitDistribution;
use soroban_sdk::{Address, Env, Map, Vec};

impl ProfitDistribution for CooperativeManagementContract {
    fn distribute_profits(
        env: Env,
        profits: i128,
        members: Vec<Address>,
    ) -> Result<Map<Address, i128>, CooperativeError> {
        let mut distribution = Map::new(&env);
        if members.is_empty() {
            return Err(CooperativeError::InvalidInput);
        }
        let share = profits / (members.len() as i128);
        for member in members.iter() {
            distribution.set(member.clone(), share);
        }
        Ok(distribution)
    }

    fn share_expenses(
        env: Env,
        total_expense: i128,
        members: Vec<Address>,
    ) -> Result<Map<Address, i128>, CooperativeError> {
        let mut expenses = Map::new(&env);
        if members.is_empty() {
            return Err(CooperativeError::InvalidInput);
        }
        let share = total_expense / (members.len() as i128);
        for member in members.iter() {
            expenses.set(member.clone(), share);
        }
        Ok(expenses)
    }

    fn pool_investment(env: Env, investor: Address, amount: i128) -> Result<(), CooperativeError> {
        let key = DataKey::Investment(investor.clone());
        let mut total = env
            .storage()
            .persistent()
            .get::<DataKey, i128>(&key)
            .unwrap_or(0);
        total += amount;
        env.storage().persistent().set(&key, &total);
        Ok(())
    }

    fn process_automated_payments(
        env: Env,
        members: Vec<Address>,
        amount: i128,
    ) -> Result<(), CooperativeError> {
        for member in members.iter() {
            let key = DataKey::Balance(member.clone());
            let mut balance = env
                .storage()
                .persistent()
                .get::<DataKey, i128>(&key)
                .unwrap_or(0);
            if balance < amount {
                return Err(CooperativeError::InsufficientFunds);
            }
            balance -= amount;
            env.storage().persistent().set(&key, &balance);
        }
        Ok(())
    }
}
