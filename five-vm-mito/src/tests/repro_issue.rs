
#[cfg(test)]
mod tests {
    use crate::tests::framework::{TestUtils};
    use crate::handlers::functions::handle_functions;
    use crate::handlers::locals::handle_nibble_locals;
    use crate::handlers::arithmetic::handle_arithmetic;
    use crate::error::VMErrorCode;
    use five_protocol::{opcodes::*, ValueRef};
    use pinocchio::pubkey::Pubkey;

    #[test]
    fn test_load_param_behavior() {
        let key = Pubkey::default();
        let (_signer_acc, signer_info) = TestUtils::create_signer_account_info(&key, 100);
        let accounts = vec![signer_info];
        let bytecode = TestUtils::create_simple_bytecode(&[]);
        let mut storage = crate::stack::StackStorage::new(&bytecode);
        
        let mut vm = TestUtils::create_custom_context(
            &bytecode,
            &accounts,
            Pubkey::default(),
            &[],
            11, 
            &mut storage,
            0
        );

        // Case 1: Empty Parameter -> Should Return Error (Fixed Bug)
        vm.allocate_params(4).unwrap(); // p[0]..p[3]
        
        // p[3] is Empty.
        let res = handle_nibble_locals(LOAD_PARAM_3, &mut vm);
        assert!(res.is_err(), "Empty param should return InvalidParameter error");
        match res {
            Err(crate::error::VMErrorCode::InvalidParameter) => {},
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

    #[test]
    fn test_call_reg_stack_shift() {
        // Verify 4 stack items causes shift (Simulating FuncIdx on stack)
        let key = Pubkey::default();
        let (_signer_acc, signer_info) = TestUtils::create_signer_account_info(&key, 100);
        let accounts = vec![signer_info];
        let bytecode = TestUtils::create_simple_bytecode(&[CALL_REG, 14, 0, HALT, RET]);
        let mut storage = crate::stack::StackStorage::new(&bytecode);
        let mut vm = TestUtils::create_custom_context(&bytecode, &accounts, Pubkey::default(), &[], 11, &mut storage, 0);

        // Stack: [FuncIdx, Payer, Mint, Decimals]
        vm.push(ValueRef::U64(999)).unwrap(); // FuncIdx
        vm.push(ValueRef::AccountRef(0, 0)).unwrap(); // Payer
        vm.push(ValueRef::AccountRef(0, 0)).unwrap(); // Mint
        vm.push(ValueRef::U8(9)).unwrap(); // Decimals

        // Exec CALL_REG
        handle_functions(CALL_REG, &mut vm).unwrap();

        // Check new frame params
        // p[3] should be Mint (Shifted)
        let p3 = vm.parameters()[3];
        if matches!(p3, ValueRef::AccountRef(_, _)) {
             println!("STACK MISALIGNMENT CONFIRMED: p[3] is Account (Mint)");
        } else {
             panic!("Expected misalignment, got {:?}", p3);
        }

        // Verify frame was pushed (call_depth increased)
        assert_eq!(vm.call_depth(), 1, "Call frame should be pushed, call_depth should be 1");

        // Verify IP was set to function address (14)
        assert_eq!(vm.ip(), 14, "Instruction pointer should be set to function address (14)");
    }
    
    #[test]
    fn test_call_reg_clean_stack() {
        // Verify 3 items (Clean Stack) works correctly
        let key = Pubkey::default();
        let (_signer_acc, signer_info) = TestUtils::create_signer_account_info(&key, 100);
        let accounts = vec![signer_info];
        let bytecode = TestUtils::create_simple_bytecode(&[CALL_REG, 14, 0, HALT, RET]);
        let mut storage = crate::stack::StackStorage::new(&bytecode);
        let mut vm = TestUtils::create_custom_context(&bytecode, &accounts, Pubkey::default(), &[], 11, &mut storage, 0);

        // Stack: [Payer, Mint, Decimals] (Correct Compiler Output)
        vm.push(ValueRef::AccountRef(0, 0)).unwrap(); // Payer (p[1])
        vm.push(ValueRef::AccountRef(0, 0)).unwrap(); // Mint (p[2])
        vm.push(ValueRef::U8(9)).unwrap(); // Decimals (p[3])

        // Exec CALL_REG
        handle_functions(CALL_REG, &mut vm).unwrap();

        // Check params
        // p[1]=Payer, p[2]=Mint, p[3]=Decimals
        let p3 = vm.parameters()[3];
        if matches!(p3, ValueRef::U8(9)) {
             println!("Stack clean (p[3] is Decimals)");
        } else {
             panic!("Stack clean FAILED. p[3] is {:?}", p3);
        }

        // Verify frame was pushed (call_depth increased)
        assert_eq!(vm.call_depth(), 1, "Call frame should be pushed, call_depth should be 1");

        // Verify IP was set to function address (14)
        assert_eq!(vm.ip(), 14, "Instruction pointer should be set to function address (14)");
    }
}
