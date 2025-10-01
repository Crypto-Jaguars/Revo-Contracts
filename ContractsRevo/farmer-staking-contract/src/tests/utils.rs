#[cfg(test)]
mod tests {
    use crate::utils::*;

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(1000, 10), 100);
        assert_eq!(calculate_percentage(5000, 5), 250);
        assert_eq!(calculate_percentage(100, 50), 50);
        assert_eq!(calculate_percentage(0, 10), 0);
    }

    #[test]
    fn test_seconds_to_days() {
        assert_eq!(seconds_to_days(86400), 1);
        assert_eq!(seconds_to_days(172800), 2);
        assert_eq!(seconds_to_days(604800), 7);
        assert_eq!(seconds_to_days(0), 0);
    }

    #[test]
    fn test_days_to_seconds() {
        assert_eq!(days_to_seconds(1), 86400);
        assert_eq!(days_to_seconds(7), 604800);
        assert_eq!(days_to_seconds(30), 2592000);
        assert_eq!(days_to_seconds(0), 0);
    }

    #[test]
    fn test_time_elapsed() {
        assert_eq!(time_elapsed(100, 200), 100);
        assert_eq!(time_elapsed(200, 100), 0);
        assert_eq!(time_elapsed(100, 100), 0);
        assert_eq!(time_elapsed(0, 1000), 1000);
    }

    #[test]
    fn test_safe_add() {
        assert_eq!(safe_add(100, 200).unwrap(), 300);
        assert_eq!(safe_add(0, 0).unwrap(), 0);
        assert_eq!(safe_add(-100, 200).unwrap(), 100);
    }

    #[test]
    fn test_safe_sub() {
        assert_eq!(safe_sub(200, 100).unwrap(), 100);
        assert_eq!(safe_sub(100, 100).unwrap(), 0);
        assert_eq!(safe_sub(100, 200).unwrap(), -100);
    }

    #[test]
    fn test_safe_mul() {
        assert_eq!(safe_mul(10, 20).unwrap(), 200);
        assert_eq!(safe_mul(0, 100).unwrap(), 0);
        assert_eq!(safe_mul(-10, 20).unwrap(), -200);
    }

    #[test]
    fn test_safe_div() {
        assert_eq!(safe_div(100, 10).unwrap(), 10);
        assert_eq!(safe_div(100, 3).unwrap(), 33);
        assert!(safe_div(100, 0).is_err());
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(100).is_ok());
        assert!(validate_amount(1).is_ok());
        assert!(validate_amount(0).is_err());
        assert!(validate_amount(-1).is_err());
    }

    #[test]
    fn test_validate_lock_period() {
        assert!(validate_lock_period(100, 1000).is_ok());
        assert!(validate_lock_period(1000, 1000).is_ok());
        assert!(validate_lock_period(1001, 1000).is_err());
    }
}
