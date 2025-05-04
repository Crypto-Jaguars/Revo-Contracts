use soroban_sdk::{contracterror, contracttype, Address, BytesN, String};

#[contracttype]
pub enum DataKey {
    Loan(u32),                // Loan ID -> LoanRequest
    Funding(u32),             // Loan ID -> Vec<FundingContribution>
    Repayments(u32),          // Loan ID -> Vec<Repayment>
    BorrowerLoans(Address),   // Borrower Address -> Vec<u32>
    LenderLoans(Address),     // Lender Address -> Vec<u32>
    BorrowerMetrics(Address), // Borrower Address -> BorrowerMetrics
    NextLoanId,               // Counter for loan IDs
    TotalLoansCreated,        // Total number of loan requests created
    TotalLoansFunded,         // Total number of loans fully funded
    TotalLoansCompleted,      // Total number of loans fully repaid
    TotalLoansDefaulted,      // Total number of loans defaulted
    AssetCode,                // Token contract address for funding
    SystemStats,              // System-wide statistics
}
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoanRequest {
    pub id: u32,
    pub borrower: Address,
    pub amount: i128,
    pub purpose: String,
    pub duration_days: u32,
    pub interest_rate: u32, // Basis points (e.g., 1000 = 10%)
    pub collateral: CollateralInfo,
    pub status: LoanStatus,
    pub funded_amount: i128,                  // Total amount funded so far
    pub creation_timestamp: u64,              // Ledger timestamp when loan is created
    pub funded_timestamp: Option<u64>,        // Ledger timestamp when loan is funded
    pub repayment_due_timestamp: Option<u64>, // Ledger timestamp when repayment is due
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollateralInfo {
    pub asset_type: String, // E.g., "Savings", "Future harvest", "Equipment"
    pub estimated_value: i128,
    pub verification_data: BytesN<32>, // Hash of verification documents
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoanStatus {
    Pending,   // Loan request created but not fully funded
    Funded,    // Loan fully funded but not yet repaid
    Repaying,  // Loan in active repayment phase
    Completed, // Loan fully repaid
    Defaulted, // Loan in default status
    Cancelled, // Loan request cancelled by borrower
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FundingContribution {
    pub lender: Address,
    pub amount: i128,
    pub timestamp: u64, // Ledger timestamp of contribution
    pub claimed: bool,  // Whether repayment has been claimed
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Repayment {
    pub amount: i128,
    pub timestamp: u64, // Ledger timestamp of repayment
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BorrowerMetrics {
    pub total_loans: u32,     // Total loans requested
    pub completed_loans: u32, // Loans fully repaid
    pub defaulted_loans: u32, // Loans that defaulted
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemStats {
    pub total_loans: u32,   // Total loans created
    pub total_funded: i128, // Total amount funded
    pub total_repaid: i128, // Total amount repaid
    pub default_rate: u32,  // Basis points (e.g., 500 = 5%)
}

// === Error Definitions ===
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MicrolendingError {
    AlreadyInitialized = 1,
    InvalidAmount = 2,
    InvalidDuration = 3,
    InvalidInterestRate = 4,
    InvalidCollateral = 5,
    LoanNotFound = 6,
    Unauthorized = 7,
    InvalidLoanStatus = 8,
    LoanFullyFunded = 9,
    LoanNotRepayable = 10,
    RepaymentExceedsDue = 11,
    NotInDefault = 12,
    NoContribution = 13,
}
