    mut value: u64;
    
    init {
        value = 1;
    }
    
pub test() -> u64 {
        return value;
    }

update_value(new_value: u64) -> u64 {
        value = new_value;
        return value;
    }
