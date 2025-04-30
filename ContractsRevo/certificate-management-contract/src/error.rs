use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdminError {
    AlreadyInitialized = 1,
    UnauthorizedAccess = 2,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IssueError {
    Error = 1,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditError {
    Error = 1,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevokeError {
    Error = 1,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VerifyError {
    Error = 1,
}

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificationError {
    Error = 1,
}