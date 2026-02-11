//! Helper functions for AST generation patterns.

use super::super::types::FieldInfo;
use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use five_protocol::opcodes::*;

impl ASTGenerator {
    /// Emit optimized SET_LOCAL when the index fits.
    pub(super) fn emit_set_local<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        index: u32,
        context: &str,
    ) {
        // Optimization: Use nibble immediate opcodes for indices 0-3
        if index <= 3 {
            let opcode = match index {
                0 => SET_LOCAL_0,
                1 => SET_LOCAL_1,
                2 => SET_LOCAL_2,
                3 => SET_LOCAL_3,
                _ => unreachable!("Index checked to be 0-3"),
            };
            emitter.emit_opcode(opcode);
            #[cfg(debug_assertions)]
            println!(
                "DEBUG: Generated SET_LOCAL_{} (nibble immediate) for {}",
                index, context
            );
        } else {
            emitter.emit_opcode(SET_LOCAL);
            emitter.emit_u8(index as u8);
            #[cfg(debug_assertions)]
            println!(
                "DEBUG: Generated SET_LOCAL {} (V1 mode) for {}",
                index, context
            );
        }
    }

    /// Emit optimized local variable GET operation
    ///
    /// When index is 0-3, this emits the nibble-immediate opcodes (GET_LOCAL_0 through GET_LOCAL_3).
    /// Otherwise, it emits the standard GET_LOCAL opcode with a u8 index.
    pub(super) fn emit_get_local<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        index: u32,
        context: &str,
    ) {
        if index <= 3 {
            let opcode = match index {
                0 => GET_LOCAL_0,
                1 => GET_LOCAL_1,
                2 => GET_LOCAL_2,
                3 => GET_LOCAL_3,
                _ => unreachable!("Index checked to be 0-3"),
            };
            emitter.emit_opcode(opcode);
            #[cfg(debug_assertions)]
            println!(
                "DEBUG: Generated GET_LOCAL_{} (nibble immediate) for {}",
                index, context
            );
        } else {
            emitter.emit_opcode(GET_LOCAL);
            emitter.emit_u8(index as u8);
            #[cfg(debug_assertions)]
            println!(
                "DEBUG: Generated GET_LOCAL {} (V1 mode) for {}",
                index, context
            );
        }
    }

    /// Add a local variable to the symbol table
    ///
    /// This is a convenience method that creates a FieldInfo struct and
    /// adds it to the local symbol table while incrementing the field counter.
    ///
    /// Returns the offset assigned to the variable.
    pub(super) fn add_local_field(
        &mut self,
        name: String,
        field_type: String,
        is_mutable: bool,
        is_optional: bool,
    ) -> u32 {
        let offset = self.field_counter;
        let field_info = FieldInfo {
            offset,
            field_type,
            is_mutable,
            is_optional,
            is_parameter: false,
        };
        self.local_symbol_table.insert(name, field_info);
        self.field_counter += 1;
        offset
    }



    /// Try to emit a built-in arithmetic/comparison method
    ///
    /// Returns Some(()) if the method was a built-in and was emitted,
    /// None if the method is not a built-in.
    pub(super) fn try_emit_builtin_method<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        method: &str,
    ) -> Option<()> {
        let opcode = match method {
            "add" => ADD,
            "sub" => SUB,
            "mul" => MUL,
            "div" => DIV,
            "mod" => MOD,
            "eq" => EQ,
            "ne" => NEQ,
            "lt" => LT,
            "le" | "lte" => LTE,
            "gt" => GT,
            "ge" | "gte" => GTE,
            "and" => AND,
            "or" => OR,
            "is_some" => OPTIONAL_IS_SOME,
            "is_none" => OPTIONAL_IS_NONE,
            "get_value" => OPTIONAL_GET_VALUE,
            _ => return None,
        };
        emitter.emit_opcode(opcode);
        Some(())
    }

    /// Parse a qualified function name like "module::function"
    /// 
    /// Returns Some((module_name, function_name)) if the name contains "::"
    /// Returns None for unqualified function names
    pub(super) fn parse_qualified_name(name: &str) -> Option<(&str, &str)> {
        if let Some(idx) = name.find("::") {
            let module_name = &name[..idx];
            let func_name = &name[idx + 2..];
            if !module_name.is_empty() && !func_name.is_empty() {
                return Some((module_name, func_name));
            }
        }
        None
    }

    /// Stable 16-bit selector for external function name resolution.
    /// Uses FNV-1a and truncates to 16 bits.
    pub(super) fn external_selector(name: &str) -> u16 {
        const OFFSET: u32 = 0x811C9DC5;
        const PRIME: u32 = 0x01000193;

        let mut hash = OFFSET;
        for b in name.as_bytes() {
            hash ^= *b as u32;
            hash = hash.wrapping_mul(PRIME);
        }
        (hash & 0xFFFF) as u16
    }
}
