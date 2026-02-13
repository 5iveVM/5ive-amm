// @test-params 2 4 6
pub test(a: u64, b: u64, c: u64) -> u64 {
    let numbers = [a, b, c, 8, 10];
    let first = numbers[0];
    let third = numbers[2];
    return first + third;
}
