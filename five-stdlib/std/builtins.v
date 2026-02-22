// std::builtins
// Standardized wrappers and signatures for common compiler built-ins.
//
// These wrappers keep call sites stable if builtin naming evolves.

pub now_seconds() -> u64 {
    return get_clock();
}

pub abort_now() {
    abort();
}

pub panic_now(message: u64) {
    panic(message);
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

pub derive_program_address(seeds: u64, program_id: u64) {
    create_program_address(seeds, program_id);
}

pub find_program_address(seeds: u64, program_id: u64) {
    try_find_program_address(seeds, program_id);
}

pub clock_sysvar() {
    get_clock_sysvar();
}

pub epoch_schedule_sysvar() {
    get_epoch_schedule_sysvar();
}

pub rent_sysvar() {
    get_rent_sysvar();
}

pub return_data_get() {
    get_return_data();
}

pub return_data_set(data: u64) {
    set_return_data(data);
}

pub log_message(value: u64) {
    log(value);
}

pub log_words(a: u64, b: u64, c: u64, d: u64, e: u64) {
    log_64(a, b, c, d, e);
}

pub log_cu() {
    log_compute_units();
}

pub log_bytes(data: u64) {
    log_data(data);
}

pub log_key(key: u64) {
    log_pubkey(key);
}

pub close_account_now(source: account, destination: account) {
    close_account(source, destination);
}

pub memory_copy(dst: u64, src: u64, len: u64) {
    memcpy(dst, src, len);
}

pub memory_compare(a: u64, b: u64, len: u64) {
    memcmp(a, b, len);
}

pub recover_secp256k1(hash: u64, recid: u64, sig: u64, out: u64) {
    secp256k1_recover(hash, recid, sig, out);
}
