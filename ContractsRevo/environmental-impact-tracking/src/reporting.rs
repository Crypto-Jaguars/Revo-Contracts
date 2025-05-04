use soroban_sdk::{BytesN, Env, Vec};

use crate::EnvironmentalContract;
use crate::{
    datatypes::{CarbonCredit, DataKey, RetirementStatus},
    interfaces::ReportingContract,
};

impl ReportingContract for EnvironmentalContract {
    fn generate_impact_report(env: &Env, project_id: BytesN<32>) -> u32 {
        // Get project credits with explicit type annotation
        let credits: Vec<BytesN<32>> = match env
            .storage()
            .persistent()
            .get::<DataKey, Vec<BytesN<32>>>(&DataKey::ProjectCredits(project_id))
        {
            Some(c) => c,
            None => Vec::new(env),
        };

        let mut total_offset = 0u32;

        for credit_id in credits.iter() {
            // Get credit with explicit type annotation
            if let Some(credit) = env
                .storage()
                .persistent()
                .get::<DataKey, CarbonCredit>(&DataKey::Credit(credit_id.clone()))
            {
                if let RetirementStatus::Retired(_) = credit.retirement_status {
                    total_offset += credit.carbon_amount;
                }
            }
        }

        total_offset
    }
}
