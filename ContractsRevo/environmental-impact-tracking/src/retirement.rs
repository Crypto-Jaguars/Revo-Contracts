use soroban_sdk::{ Address, BytesN, Env};

use crate::{
    datatypes::{CarbonCredit, DataKey, RetirementStatus},
    error::ContractError,
    interfaces::RetirementContract,
    EnvironmentalContract,
};

impl RetirementContract for EnvironmentalContract {
    /// Marks credit as retired by specified address
    fn retire_credit(
        env: &Env,
        credit_id: BytesN<32>,
        retiree: Address,
    ) -> Result<(), ContractError> {
        let mut credit: CarbonCredit = match env
            .storage()
            .persistent()
            .get::<DataKey, CarbonCredit>(&DataKey::Credit(credit_id.clone()))
        {
            Some(credit) => credit,
            None => return Err(ContractError::CreditNotFound),
        };

        match credit.retirement_status {
            RetirementStatus::Available => {
                credit.retirement_status = RetirementStatus::Retired(retiree.clone());
                env.storage()
                    .persistent()
                    .set(&DataKey::Credit(credit_id), &credit);
                Ok(())
            }
            RetirementStatus::Retired(_) => Err(ContractError::AlreadyRetired),
        }
    }

    /// Updates credit's retirement status (admin function)
    fn set_retirement_status(
        env: &Env,
        credit_id: BytesN<32>,
        status: RetirementStatus,
    ) -> Result<(), ContractError> {
        let mut credit: CarbonCredit = match env
            .storage()
            .persistent()
            .get::<DataKey, CarbonCredit>(&DataKey::Credit(credit_id.clone()))
        {
            Some(credit) => credit,
            None => return Err(ContractError::CreditNotFound),
        };

        credit.retirement_status = status;
        env.storage()
            .persistent()
            .set(&DataKey::Credit(credit_id), &credit);
        Ok(())
    }

    /// Returns current retirement status
    fn get_retirement_status(
        env: &Env,
        credit_id: BytesN<32>,
    ) -> Result<RetirementStatus, ContractError> {
        match env
            .storage()
            .persistent()
            .get::<DataKey, CarbonCredit>(&DataKey::Credit(credit_id))
        {
            Some(credit) => Ok(credit.retirement_status),
            None => Err(ContractError::CreditNotFound),
        }
    }
}
