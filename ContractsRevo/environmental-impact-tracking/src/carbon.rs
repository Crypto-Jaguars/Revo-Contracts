use soroban_sdk::{contractimpl, BytesN, Env, String, Symbol, Vec};

use crate::datatypes::{CarbonCredit, DataKey, RetirementStatus};
use crate::error::ContractError;
use crate::interfaces::CarbonContract;
use crate::{EnvironmentalContract, EnvironmentalContractArgs, EnvironmentalContractClient};

#[contractimpl]
impl CarbonContract for EnvironmentalContract {
    /// Issues new carbon credit after validating all parameters
    fn issue_carbon_credit(
        env: &Env,
        credit_id: BytesN<32>,
        project_id: BytesN<32>,
        carbon_amount: u32,
        verification_method: String,
    ) -> Result<(), ContractError> {
        if carbon_amount == 0 {
            return Err(ContractError::ZeroAmount);
        }

        const MAX_AMOUNT: u32 = 1_000_000_000;
        if carbon_amount > MAX_AMOUNT {
            return Err(ContractError::InvalidAmount);
        }

        if credit_id == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(ContractError::InvalidIdentifier);
        }

        if project_id == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(ContractError::InvalidIdentifier);
        }

        if verification_method.is_empty() {
            return Err(ContractError::InvalidVerificationMethod);
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Credit(credit_id.clone()))
        {
            return Err(ContractError::CreditAlreadyExists);
        }

        let issuance_date = env.ledger().timestamp();
        let credit = CarbonCredit {
            project_id: project_id.clone(),
            carbon_amount,
            verification_method,
            issuance_date,
            retirement_status: RetirementStatus::Available,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Credit(credit_id.clone()), &credit);

        let project_credits: Vec<BytesN<32>> = match env
            .storage()
            .persistent()
            .get(&DataKey::ProjectCredits(project_id.clone()))
        {
            Some(credits) => credits,
            None => Vec::new(env),
        };

        let mut updated_project_credits = project_credits;
        updated_project_credits.push_back(credit_id.clone());
        env.storage().persistent().set(
            &DataKey::ProjectCredits(project_id.clone()),
            &updated_project_credits,
        );

        env.events().publish(
            (Symbol::new(&env, "Carbon_Credit_Issued"), credit_id),
            (project_id, issuance_date, carbon_amount),
        );
        Ok(())
    }

    /// Returns retirement status of specified credit
    fn get_credit_status(
        env: &Env,
        credit_id: BytesN<32>,
    ) -> Result<RetirementStatus, ContractError> {
        if credit_id == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(ContractError::InvalidIdentifier);
        }

        let credit: CarbonCredit = match env.storage().persistent().get(&DataKey::Credit(credit_id))
        {
            Some(x) => x,
            None => return Err(ContractError::CreditNotFound),
        };

        Ok(credit.retirement_status)
    }

    /// Lists all credit IDs associated with a project
    fn list_credits_by_project(
        env: &Env,
        project_id: BytesN<32>,
    ) -> Result<Vec<BytesN<32>>, ContractError> {
        if project_id == BytesN::from_array(&env, &[0u8; 32]) {
            return Err(ContractError::InvalidIdentifier);
        }

        match env
            .storage()
            .persistent()
            .get(&DataKey::ProjectCredits(project_id))
        {
            Some(credits) => Ok(credits),
            None => Ok(Vec::new(env)),
        }
    }
}
