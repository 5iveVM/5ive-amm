#[cfg(test)]
mod tests {
    use crate::error::VMErrorCode;
    use crate::handlers::arithmetic::handle_arithmetic;
    use crate::handlers::functions::handle_functions;
    use crate::handlers::locals::handle_nibble_locals;
    use crate::tests::framework::TestUtils;
    use five_protocol::{opcodes::*, ValueRef};
    use pinocchio::pubkey::Pubkey;

    #[test]
    fn test_load_param_behavior() {
        let key = Pubkey::default();
        let (_signer_acc, signer_info) = TestUtils::create_signer_account_info(&key, 100);
        let accounts = vec![signer_info];
        let bytecode = TestUtils::create_simple_bytecode(&[]);
        let mut storage = crate::stack::StackStorage::new();

        let mut vm = TestUtils::create_custom_context(
            &bytecode,
            &accounts,
            Pubkey::default(),
            &[],
            11,
            &mut storage,
            0,
        );

        // Case 1: Empty Parameter -> Should Return Error (Fixed Bug)
        vm.allocate_params(4).unwrap(); // p[0]..p[3]

        // p[3] is Empty.
        let res = handle_nibble_locals(LOAD_PARAM_3, &mut vm);
        assert!(
            res.is_err(),
            "Empty param should return InvalidParameter error"
        );
        match res {
            Err(crate::error::VMErrorCode::InvalidParameter) => {}
            _ => panic!("Expected InvalidParameter error, got {:?}", res),
        }

        // Case 2: U8(9) Parameter -> Returns 9.
        vm.parameters_mut()[3] = ValueRef::U8(9);
        let res = handle_nibble_locals(LOAD_PARAM_3, &mut vm);
        assert!(res.is_ok());
        let top = vm.pop().unwrap();
        assert_eq!(top.as_u64().unwrap(), 9);

        // Case 3: AccountRef Parameter
        vm.parameters_mut()[3] = ValueRef::AccountRef(0, 0);
        // We just verify it loads. Comparison check requires context.
        handle_nibble_locals(LOAD_PARAM_3, &mut vm).unwrap();
        let top = vm.pop().unwrap();
        assert!(matches!(top, ValueRef::AccountRef(_, _)));
    }
}
