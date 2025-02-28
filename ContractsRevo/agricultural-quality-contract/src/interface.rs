use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};
use crate::datatypes::*;

/// Handles quality standards management operations
pub trait QualityStandardsOps {
    /// Register a new quality metric for a standard
    /// * `authority` - Address authorized to register metrics
    /// * `standard` - Quality standard being registered
    /// * `name` - Name of the metric
    /// * `min_score` - Minimum acceptable score (0-100)
    /// * `weight` - Weight of this metric in overall scoring (0-100)
    fn register_metric(
        env: Env,
        authority: Address,
        standard: QualityStandard,
        name: Symbol,
        min_score: u32,
        weight: u32,
    ) -> Result<(), AgricQualityError>;

    /// Update an existing quality metric
    /// * `authority` - Address authorized to update metrics
    /// * `standard` - Quality standard being updated
    /// * `name` - Name of the metric to update
    /// * `new_min_score` - New minimum acceptable score
    /// * `new_weight` - New weight for the metric
    fn update_metric(
        env: Env,
        authority: Address,
        standard: QualityStandard,
        name: Symbol,
        new_min_score: u32,
        new_weight: u32,
    ) -> Result<(), AgricQualityError>;

    /// Get all metrics for a specific standard
    /// * `standard` - Quality standard to get metrics for
    fn get_standard_metrics(
        env: Env,
        standard: QualityStandard,
    ) -> Result<Vec<QualityMetric>, AgricQualityError>;

    /// Check compliance against quality standards
    /// * `certification_id` - ID of certification to check
    /// * `inspector` - Address of authorized inspector
    fn check_compliance(
        env: Env,
        certification_id: BytesN<32>,
        inspector: Address,
    ) -> Result<InspectionReport, AgricQualityError>;
}

/// Manages verification and certification processes
pub trait VerificationOps {
    /// Submit for certification under a quality standard
    /// * `holder` - Address requesting certification
    /// * `standard` - Quality standard to certify against
    /// * `conditions` - Specific conditions or requirements
    fn submit_for_certification(
        env: Env,
        holder: Address,
        standard: QualityStandard,
        conditions: Vec<String>,
    ) -> Result<BytesN<32>, AgricQualityError>;

    /// Record inspection results for a certification
    /// * `inspector` - Address of authorized inspector
    /// * `certification_id` - ID of certification being inspected
    /// * `metrics` - List of metric scores
    /// * `findings` - Detailed inspection findings
    /// * `recommendations` - Improvement recommendations
    fn record_inspection(
        env: Env,
        inspector: Address,
        certification_id: BytesN<32>,
        metrics: Vec<(Symbol, u32)>,
        findings: Vec<String>,
        recommendations: Vec<String>,
    ) -> Result<(), AgricQualityError>;

    /// Process certification approval or rejection
    /// * `issuer` - Address authorized to issue certifications
    /// * `certification_id` - ID of certification to process
    /// * `approved` - Whether certification is approved
    /// * `validity_period` - Duration of validity in seconds
    fn process_certification(
        env: Env,
        issuer: Address,
        certification_id: BytesN<32>,
        approved: bool,
        validity_period: u64,
    ) -> Result<(), AgricQualityError>;

    /// Get certification history for a holder
    /// * `holder` - Address to get history for
    fn get_certification_history(
        env: Env,
        holder: Address,
    ) -> Result<Vec<CertificationData>, AgricQualityError>;
}

/// Handles dispute filing and management
pub trait DisputeOps {
    /// File a new dispute against a certification
    /// * `complainant` - Address filing the dispute
    /// * `certification_id` - ID of disputed certification
    /// * `description` - Detailed description of dispute
    /// * `evidence` - List of evidence hashes
    fn file_dispute(
        env: Env,
        complainant: Address,
        certification_id: BytesN<32>,
        description: String,
        evidence: Vec<BytesN<32>>,
    ) -> Result<BytesN<32>, AgricQualityError>;

    /// Submit evidence for an existing dispute
    /// * `handler` - Address submitting evidence
    /// * `dispute_id` - ID of dispute
    /// * `description` - Description of evidence
    /// * `data_type` - Type of evidence data
    /// * `metadata` - Additional evidence metadata
    fn submit_evidence(
        env: Env,
        handler: Address,
        dispute_id: BytesN<32>,
        description: String,
        data_type: Symbol,
        metadata: Vec<(Symbol, String)>,
    ) -> Result<BytesN<32>, AgricQualityError>;

    /// Assign a mediator to handle a dispute
    /// * `authority` - Address authorized to assign mediators
    /// * `dispute_id` - ID of dispute
    /// * `mediator` - Address of assigned mediator
    fn assign_mediator(
        env: Env,
        authority: Address,
        dispute_id: BytesN<32>,
        mediator: Address,
    ) -> Result<(), AgricQualityError>;

    /// Get details of a specific dispute
    /// * `dispute_id` - ID of dispute to get details for
    fn get_dispute_details(
        env: Env,
        dispute_id: BytesN<32>,
    ) -> Result<DisputeData, AgricQualityError>;
}

/// Manages dispute resolution and enforcement
pub trait ResolutionOps {
    /// Resolve a dispute with final outcome
    /// * `mediator` - Address of authorized mediator
    /// * `dispute_id` - ID of dispute to resolve
    /// * `outcome` - Resolution outcome
    /// * `notes` - Detailed resolution notes
    fn resolve_dispute(
        env: Env,
        mediator: Address,
        dispute_id: BytesN<32>,
        outcome: ResolutionOutcome,
        notes: String,
    ) -> Result<(), AgricQualityError>;

    /// Process an appeal against a resolution
    /// * `appellant` - Address filing appeal
    /// * `dispute_id` - ID of disputed resolution
    /// * `new_evidence` - New evidence for appeal
    /// * `justification` - Justification for appeal
    fn process_appeal(
        env: Env,
        appellant: Address,
        dispute_id: BytesN<32>,
        new_evidence: Vec<BytesN<32>>,
        justification: String,
    ) -> Result<(), AgricQualityError>;

    /// Calculate compensation amount for resolved dispute
    /// * `dispute_id` - ID of resolved dispute
    fn calculate_compensation(
        env: Env,
        dispute_id: BytesN<32>,
    ) -> Result<u32, AgricQualityError>;

    /// Track enforcement of resolution
    /// * `authority` - Address authorized to track enforcement
    /// * `dispute_id` - ID of resolved dispute
    /// * `enforced` - Whether resolution was enforced
    /// * `notes` - Enforcement notes
    fn track_enforcement(
        env: Env,
        authority: Address,
        dispute_id: BytesN<32>,
        enforced: bool,
        notes: String,
    ) -> Result<(), AgricQualityError>;
} 
