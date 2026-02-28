//! Function and method call generation.

use super::super::OpcodeEmitter;
use super::assignments::{collect_byte_array_literal_bytes, fixed_u8_array_len};
use super::types::ASTGenerator;
use crate::ast::{AstNode, InstructionParameter, TypeNode};
use crate::bytecode_generator::account_utils::account_index_from_param_index;
use crate::type_checker::{InterfaceInfo, InterfaceMethod};
use core::cmp::Ordering;
use five_protocol::opcodes::*;
use five_protocol::Value;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    fn resolve_account_literal_offset(
        &self,
        account_arg: &AstNode,
        offset_arg: &AstNode,
    ) -> Result<(u8, u32), VMError> {
        let account_name = match account_arg {
            AstNode::Identifier(name) => name,
            _ => return Err(VMError::InvalidOperation),
        };
        let account_index = self
            .resolve_account_param_by_name(account_name)
            .ok_or(VMError::InvalidOperation)?;

        let offset = match offset_arg {
            AstNode::Literal(Value::U8(v)) => *v as u32,
            AstNode::Literal(Value::U64(v)) => {
                u32::try_from(*v).map_err(|_| VMError::InvalidOperation)?
            }
            _ => return Err(VMError::InvalidOperation),
        };

        Ok((account_index, offset))
    }

    fn builtin_expected_arg_type(name: &str, arg_idx: usize) -> Option<TypeNode> {
        match (name, arg_idx) {
            ("sha256", 1) | ("keccak256", 1) | ("blake3", 1) => Some(TypeNode::Array {
                element_type: Box::new(TypeNode::Primitive("u8".to_string())),
                size: Some(32),
            }),
            ("verify_ed25519_instruction" | "__verify_ed25519_instruction", 3) => {
                Some(TypeNode::Array {
                    element_type: Box::new(TypeNode::Primitive("u8".to_string())),
                    size: Some(64),
                })
            }
            _ => None,
        }
    }

    fn builtin_allows_untyped_byte_literal(name: &str, arg_idx: usize) -> bool {
        matches!(
            (name, arg_idx),
            ("sha256", 0)
                | ("keccak256", 0)
                | ("blake3", 0)
                | (
                    "verify_ed25519_instruction" | "__verify_ed25519_instruction",
                    2
                )
        )
    }

    fn emit_untyped_byte_array_literal<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        arg: &AstNode,
    ) -> Result<bool, VMError> {
        let Some(bytes) = collect_byte_array_literal_bytes(arg) else {
            return Ok(false);
        };
        emitter.emit_const_bytes(&bytes)?;
        Ok(true)
    }

    fn emit_argument_with_expected_type<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        arg: &AstNode,
        expected_type: Option<&TypeNode>,
        context: &str,
    ) -> Result<(), VMError> {
        if !self.emit_typed_byte_array_literal(
            emitter,
            arg,
            fixed_u8_array_len(expected_type),
            context,
        )? {
            self.generate_ast_node(emitter, arg)?;
        }

        Ok(())
    }

    fn interface_param_has_attribute(param: &InstructionParameter, attr_name: &str) -> bool {
        param.attributes.iter().any(|attr| attr.name == attr_name)
    }

    fn current_function_param_by_name(&self, name: &str) -> Option<&InstructionParameter> {
        self.current_function_parameters
            .as_ref()
            .and_then(|params| params.iter().find(|param| param.name == name))
    }

    fn emit_pda_bump_value<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        authority_param: &InstructionParameter,
    ) -> Result<(), VMError> {
        let bump_alias = Self::init_ctx_bump_alias(&authority_param.name);
        if self.local_symbol_table.contains_key(&bump_alias)
            || self.global_symbol_table.contains_key(&bump_alias)
        {
            self.generate_ast_node(emitter, &AstNode::Identifier(bump_alias))?;
            return Ok(());
        }

        let pda_config = authority_param
            .pda_config
            .as_ref()
            .ok_or(VMError::InvalidScript)?;

        if let Some(bump_var) = &pda_config.bump {
            self.generate_ast_node(emitter, &AstNode::Identifier(bump_var.clone()))?;
            return Ok(());
        }

        for seed in &pda_config.seeds {
            self.generate_ast_node(emitter, seed)?;
        }
        emitter.emit_const_u8(pda_config.seeds.len() as u8)?;
        emitter.emit_opcode(PUSH_0);
        emitter.emit_opcode(FIND_PDA);
        emitter.emit_opcode(UNPACK_TUPLE);
        emitter.emit_opcode(SWAP);
        emitter.emit_opcode(DROP);
        Ok(())
    }

    fn emit_pda_signer_group<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        authority_param: &InstructionParameter,
    ) -> Result<(), VMError> {
        let pda_config = authority_param
            .pda_config
            .as_ref()
            .ok_or(VMError::InvalidScript)?;

        for seed in &pda_config.seeds {
            self.generate_ast_node(emitter, seed)?;
        }
        self.emit_pda_bump_value(emitter, authority_param)?;
        emitter.emit_opcode(PUSH_ARRAY_LITERAL);
        emitter.emit_u8((pda_config.seeds.len() + 1) as u8);
        Ok(())
    }

    fn collect_interface_signer_groups<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        interface_method: &InterfaceMethod,
        args: &[AstNode],
        emit: bool,
    ) -> Result<u8, VMError> {
        let mut signer_group_count = 0u8;

        for (param, arg) in interface_method.parameters.iter().zip(args.iter()) {
            if !Self::interface_param_has_attribute(param, "authority") {
                continue;
            }

            let AstNode::Identifier(authority_name) = arg else {
                return Err(VMError::InvalidOperation);
            };

            let authority_param = self
                .current_function_param_by_name(authority_name)
                .cloned()
                .ok_or(VMError::InvalidScript)?;

            if authority_param.pda_config.is_some() {
                if emit {
                    self.emit_pda_signer_group(emitter, &authority_param)?;
                }
                signer_group_count = signer_group_count
                    .checked_add(1)
                    .ok_or(VMError::InvalidOperation)?;
            } else if !Self::interface_param_has_attribute(&authority_param, "signer") {
                return Err(VMError::ConstraintViolation);
            }
        }

        Ok(signer_group_count)
    }

    fn try_emit_unqualified_external_call<T: OpcodeEmitter>(
        &self,
        emitter: &mut T,
        name: &str,
        arg_count: usize,
    ) -> Result<bool, VMError> {
        // Candidate set is deduplicated by account index because each external import is
        // currently registered under multiple keys (e.g. extN and identifier alias).
        let mut candidates: Vec<(String, u8, u16)> = Vec::new();
        for (module_name, ext_import) in self.external_imports.iter() {
            let selector = if let Some(sel) = ext_import.functions.get(name) {
                *sel
            } else if ext_import.allow_any_function {
                Self::external_selector(name)
            } else {
                continue;
            };

            if let Some(existing) = candidates
                .iter_mut()
                .find(|(_, acc_idx, _)| *acc_idx == ext_import.account_index)
            {
                // Prefer deterministic synthetic aliases (`extN`) when available
                // because they map directly to common runtime account parameter names.
                let preferred = format!("ext{}", ext_import.account_index);
                if module_name == &preferred {
                    existing.0 = module_name.clone();
                }
            } else {
                candidates.push((module_name.clone(), ext_import.account_index, selector));
            }
        }

        match candidates.len().cmp(&1) {
            Ordering::Equal => {
                let (module_name, default_account_index, selector) = &candidates[0];
                let account_index =
                    self.resolve_external_account_index(module_name, *default_account_index)?;
                emitter.emit_opcode(CALL_EXTERNAL);
                emitter.emit_u8(account_index);
                emitter.emit_u16(*selector);
                emitter.emit_u8(arg_count as u8);
                Ok(true)
            }
            Ordering::Greater => Err(VMError::InvalidScript),
            Ordering::Less => Ok(false),
        }
    }

    /// Generate method call bytecode
    pub(super) fn generate_method_call<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        method: &str,
        object: &AstNode,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        // Imported interface from `use "<account>"::{interface Name}`.
        // These calls execute callee bytecode via CALL_EXTERNAL rather than
        // synthesizing caller-side INVOKE payloads.
        if let AstNode::Identifier(import_name) = object {
            if let Some(ext_import) = self.external_imports.get(import_name) {
                let ext_import = ext_import.clone();
                // Generate arguments first (CALL_EXTERNAL pops from stack).
                for (arg_idx, arg) in args.iter().enumerate() {
                    self.emit_argument_with_expected_type(
                        emitter,
                        arg,
                        None,
                        &format!("call argument {} for `{}`", arg_idx, method),
                    )?;
                }

                let selector = if let Some(sel) = ext_import.functions.get(method) {
                    *sel
                } else if ext_import.allow_any_function {
                    Self::external_selector(method)
                } else {
                    return Err(VMError::InvalidScript);
                };

                let account_index = self.resolve_external_account_index_strict(import_name)?;
                emitter.emit_opcode(CALL_EXTERNAL);
                emitter.emit_u8(account_index);
                emitter.emit_u16(selector);
                emitter.emit_u8(args.len() as u8);
                return Ok(());
            }
        }

        // Check if this is an interface method call first
        if let AstNode::Identifier(interface_name) = object {
            if let Some(interface_info) = self.interface_registry.get(interface_name) {
                // This is an interface method call - generate INVOKE opcode
                if let Some(interface_method) = interface_info.methods.get(method) {
                    // Clone to avoid simultaneous borrow of self.interface_registry (immutable)
                    // and self (mutable) in emit_interface_invoke
                    let info = interface_info.clone();
                    let method_info = interface_method.clone();

                    return self.emit_interface_invoke(emitter, &info, &method_info, args);
                } else {
                    return Err(VMError::InvalidOperation); // Method not found in interface
                }
            }
        }

        // Generate object first (for non-interface method calls)
        self.generate_ast_node(emitter, object)?;

        // Generate arguments
        for arg in args {
            self.generate_ast_node(emitter, arg)?;
        }

        // Try built-in arithmetic/comparison methods first
        if self.try_emit_builtin_method(emitter, method).is_some() {
            return Ok(());
        }

        // Custom method call - use CALL opcode for function dispatch (legacy mode)
        // This ensures coordination with the simplified function dispatcher
        let param_count = (args.len() + 1) as u8; // +1 for the object itself

        // Standard CALL emission without metadata (VM does not support metadata bytes)
        emitter.emit_opcode(five_protocol::opcodes::CALL);
        emitter.emit_u8(param_count);

        // Record function patch for the u16 function address
        let patch_position = emitter.get_position();
        emitter.emit_u16(0x0000); // Placeholder offset

        self.record_function_patch_at_position(patch_position, method.to_string());

        Ok(())
    }

    /// Generate function call - produces CALL opcodes only for function dispatch
    pub(super) fn generate_function_call<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        name: &str,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        // Module-qualified interface call:
        //   alias::method(...)
        //   full::module::path::method(...)
        if let Some((module_name, method_name)) = Self::parse_qualified_name(name) {
            let interface_name = self
                .module_interface_aliases
                .get(module_name)
                .cloned()
                .or_else(|| {
                    module_name
                        .rsplit("::")
                        .next()
                        .and_then(|last| self.module_interface_aliases.get(last).cloned())
                });

            if let Some(interface_name) = interface_name {
                if let Some(interface_info) = self.interface_registry.get(&interface_name) {
                    if let Some(interface_method) = interface_info.methods.get(method_name) {
                        let info = interface_info.clone();
                        let method_info = interface_method.clone();
                        return self.emit_interface_invoke(emitter, &info, &method_info, args);
                    }
                }
                return Err(VMError::InvalidOperation);
            }
        }

        // Most built-ins consume pre-generated arguments.
        // A few have custom argument lowering and must not pre-generate here.
        let has_custom_arg_lowering = matches!(
            name,
            "derive_pda"
                | "invoke_signed"
                | "transfer_lamports"
                | "close_account"
                | "load_account_u64"
                | "load_account_u64_word"
        );

        if !has_custom_arg_lowering {
            let user_defined_param_types = self.function_parameter_types.get(name).cloned();
            for (arg_idx, arg) in args.iter().enumerate() {
                let builtin_expected_type = Self::builtin_expected_arg_type(name, arg_idx);
                let user_expected_type = user_defined_param_types
                    .as_ref()
                    .and_then(|types| types.get(arg_idx))
                    .cloned();
                let expected_type = builtin_expected_type.or(user_expected_type);

                if !(Self::builtin_allows_untyped_byte_literal(name, arg_idx)
                    && self.emit_untyped_byte_array_literal(emitter, arg)?)
                {
                    self.emit_argument_with_expected_type(
                        emitter,
                        arg,
                        expected_type.as_ref(),
                        &format!("call argument {} for `{}`", arg_idx, name),
                    )?;
                }
            }
        }

        // Handle built-in functions (these don't use function dispatch)
        match name {
            "require" => {
                emitter.emit_opcode(REQUIRE);
            }
            "get_clock" => {
                emitter.emit_opcode(GET_CLOCK);
            }
            "load_account_u64" | "load_account_u64_word" => {
                if args.len() != 2 {
                    return Err(VMError::InvalidParameterCount);
                }
                let (account_index, offset) =
                    self.resolve_account_literal_offset(&args[0], &args[1])?;
                emitter.emit_opcode(LOAD_EXTERNAL_FIELD);
                emitter.emit_u8(account_index);
                emitter.emit_u32(offset);
            }
            "string_length" => {
                if args.len() != 1 {
                    return Err(VMError::InvalidParameterCount);
                }
                emitter.emit_opcode(ARRAY_LENGTH);
            }
            "string_concat" => {
                if args.len() != 2 {
                    return Err(VMError::InvalidParameterCount);
                }
                emitter.emit_opcode(ARRAY_CONCAT);
            }
            "bytes_concat" => {
                if args.len() != 2 {
                    return Err(VMError::InvalidParameterCount);
                }
                emitter.emit_opcode(ARRAY_CONCAT);
            }
            "Some" => {
                // Option<T> constructor: Some(value) - wraps value in Some variant
                // The argument should already be on the stack from above
                if args.len() != 1 {
                    return Err(VMError::InvalidParameterCount);
                }
                emitter.emit_opcode(OPTIONAL_SOME);
            }
            "Ok" => {
                // Result<T,E> success constructor: Ok(value) - wraps value in Ok variant
                // The argument should already be on the stack from above
                if args.len() != 1 {
                    return Err(VMError::InvalidParameterCount);
                }
                emitter.emit_opcode(RESULT_OK);
            }
            "Err" => {
                // Result<T,E> error constructor: Err(error) - wraps error in Err variant
                // The argument should already be on the stack from above
                if args.len() != 1 {
                    return Err(VMError::InvalidParameterCount);
                }
                emitter.emit_opcode(RESULT_ERR);
            }
            "derive_pda" => {
                if args.is_empty() {
                    return Err(VMError::InvalidParameterCount);
                }

                // Validation mode: derive_pda(seed1, seed2, ..., bump: u8) -> pubkey
                // Find mode:       derive_pda(seed1, seed2, ...)          -> (pubkey, u8)
                // This must align with type_checker::expressions::infer_function_call_type.
                let returns_pubkey_only = if args.len() >= 2 {
                    let last = &args[args.len() - 1];
                    match last {
                        AstNode::Literal(Value::U8(_)) => true,
                        AstNode::Cast { target_type, .. } => {
                            matches!(target_type.as_ref(), AstNode::Identifier(t) if t == "u8")
                        }
                        AstNode::Identifier(name) => self
                            .local_symbol_table
                            .get(name)
                            .map(|f| f.field_type == "u8")
                            .or_else(|| {
                                self.global_symbol_table
                                    .get(name)
                                    .map(|f| f.field_type == "u8")
                            })
                            .unwrap_or(false),
                        _ => self
                            .infer_type_from_node(last)
                            .map(|t| t == "u8")
                            .unwrap_or(false),
                    }
                } else {
                    false
                };

                // Generate all seed arguments first (they will be on stack)
                for arg in args {
                    self.generate_ast_node(emitter, arg)?;
                }

                // Push seeds count onto the stack as a value
                // VM handler expects to pop seeds_count from the stack (not as a raw bytecode byte)
                let seeds_count = args.len() as u8;
                emitter.emit_const_u8(seeds_count)?;

                // Push Five VM program ID as current program (32 bytes)
                // Five VM Program ID constant (matches five-vm-mito::FIVE_VM_PROGRAM_ID)
                let five_vm_program_id = [
                    0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                    0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a,
                    0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
                ];
                emitter.emit_const_pubkey(&five_vm_program_id)?;

                // Invoke PDA operation (handler pops: program_id, seeds_count, then each seed)
                // - validation mode (explicit bump): DERIVE_PDA
                // - find mode (no bump): FIND_PDA
                if returns_pubkey_only {
                    emitter.emit_opcode(DERIVE_PDA);
                } else {
                    emitter.emit_opcode(FIND_PDA);
                }

                // DERIVE_PDA currently returns a tuple (pubkey, bump).
                // In validation mode we expose only the pubkey to match language typing.
                if returns_pubkey_only {
                    emitter.emit_opcode(UNPACK_TUPLE);
                    // Stack after UNPACK_TUPLE: [pubkey, bump] (bump on top)
                    // Drop bump so only pubkey remains.
                    emitter.emit_opcode(DROP);
                }
            }
            "invoke_signed" => {
                // New logic for handling invoke_signed
                // The arguments on the stack should be: [program_id, instruction_data, accounts_count, seeds_count, seed1_len, seed1_data, ...]
                self.generate_invoke_signed(emitter, args)?;
            }
            "transfer_lamports" => {
                // transfer_lamports(from: account, to: account, amount: u64)
                // TRANSFER pops: amount, to_idx, from_idx, so emit in that order.
                if args.len() != 3 {
                    return Err(VMError::InvalidParameterCount);
                }
                let from_idx = self.resolve_account_argument(&args[0])?;
                let to_idx = self.resolve_account_argument(&args[1])?;
                emitter.emit_const_u8(from_idx)?;
                emitter.emit_const_u8(to_idx)?;
                self.generate_ast_node(emitter, &args[2])?;
                emitter.emit_opcode(TRANSFER);
            }
            "close_account" => {
                // close_account(source: account, destination: account)
                if args.len() != 2 {
                    return Err(VMError::InvalidParameterCount);
                }
                let source_idx = self.resolve_account_argument(&args[0])?;
                let destination_idx = self.resolve_account_argument(&args[1])?;
                emitter.emit_const_u8(source_idx)?;
                emitter.emit_const_u8(destination_idx)?;
                emitter.emit_opcode(CLOSE_ACCOUNT);
            }
            "pubkey" => {
                // Compatibility constructor: pubkey(x).
                // Arguments were already generated onto the stack above.
                // Keep the argument value as-is (identity). Type checker enforces valid forms.
                if args.len() != 1 {
                    return Err(VMError::InvalidParameterCount);
                }
            }

            // ===== NATIVE SYSCALL FUNCTIONS =====
            // These functions provide direct access to Solana/Pinocchio syscalls via CALL_NATIVE.
            // Each function maps to a specific syscall ID and includes parameter validation.
            // All syscalls maintain Five VM's zero-allocation execution model.
            // Control syscalls - program termination and error handling
            // abort() -> never (~50 CU) - Immediately terminates execution
            "abort" => emit_syscall!(emitter, args, 1, args empty),

            // panic(message?: string) -> never (~50-100 CU) - Terminates with optional message
            "panic" => emit_syscall!(emitter, args, 2, args max 1),

            // PDA/Address syscalls - program-derived address generation
            // create_program_address(seeds, program_id) -> Result<pubkey, error> (~1,500 CU)
            "create_program_address" => emit_syscall!(emitter, args, 10, args = 2),

            // try_find_program_address(seeds, program_id) -> (Result<pubkey, error>, u8) (~2,000-3,000 CU)
            "try_find_program_address" => emit_syscall!(emitter, args, 11, args = 2),

            // Sysvar syscalls - access to blockchain state variables
            // get_clock_sysvar() -> Clock (~200 CU)
            "get_clock_sysvar" => emit_syscall!(emitter, args, 20, args empty),

            // get_epoch_schedule_sysvar() -> EpochSchedule (~200 CU)
            "get_epoch_schedule_sysvar" => emit_syscall!(emitter, args, 21, args empty),

            // get_rent_sysvar() -> Rent (~200 CU)
            "get_rent_sysvar" => emit_syscall!(emitter, args, 25, args empty),

            // Program data syscalls
            "get_return_data" => emit_syscall!(emitter, args, 30, args empty),
            "set_return_data" => emit_syscall!(emitter, args, 31, args = 1),
            "remaining_compute_units" => emit_syscall!(emitter, args, 50, args empty),

            // Logging syscalls
            "log" => emit_syscall!(emitter, args, 60, args = 1),
            "log_64" => emit_syscall!(emitter, args, 61, args max 5),
            "log_compute_units" => emit_syscall!(emitter, args, 62, args empty),
            "log_data" => emit_syscall!(emitter, args, 63, args = 1),
            "log_pubkey" => emit_syscall!(emitter, args, 64, args = 1),

            // Memory syscalls
            "memcpy" => emit_syscall!(emitter, args, 70, args = 3),
            "memcmp" => emit_syscall!(emitter, args, 73, args = 3),

            // Cryptography syscalls
            "sha256" => emit_syscall!(emitter, args, 80, args = 2),
            "keccak256" => emit_syscall!(emitter, args, 81, args = 2),
            "blake3" => emit_syscall!(emitter, args, 82, args = 2),
            "secp256k1_recover" => emit_syscall!(emitter, args, 84, args = 4),
            "verify_ed25519_instruction" | "__verify_ed25519_instruction" => {
                emit_syscall!(emitter, args, 92, args = 4)
            }

            _ => {
                // Check for qualified function names like "math_lib::add"
                // If the module is registered as external, emit CALL_EXTERNAL instead of CALL
                if let Some((module_name, func_name)) = Self::parse_qualified_name(name) {
                    if let Some(ext_import) = self.external_imports.get(module_name) {
                        let selector = if let Some(sel) = ext_import.functions.get(func_name) {
                            *sel
                        } else if ext_import.allow_any_function {
                            Self::external_selector(func_name)
                        } else {
                            return Err(VMError::InvalidScript);
                        };

                        let account_index = self.resolve_external_account_index(
                            module_name,
                            ext_import.account_index,
                        )?;

                        // Found external import - emit CALL_EXTERNAL opcode.
                        // Keep selector as raw u16 for protocol/runtime compatibility.
                        // CALL_EXTERNAL format: opcode(1) + account_index(1) + selector(u16) + param_count(1)
                        emitter.emit_opcode(CALL_EXTERNAL);
                        emitter.emit_u8(account_index);
                        emitter.emit_u16(selector);
                        emitter.emit_u8(args.len() as u8);
                        return Ok(());
                    }
                }

                // Unqualified imported function call.
                // Example:
                //   use "<address>"::{transfer};
                //   transfer(...)
                if self.try_emit_unqualified_external_call(emitter, name, args.len())? {
                    return Ok(());
                }

                // User-defined function call - always use CALL opcode
                // Track function call for resource allocation
                self.track_function_call();

                // Track parameters for stack resource calculation
                self.track_function_parameters(args.len() as u8);

                // Legacy mode: emit CALL with direct function offset
                // The function offset will be resolved by the function dispatcher
                // during the compilation process when function offsets are known
                let param_count = args.len() as u8;

                // Standard CALL emission without metadata (VM does not support metadata bytes)
                emitter.emit_opcode(five_protocol::opcodes::CALL);
                emitter.emit_u8(param_count);

                // Record function patch for the u16 function address
                let patch_position = emitter.get_position();
                emitter.emit_u16(0x0000); // Placeholder offset

                self.record_function_patch_at_position(patch_position, name.to_string());

                // Track return from function call (for proper call depth management)
                self.track_function_return();
            }
        }

        Ok(())
    }

    fn resolve_external_account_index(
        &self,
        module_name: &str,
        default_index: u8,
    ) -> Result<u8, VMError> {
        let params = self
            .current_function_parameters
            .as_ref()
            .ok_or(VMError::InvalidScript)?;

        let mut account_params: Vec<&InstructionParameter> = Vec::new();
        let registry = self
            .account_system
            .as_ref()
            .map(|s| s.get_account_registry());
        for p in params {
            if super::super::account_utils::is_account_parameter(
                &p.param_type,
                &p.attributes,
                registry,
            ) {
                account_params.push(p);
            }
        }

        if account_params.is_empty() {
            return Err(VMError::InvalidScript);
        }

        if let Some((idx, _)) = account_params
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == module_name)
        {
            return Ok(account_index_from_param_index(idx as u8));
        }

        // Devex path: allow descriptive names like `token_bytecode` instead of synthetic
        // `extN` parameter names for external import bindings.
        let bytecode_param_positions: Vec<usize> = account_params
            .iter()
            .enumerate()
            .filter_map(|(idx, p)| {
                let name = p.name.as_str();
                if name.ends_with("_bytecode") || name.contains("bytecode") {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();

        if let Some((idx, _)) = account_params
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == "token_bytecode")
        {
            return Ok(account_index_from_param_index(idx as u8));
        }

        if !bytecode_param_positions.is_empty() {
            let selected = if (default_index as usize) < bytecode_param_positions.len() {
                bytecode_param_positions[default_index as usize]
            } else {
                bytecode_param_positions[0]
            };
            return Ok(account_index_from_param_index(selected as u8));
        }

        if (default_index as usize) < account_params.len() {
            Ok(account_index_from_param_index(default_index))
        } else {
            Err(VMError::InvalidScript)
        }
    }

    fn resolve_external_account_index_strict(&self, module_name: &str) -> Result<u8, VMError> {
        let params = self
            .current_function_parameters
            .as_ref()
            .ok_or(VMError::InvalidScript)?;

        let mut account_params: Vec<&InstructionParameter> = Vec::new();
        let registry = self
            .account_system
            .as_ref()
            .map(|s| s.get_account_registry());
        for p in params {
            if super::super::account_utils::is_account_parameter(
                &p.param_type,
                &p.attributes,
                registry,
            ) {
                account_params.push(p);
            }
        }

        if let Some((idx, _)) = account_params
            .iter()
            .enumerate()
            .find(|(_, p)| p.name == module_name)
        {
            return Ok(account_index_from_param_index(idx as u8));
        }

        // Imported interface execution requires explicit account binding.
        Err(VMError::InvalidScript)
    }

    /// Generate bytecode for the invoke_signed function
    pub(super) fn generate_invoke_signed<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        if args.len() != 4 {
            return Err(VMError::InvalidParameterCount);
        }

        // 1. program_id (must be a Pubkey)
        self.generate_ast_node(emitter, &args[0])?;

        // 2. instruction_data (must be a byte array)
        self.generate_byte_array(emitter, &args[1])?;

        // 3. accounts (must be an array of AccountMeta)
        self.generate_array(emitter, &args[2])?;

        // 4. seeds (must be an array of byte arrays)
        self.generate_array(emitter, &args[3])?;

        emitter.emit_opcode(INVOKE_SIGNED);
        Ok(())
    }

    /// Emit INVOKE for interface calls with correct account/data partitioning.
    /// Stack contract (bottom to top):
    ///   program_id → instruction_data → account_indices[] → accounts_count → INVOKE
    fn emit_interface_invoke<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        interface_info: &InterfaceInfo,
        interface_method: &InterfaceMethod,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        // Step 1: Partition arguments into account indices and data argument indices
        let (account_indices, data_arg_indices) =
            self.partition_interface_arguments(interface_method, args)?;

        // Step 2: Determine whether this is a plain INVOKE or automatic signed CPI.
        let signer_group_count =
            self.collect_interface_signer_groups(emitter, interface_method, args, false)?;

        if signer_group_count == 0 {
            // Plain INVOKE stack contract (bottom -> top):
            //   program_id, instruction_data, account_indices..., accounts_count
            let program_id_bytes = self.parse_program_id(&interface_info.program_id)?;
            emitter.emit_const_pubkey(&program_id_bytes)?;
            self.emit_instruction_data_construction(
                emitter,
                interface_method,
                &data_arg_indices,
                args,
            )?;
            for &account_idx in account_indices.iter().rev() {
                emitter.emit_const_u8(account_idx)?;
            }
            emitter.emit_const_u8(account_indices.len() as u8)?;
            emitter.emit_opcode(INVOKE);
        } else {
            // Grouped INVOKE_SIGNED stack contract (bottom -> top):
            //   account_indices..., program_id, instruction_data, accounts_count, signer_groups
            //
            // The runtime pops signer_groups/count/data/program_id first and then consumes the
            // remaining account indices, so the indices must stay below the call payload.
            for &account_idx in &account_indices {
                emitter.emit_const_u8(account_idx)?;
            }
            let program_id_bytes = self.parse_program_id(&interface_info.program_id)?;
            emitter.emit_const_pubkey(&program_id_bytes)?;
            self.emit_instruction_data_construction(
                emitter,
                interface_method,
                &data_arg_indices,
                args,
            )?;
            emitter.emit_const_u8(account_indices.len() as u8)?;
            self.collect_interface_signer_groups(emitter, interface_method, args, true)?;
            emitter.emit_opcode(PUSH_ARRAY_LITERAL);
            emitter.emit_u8(signer_group_count);
            emitter.emit_opcode(INVOKE_SIGNED);
        }

        Ok(())
    }

    /// Check if a TypeNode represents an account-meta parameter.
    /// Only explicit `Account` parameters are emitted as account metas.
    /// `pubkey` parameters are serialized into instruction data.
    fn is_account_meta_type(type_node: &TypeNode) -> bool {
        matches!(type_node, TypeNode::Account)
            || matches!(type_node, TypeNode::Named(name) if name.eq_ignore_ascii_case("account"))
    }

    /// Check if a field_type string represents an account type
    fn is_account_type_str(field_type: &str) -> bool {
        field_type == "Account" || field_type == "account" || field_type.starts_with("Account<")
    }

    /// Resolve an account argument to its parameter index.
    /// Account arguments must be simple identifiers that resolve to function parameters
    /// of Account type. Returns the parameter index if valid.
    fn resolve_account_argument(&self, arg: &AstNode) -> Result<u8, VMError> {
        match arg {
            AstNode::Identifier(name) => {
                if let Some(param_idx) = self.resolve_account_param_by_name(name) {
                    return Ok(param_idx);
                }

                // Look up in local symbol table (function parameters)
                if let Some(field_info) = self.local_symbol_table.get(name) {
                    // Validate it's an account type
                    if !Self::is_account_type_str(&field_info.field_type) {
                        return Err(VMError::TypeMismatch);
                    }

                    Ok(
                        super::super::account_utils::account_index_from_param_offset(
                            field_info.offset,
                        ),
                    )
                } else {
                    Err(VMError::InvalidScript) // Undefined identifier
                }
            }
            AstNode::FieldAccess { object, field } => {
                // Handle account.key access - resolves to the account index
                if field == "key" {
                    if let AstNode::Identifier(_name) = object.as_ref() {
                        return self.resolve_account_argument(object);
                    }
                }
                Err(VMError::InvalidOperation)
            }
            _ => {
                // Only identifiers are allowed as account arguments
                Err(VMError::InvalidOperation) // Complex expressions not allowed for accounts
            }
        }
    }

    pub(super) fn resolve_account_param_by_name(&self, name: &str) -> Option<u8> {
        let params = self.current_function_parameters.as_ref()?;
        let registry = self
            .account_system
            .as_ref()
            .map(|s| s.get_account_registry());

        for (idx, param) in params.iter().enumerate() {
            if param.name != name {
                continue;
            }
            if super::super::account_utils::is_account_parameter(
                &param.param_type,
                &param.attributes,
                registry,
            ) {
                return Some(account_index_from_param_index(idx as u8));
            }
        }
        None
    }

    /// Partition interface method arguments into account indices and data arguments.
    /// Account parameters (Account type) have their indices extracted.
    /// Data parameters are collected for serialization.
    /// Returns (account_indices, data_args).
    fn partition_interface_arguments(
        &self,
        interface_method: &InterfaceMethod,
        args: &[AstNode],
    ) -> Result<(Vec<u8>, Vec<usize>), VMError> {
        // Validate argument count matches parameter count
        if interface_method.parameters.len() != args.len() {
            return Err(VMError::InvalidParameterCount);
        }

        let mut account_indices = Vec::new();
        let mut data_arg_indices = Vec::new();

        // Iterate through parameters and corresponding arguments
        for (idx, (param, arg)) in interface_method
            .parameters
            .iter()
            .zip(args.iter())
            .enumerate()
        {
            if Self::is_account_meta_type(&param.param_type) {
                // This is an account parameter - resolve to index
                let param_idx = self.resolve_account_argument(arg)?;
                account_indices.push(param_idx);
            } else {
                // This is a data parameter - collect index
                data_arg_indices.push(idx);
            }
        }

        Ok((account_indices, data_arg_indices))
    }

    /// Emit opcodes to construct instruction data values on the stack.
    /// Supports both literals and variables by emitting values dynamically.
    fn emit_instruction_data_construction<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        interface_method: &InterfaceMethod,
        data_arg_indices: &[usize],
        args: &[AstNode],
    ) -> Result<(), VMError> {
        let discriminator_bytes = interface_method
            .discriminator_bytes
            .clone()
            .unwrap_or_else(|| vec![interface_method.discriminator]);
        emitter.emit_const_bytes(&discriminator_bytes)?;

        // Append each argument as a serialized byte chunk.
        for &arg_idx in data_arg_indices {
            let param_type = if let Some(param) = interface_method.parameters.get(arg_idx) {
                &param.param_type
            } else {
                return Err(VMError::InvalidParameterCount);
            };

            let arg = &args[arg_idx];
            self.emit_argument_serialization(emitter, param_type, arg)?;
            emitter.emit_opcode(ARRAY_CONCAT);
        }

        Ok(())
    }

    /// Emit a single serialized argument chunk as a bytes-like value on the stack.
    /// The caller is responsible for concatenating it into the instruction payload.
    fn emit_argument_serialization<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        param_type: &TypeNode,
        arg: &AstNode,
    ) -> Result<(), VMError> {
        match param_type {
            TypeNode::Array { element_type, .. } if matches!(element_type.as_ref(), TypeNode::Primitive(name) if name == "u8") =>
            {
                if !self.emit_typed_byte_array_literal(
                    emitter,
                    arg,
                    fixed_u8_array_len(Some(param_type)),
                    "interface instruction data argument",
                )? {
                    self.generate_ast_node(emitter, arg)?;
                }
                Ok(())
            }
            TypeNode::Primitive(name) if name == "u8" => {
                if let AstNode::Literal(val) = arg {
                    let byte = val
                        .as_u8()
                        .or_else(|| {
                            val.as_u64()
                                .filter(|v| *v <= u8::MAX as u64)
                                .map(|v| v as u8)
                        })
                        .or_else(|| {
                            val.as_i64()
                                .filter(|v| (0..=u8::MAX as i64).contains(v))
                                .map(|v| v as u8)
                        })
                        .ok_or(VMError::TypeMismatch)?;
                    emitter.emit_const_bytes(&[byte])?;
                } else {
                    self.generate_ast_node(emitter, arg)?;
                    emitter.emit_opcode(PUSH_ARRAY_LITERAL);
                    emitter.emit_u8(1);
                }
                Ok(())
            }
            TypeNode::Primitive(name) if name == "u32" => {
                if let AstNode::Literal(val) = arg {
                    let word = val
                        .as_u64()
                        .or_else(|| val.as_i64().filter(|v| *v >= 0).map(|v| v as u64))
                        .filter(|v| *v <= u32::MAX as u64)
                        .ok_or(VMError::TypeMismatch)? as u32;
                    emitter.emit_const_bytes(&word.to_le_bytes())?;
                } else {
                    self.generate_ast_node(emitter, arg)?;
                    emitter.emit_opcode(PUSH_ARRAY_LITERAL);
                    emitter.emit_u8(1);
                }
                Ok(())
            }
            TypeNode::Primitive(name)
                if name == "u64" || name == "i64" || name == "bool" || name == "pubkey" =>
            {
                if let AstNode::Literal(Value::Pubkey(pk)) = arg {
                    emitter.emit_const_bytes(pk)?;
                } else if let (TypeNode::Primitive(name), AstNode::Literal(val)) = (param_type, arg)
                {
                    if name == "u64" {
                        let word = val
                            .as_u64()
                            .or_else(|| val.as_i64().filter(|v| *v >= 0).map(|v| v as u64))
                            .ok_or(VMError::TypeMismatch)?;
                        emitter.emit_const_bytes(&word.to_le_bytes())?;
                    } else if name == "i64" {
                        let word = val.as_i64().ok_or(VMError::TypeMismatch)?;
                        emitter.emit_const_bytes(&word.to_le_bytes())?;
                    } else if name == "bool" {
                        let flag = matches!(val, Value::Bool(true));
                        emitter.emit_const_bytes(&[u8::from(flag)])?;
                    } else {
                        self.generate_ast_node(emitter, arg)?;
                        emitter.emit_opcode(PUSH_ARRAY_LITERAL);
                        emitter.emit_u8(1);
                    }
                } else {
                    self.generate_ast_node(emitter, arg)?;
                    emitter.emit_opcode(PUSH_ARRAY_LITERAL);
                    emitter.emit_u8(1);
                }
                Ok(())
            }
            _ => {
                eprintln!("DEBUG: unsupported CPI serialization for {:?}", param_type);
                Err(VMError::TypeMismatch)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Attribute, InstructionParameter, TypeNode};
    use crate::bytecode_generator::OpcodeEmitter;
    use std::collections::HashMap;

    struct MockEmitter {
        bytes: Vec<u8>,
    }

    impl MockEmitter {
        fn new() -> Self {
            Self { bytes: Vec::new() }
        }
    }

    impl OpcodeEmitter for MockEmitter {
        fn emit_opcode(&mut self, opcode: u8) {
            self.bytes.push(opcode);
        }
        fn emit_u8(&mut self, value: u8) {
            self.bytes.push(value);
        }
        fn emit_u16(&mut self, value: u16) {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn emit_u32(&mut self, value: u32) {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn emit_u64(&mut self, value: u64) {
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
        fn emit_bytes(&mut self, bytes: &[u8]) {
            self.bytes.extend_from_slice(bytes);
        }
        fn get_position(&self) -> usize {
            self.bytes.len()
        }
        fn patch_u32(&mut self, position: usize, value: u32) {
            self.bytes[position..position + 4].copy_from_slice(&value.to_le_bytes());
        }
        fn patch_u16(&mut self, position: usize, value: u16) {
            self.bytes[position..position + 2].copy_from_slice(&value.to_le_bytes());
        }
        fn should_include_tests(&self) -> bool {
            false
        }
        fn emit_const_u8(&mut self, value: u8) -> Result<(), VMError> {
            self.emit_u8(value);
            Ok(())
        }
        fn emit_const_u16(&mut self, value: u16) -> Result<(), VMError> {
            self.emit_u16(value);
            Ok(())
        }
        fn emit_const_u32(&mut self, value: u32) -> Result<(), VMError> {
            self.emit_u32(value);
            Ok(())
        }
        fn emit_const_u64(&mut self, value: u64) -> Result<(), VMError> {
            self.emit_u64(value);
            Ok(())
        }
        fn emit_const_i64(&mut self, value: i64) -> Result<(), VMError> {
            self.emit_u64(value as u64);
            Ok(())
        }
        fn emit_const_bool(&mut self, value: bool) -> Result<(), VMError> {
            self.emit_u8(u8::from(value));
            Ok(())
        }
        fn emit_const_u128(&mut self, value: u128) -> Result<(), VMError> {
            self.emit_bytes(&value.to_le_bytes());
            Ok(())
        }
        fn emit_const_pubkey(&mut self, value: &[u8; 32]) -> Result<(), VMError> {
            self.emit_bytes(value);
            Ok(())
        }
        fn emit_const_string(&mut self, value: &[u8]) -> Result<(), VMError> {
            self.emit_bytes(value);
            Ok(())
        }
        fn emit_const_bytes(&mut self, value: &[u8]) -> Result<(), VMError> {
            self.emit_opcode(PUSH_BYTES);
            self.emit_u8(value.len() as u8);
            self.emit_bytes(value);
            Ok(())
        }
        fn intern_u16_const(&mut self, _value: u16) -> Result<u16, VMError> {
            Ok(0)
        }
    }

    #[test]
    fn emits_call_external_for_qualified_external_module() {
        let mut gen = ASTGenerator::new();
        gen.current_function_parameters = Some(vec![InstructionParameter {
            name: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            param_type: TypeNode::Primitive("Account".to_string()),
            is_optional: false,
            default_value: None,
            attributes: vec![Attribute {
                name: "mut".to_string(),
                args: vec![],
            }],
            is_init: false,
            init_config: None,
            pda_config: None,
        }]);
        gen.register_external_import(
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
            0,
            true,
            HashMap::new(),
        );

        let mut emitter = MockEmitter::new();
        gen.generate_function_call(
            &mut emitter,
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA::transfer",
            &[],
        )
        .expect("call generation should succeed");

        assert_eq!(emitter.bytes[0], CALL_EXTERNAL);
        assert_eq!(emitter.bytes[1], 1); // account index with runtime offset
        assert_eq!(emitter.bytes[4], 0); // param count
    }

    #[test]
    fn emits_call_external_for_unqualified_imported_function() {
        let mut gen = ASTGenerator::new();
        gen.current_function_parameters = Some(vec![InstructionParameter {
            name: "any_account".to_string(),
            param_type: TypeNode::Primitive("Account".to_string()),
            is_optional: false,
            default_value: None,
            attributes: vec![Attribute {
                name: "mut".to_string(),
                args: vec![],
            }],
            is_init: false,
            init_config: None,
            pda_config: None,
        }]);
        let mut funcs = HashMap::new();
        funcs.insert(
            "transfer".to_string(),
            ASTGenerator::external_selector("transfer"),
        );
        gen.register_external_import("ext0".to_string(), 0, false, funcs);

        let mut emitter = MockEmitter::new();
        gen.generate_function_call(&mut emitter, "transfer", &[])
            .expect("unqualified imported call should succeed");

        assert_eq!(emitter.bytes[0], CALL_EXTERNAL);
        assert_eq!(emitter.bytes[1], 1);
        assert_eq!(emitter.bytes[4], 0);
    }

    #[test]
    fn rejects_ambiguous_unqualified_external_call() {
        let mut gen = ASTGenerator::new();
        let mut funcs = HashMap::new();
        funcs.insert(
            "transfer".to_string(),
            ASTGenerator::external_selector("transfer"),
        );
        gen.register_external_import("ext0".to_string(), 0, false, funcs.clone());
        gen.register_external_import("ext1".to_string(), 1, false, funcs);

        let mut emitter = MockEmitter::new();
        let result = gen.generate_function_call(&mut emitter, "transfer", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn resolves_external_account_from_token_bytecode_param_name() {
        let mut gen = ASTGenerator::new();
        gen.current_function_parameters = Some(vec![
            InstructionParameter {
                name: "source".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![Attribute {
                    name: "mut".to_string(),
                    args: vec![],
                }],
                is_init: false,
                init_config: None,
                pda_config: None,
            },
            InstructionParameter {
                name: "destination".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![Attribute {
                    name: "mut".to_string(),
                    args: vec![],
                }],
                is_init: false,
                init_config: None,
                pda_config: None,
            },
            InstructionParameter {
                name: "owner".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![Attribute {
                    name: "signer".to_string(),
                    args: vec![],
                }],
                is_init: false,
                init_config: None,
                pda_config: None,
            },
            InstructionParameter {
                name: "token_bytecode".to_string(),
                param_type: TypeNode::Primitive("Account".to_string()),
                is_optional: false,
                default_value: None,
                attributes: vec![],
                is_init: false,
                init_config: None,
                pda_config: None,
            },
        ]);

        let mut funcs = HashMap::new();
        funcs.insert(
            "transfer".to_string(),
            ASTGenerator::external_selector("transfer"),
        );
        gen.register_external_import("ext0".to_string(), 0, false, funcs);

        let mut emitter = MockEmitter::new();
        gen.generate_function_call(&mut emitter, "transfer", &[])
            .expect("call generation should succeed");

        assert_eq!(emitter.bytes[0], CALL_EXTERNAL);
        assert_eq!(emitter.bytes[1], 4); // token_bytecode parameter is the 4th account argument
    }

    #[test]
    fn interface_fixed_byte_array_argument_uses_push_bytes_and_concat() {
        let mut gen = ASTGenerator::new();
        gen.current_function_parameters = Some(vec![InstructionParameter {
            name: "authority".to_string(),
            param_type: TypeNode::Account,
            is_optional: false,
            default_value: None,
            attributes: vec![Attribute {
                name: "signer".to_string(),
                args: vec![],
            }],
            is_init: false,
            init_config: None,
            pda_config: None,
        }]);

        let method = InterfaceMethod {
            discriminator: 9,
            discriminator_bytes: None,
            is_anchor: false,
            parameters: vec![
                InstructionParameter {
                    name: "authority".to_string(),
                    param_type: TypeNode::Account,
                    is_optional: false,
                    default_value: None,
                    attributes: vec![],
                    is_init: false,
                    init_config: None,
                    pda_config: None,
                },
                InstructionParameter {
                    name: "payload".to_string(),
                    param_type: TypeNode::Array {
                        element_type: Box::new(TypeNode::Primitive("u8".to_string())),
                        size: Some(64),
                    },
                    is_optional: false,
                    default_value: None,
                    attributes: vec![],
                    is_init: false,
                    init_config: None,
                    pda_config: None,
                },
            ],
            return_type: None,
        };
        let interface = InterfaceInfo {
            program_id: "11111111111111111111111111111111".to_string(),
            serializer: crate::type_checker::InterfaceSerializer::Raw,
            is_anchor: false,
            methods: HashMap::new(),
        };
        let args = vec![
            AstNode::Identifier("authority".to_string()),
            AstNode::ArrayLiteral {
                elements: (0..64).map(|i| AstNode::Literal(Value::U64(i))).collect(),
            },
        ];

        let mut emitter = MockEmitter::new();
        gen.emit_interface_invoke(&mut emitter, &interface, &method, &args)
            .expect("interface invoke generation should succeed");

        assert!(emitter.bytes.contains(&PUSH_BYTES));
        assert!(emitter.bytes.contains(&ARRAY_CONCAT));
        assert!(emitter.bytes.contains(&INVOKE));
        assert!(!emitter.bytes.contains(&PUSH_ARRAY_LITERAL));
    }
}
