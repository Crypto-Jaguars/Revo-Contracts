#[cfg(test)]
mod tests {
    use crate::rewards::calculate_lock_multiplier;

    #[test]
    fn test_lock_multiplier_no_lock() {
        // No lock period should give base multiplier
        let multiplier = calculate_lock_multiplier(0);
        assert_eq!(multiplier, 100);
    }

    #[test]
    fn test_lock_multiplier_one_week() {
        // Less than 1 week
        let multiplier = calculate_lock_multiplier(604800 - 1);
        assert_eq!(multiplier, 105);
    }

    #[test]
    fn test_lock_multiplier_one_month() {
        // One month
        let multiplier = calculate_lock_multiplier(2592000);
        assert_eq!(multiplier, 120);
    }

    #[test]
    fn test_lock_multiplier_one_year() {
        // One year or more
        let multiplier = calculate_lock_multiplier(31536000);
        assert_eq!(multiplier, 175);
    }

    #[test]
    fn test_reward_calculation() {
        // Test basic reward calculation logic
        let stake_amount = 1000i128;
        let reward_rate = 100i128;
        let total_staked = 10000i128;

        let user_share = stake_amount * 1_000_000 / total_staked;
        let base_rewards = reward_rate * user_share / 1_000_000;

        assert_eq!(user_share, 100_000);
        assert_eq!(base_rewards, 10);
    }
}
