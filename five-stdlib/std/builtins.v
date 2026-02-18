// std::builtins
// Standardized wrappers and signatures for common compiler built-ins.
//
// These wrappers keep call sites stable if builtin naming evolves.

pub now_seconds() -> u64 {
    return get_clock();
}

pub hash_sha256(input: u64) {
    sha256(input);
}

pub hash_keccak256(input: u64) {
    keccak256(input);
}

pub hash_blake3(input: u64) {
    blake3(input);
}

pub remaining_cu() {
    remaining_compute_units();
}
