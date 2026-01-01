// Simple Five VM test script
// Tests basic arithmetic operation

func add(a: u64, b: u64) -> u64 {
    return a + b;
}

func main() -> u64 {
    return add(5, 3);
}