use five_protocol::{opcodes::HALT, ValueRef};
use five_vm_mito::{
    error::VMErrorCode,
    handlers::system::{
        compute::handle_syscall_remaining_compute_units,
        sysvars::{
            handle_syscall_get_clock_sysvar, handle_syscall_get_epoch_schedule_sysvar,
            handle_syscall_get_rent_sysvar,
        },
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

#[test]
fn remaining_compute_units_returns_mock_value_offchain() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    handle_syscall_remaining_compute_units(&mut ctx).expect("remaining CU syscall");
    assert_eq!(ctx.pop().unwrap(), ValueRef::U64(200_000));
}

#[test]
fn clock_sysvar_returns_tuple_ref() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    handle_syscall_get_clock_sysvar(&mut ctx).expect("clock sysvar syscall");
    assert!(matches!(ctx.pop().unwrap(), ValueRef::TupleRef(_, 45)));
}

#[test]
fn rent_sysvar_returns_lamports_per_byte_year() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    handle_syscall_get_rent_sysvar(&mut ctx).expect("rent sysvar syscall");
    assert!(matches!(ctx.pop().unwrap(), ValueRef::U64(_)));
}

#[test]
fn epoch_schedule_sysvar_requires_runtime_integration() {
    let mut storage = StackStorage::new();
    let mut ctx = new_context(&mut storage);

    let err =
        handle_syscall_get_epoch_schedule_sysvar(&mut ctx).expect_err("epoch schedule syscall");
    assert_eq!(err, VMErrorCode::RuntimeIntegrationRequired);
    assert_eq!(ctx.pop(), Err(VMErrorCode::StackUnderflow));
}
