use super::DslBytecodeGenerator;

impl DslBytecodeGenerator {
    /// Log a bytecode operation with human-readable description
    pub fn log_opcode(&mut self, opcode_name: &str, description: &str) {
        let offset = self.position;
        self.compilation_log
            .push(format!("{:04X}: {} - {}", offset, opcode_name, description));
    }

    /// Log a bytecode operation with parameters
    pub fn log_opcode_with_params(&mut self, opcode_name: &str, params: &str, description: &str) {
        let offset = self.position;
        self.compilation_log.push(format!(
            "{:04X}: {} {} - {}",
            offset, opcode_name, params, description
        ));
    }

    /// Log header generation
    pub fn log_header(&mut self, header_type: &str, description: &str) {
        self.compilation_log
            .push(format!("HEADER: {} - {}", header_type, description));
    }

    /// Get the compilation log (disassembly)
    pub fn get_compilation_log(&self) -> &[String] {
        &self.compilation_log
    }

    /// Clear the compilation log
    pub fn clear_compilation_log(&mut self) {
        self.compilation_log.clear();
    }

    /// Get human-readable opcode name from opcode value
    fn get_opcode_name(opcode: u8) -> &'static str {
        five_protocol::opcodes::opcode_name(opcode)
    }

    /// Emit opcode with logging
    pub fn emit_opcode_logged(&mut self, opcode: u8, description: &str) {
        let opcode_name = Self::get_opcode_name(opcode);
        self.log_opcode(opcode_name, description);
        self.emit_opcode(opcode);
    }

    /// Emit opcode with parameters and logging
    pub fn emit_opcode_with_params(&mut self, opcode: u8, params: &str, description: &str) {
        let opcode_name = Self::get_opcode_name(opcode);
        self.log_opcode_with_params(opcode_name, params, description);
        self.emit_opcode(opcode);
    }
}
