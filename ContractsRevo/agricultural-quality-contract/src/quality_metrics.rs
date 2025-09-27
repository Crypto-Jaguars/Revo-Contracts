use crate::datatypes::*;
use soroban_sdk::{log, symbol_short, vec, Address, BytesN, Env, String, Symbol, Vec};

// Helper function to verify authority
fn verify_authority(env: &Env, authority: &Address) -> Result<(), AgricQualityError> {
    let authorities: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Authorities)
        .unwrap_or_else(|| vec![env]);

    if !authorities.contains(authority) {
        return Err(AgricQualityError::Unauthorized);
    }
    authority.require_auth();
    Ok(())
}

// Helper function to validate metric values
fn validate_metric(min_score: u32, weight: u32) -> Result<(), AgricQualityError> {
    if min_score > 100 || weight > 100 {
        return Err(AgricQualityError::InvalidInput);
    }
    Ok(())
}

// Helper function to get standard-specific requirements
fn get_standard_requirements(standard: &QualityStandard) -> (u32, Vec<Symbol>) {
    match standard {
        QualityStandard::GlobalGAP => (
            70,
            vec![
                &Env::default(),
                symbol_short!("food_safe"), //food_safety
                symbol_short!("traceabil"), // traceability
                symbol_short!("environm"),  // environmental
                symbol_short!("worker_sa"), // worker_safety
            ],
        ),
        QualityStandard::Organic => (
            85,
            vec![
                &Env::default(),
                symbol_short!("pes_free"), // pesticide_free
                symbol_short!("s_health"), // soil_health
                symbol_short!("bio_div"),  //biodiversity
                symbol_short!("gmo_free"), // gmo_free
            ],
        ),
        QualityStandard::Fairtrade => (
            75,
            vec![
                &Env::default(),
                symbol_short!("fairprice"), // fair_price
                symbol_short!("work_cond"), // working_conditions
                symbol_short!("com_dev"),   // community_development
                symbol_short!("env_prot"),  // environmental_protection
            ],
        ),
        QualityStandard::UTZ => (
            80,
            vec![
                &Env::default(),
                symbol_short!("farm_prac"), // farming_practices
                symbol_short!("soc_cond"),  // social_conditions
                symbol_short!("env_man"),   // environmental_management
                symbol_short!("farm_man"),  // farm_management
            ],
        ),
        QualityStandard::NonGMO => (
            95,
            vec![
                &Env::default(),
                symbol_short!("gmo_test"), // gmo_testing
                symbol_short!("segregat"), //segregation
                symbol_short!("traceab"),  // traceability
                symbol_short!("risk_man"), //risk_management
            ],
        ),
        QualityStandard::PDO | QualityStandard::PGI => (
            90,
            vec![
                &Env::default(),
                symbol_short!("orig_ver"),  // origin_verification
                symbol_short!("trad_meth"), //traditional_methods
                symbol_short!("qual_char"), // quality_characteristics
                symbol_short!("local_ing"), //local_ingredients
            ],
        ),
        QualityStandard::Kosher => (
            100,
            vec![
                &Env::default(),
                symbol_short!("ingr_ver"),  // ingredients_verification
                symbol_short!("process_c"), // process_compliance
                symbol_short!("equip_st"),  //equipment_standards
                symbol_short!("supervis"),  //supervision
            ],
        ),
        QualityStandard::GOTS => (
            85,
            vec![
                &Env::default(),
                symbol_short!("org_fiber"), // organic_fiber
                symbol_short!("process"),   //processing
                symbol_short!("soc_cri"),   // social_criteria
                symbol_short!("che_inp"),   // chemical_inputs
            ],
        ),
        QualityStandard::Demeter => (
            90,
            vec![
                &Env::default(),
                symbol_short!("biodynami"), //biodynamic_practices
                symbol_short!("biodiver"),  // biodiversity
                symbol_short!("soil_fert"), // soil_fertility
                symbol_short!("an_wel"),    // animal_welfare
            ],
        ),
        QualityStandard::Custom(_) => (
            75,
            vec![
                &Env::default(),
                symbol_short!("custom_1"), // custom_requirement_1
                symbol_short!("custom_2"), // custom_requirement_2
                symbol_short!("custom_3"), //custom_requirement_3
            ],
        ),
    }
}

pub fn register_metric(
    env: &Env,
    authority: &Address,
    standard: QualityStandard,
    name: Symbol,
    min_score: u32,
    weight: u32,
) -> Result<(), AgricQualityError> {
    // Verify authority and validate inputs
    verify_authority(env, authority)?;
    validate_metric(min_score, weight)?;

    // Check if metric already exists
    let key = DataKey::Metric(standard.clone(), name.clone());
    if env.storage().persistent().has(&key) {
        return Err(AgricQualityError::AlreadyExists);
    }

    // Create new metric
    let metric = QualityMetric {
        name: name.clone(),
        standard: standard.clone(),
        min_score,
        weight,
        version: 1,
        authority: authority.clone(),
    };

    // Store metric
    env.storage().persistent().set(&key, &metric);

    // Update standard metrics list
    let mut metrics: Vec<Symbol> = env
        .storage()
        .persistent()
        .get(&DataKey::StandardMetrics(standard.clone()))
        .unwrap_or_else(|| Vec::new(&env));
    metrics.push_back(name.clone());
    env.storage()
        .persistent()
        .set(&DataKey::StandardMetrics(standard), &metrics);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "metric_registered"),),
        (authority, name, min_score, weight),
    );

    Ok(())
}

pub fn update_metric(
    env: &Env,
    authority: &Address,
    standard: QualityStandard,
    name: Symbol,
    new_min_score: u32,
    new_weight: u32,
) -> Result<(), AgricQualityError> {
    // Verify authority and validate inputs
    verify_authority(env, authority)?;
    validate_metric(new_min_score, new_weight)?;

    // Get existing metric
    let key = DataKey::Metric(standard.clone(), name.clone());
    let mut metric: QualityMetric = env
        .storage()
        .persistent()
        .get(&key)
        .ok_or(AgricQualityError::NotFound)?;

    // Verify authority
    if metric.authority != *authority {
        return Err(AgricQualityError::Unauthorized);
    }

    // Update metric
    metric.min_score = new_min_score;
    metric.weight = new_weight;
    metric.version += 1;

    // Store updated metric
    env.storage().persistent().set(&key, &metric);

    // Emit event
    env.events().publish(
        (Symbol::new(env, "metric_updated"),),
        (authority, name, new_min_score, new_weight),
    );

    Ok(())
}

pub fn get_standard_metrics(
    env: &Env,
    standard: &QualityStandard,
) -> Result<Vec<QualityMetric>, AgricQualityError> {
    let metric_names = env
        .storage()
        .persistent()
        .get(&DataKey::StandardMetrics(standard.clone()))
        .unwrap_or_else(|| Vec::new(&env));

    let mut metrics = vec![env];
    for name in metric_names.iter() {
        if let Some(metric) = env
            .storage()
            .persistent()
            .get(&DataKey::Metric(standard.clone(), name))
        {
            metrics.push_back(metric);
        }
    }

    Ok(metrics)
}

pub fn check_compliance(
    env: &Env,
    certification_id: &BytesN<32>,
    inspector: &Address,
) -> Result<InspectionReport, AgricQualityError> {
    // Verify inspector authorization
    let inspectors: Vec<Address> = env
        .storage()
        .instance()
        .get(&DataKey::Inspectors)
        .unwrap_or_else(|| vec![env]);

    if !inspectors.contains(inspector) {
        return Err(AgricQualityError::Unauthorized);
    }
    inspector.require_auth();
    // Get certification data
    let certification: CertificationData = env
        .storage()
        .persistent()
        .get(&DataKey::Certification(certification_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Get standard requirements
    let (min_overall_score, required_metrics) = get_standard_requirements(&certification.standard);

    // Get metrics for the standard
    let metrics = get_standard_metrics(env, &certification.standard)?;

    // Calculate scores for each required metric
    let mut total_score = 0u32;
    let mut total_weight = 0u32;
    let mut metric_scores = vec![env];
    let mut findings = vec![env];
    let mut recommendations = vec![env];

    for metric in metrics.iter() {
        let score = calculate_metric_score(env, certification_id, &metric)?;

        // Add findings and recommendations based on score
        if score < metric.min_score {
            findings.push_back(String::from_str(
                env,
                "Score below minimum required threshold",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Improve metric score to meet minimum requirements",
            ));
        }

        total_score += score * metric.weight;
        total_weight += metric.weight;
        metric_scores.push_back((metric.name.clone(), score));
    }

    // Calculate overall score
    let overall_score = if total_weight > 0 {
        total_score / total_weight
    } else {
        0
    };

    // Create inspection report
    let report = InspectionReport {
        inspector: inspector.clone(),
        timestamp: env.ledger().timestamp(),
        metrics: metric_scores,
        overall_score,
        findings,
        recommendations,
    };

    // Emit event
    env.events().publish(
        (Symbol::new(env, "compliance_checked"),),
        (certification_id, overall_score),
    );

    Ok(report)
}

// Helper function to calculate score for a specific metric
fn calculate_metric_score(
    env: &Env,
    certification_id: &BytesN<32>,
    metric: &QualityMetric,
) -> Result<u32, AgricQualityError> {
    // Get certification data
    let certification: CertificationData = env
        .storage()
        .persistent()
        .get(&DataKey::Certification(certification_id.clone()))
        .ok_or(AgricQualityError::NotFound)?;

    // Get the latest inspection report
    let inspection: Option<InspectionReport> = env
        .storage()
        .persistent()
        .get(&DataKey::Inspection(certification_id.clone()));

    let base_score = if let Some(report) = inspection {
        // Find the score for this metric in the report
        report
            .metrics
            .iter()
            .find(|(name, _)| name == &metric.name)
            .map(|(_, score)| score)
            .unwrap_or(0)
    } else {
        0
    };

    // Apply time decay factor (reduce score by 1% per day after certification)
    let days_since_cert = (env.ledger().timestamp() - certification.issue_date) / (24 * 60 * 60);
    let time_factor = if days_since_cert > 0 {
        let decay = (days_since_cert) * (1 / 100); // 1% per day
        if decay >= 1 {
            0
        } else {
            ((1 - decay) * base_score as u64) as u32
        }
    } else {
        base_score
    };

    // Apply standard-specific adjustments
    let adjusted_score = match certification.standard {
        QualityStandard::Organic | QualityStandard::NonGMO => {
            // Stricter scoring for organic and non-GMO certifications
            if time_factor < (metric.min_score * 95 / 100) {
                0 // Fail if below 95% of minimum
            } else {
                time_factor
            }
        }
        QualityStandard::Kosher => {
            // Binary scoring for kosher certification
            if time_factor >= metric.min_score {
                100
            } else {
                0
            }
        }
        _ => time_factor,
    };

    // Ensure score doesn't exceed 100
    Ok(adjusted_score.min(100))
}
