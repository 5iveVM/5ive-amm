test_function() -> u64 {
        let x = get_clock().slot;
        let y = x - 100;
        return y;
    }
