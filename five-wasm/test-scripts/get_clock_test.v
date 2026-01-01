    mut current_time: u64;
    mut last_timestamp: u64;
    
    init {
        current_time = 0;
        last_timestamp = 0;
    }
    
update_time() -> u64 {
        current_time = get_clock();
        return current_time;
    }
    
get_current_time() -> u64 {
        let time = get_clock();
        return time;
    }
    
time_since_last_update() -> u64 {
        let current = get_clock();
        let difference = current - last_timestamp;
        last_timestamp = current;
        return difference;
    }
