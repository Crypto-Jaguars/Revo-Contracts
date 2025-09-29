use soroban_sdk::{Address, Env, Symbol};

pub(crate) fn add_liquidity(e: &Env, to: Address, amount_a: i128, amount_b: i128) {
    let topics = (Symbol::new(e, "add_liquidity"), to);
    e.events().publish(topics, (amount_a, amount_b));
}

pub(crate) fn remove_liquidity(e: &Env, to: Address, amount_a: i128, amount_b: i128) {
    let topics = (Symbol::new(e, "remove_liquidity"), to);
    e.events().publish(topics, (amount_a, amount_b));
}

pub(crate) fn swap(e: &Env, to: Address, buy_a: bool, sell_amount: i128, buy_amount: i128) {
    let topics = (Symbol::new(e, "swap"), to, buy_a);
    e.events().publish(topics, (buy_amount, sell_amount));
}

pub(crate) fn fees_collected(e: &Env, admin: Address, fee_a: i128, fee_b: i128) {
    let topics = (Symbol::new(e, "fees_col"), admin);
    let data = (fee_a, fee_b);
    e.events().publish(topics, data);
}
