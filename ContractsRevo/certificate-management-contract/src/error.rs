use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminError {
    AlreadyInitialized = 1,
    Uninitialized = 2,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssueError {
    InvalidExpirationDate = 1,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditError {
    NotFound = 1,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevokeError {
    NotFound = 1,
    AlreadyRevoked = 2,
    Unauthorized = 3,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    NotFound = 1,
    HashMismatch = 2,
    Expired = 3,
    Revoked = 4,
    ExpirationDue = 5,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificationError {
    NotFound = 1,
    AlreadyExpired = 2,
    NotExpired = 3,
}
