// Test comprehensive match expressions with arithmetic operations

pub test() -> u64 {
    let x = 10;
    let y = Some(5);
    match y {
        Some(divisor) => {
            if (divisor == 0) {
                return 0;
            } else {
                let result = x * divisor;
                match (result > 30) {
                    true => {
                        return result + 10;
                    }
                    false => {
                        return result;
                    }
                }
            }
        }
        None => {
            return 0;
        }
    }
}