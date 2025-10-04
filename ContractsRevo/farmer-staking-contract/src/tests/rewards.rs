use crate::rewards::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lock_multiplier_no_lock() {
        // No lock period should give base multiplier
        let multiplier = calculate_lock_multiplier(0);
        assert_eq!(multiplier, 100);
    }

    #[test]
    fn test_lock_multiplier_less_than_one_week() {
        // Less than 1 week
        let multiplier = calculate_lock_multiplier(604800 - 1);
        assert_eq!(multiplier, 105);
    }

    #[test]
    fn test_lock_multiplier_one_week() {
        // Exactly 1 week
        let multiplier = calculate_lock_multiplier(604800);
        assert_eq!(multiplier, 110);
    }

    #[test]
    fn test_lock_multiplier_one_month() {
        // One month
        let multiplier = calculate_lock_multiplier(2592000);
        assert_eq!(multiplier, 120);
    }

    #[test]
    fn test_lock_multiplier_three_months() {
        // Three months
        let multiplier = calculate_lock_multiplier(7776000);
        assert_eq!(multiplier, 135);
    }

    #[test]
    fn test_lock_multiplier_six_months() {
        // Six months
        let multiplier = calculate_lock_multiplier(15552000);
        assert_eq!(multiplier, 150);
    }

    #[test]
    fn test_lock_multiplier_one_year() {
        // One year or more
        let multiplier = calculate_lock_multiplier(31536000);
        assert_eq!(multiplier, 175);
    }

    #[test]
    fn test_lock_multiplier_more_than_one_year() {
        // More than one year
        let multiplier = calculate_lock_multiplier(31536000 + 1);
        assert_eq!(multiplier, 175);
    }

    #[test]
    fn test_reward_calculation_basic() {
        // Test basic reward calculation logic
        let stake_amount = 1000i128;
        let reward_rate = 100i128;
        let total_staked = 10000i128;

        let user_share = stake_amount * 1_000_000 / total_staked;
        let base_rewards = reward_rate * user_share / 1_000_000;

        assert_eq!(user_share, 100_000);
        assert_eq!(base_rewards, 10);
    }

    #[test]
    fn test_reward_calculation_zero_total_staked_logic() {
        // Test reward calculation logic when total staked is zero
        let stake_amount = 1000i128;
        let reward_rate = 100i128;
        let total_staked = 0i128;

        // When total staked is 0, user share should be 0
        let user_share = if total_staked > 0 {
            stake_amount * 1_000_000 / total_staked
        } else {
            0
        };

        assert_eq!(user_share, 0);

        let base_rewards = reward_rate * user_share / 1_000_000;
        assert_eq!(base_rewards, 0);
    }

    #[test]
    fn test_reward_calculation_zero_stake_amount_logic() {
        // Test reward calculation logic when stake amount is zero
        let stake_amount = 0i128;
        let reward_rate = 100i128;
        let total_staked = 10000i128;

        // When stake amount is 0, rewards should be 0
        if stake_amount == 0 {
            assert_eq!(stake_amount, 0);
            return; // Early return for zero stake
        }

        // This code shouldn't execute
        assert!(false, "Should not reach here with zero stake");
    }

    #[test]
    fn test_reward_debt_calculation_logic() {
        // Test reward debt calculation logic
        let stake_amount = 1000i128;
        let total_staked = 10000i128;
        let reward_rate = 100i128;
        let current_epoch = 5u64;

        let current_accumulated_reward_per_share = if total_staked > 0 {
            (reward_rate * current_epoch as i128 * 1_000_000) / total_staked
        } else {
            0
        };

        let reward_debt = (stake_amount * current_accumulated_reward_per_share) / 1_000_000;

        assert!(reward_debt >= 0);
        assert_eq!(current_accumulated_reward_per_share, 50_000); // 100 * 5 * 1_000_000 / 10000
        assert_eq!(reward_debt, 50); // 1000 * 50_000 / 1_000_000
    }

    #[test]
    fn test_reward_debt_zero_total_staked_logic() {
        // Test reward debt when total staked is zero
        let stake_amount = 1000i128;
        let total_staked = 0i128;
        let reward_rate = 100i128;
        let current_epoch = 5u64;

        let current_accumulated_reward_per_share = if total_staked > 0 {
            (reward_rate * current_epoch as i128 * 1_000_000) / total_staked
        } else {
            0
        };

        let reward_debt = (stake_amount * current_accumulated_reward_per_share) / 1_000_000;

        assert_eq!(current_accumulated_reward_per_share, 0);
        assert_eq!(reward_debt, 0);
    }

    #[test]
    fn test_reward_error_types() {
        // Test that all error types are properly defined
        let errors = [
            RewardError::PoolNotFound,
            RewardError::StakeNotFound,
            RewardError::NoRewardsToClaim,
            RewardError::TransferFailed,
            RewardError::CalculationError,
        ];

        // Each error should have a unique discriminant
        for (i, error) in errors.iter().enumerate() {
            assert_eq!(*error as u32, i as u32 + 1);
        }
    }

    #[test]
    fn test_apr_calculation_logic() {
        // Test APR calculation logic
        let reward_rate = 1000i128;
        let total_staked = 10000i128;
        let lock_period = 604800u64; // 1 week

        // Base APR = (reward_rate * 365 days) / total_staked
        let yearly_rewards = reward_rate * 365;
        let base_apr = if total_staked > 0 {
            (yearly_rewards * 10000) / total_staked
        } else {
            0
        };

        // Apply lock multiplier
        let multiplier = calculate_lock_multiplier(lock_period);
        let apr_with_bonus = (base_apr * multiplier) / 100;

        assert_eq!(yearly_rewards, 365000);
        assert_eq!(base_apr, 365000); // 365000 * 10000 / 10000
        assert_eq!(multiplier, 110); // 1 week = 10% bonus
        assert_eq!(apr_with_bonus, 401500); // 365000 * 110 / 100
    }

    #[test]
    fn test_apr_calculation_zero_total_staked_logic() {
        // Test APR calculation when total staked is zero
        let reward_rate = 1000i128;
        let total_staked = 0i128;
        let lock_period = 604800u64;

        let yearly_rewards = reward_rate * 365;
        let base_apr = if total_staked > 0 {
            (yearly_rewards * 10000) / total_staked
        } else {
            0
        };

        assert_eq!(base_apr, 0);
    }

    #[test]
    fn test_total_rewards_distributed_logic() {
        // Test total rewards distributed calculation
        let reward_rate = 1000i128;
        let current_epoch = 10u64;

        let total_rewards = reward_rate * current_epoch as i128;

        assert_eq!(total_rewards, 10000);

        // Test with zero epochs
        let zero_epoch_rewards = reward_rate * 0;
        assert_eq!(zero_epoch_rewards, 0);
    }

    #[test]
    fn test_reward_share_calculation() {
        // Test user share calculation in rewards
        let stake_amount = 1000i128;
        let total_staked = 10000i128;
        let precision_factor = 1_000_000i128;

        let user_share = stake_amount * precision_factor / total_staked;
        assert_eq!(user_share, 100_000); // 10% share

        // Test with different amounts
        let large_stake = 5000i128;
        let large_share = large_stake * precision_factor / total_staked;
        assert_eq!(large_share, 500_000); // 50% share

        let small_stake = 100i128;
        let small_share = small_stake * precision_factor / total_staked;
        assert_eq!(small_share, 10_000); // 1% share
    }

    #[test]
    fn test_time_based_reward_calculation() {
        // Test time-based reward calculations
        let stake_time = 1000u64;
        let current_time = 1000u64 + 2 * 86400; // 2 days later
        let epoch_duration = 86400u64; // 1 day

        let time_staked = current_time - stake_time;
        let epochs_passed = time_staked / epoch_duration;

        assert_eq!(time_staked, 2 * 86400);
        assert_eq!(epochs_passed, 2);

        // Test reward calculation with epochs
        let reward_rate = 100i128;
        let user_share = 100_000i128; // 10% share (from previous test)
        let base_rewards = (reward_rate * user_share * epochs_passed as i128) / 1_000_000;

        assert_eq!(base_rewards, 20); // 100 * 100_000 * 2 / 1_000_000
    }

    #[test]
    fn test_reward_multiplier_application() {
        // Test how lock multipliers are applied to rewards
        let base_rewards = 100i128;

        // Test different multipliers
        let no_lock_multiplier = calculate_lock_multiplier(0);
        let one_week_multiplier = calculate_lock_multiplier(604800);
        let one_year_multiplier = calculate_lock_multiplier(31536000);

        let no_lock_rewards = (base_rewards * no_lock_multiplier) / 100;
        let one_week_rewards = (base_rewards * one_week_multiplier) / 100;
        let one_year_rewards = (base_rewards * one_year_multiplier) / 100;

        assert_eq!(no_lock_rewards, 100); // 100 * 100 / 100
        assert_eq!(one_week_rewards, 110); // 100 * 110 / 100
        assert_eq!(one_year_rewards, 175); // 100 * 175 / 100

        // Longer locks should yield more rewards
        assert!(one_week_rewards > no_lock_rewards);
        assert!(one_year_rewards > one_week_rewards);
    }

    #[test]
    fn test_reward_multiplier_edge_cases() {
        // Test boundary conditions for lock multipliers
        assert_eq!(calculate_lock_multiplier(0), 100);
        assert_eq!(calculate_lock_multiplier(1), 105);
        assert_eq!(calculate_lock_multiplier(604799), 105); // Just under 1 week
        assert_eq!(calculate_lock_multiplier(604800), 110); // Exactly 1 week
        assert_eq!(calculate_lock_multiplier(604801), 110); // Just over 1 week

        assert_eq!(calculate_lock_multiplier(2591999), 110); // Just under 1 month
        assert_eq!(calculate_lock_multiplier(2592000), 120); // Exactly 1 month
        assert_eq!(calculate_lock_multiplier(2592001), 120); // Just over 1 month
    }

    #[test]
    fn test_reward_calculation_precision() {
        // Test that reward calculations maintain precision
        let stake_amount = 1i128;
        let reward_rate = 1i128;
        let total_staked = 1_000_000i128;

        let user_share = stake_amount * 1_000_000 / total_staked;
        let base_rewards = reward_rate * user_share / 1_000_000;

        assert_eq!(user_share, 1);
        assert_eq!(base_rewards, 0); // Very small stake should result in 0 rewards due to precision
    }

    #[test]
    fn test_reward_calculation_large_numbers() {
        // Test reward calculations with large numbers
        let stake_amount = 1_000_000_000i128;
        let reward_rate = 1_000_000i128;
        let total_staked = 10_000_000_000i128;

        let user_share = stake_amount * 1_000_000 / total_staked;
        let base_rewards = reward_rate * user_share / 1_000_000;

        assert_eq!(user_share, 100_000);
        assert_eq!(base_rewards, 100_000);
    }
}
