use five_vm_mito::{ExecutionContext, FIVE_VM_PROGRAM_ID, StackStorage, ValueRef};
use pinocchio::account_info::AccountInfo;
use pinocchio::pubkey::Pubkey;

#[test]
fn test_temp_value_roundtrip_all_variants() {
    let bytecode: &[u8] = &[];
    let accounts: &[AccountInfo] = &[];
    let program_id = Pubkey::default();
    let instruction_data: &[u8] = &[];

    let variants = [
        ValueRef::Empty,
        ValueRef::U8(1),
        ValueRef::U64(2),
        ValueRef::I64(-3),
        ValueRef::Bool(true),
        ValueRef::PubkeyRef(4),
        ValueRef::AccountRef(5, 6),
        ValueRef::StringRef(7),
        ValueRef::ArrayRef(8),
        ValueRef::HeapString(9),
        ValueRef::HeapArray(10),
        ValueRef::ResultRef(11, 12),
        ValueRef::InputRef(13),
        ValueRef::TempRef(14, 15),
        ValueRef::TupleRef(16, 17),
        ValueRef::OptionalRef(18, 19),
    ];

    for v in variants.iter() {
        let mut storage = StackStorage::new(bytecode);
        let mut ctx = ExecutionContext::new(
            bytecode,
            accounts,
            program_id,
            instruction_data,
            0,
            &mut storage,
            0,
            0,
        );
        let offset = ctx.write_value_to_temp(v).expect("write to temp");
        let decoded = ctx.read_value_from_temp(offset).expect("read from temp");
        assert_eq!(*v, decoded);
    }
}
