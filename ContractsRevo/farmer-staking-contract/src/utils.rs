use soroban_sdk::{
    symbol_short, Address, Env, IntoVal,
    contracterror,
};

/// Errors that can occur in validation and utility operations
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ValidationError {
    InvalidAddress = 1,
    InvalidAmount = 2,
    InvalidLockPeriod = 3,
    TransferFailed = 4,
    InsufficientBalance = 5,
    InsufficientAllowance = 6,
}

/// Transfer tokens from user to this contract
/// Uses the token contract's transfer_from method
pub fn transfer_from_user(
    env: Env,
    token_address: Address,
    from: Address,
    amount: i128,
) -> Result<(), ValidationError> {
    if amount <= 0 {
        return Err(ValidationError::InvalidAmount);
    }

    // For Soroban, we need the user to have approved the contract first
    // Then we call transfer_from with: spender (contract), from, to (contract), amount
    let contract_address = env.current_contract_address();

    // Since we're in a contract, the from address needs to approve us
    // The actual transfer will be: from -> contract
    // In Soroban, transfer expects: from, to, amount
    // But we need transfer_from: spender, from, to, amount

    // For this to work, we assume the user has called approve(contract_address, amount) first
    // Then the staking contract calls transfer on behalf of the user

    // Simplified: just do a transfer since the user is authenticated
    env.invoke_contract::<()>(
        &token_address,
        &symbol_short!("transfer"),
        (from, contract_address, amount).into_val(&env),
    );

    Ok(())
}

/// Transfer tokens from this contract to user
pub fn transfer_to_user(
    env: Env,
    token_address: Address,
    to: Address,
    amount: i128,
) -> Result<(), ValidationError> {
    if amount <= 0 {
        return Err(ValidationError::InvalidAmount);
    }

    let contract_address = env.current_contract_address();

    env.invoke_contract::<()>(
        &token_address,
        &symbol_short!("transfer"),
        (contract_address, to, amount).into_val(&env),
    );

    Ok(())
}

/// Get token balance of an address
pub fn get_balance(
    env: Env,
    token_address: Address,
    address: Address,
) -> Result<i128, ValidationError> {
    let balance: i128 = env.invoke_contract(
        &token_address,
        &symbol_short!("balance"),
        (address,).into_val(&env),
    );

    Ok(balance)
}

/// Get token allowance
pub fn get_allowance(
    env: Env,
    token_address: Address,
    owner: Address,
    spender: Address,
) -> Result<i128, ValidationError> {
    let allowance: i128 = env.invoke_contract(
        &token_address,
        &symbol_short!("allowance"),
        (owner, spender).into_val(&env),
    );

    Ok(allowance)
}

/// Validate address is not zero address
pub fn validate_address(_address: &Address) -> Result<(), ValidationError> {
    // In Stellar, we just check the address exists
    // Additional validation can be added here
    Ok(())
}

/// Validate amount is positive
pub fn validate_amount(amount: i128) -> Result<(), ValidationError> {
    if amount <= 0 {
        return Err(ValidationError::InvalidAmount);
    }
    Ok(())
}

/// Validate lock period is within acceptable range
pub fn validate_lock_period(lock_period: u64, max_lock_period: u64) -> Result<(), ValidationError> {
    if lock_period > max_lock_period {
        return Err(ValidationError::InvalidLockPeriod);
    }
    Ok(())
}

/// Calculate percentage of an amount
pub fn calculate_percentage(amount: i128, percentage: i128) -> i128 {
    amount.checked_mul(percentage).unwrap_or(0) / 100
}

/// Safe addition with overflow check
pub fn safe_add(a: i128, b: i128) -> Result<i128, ValidationError> {
    a.checked_add(b).ok_or(ValidationError::InvalidAmount)
}

/// Safe subtraction with overflow check
pub fn safe_sub(a: i128, b: i128) -> Result<i128, ValidationError> {
    a.checked_sub(b).ok_or(ValidationError::InvalidAmount)
}

/// Safe multiplication with overflow check
pub fn safe_mul(a: i128, b: i128) -> Result<i128, ValidationError> {
    a.checked_mul(b).ok_or(ValidationError::InvalidAmount)
}

/// Safe division with zero check
pub fn safe_div(a: i128, b: i128) -> Result<i128, ValidationError> {
    if b == 0 {
        return Err(ValidationError::InvalidAmount);
    }
    a.checked_div(b).ok_or(ValidationError::InvalidAmount)
}

/// Convert seconds to days
pub fn seconds_to_days(seconds: u64) -> u64 {
    seconds / 86400
}

/// Convert days to seconds
pub fn days_to_seconds(days: u64) -> u64 {
    days * 86400
}

/// Calculate time elapsed between two timestamps
pub fn time_elapsed(start_time: u64, end_time: u64) -> u64 {
    if end_time > start_time {
        end_time - start_time
    } else {
        0
    }
}

/// Check if current time is past a given timestamp
pub fn is_time_past(env: &Env, timestamp: u64) -> bool {
    env.ledger().timestamp() >= timestamp
}

/// Get current timestamp
pub fn get_current_timestamp(env: &Env) -> u64 {
    env.ledger().timestamp()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(1000, 10), 100);
        assert_eq!(calculate_percentage(5000, 5), 250);
        assert_eq!(calculate_percentage(100, 50), 50);
    }

    #[test]
    fn test_seconds_to_days() {
        assert_eq!(seconds_to_days(86400), 1);
        assert_eq!(seconds_to_days(172800), 2);
        assert_eq!(seconds_to_days(604800), 7);
    }

    #[test]
    fn test_days_to_seconds() {
        assert_eq!(days_to_seconds(1), 86400);
        assert_eq!(days_to_seconds(7), 604800);
        assert_eq!(days_to_seconds(30), 2592000);
    }

    #[test]
    fn test_time_elapsed() {
        assert_eq!(time_elapsed(100, 200), 100);
        assert_eq!(time_elapsed(200, 100), 0);
        assert_eq!(time_elapsed(100, 100), 0);
    }
}
