use five_protocol::{opcodes::ARRAY_CONCAT, opcodes::HALT, ValueRef};
use five_vm_mito::{
    handlers::{
        arrays::handle_arrays,
        system::crypto::{
            handle_syscall_blake3, handle_syscall_keccak256, handle_syscall_sha256,
            handle_syscall_verify_ed25519_instruction,
        },
    },
    AccountInfo, ExecutionContext, StackStorage, FIVE_VM_PROGRAM_ID,
};

const ED25519_PROGRAM_ID_BYTES: [u8; 32] = [
    0x03, 0x7d, 0x46, 0xd6, 0x7c, 0x93, 0xfb, 0xbe, 0x12, 0xf9, 0x42, 0x8f, 0x83, 0x8d, 0x40, 0xff,
    0x05, 0x70, 0x74, 0x49, 0x27, 0xf4, 0x8a, 0x64, 0xfc, 0xca, 0x70, 0x44, 0x80, 0x00, 0x00, 0x00,
];

fn build_ed25519_sysvar_payload(
    signed_pubkey: &[u8; 32],
    signed_signature: &[u8; 64],
    signed_message: &[u8],
    program_id: &[u8; 32],
) -> Vec<u8> {
    let signature_offset = 48u16;
    let pubkey_offset = 16u16;
    let message_offset = 112u16;
    let message_size = signed_message.len() as u16;

    let mut ed_data = vec![0u8; message_offset as usize + signed_message.len()];
    ed_data[0] = 1; // signature_count
    ed_data[1] = 0; // padding
    ed_data[2..4].copy_from_slice(&signature_offset.to_le_bytes());
    ed_data[4..6].copy_from_slice(&u16::MAX.to_le_bytes()); // signature_instruction_index
    ed_data[6..8].copy_from_slice(&pubkey_offset.to_le_bytes());
    ed_data[8..10].copy_from_slice(&u16::MAX.to_le_bytes()); // pubkey_instruction_index
    ed_data[10..12].copy_from_slice(&message_offset.to_le_bytes());
    ed_data[12..14].copy_from_slice(&message_size.to_le_bytes());
    ed_data[14..16].copy_from_slice(&u16::MAX.to_le_bytes()); // message_instruction_index
    ed_data[16..48].copy_from_slice(signed_pubkey);
    ed_data[48..112].copy_from_slice(signed_signature);
    ed_data[112..112 + signed_message.len()].copy_from_slice(signed_message);

    let mut sysvar_payload = vec![];
    sysvar_payload.extend_from_slice(&1u16.to_le_bytes()); // instruction_count
    sysvar_payload.extend_from_slice(&4u16.to_le_bytes()); // first instruction offset
    sysvar_payload.extend_from_slice(&0u16.to_le_bytes()); // account_count
    sysvar_payload.extend_from_slice(program_id);
    sysvar_payload.extend_from_slice(&(ed_data.len() as u16).to_le_bytes());
    sysvar_payload.extend_from_slice(&ed_data);
    sysvar_payload
}

#[test]
fn test_sha256_known_vector_abc() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[HALT],
        &[],
        FIVE_VM_PROGRAM_ID,
        &[],
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let input = b"abc";
    let input_off = ctx.alloc_temp(input.len() as u8).unwrap();
    ctx.temp_buffer_mut()[input_off as usize..input_off as usize + input.len()]
        .copy_from_slice(input);
    let out_off = ctx.alloc_temp(32).unwrap();

    ctx.push(ValueRef::TempRef(input_off, input.len() as u8))
        .unwrap();
    ctx.push(ValueRef::TempRef(out_off, 32)).unwrap();
    handle_syscall_sha256(&mut ctx).unwrap();

    let expected: [u8; 32] = [
        0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae, 0x22,
        0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61, 0xf2, 0x00,
        0x15, 0xad,
    ];
    let got = &ctx.temp_buffer()[out_off as usize..out_off as usize + 32];
    assert_eq!(got, &expected);
}

#[test]
fn test_keccak256_known_vector_abc() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[HALT],
        &[],
        FIVE_VM_PROGRAM_ID,
        &[],
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let input = b"abc";
    let input_off = ctx.alloc_temp(input.len() as u8).unwrap();
    ctx.temp_buffer_mut()[input_off as usize..input_off as usize + input.len()]
        .copy_from_slice(input);
    let out_off = ctx.alloc_temp(32).unwrap();

    ctx.push(ValueRef::TempRef(input_off, input.len() as u8))
        .unwrap();
    ctx.push(ValueRef::TempRef(out_off, 32)).unwrap();
    handle_syscall_keccak256(&mut ctx).unwrap();

    let expected: [u8; 32] = [
        0x4e, 0x03, 0x65, 0x7a, 0xea, 0x45, 0xa9, 0x4f, 0xc7, 0xd4, 0x7b, 0xa8, 0x26, 0xc8, 0xd6,
        0x67, 0xc0, 0xd1, 0xe6, 0xe3, 0x3a, 0x64, 0xa0, 0x36, 0xec, 0x44, 0xf5, 0x8f, 0xa1, 0x2d,
        0x6c, 0x45,
    ];
    let got = &ctx.temp_buffer()[out_off as usize..out_off as usize + 32];
    assert_eq!(got, &expected);
}

#[test]
fn test_blake3_known_vector_abc() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[HALT],
        &[],
        FIVE_VM_PROGRAM_ID,
        &[],
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let input = b"abc";
    let input_off = ctx.alloc_temp(input.len() as u8).unwrap();
    ctx.temp_buffer_mut()[input_off as usize..input_off as usize + input.len()]
        .copy_from_slice(input);
    let out_off = ctx.alloc_temp(32).unwrap();

    ctx.push(ValueRef::TempRef(input_off, input.len() as u8))
        .unwrap();
    ctx.push(ValueRef::TempRef(out_off, 32)).unwrap();
    handle_syscall_blake3(&mut ctx).unwrap();

    let expected: [u8; 32] = [
        0x64, 0x37, 0xb3, 0xac, 0x38, 0x46, 0x51, 0x33, 0xff, 0xb6, 0x3b, 0x75, 0x27, 0x3a, 0x8d,
        0xb5, 0x48, 0xc5, 0x58, 0x46, 0x5d, 0x79, 0xdb, 0x03, 0xfd, 0x35, 0x9c, 0x6c, 0xd5, 0xbd,
        0x9d, 0x85,
    ];
    let got = &ctx.temp_buffer()[out_off as usize..out_off as usize + 32];
    assert_eq!(got, &expected);
}

#[test]
fn test_array_concat_dynamic_entropy_preimage() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[HALT],
        &[],
        FIVE_VM_PROGRAM_ID,
        &[],
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let house_entropy = [1u8; 32];
    let user_seed = [2u8; 16];
    let slot = [3u8; 8];
    let unix_ts = [4u8; 8];
    let request_count = [5u8; 8];

    let house_off = ctx.alloc_temp(32).unwrap();
    ctx.temp_buffer_mut()[house_off as usize..house_off as usize + 32]
        .copy_from_slice(&house_entropy);
    let user_off = ctx.alloc_temp(16).unwrap();
    ctx.temp_buffer_mut()[user_off as usize..user_off as usize + 16].copy_from_slice(&user_seed);
    let slot_off = ctx.alloc_temp(8).unwrap();
    ctx.temp_buffer_mut()[slot_off as usize..slot_off as usize + 8].copy_from_slice(&slot);
    let ts_off = ctx.alloc_temp(8).unwrap();
    ctx.temp_buffer_mut()[ts_off as usize..ts_off as usize + 8].copy_from_slice(&unix_ts);
    let req_off = ctx.alloc_temp(8).unwrap();
    ctx.temp_buffer_mut()[req_off as usize..req_off as usize + 8].copy_from_slice(&request_count);

    // ((((house + user_seed) + slot) + unix_ts) + request_count)
    ctx.push(ValueRef::TempRef(house_off, 32)).unwrap();
    ctx.push(ValueRef::TempRef(user_off, 16)).unwrap();
    handle_arrays(ARRAY_CONCAT, &mut ctx).unwrap();
    let mut merged = ctx.pop().unwrap();

    ctx.push(merged).unwrap();
    ctx.push(ValueRef::TempRef(slot_off, 8)).unwrap();
    handle_arrays(ARRAY_CONCAT, &mut ctx).unwrap();
    merged = ctx.pop().unwrap();

    ctx.push(merged).unwrap();
    ctx.push(ValueRef::TempRef(ts_off, 8)).unwrap();
    handle_arrays(ARRAY_CONCAT, &mut ctx).unwrap();
    merged = ctx.pop().unwrap();

    ctx.push(merged).unwrap();
    ctx.push(ValueRef::TempRef(req_off, 8)).unwrap();
    handle_arrays(ARRAY_CONCAT, &mut ctx).unwrap();
    merged = ctx.pop().unwrap();

    let (len, bytes) = ctx.extract_string_slice(&merged).unwrap();
    assert_eq!(len, 72);

    let mut expected = vec![];
    expected.extend_from_slice(&house_entropy);
    expected.extend_from_slice(&user_seed);
    expected.extend_from_slice(&slot);
    expected.extend_from_slice(&unix_ts);
    expected.extend_from_slice(&request_count);
    assert_eq!(bytes, expected.as_slice());
}

#[test]
fn test_array_concat_overflow_fails() {
    let mut storage = StackStorage::new();
    let mut ctx = ExecutionContext::new(
        &[HALT],
        &[],
        FIVE_VM_PROGRAM_ID,
        &[],
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );

    let left_off = ctx.alloc_temp(200).unwrap();
    for b in &mut ctx.temp_buffer_mut()[left_off as usize..left_off as usize + 200] {
        *b = 7;
    }
    let right_heap_id = ctx.heap_alloc(104).unwrap();
    ctx.get_heap_data_mut(right_heap_id, 4)
        .unwrap()
        .copy_from_slice(&(100u32).to_le_bytes());
    for b in ctx.get_heap_data_mut(right_heap_id + 4, 100).unwrap() {
        *b = 9;
    }

    ctx.push(ValueRef::TempRef(left_off, 200)).unwrap();
    ctx.push(ValueRef::HeapString(right_heap_id)).unwrap();
    let err = handle_arrays(ARRAY_CONCAT, &mut ctx).unwrap_err();
    assert_eq!(err, five_vm_mito::error::VMErrorCode::OutOfMemory);
}

#[test]
fn test_ed25519_sysvar_positive_and_negatives() {
    let expected_pubkey = [11u8; 32];
    let signature = [22u8; 64];
    let message = [33u8; 32];
    let wrong_pubkey = [44u8; 32];
    let wrong_signature = [55u8; 64];
    let wrong_message = [66u8; 32];

    // Positive case
    {
        let mut sysvar_lamports = 1u64;
        let mut sysvar_data = build_ed25519_sysvar_payload(
            &expected_pubkey,
            &signature,
            &message,
            &ED25519_PROGRAM_ID_BYTES,
        );
        let sysvar_key = [42u8; 32];
        let sysvar_account = AccountInfo::new(
            &sysvar_key,
            false,
            false,
            &mut sysvar_lamports,
            &mut sysvar_data,
            &FIVE_VM_PROGRAM_ID,
            false,
            0,
        );
        let accounts = [sysvar_account];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[HALT],
            &accounts,
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        let pk_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[pk_off as usize..pk_off as usize + 32]
            .copy_from_slice(&expected_pubkey);
        let msg_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[msg_off as usize..msg_off as usize + 32].copy_from_slice(&message);
        let sig_off = ctx.alloc_temp(64).unwrap();
        ctx.temp_buffer_mut()[sig_off as usize..sig_off as usize + 64].copy_from_slice(&signature);

        ctx.push(ValueRef::AccountRef(0, 0)).unwrap();
        ctx.push(ValueRef::TempRef(pk_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(msg_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(sig_off, 64)).unwrap();
        handle_syscall_verify_ed25519_instruction(&mut ctx).unwrap();
        assert_eq!(ctx.pop().unwrap(), ValueRef::Bool(true));
    }

    // Wrong signer pubkey (should fail)
    {
        let mut sysvar_lamports = 1u64;
        let mut sysvar_data = build_ed25519_sysvar_payload(
            &expected_pubkey,
            &signature,
            &message,
            &ED25519_PROGRAM_ID_BYTES,
        );
        let sysvar_key = [43u8; 32];
        let sysvar_account = AccountInfo::new(
            &sysvar_key,
            false,
            false,
            &mut sysvar_lamports,
            &mut sysvar_data,
            &FIVE_VM_PROGRAM_ID,
            false,
            0,
        );
        let accounts = [sysvar_account];

        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[HALT],
            &accounts,
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );

        let pk_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[pk_off as usize..pk_off as usize + 32].copy_from_slice(&wrong_pubkey);
        let msg_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[msg_off as usize..msg_off as usize + 32].copy_from_slice(&message);
        let sig_off = ctx.alloc_temp(64).unwrap();
        ctx.temp_buffer_mut()[sig_off as usize..sig_off as usize + 64].copy_from_slice(&signature);

        ctx.push(ValueRef::AccountRef(0, 0)).unwrap();
        ctx.push(ValueRef::TempRef(pk_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(msg_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(sig_off, 64)).unwrap();
        handle_syscall_verify_ed25519_instruction(&mut ctx).unwrap();
        assert_eq!(ctx.pop().unwrap(), ValueRef::Bool(false));
    }

    // Wrong message/signature and malformed ix program-id/missing sysvar fail
    {
        let mut sysvar_lamports = 1u64;
        let mut sysvar_data = build_ed25519_sysvar_payload(
            &expected_pubkey,
            &signature,
            &message,
            &ED25519_PROGRAM_ID_BYTES,
        );
        let sysvar_key = [44u8; 32];
        let sysvar_account = AccountInfo::new(
            &sysvar_key,
            false,
            false,
            &mut sysvar_lamports,
            &mut sysvar_data,
            &FIVE_VM_PROGRAM_ID,
            false,
            0,
        );
        let accounts = [sysvar_account];

        // Wrong message
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[HALT],
            &accounts,
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        let pk_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[pk_off as usize..pk_off as usize + 32]
            .copy_from_slice(&expected_pubkey);
        let msg_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[msg_off as usize..msg_off as usize + 32]
            .copy_from_slice(&wrong_message);
        let sig_off = ctx.alloc_temp(64).unwrap();
        ctx.temp_buffer_mut()[sig_off as usize..sig_off as usize + 64].copy_from_slice(&signature);
        ctx.push(ValueRef::AccountRef(0, 0)).unwrap();
        ctx.push(ValueRef::TempRef(pk_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(msg_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(sig_off, 64)).unwrap();
        handle_syscall_verify_ed25519_instruction(&mut ctx).unwrap();
        assert_eq!(ctx.pop().unwrap(), ValueRef::Bool(false));

        // Wrong signature
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[HALT],
            &accounts,
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        let pk_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[pk_off as usize..pk_off as usize + 32]
            .copy_from_slice(&expected_pubkey);
        let msg_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[msg_off as usize..msg_off as usize + 32].copy_from_slice(&message);
        let sig_off = ctx.alloc_temp(64).unwrap();
        ctx.temp_buffer_mut()[sig_off as usize..sig_off as usize + 64]
            .copy_from_slice(&wrong_signature);
        ctx.push(ValueRef::AccountRef(0, 0)).unwrap();
        ctx.push(ValueRef::TempRef(pk_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(msg_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(sig_off, 64)).unwrap();
        handle_syscall_verify_ed25519_instruction(&mut ctx).unwrap();
        assert_eq!(ctx.pop().unwrap(), ValueRef::Bool(false));

        // Malformed ix (wrong program-id in sysvar payload)
        let mut bad_sysvar_lamports = 1u64;
        let mut bad_sysvar_data =
            build_ed25519_sysvar_payload(&expected_pubkey, &signature, &message, &[9u8; 32]);
        let bad_sysvar_key = [45u8; 32];
        let bad_sysvar_account = AccountInfo::new(
            &bad_sysvar_key,
            false,
            false,
            &mut bad_sysvar_lamports,
            &mut bad_sysvar_data,
            &FIVE_VM_PROGRAM_ID,
            false,
            0,
        );
        let bad_accounts = [bad_sysvar_account];
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[HALT],
            &bad_accounts,
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        let pk_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[pk_off as usize..pk_off as usize + 32]
            .copy_from_slice(&expected_pubkey);
        let msg_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[msg_off as usize..msg_off as usize + 32].copy_from_slice(&message);
        let sig_off = ctx.alloc_temp(64).unwrap();
        ctx.temp_buffer_mut()[sig_off as usize..sig_off as usize + 64].copy_from_slice(&signature);
        ctx.push(ValueRef::AccountRef(0, 0)).unwrap();
        ctx.push(ValueRef::TempRef(pk_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(msg_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(sig_off, 64)).unwrap();
        handle_syscall_verify_ed25519_instruction(&mut ctx).unwrap();
        assert_eq!(ctx.pop().unwrap(), ValueRef::Bool(false));

        // Missing sysvar account: should fail before verification.
        let mut storage = StackStorage::new();
        let mut ctx = ExecutionContext::new(
            &[HALT],
            &[],
            FIVE_VM_PROGRAM_ID,
            &[],
            0,
            &mut storage,
            0,
            0,
            0,
            0,
            0,
            0,
        );
        let pk_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[pk_off as usize..pk_off as usize + 32]
            .copy_from_slice(&expected_pubkey);
        let msg_off = ctx.alloc_temp(32).unwrap();
        ctx.temp_buffer_mut()[msg_off as usize..msg_off as usize + 32].copy_from_slice(&message);
        let sig_off = ctx.alloc_temp(64).unwrap();
        ctx.temp_buffer_mut()[sig_off as usize..sig_off as usize + 64].copy_from_slice(&signature);
        ctx.push(ValueRef::AccountRef(0, 0)).unwrap();
        ctx.push(ValueRef::TempRef(pk_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(msg_off, 32)).unwrap();
        ctx.push(ValueRef::TempRef(sig_off, 64)).unwrap();
        assert!(handle_syscall_verify_ed25519_instruction(&mut ctx).is_err());
    }
}
