test_direct_binary() -> u64 {
        // Direct binary operation without local variables (should work)
        return get_clock().slot - 100;
    }
