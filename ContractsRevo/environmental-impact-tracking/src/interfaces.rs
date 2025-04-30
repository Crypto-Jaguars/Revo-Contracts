use soroban_sdk::{Address, BytesN, Env, String, Vec};

use crate::{datatypes::RetirementStatus, error::ContractError};

pub trait CarbonContract {
    fn issue_carbon_credit(
        env: &Env,
        credit_id: BytesN<32>,
        project_id: BytesN<32>,
        carbon_amount: u32,
        verification_method: String,
    ) -> Result<(), ContractError>;

    fn get_credit_status(
        env: &Env,
        credit_id: BytesN<32>,
    ) -> Result<RetirementStatus, ContractError>;

    fn list_credits_by_project(
        env: &Env,
        project_id: BytesN<32>,
    ) -> Result<Vec<BytesN<32>>, ContractError>;
}

#[allow(dead_code)]
pub trait VerificationContract {
    fn verify_credit(env: &Env, credit_id: BytesN<32>) -> Result<bool, ContractError>;
}

#[allow(dead_code)]
pub trait ReportingContract {
    fn generate_impact_report(env: &Env, project_id: BytesN<32>) -> u32;
}

#[allow(dead_code)]
pub trait RetirementContract {
    fn retire_credit(
        env: &Env,
        credit_id: BytesN<32>,
        retiree: Address,
    ) -> Result<(), ContractError>;
    fn set_retirement_status(
        env: &Env,
        credit_id: BytesN<32>,
        status: RetirementStatus,
    ) -> Result<(), ContractError>;
    fn get_retirement_status(
        env: &Env,
        credit_id: BytesN<32>,
    ) -> Result<RetirementStatus, ContractError>;
}
