use soroban_sdk::{Address, Env, Vec};

use crate::{AuditError, CertStatus, Certification, DataKey, UsersCertificates};

pub fn generate_cert_audit_report(
    env: Env,
    owner: Address,
    issuer: Option<Address>,
    status_filter: Option<CertStatus>,
    after_timestamp: Option<u64>,
) -> Result<Vec<Certification>, AuditError> {
    let mut report = Vec::new(&env);

    let users_certificates: UsersCertificates = env
        .storage()
        .instance()
        .get(&DataKey::UsersCertificates)
        .ok_or(AuditError::NotFound)?;

    let user_certificates = users_certificates
        .get(owner.clone())
        .ok_or(AuditError::NotFound)?;

    for (_, cert) in user_certificates.iter() {
        if let Some(ref issuer) = issuer {
            if cert.issuer != *issuer {
                continue;
            }
        }

        if let Some(ref status) = status_filter {
            if cert.status != *status {
                continue;
            }
        }

        if let Some(min_time) = after_timestamp {
            if cert.issued_date < min_time {
                continue;
            }
        }

        report.push_back(cert);
    }

    Ok(report)
}