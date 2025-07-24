#![no_std]
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol, Vec};

mod datatypes;
mod dispute_handling;
mod interface;
mod quality_metrics;
mod resolution;
mod verification;
mod test;

use datatypes::*;
use interface::*;

#[contract]
pub struct AgricQualityContract;

#[contractimpl]
impl AgricQualityContract {
    pub fn initialize(env: Env, admin: Address) -> Result<(), AdminError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AdminError::AlreadyInitialized);
        }

        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);

        env.events().publish(
            (Symbol::new(&env, "contract_initialized"), admin.clone()),
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, AdminError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AdminError::UnauthorizedAccess)
    }
}

#[contractimpl]
impl QualityStandardsOps for AgricQualityContract {
    fn register_metric(
        env: Env,
        authority: Address,
        standard: QualityStandard,
        name: Symbol,
        min_score: u32,
        weight: u32,
    ) -> Result<(), AgricQualityError> {
        quality_metrics::register_metric(&env, &authority, standard, name, min_score, weight)
    }

    fn update_metric(
        env: Env,
        authority: Address,
        standard: QualityStandard,
        name: Symbol,
        new_min_score: u32,
        new_weight: u32,
    ) -> Result<(), AgricQualityError> {
        quality_metrics::update_metric(&env, &authority, standard, name, new_min_score, new_weight)
    }

    fn get_standard_metrics(
        env: Env,
        standard: QualityStandard,
    ) -> Result<Vec<QualityMetric>, AgricQualityError> {
        quality_metrics::get_standard_metrics(&env, &standard)
    }

    fn check_compliance(
        env: Env,
        certification_id: BytesN<32>,
        inspector: Address,
    ) -> Result<InspectionReport, AgricQualityError> {
        quality_metrics::check_compliance(&env, &certification_id, &inspector)
    }
}

#[contractimpl]
impl VerificationOps for AgricQualityContract {
    fn submit_for_certification(
        env: Env,
        holder: Address,
        standard: QualityStandard,
        conditions: Vec<String>,
    ) -> Result<BytesN<32>, AgricQualityError> {
        verification::submit_for_certification(&env, &holder, standard, conditions)
    }

    fn record_inspection(
        env: Env,
        inspector: Address,
        certification_id: BytesN<32>,
        metrics: Vec<(Symbol, u32)>,
        findings: Vec<String>,
        recommendations: Vec<String>,
    ) -> Result<(), AgricQualityError> {
        verification::record_inspection(
            &env,
            &inspector,
            &certification_id,
            metrics,
            findings,
            recommendations,
        )
    }

    fn process_certification(
        env: Env,
        issuer: Address,
        certification_id: BytesN<32>,
        approved: bool,
        validity_period: u64,
    ) -> Result<(), AgricQualityError> {
        verification::process_certification(
            &env,
            &issuer,
            &certification_id,
            approved,
            validity_period,
        )
    }

    fn get_certification_history(
        env: Env,
        holder: Address,
    ) -> Result<Vec<CertificationData>, AgricQualityError> {
        verification::get_certification_history(&env, &holder)
    }
}

#[contractimpl]
impl DisputeOps for AgricQualityContract {
    fn file_dispute(
        env: Env,
        complainant: Address,
        certification_id: BytesN<32>,
        description: String,
        evidence: Vec<BytesN<32>>,
    ) -> Result<BytesN<32>, AgricQualityError> {
        dispute_handling::file_dispute(&env, &complainant, &certification_id, description, evidence)
    }

    fn submit_evidence(
        env: Env,
        handler: Address,
        dispute_id: BytesN<32>,
        description: String,
        data_type: Symbol,
        metadata: Vec<(Symbol, String)>,
    ) -> Result<BytesN<32>, AgricQualityError> {
        dispute_handling::submit_evidence(
            &env,
            &handler,
            &dispute_id,
            description,
            data_type,
            metadata,
        )
    }

    fn assign_mediator(
        env: Env,
        authority: Address,
        dispute_id: BytesN<32>,
        mediator: Address,
    ) -> Result<(), AgricQualityError> {
        dispute_handling::assign_mediator(&env, &authority, &dispute_id, &mediator)
    }

    fn get_dispute_details(
        env: Env,
        dispute_id: BytesN<32>,
    ) -> Result<DisputeData, AgricQualityError> {
        dispute_handling::get_dispute_details(&env, &dispute_id)
    }
}

#[contractimpl]
impl ResolutionOps for AgricQualityContract {
    fn resolve_dispute(
        env: Env,
        mediator: Address,
        dispute_id: BytesN<32>,
        outcome: ResolutionOutcome,
        notes: String,
    ) -> Result<(), AgricQualityError> {
        resolution::resolve_dispute(&env, &mediator, &dispute_id, outcome, notes)
    }

    fn process_appeal(
        env: Env,
        appellant: Address,
        dispute_id: BytesN<32>,
        new_evidence: Vec<BytesN<32>>,
        justification: String,
    ) -> Result<(), AgricQualityError> {
        resolution::process_appeal(&env, &appellant, &dispute_id, new_evidence, justification)
    }

    fn calculate_compensation(env: Env, dispute_id: BytesN<32>) -> Result<u32, AgricQualityError> {
        resolution::calculate_compensation(&env, &dispute_id)
    }

    fn track_enforcement(
        env: Env,
        authority: Address,
        dispute_id: BytesN<32>,
        enforced: bool,
        notes: String,
    ) -> Result<(), AgricQualityError> {
        resolution::track_enforcement(&env, &authority, &dispute_id, enforced, notes)
    }
}
