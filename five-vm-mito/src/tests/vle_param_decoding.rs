#[cfg(test)]
mod tests {
    use crate::utils::parse_vle_parameters_unified;
    use crate::context::ExecutionManager;
    use crate::stack::StackStorage;
    use five_protocol::{ValueRef, VLE};
    use pinocchio::pubkey::Pubkey;

    #[test]
    fn test_vle_decode_u64_parameter() {
        // Construct VLE data for [function_index=0, param_count=1, param1=u64_max]
        // 0 -> [0x00]
        // 1 -> [0x01]
        // u64::MAX -> 10 bytes: [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01]

        let mut input_data = Vec::new();
        input_data.push(0x00); // function index
        input_data.push(0x01); // param count

        // Encode u64::MAX
        let (size, bytes) = VLE::encode_u64(u64::MAX);
        input_data.extend_from_slice(&bytes[..size]);

        let mut params = [ValueRef::Empty; 8];
        let script = [0u8; 10]; // Minimal script
        let mut storage = StackStorage::new(&script);

        let mut ctx = ExecutionManager::new(
            &script,
            &[],
            Pubkey::default(),
            &input_data,
            0,
            &mut storage,
            0,
            0
        );

        let result = parse_vle_parameters_unified(&mut ctx, &input_data, &mut params);
        assert!(result.is_ok(), "Parsing failed: {:?}", result.err());

        // Check params[0] is function index
        if let ValueRef::U64(val) = params[0] {
            assert_eq!(val, 0, "Function index mismatch");
        } else {
            panic!("Function index not U64");
        }

        // Check params[1] is the u64::MAX value
        if let ValueRef::U64(val) = params[1] {
            assert_eq!(val, u64::MAX, "Parameter value mismatch");
        } else {
            panic!("Parameter 1 is not U64: {:?}", params[1]);
        }
    }
}
