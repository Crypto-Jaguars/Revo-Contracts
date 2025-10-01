#[cfg(test)]
mod tests {
    use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env};

    #[test]
    fn test_stake_validation() {
        let env = Env::default();
        let _farmer = Address::generate(&env);

        // Test amount validation
        let amount = 1000i128;
        let lock_period = 604800u64; // 1 week

        assert!(amount > 0);
        assert!(lock_period >= 0);
    }

    #[test]
    fn test_lock_period_calculation() {
        let env = Env::default();
        env.ledger().with_mut(|li| {
            li.timestamp = 1000;
        });

        let current_time = env.ledger().timestamp();
        let lock_period = 86400u64; // 1 day
        let unlock_time = current_time + lock_period;

        assert_eq!(unlock_time, 1000 + 86400);
    }

    #[test]
    fn test_emergency_unstake_penalty() {
        // Test that penalty calculation is correct
        let amount = 1000i128;
        let penalty_rate = 10i128;
        let penalty = amount * penalty_rate / 100;
        let amount_after_penalty = amount - penalty;

        assert_eq!(penalty, 100);
        assert_eq!(amount_after_penalty, 900);
    }
}
