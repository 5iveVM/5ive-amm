use five_protocol::{opcodes::HALT, ValueRef};
use five_vm_mito::{
    handlers::system::memory::{handle_syscall_memcmp, handle_syscall_memcpy},
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

#[test]
fn memcpy_copies_bytes_between_temp_buffers() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let source_bytes = [1u8, 2, 3, 4];
    let src_offset = ctx.alloc_temp(source_bytes.len() as u8).expect("alloc src");
    let dst_offset = ctx.alloc_temp(source_bytes.len() as u8).expect("alloc dst");
    ctx.temp_buffer_mut()[src_offset as usize..src_offset as usize + source_bytes.len()]
        .copy_from_slice(&source_bytes);

    ctx.push(ValueRef::TempRef(dst_offset, source_bytes.len() as u8)).unwrap();
    ctx.push(ValueRef::TempRef(src_offset, source_bytes.len() as u8)).unwrap();
    ctx.push(ValueRef::U64(source_bytes.len() as u64)).unwrap();

    handle_syscall_memcpy(&mut ctx).expect("memcpy syscall");

    let copied =
        &ctx.temp_buffer()[dst_offset as usize..dst_offset as usize + source_bytes.len()];
    assert_eq!(copied, &source_bytes);
}

#[test]
fn memcmp_writes_zero_for_equal_buffers() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let bytes = [5u8, 6, 7, 8];
    let left_offset = ctx.alloc_temp(bytes.len() as u8).expect("alloc left");
    let right_offset = ctx.alloc_temp(bytes.len() as u8).expect("alloc right");
    let result_offset = ctx.alloc_temp(4).expect("alloc result");
    ctx.temp_buffer_mut()[left_offset as usize..left_offset as usize + bytes.len()]
        .copy_from_slice(&bytes);
    ctx.temp_buffer_mut()[right_offset as usize..right_offset as usize + bytes.len()]
        .copy_from_slice(&bytes);

    ctx.push(ValueRef::TempRef(left_offset, bytes.len() as u8)).unwrap();
    ctx.push(ValueRef::TempRef(right_offset, bytes.len() as u8)).unwrap();
    ctx.push(ValueRef::U64(bytes.len() as u64)).unwrap();
    ctx.push(ValueRef::TempRef(result_offset, 4)).unwrap();

    handle_syscall_memcmp(&mut ctx).expect("memcmp syscall");

    let result_bytes = &ctx.temp_buffer()[result_offset as usize..result_offset as usize + 4];
    assert_eq!(i32::from_le_bytes(result_bytes.try_into().unwrap()), 0);
}
