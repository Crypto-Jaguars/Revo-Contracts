use soroban_sdk::{BytesN, Env};

use crate::{
    datatypes::{CarbonCredit, DataKey},
    error::ContractError,
    interfaces::VerificationContract,
    EnvironmentalContract,
};

impl VerificationContract for EnvironmentalContract {
    /// Checks if credit has a non-empty verification method
    fn verify_credit(env: &Env, credit_id: BytesN<32>) -> Result<bool, ContractError> {
        match env
            .storage()
            .persistent()
            .get::<DataKey, CarbonCredit>(&DataKey::Credit(credit_id))
        {
            Some(credit) => Ok(!credit.verification_method.is_empty()),
            None => Err(ContractError::CreditNotFound),
        }
    }
}
