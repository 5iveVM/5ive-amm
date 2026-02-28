use five_protocol::{opcodes::HALT, ValueRef};
use five_vm_mito::{
    handlers::system::logging::{
        handle_syscall_log, handle_syscall_log_64, handle_syscall_log_compute_units,
        handle_syscall_log_data, handle_syscall_log_pubkey,
    },
    ExecutionContext, StackStorage, FIVE_VM_PROGRAM_ID,
};

fn new_context<'a>(storage: &'a mut StackStorage) -> ExecutionContext<'a> {
    ExecutionContext::new(
        &[HALT],
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

fn push_string_ref(ctx: &mut ExecutionContext<'_>, bytes: &[u8]) -> ValueRef {
    let offset = ctx
        .alloc_temp((bytes.len() + 2) as u8)
        .expect("alloc string temp");
    let start = offset as usize;
    ctx.temp_buffer_mut()[start] = bytes.len() as u8;
    ctx.temp_buffer_mut()[start + 1] = 0;
    ctx.temp_buffer_mut()[start + 2..start + 2 + bytes.len()].copy_from_slice(bytes);
    ValueRef::StringRef(offset as u16)
}

#[test]
fn log_message_accepts_temp_string_buffer() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let msg = b"hello from stdlib";
    let msg_ref = push_string_ref(&mut ctx, msg);
    ctx.push(msg_ref).unwrap();

    handle_syscall_log(&mut ctx).expect("log message syscall");
}

#[test]
fn log_words_accepts_five_u64_values() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    for value in [1u64, 2, 3, 4, 5] {
        ctx.push(ValueRef::U64(value)).unwrap();
    }

    handle_syscall_log_64(&mut ctx).expect("log_64 syscall");
}

#[test]
fn log_compute_units_succeeds_without_arguments() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);
    handle_syscall_log_compute_units(&mut ctx).expect("log compute units syscall");
}

#[test]
fn log_data_accepts_temp_buffer() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let bytes = [7u8, 8, 9, 10];
    let data_ref = push_string_ref(&mut ctx, &bytes);
    ctx.push(data_ref).unwrap();

    handle_syscall_log_data(&mut ctx).expect("log data syscall");
}

#[test]
fn log_pubkey_accepts_temp_buffer_pubkey() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let pubkey = [9u8; 32];
    let offset = ctx.alloc_temp(32).expect("alloc temp");
    ctx.temp_buffer_mut()[offset as usize..offset as usize + 32].copy_from_slice(&pubkey);
    ctx.push(ValueRef::TempRef(offset, 32)).unwrap();

    handle_syscall_log_pubkey(&mut ctx).expect("log pubkey syscall");
}
