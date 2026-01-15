#[cfg(test)]
mod tests {
    use five_protocol::opcodes;

    #[test]
    fn test_opcode_lookup() {
        let opcode = 0xdc; // LOAD_PARAM_0
        let info = opcodes::get_opcode_info(opcode);
        assert!(info.is_some(), "LOAD_PARAM_0 (0xdc) should be found");
        assert_eq!(info.unwrap().name, "LOAD_PARAM_0");
    }
}
