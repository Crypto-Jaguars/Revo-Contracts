use soroban_sdk::{contracterror, contracttype, Address, Env, Map, String, Symbol};

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TokenError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InsufficientBalance = 3,
    InsufficientAllowance = 4,
    InvalidAmount = 5,
    Paused = 6,
    Unauthorized = 7,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub total_supply: i128,
}

#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    TokenMetadata,
    Balance(Address),
    Allowance(Address, Address), // (owner, spender)
    TotalSupply,
    Minters,
    Paused,
}

pub type Balances = Map<Address, i128>;
pub type Allowances = Map<(Address, Address), i128>;
pub type Minters = Map<Address, bool>;

/// Initialize the token contract
pub fn initialize(
    env: Env,
    admin: Address,
    name: String,
    symbol: String,
    decimals: u32,
) -> Result<(), TokenError> {
    if env.storage().instance().has(&DataKey::Admin) {
        return Err(TokenError::AlreadyInitialized);
    }

    admin.require_auth();

    // Set admin
    env.storage().instance().set(&DataKey::Admin, &admin);

    // Set token metadata
    let metadata = TokenMetadata {
        name: name.clone(),
        symbol: symbol.clone(),
        decimals,
        total_supply: 0i128,
    };
    env.storage()
        .instance()
        .set(&DataKey::TokenMetadata, &metadata);

    // Initialize total supply
    env.storage().instance().set(&DataKey::TotalSupply, &0i128);

    // Initialize minters map and add admin as first minter
    let mut minters: Minters = Map::new(&env);
    minters.set(admin.clone(), true);
    env.storage().persistent().set(&DataKey::Minters, &minters);

    // Set paused state to false
    env.storage().instance().set(&DataKey::Paused, &false);

    // Emit initialization event
    env.events().publish(
        (Symbol::new(&env, "initialize"), admin),
        (name, symbol, decimals),
    );

    Ok(())
}

/// Transfer tokens from one address to another
pub fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), TokenError> {
    from.require_auth();

    if amount <= 0 {
        return Err(TokenError::InvalidAmount);
    }

    // Check if paused
    if is_paused(&env) {
        return Err(TokenError::Paused);
    }

    let from_balance = get_balance(&env, &from);
    if from_balance < amount {
        return Err(TokenError::InsufficientBalance);
    }

    // Update balances
    set_balance(&env, &from, from_balance - amount);
    set_balance(&env, &to, get_balance(&env, &to) + amount);

    // Emit transfer event
    env.events().publish(
        (Symbol::new(&env, "transfer"), from.clone(), to.clone()),
        amount,
    );

    Ok(())
}

/// Transfer tokens on behalf of another address
pub fn transfer_from(
    env: Env,
    spender: Address,
    from: Address,
    to: Address,
    amount: i128,
) -> Result<(), TokenError> {
    spender.require_auth();

    if amount <= 0 {
        return Err(TokenError::InvalidAmount);
    }

    // Check if paused
    if is_paused(&env) {
        return Err(TokenError::Paused);
    }

    let allowance = get_allowance(&env, &from, &spender);
    if allowance < amount {
        return Err(TokenError::InsufficientAllowance);
    }

    let from_balance = get_balance(&env, &from);
    if from_balance < amount {
        return Err(TokenError::InsufficientBalance);
    }

    // Update balances and allowance
    set_balance(&env, &from, from_balance - amount);
    set_balance(&env, &to, get_balance(&env, &to) + amount);
    set_allowance(&env, &from, &spender, allowance - amount);

    // Emit transfer event
    env.events().publish(
        (
            Symbol::new(&env, "transfer_from"),
            spender,
            from.clone(),
            to.clone(),
        ),
        amount,
    );

    Ok(())
}

/// Approve an address to spend tokens on behalf of the owner
pub fn approve(
    env: Env,
    owner: Address,
    spender: Address,
    amount: i128,
) -> Result<(), TokenError> {
    owner.require_auth();

    if amount < 0 {
        return Err(TokenError::InvalidAmount);
    }

    set_allowance(&env, &owner, &spender, amount);

    // Emit approval event
    env.events().publish(
        (Symbol::new(&env, "approve"), owner, spender),
        amount,
    );

    Ok(())
}

/// Get the balance of an address
pub fn balance(env: Env, owner: Address) -> i128 {
    get_balance(&env, &owner)
}

/// Get the allowance of a spender for an owner
pub fn allowance(env: Env, owner: Address, spender: Address) -> i128 {
    get_allowance(&env, &owner, &spender)
}

/// Get the total supply of tokens
pub fn total_supply(env: Env) -> i128 {
    env.storage()
        .instance()
        .get(&DataKey::TotalSupply)
        .unwrap_or(0)
}

/// Get token metadata
pub fn token_metadata(env: Env) -> TokenMetadata {
    env.storage()
        .instance()
        .get(&DataKey::TokenMetadata)
        .unwrap_or(TokenMetadata {
            name: String::from_str(&env, "Unknown"),
            symbol: String::from_str(&env, "UNK"),
            decimals: 7,
            total_supply: 0,
        })
}

// Internal helper functions

fn get_balance(env: &Env, address: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Balance(address.clone()))
        .unwrap_or(0)
}

fn set_balance(env: &Env, address: &Address, balance: i128) {
    if balance == 0 {
        env.storage()
            .persistent()
            .remove(&DataKey::Balance(address.clone()));
    } else {
        env.storage()
            .persistent()
            .set(&DataKey::Balance(address.clone()), &balance);
    }
}

fn get_allowance(env: &Env, owner: &Address, spender: &Address) -> i128 {
    env.storage()
        .persistent()
        .get(&DataKey::Allowance(owner.clone(), spender.clone()))
        .unwrap_or(0)
}

fn set_allowance(env: &Env, owner: &Address, spender: &Address, allowance: i128) {
    if allowance == 0 {
        env.storage()
            .persistent()
            .remove(&DataKey::Allowance(owner.clone(), spender.clone()));
    } else {
        env.storage()
            .persistent()
            .set(&DataKey::Allowance(owner.clone(), spender.clone()), &allowance);
    }
}

fn is_paused(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::Paused)
        .unwrap_or(false)
}

pub fn update_total_supply(env: &Env, new_supply: i128) {
    env.storage()
        .instance()
        .set(&DataKey::TotalSupply, &new_supply);
    
    // Update metadata
    let mut metadata = token_metadata(env.clone());
    metadata.total_supply = new_supply;
    env.storage()
        .instance()
        .set(&DataKey::TokenMetadata, &metadata);
}