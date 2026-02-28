use five_dsl_compiler::DslCompiler;
use five_protocol::{opcodes::ARRAY_CONCAT, ValueRef};
use five_vm_mito::{
    handlers::{arrays::handle_arrays, system::crypto::handle_syscall_sha256},
    ExecutionContext, StackStorage, FIVE_VM_PROGRAM_ID,
};

fn new_context<'a>(storage: &'a mut StackStorage) -> ExecutionContext<'a> {
    ExecutionContext::new(
        &[0u8],
        &[],
        FIVE_VM_PROGRAM_ID,
        &[],
        0,
        storage,
        0,
        0,
        0,
        0,
        0,
        0,
    )
}

fn temp_ref(ctx: &mut ExecutionContext<'_>, bytes: &[u8]) -> ValueRef {
    let off = ctx.alloc_temp(bytes.len() as u8).expect("alloc temp");
    ctx.temp_buffer_mut()[off as usize..off as usize + bytes.len()].copy_from_slice(bytes);
    ValueRef::TempRef(off, bytes.len() as u8)
}

fn concat(ctx: &mut ExecutionContext<'_>, left: ValueRef, right: ValueRef) -> ValueRef {
    ctx.push(left).expect("push left");
    ctx.push(right).expect("push right");
    handle_arrays(ARRAY_CONCAT, ctx).expect("array concat");
    ctx.pop().expect("concat result")
}

fn hash_preimage(ctx: &mut ExecutionContext<'_>, preimage: ValueRef) -> [u8; 32] {
    let out_off = ctx.alloc_temp(32).expect("alloc out");
    ctx.push(preimage).expect("push preimage");
    ctx.push(ValueRef::TempRef(out_off, 32))
        .expect("push output");
    handle_syscall_sha256(ctx).expect("sha256");

    let mut out = [0u8; 32];
    out.copy_from_slice(&ctx.temp_buffer()[out_off as usize..out_off as usize + 32]);
    out
}

fn first_8_le_u64(bytes: [u8; 32]) -> u64 {
    u64::from_le_bytes(bytes[..8].try_into().expect("first 8 bytes"))
}

#[test]
fn canonical_preimage_hash_vector_matches_anchor_layout() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let house_entropy = [1u8; 32];
    let seed_bytes = [
        255, 238, 221, 204, 187, 170, 153, 136, 119, 102, 85, 68, 51, 34, 17, 0,
    ];
    let clock_bytes = [8u8, 7, 6, 5, 4, 3, 2, 1];
    let request_bytes = [9u8, 0, 0, 0, 0, 0, 0, 0];

    let house_ref = temp_ref(&mut ctx, &house_entropy);
    let seed_ref = temp_ref(&mut ctx, &seed_bytes);
    let clock_ref = temp_ref(&mut ctx, &clock_bytes);
    let request_ref = temp_ref(&mut ctx, &request_bytes);

    let p0 = concat(&mut ctx, house_ref, seed_ref);
    let p1 = concat(&mut ctx, p0, clock_ref);
    let preimage = concat(&mut ctx, p1, request_ref);

    let (len, bytes) = ctx
        .extract_string_slice(&preimage)
        .expect("extract preimage");
    assert_eq!(len, 64);

    let mut expected_preimage = Vec::new();
    expected_preimage.extend_from_slice(&house_entropy);
    expected_preimage.extend_from_slice(&seed_bytes);
    expected_preimage.extend_from_slice(&clock_bytes);
    expected_preimage.extend_from_slice(&request_bytes);
    assert_eq!(bytes, expected_preimage.as_slice());

    let out = hash_preimage(&mut ctx, preimage);
    assert_eq!(&out[..8], &[3, 35, 72, 130, 171, 129, 214, 183]);
    assert_eq!(first_8_le_u64(out), 13246917927582049027);
}

#[test]
fn request_count_changes_hash_path() {
    let house_entropy = [1u8; 32];
    let seed_bytes = [
        255, 238, 221, 204, 187, 170, 153, 136, 119, 102, 85, 68, 51, 34, 17, 0,
    ];
    let clock_bytes = [8u8, 7, 6, 5, 4, 3, 2, 1];
    let request_a = [9u8, 0, 0, 0, 0, 0, 0, 0];
    let request_b = [10u8, 0, 0, 0, 0, 0, 0, 0];

    let out_a = {
        let mut storage = StackStorage::new();
        let mut ctx = new_context(&mut storage);
        let house_ref = temp_ref(&mut ctx, &house_entropy);
        let seed_ref = temp_ref(&mut ctx, &seed_bytes);
        let clock_ref = temp_ref(&mut ctx, &clock_bytes);
        let request_ref = temp_ref(&mut ctx, &request_a);
        let p0 = concat(&mut ctx, house_ref, seed_ref);
        let p1 = concat(&mut ctx, p0, clock_ref);
        let preimage = concat(&mut ctx, p1, request_ref);
        hash_preimage(&mut ctx, preimage)
    };

    let out_b = {
        let mut storage = StackStorage::new();
        let mut ctx = new_context(&mut storage);
        let house_ref = temp_ref(&mut ctx, &house_entropy);
        let seed_ref = temp_ref(&mut ctx, &seed_bytes);
        let clock_ref = temp_ref(&mut ctx, &clock_bytes);
        let request_ref = temp_ref(&mut ctx, &request_b);
        let p0 = concat(&mut ctx, house_ref, seed_ref);
        let p1 = concat(&mut ctx, p0, clock_ref);
        let preimage = concat(&mut ctx, p1, request_ref);
        hash_preimage(&mut ctx, preimage)
    };

    assert_ne!(out_a, out_b);
    assert_eq!(first_8_le_u64(out_a), 13246917927582049027);
    assert_eq!(first_8_le_u64(out_b), 1387529471207707633);
}

#[test]
fn downstream_vrng_contract_compiles() {
    let source = std::fs::read_to_string("/Users/ivmidable/Development/5ive-vrng-2/src/main.v")
        .expect("read downstream vrng source");
    DslCompiler::compile_dsl(&source).expect("downstream vrng should compile");
}
