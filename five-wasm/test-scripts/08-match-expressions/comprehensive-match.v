// Comprehensive test

pub test() -> u64 {
    let x = 10;
    let y = 5;
    if (y == 0) {
        return 0;
    } else {
        let result = x * y;
        if (result > 30) {
            return result + 10;
        } else {
            return result;
        }
    }
}
