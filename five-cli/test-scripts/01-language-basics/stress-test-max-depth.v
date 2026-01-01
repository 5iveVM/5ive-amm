// Stress test with maximum call depth and local variables
// Tests the limits of call stack and local storage

pub test_max_depth() -> u64 {
    return f1(1);
}

f1(v: u64) -> u64 {
    let l = v + 1;
    return f2(l) + l;
}

f2(v: u64) -> u64 {
    let l = v + 1;
    return f3(l) + l;
}

f3(v: u64) -> u64 {
    let l = v + 1;
    return f4(l) + l;
}

f4(v: u64) -> u64 {
    let l = v + 1;
    return l;
}
