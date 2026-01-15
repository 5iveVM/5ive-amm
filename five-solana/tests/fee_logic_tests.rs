#[cfg(test)]
mod fee_logic_tests {
    use five::instructions::fees::calculate_fee;

    #[test]
    fn test_calculate_fee_standard() {
        // 100 * 500 / 10000 = 50000 / 10000 = 5
        assert_eq!(calculate_fee(100, 500), 5);

        // 200 * 5000 / 10000 = 1000000 / 10000 = 100
        assert_eq!(calculate_fee(200, 5000), 100);
    }

    #[test]
    fn test_calculate_fee_zero_bps() {
        assert_eq!(calculate_fee(1000, 0), 0);
    }

    #[test]
    fn test_calculate_fee_zero_amount() {
        assert_eq!(calculate_fee(0, 500), 0);
    }

    #[test]
    fn test_calculate_fee_rounding() {
        // 100 * 50 / 10000 = 5000 / 10000 = 0.5 -> 0 (integer division)
        assert_eq!(calculate_fee(100, 50), 0);

        // 100 * 150 / 10000 = 15000 / 10000 = 1.5 -> 1
        assert_eq!(calculate_fee(100, 150), 1);
    }

    #[test]
    fn test_calculate_fee_large_numbers() {
        // u64 max is approx 1.8e19
        // u128 max is approx 3.4e38
        // 10000 is 1e4
        // So amount * bps should not exceed u128 max.
        // amount is u64, bps is u32. u64 * u32 fits in u96, so it definitely fits in u128.

        let amount = u64::MAX;
        let bps = 10000; // 100%

        // Should return amount exactly
        assert_eq!(calculate_fee(amount, bps), amount);

        let bps = 5000; // 50%
        assert_eq!(calculate_fee(amount, bps), amount / 2);
    }
}
