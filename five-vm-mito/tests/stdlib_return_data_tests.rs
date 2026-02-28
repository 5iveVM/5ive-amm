use five_protocol::{opcodes::HALT, ValueRef};
use five_vm_mito::{
    handlers::system::program::{handle_syscall_get_return_data, handle_syscall_set_return_data},
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
    let offset = ctx.alloc_temp((bytes.len() + 2) as u8).expect("alloc string temp");
    let start = offset as usize;
    ctx.temp_buffer_mut()[start] = bytes.len() as u8;
    ctx.temp_buffer_mut()[start + 1] = 0;
    ctx.temp_buffer_mut()[start + 2..start + 2 + bytes.len()].copy_from_slice(bytes);
    ValueRef::StringRef(offset as u16)
}

#[test]
fn set_return_data_accepts_temp_string_buffer() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let bytes = [1u8, 2, 3];
    let data_ref = push_string_ref(&mut ctx, &bytes);
    ctx.push(data_ref).unwrap();

    handle_syscall_set_return_data(&mut ctx).expect("set return data syscall");
}

#[test]
fn get_return_data_returns_mock_length_on_host() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let data_offset = ctx.alloc_temp(8).expect("alloc data");
    let pid_offset = ctx.alloc_temp(32).expect("alloc pid");
    ctx.push(ValueRef::TempRef(data_offset, 8)).unwrap();
    ctx.push(ValueRef::U64(8)).unwrap();
    ctx.push(ValueRef::TempRef(pid_offset, 32)).unwrap();

    handle_syscall_get_return_data(&mut ctx).expect("get return data syscall");
    assert_eq!(ctx.pop().unwrap(), ValueRef::U64(0));
}
