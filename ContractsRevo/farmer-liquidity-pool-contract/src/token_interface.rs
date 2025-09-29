use soroban_sdk::{ Address, Env, String};


pub trait TokenTrait {
    // Admin / metadata
    fn initialize(e: Env, admin: Address, decimal: u32, name: String, symbol: String);
    fn set_admin(e: Env, new_admin: Address);

    // Mint / burn
    fn mint(e: Env, to: Address, amount: i128);
    fn burn(e: Env, from: Address, amount: i128);
    fn burn_from(e: Env, spender: Address, from: Address, amount: i128);

    // ERC20-style
    fn allowance(e: Env, from: Address, spender: Address) -> i128;
    fn approve(e: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32);
    fn balance(e: Env, id: Address) -> i128;
    fn transfer(e: Env, from: Address, to: Address, amount: i128);
    fn transfer_from(e: Env, spender: Address, from: Address, to: Address, amount: i128);

    // Metadata
    fn decimals(e: Env) -> u32;
    fn name(e: Env) -> String;
    fn symbol(e: Env) -> String;
}
