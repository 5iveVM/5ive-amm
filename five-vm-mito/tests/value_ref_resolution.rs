use five_protocol::ValueRef;
use five_vm_mito::{error::VMErrorCode, AccountInfo, ExecutionContext, MitoVM, Pubkey, StackStorage, Value};

#[test]
fn resolve_input_ref_to_u64() {
    let data = 42u64.to_le_bytes();
    let program_id = Pubkey::default();
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(&[], &accounts, program_id, &data, 0, &mut storage, 0, 0, 0, 0, 0, 0);
    let value = MitoVM::resolve_value_ref(&ValueRef::InputRef(0), &ctx).unwrap();
    assert_eq!(value, Value::U64(42));
}

#[test]
fn resolve_pubkey_ref() {
    let accounts: [AccountInfo; 0] = [];
    let pk_bytes = [3u8; 32];
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &pk_bytes,
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );
    let value = MitoVM::resolve_value_ref(&ValueRef::PubkeyRef(0), &ctx).unwrap();
    assert_eq!(value, Value::Pubkey(Pubkey::from(pk_bytes)));
}

#[test]
fn input_ref_out_of_bounds() {
    let data = [0u8; 4];
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
        &data,
        0,
        &mut storage,
        0,
        0,
        0,
        0,
        0,
        0,
    );
    let err = MitoVM::resolve_value_ref(&ValueRef::InputRef(1), &ctx).unwrap_err();
    assert_eq!(err, VMErrorCode::InvalidOperation);
}

#[test]
fn pubkey_ref_out_of_bounds() {
    let accounts: [AccountInfo; 0] = [];
    let mut storage = StackStorage::new();
    let ctx = ExecutionContext::new(
        &[],
        &accounts,
        Pubkey::default(),
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
    let err = MitoVM::resolve_value_ref(&ValueRef::PubkeyRef(40), &ctx).unwrap_err();
    assert_eq!(err, VMErrorCode::InvalidOperation);
}
