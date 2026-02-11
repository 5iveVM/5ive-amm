//! Function and method call generation.

use super::super::OpcodeEmitter;
use super::types::ASTGenerator;
use crate::ast::{AstNode, TypeNode};
use crate::type_checker::{InterfaceInfo, InterfaceMethod};
use five_protocol::opcodes::*;
use five_protocol::Value;
use five_vm_mito::error::VMError;

impl ASTGenerator {
    /// Generate method call bytecode
    pub(super) fn generate_method_call<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        method: &str,
        object: &AstNode,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        println!("DEBUG: generate_method_call method='{}'", method);
        // Check if this is an interface method call first
        if let AstNode::Identifier(interface_name) = object {
            if let Some(interface_info) = self.interface_registry.get(interface_name) {
                // This is an interface method call - generate INVOKE opcode
                if let Some(interface_method) = interface_info.methods.get(method) {
                    // Clone to avoid simultaneous borrow of self.interface_registry (immutable) 
                    // and self (mutable) in emit_interface_invoke
                    let info = interface_info.clone();
                    let method_info = interface_method.clone();
                    
                    return self.emit_interface_invoke(
                        emitter,
                        &info,
                        &method_info,
                        args,
                    );
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
        
        self.record_function_patch_at_position(
            patch_position,
            method.to_string(),
        );

        Ok(())
    }

    /// Generate function call - produces CALL opcodes only for function dispatch
    pub(super) fn generate_function_call<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        name: &str,
        args: &[AstNode],
    ) -> Result<(), VMError> {
        println!("DEBUG: generate_function_call name='{}'", name);
        // Generate arguments first (they will be consumed by the function)
        for arg in args {
            self.generate_ast_node(emitter, arg)?;
        }

        // Handle built-in functions (these don't use function dispatch)
        match name {
            "require" => {
                emitter.emit_opcode(REQUIRE);
            }
            "get_clock" => {
                emitter.emit_opcode(GET_CLOCK);
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
                // ARRAY_CONCAT removed - would need to use separate operations or implement differently
                return Err(VMError::InvalidOperation); // Temporarily disabled
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

                // Invoke PDA derivation (handler pops: program_id, seeds_count, then each seed)
                emitter.emit_opcode(DERIVE_PDA);
            }
            "invoke_signed" => {
                // New logic for handling invoke_signed
                // The arguments on the stack should be: [program_id, instruction_data, accounts_count, seeds_count, seed1_len, seed1_data, ...]
                self.generate_invoke_signed(emitter, args)?;
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
            "sha256" => emit_syscall!(emitter, args, 80, args = 1),
            "keccak256" => emit_syscall!(emitter, args, 81, args = 1),
            "blake3" => emit_syscall!(emitter, args, 82, args = 1),
            "secp256k1_recover" => emit_syscall!(emitter, args, 84, args = 4),

            _ => {
                // Check for qualified function names like "math_lib::add"
                // If the module is registered as external, emit CALL_EXTERNAL instead of CALL
                
                println!("DEBUG: Opcode check - CALL: {}, CALL_EXTERNAL: {}", CALL, CALL_EXTERNAL);
                println!("DEBUG: Checking if '{}' is a qualified external call...", name);
                if let Some((module_name, func_name)) = Self::parse_qualified_name(name) {
                    println!("DEBUG: Parsed qualified name: mod='{}', func='{}'", module_name, func_name);
                    
                    // debug print all keys
                    println!("DEBUG: Available external_imports: {:?}", self.external_imports.keys());
                    
                    if let Some(ext_import) = self.external_imports.get(module_name) {
                        // Found external import - emit CALL_EXTERNAL opcode
                        println!("DEBUG: SUCCESS! Found import for module '{}', emitting CALL_EXTERNAL (145)", module_name);
                        
                        // CALL_EXTERNAL format: opcode(1) + account_index(1) + func_offset(u16) + param_count(1)
                        emitter.emit_opcode(CALL_EXTERNAL);
                        emitter.emit_u8(ext_import.account_index);
                        
                        // Get function offset (error if not found)
                        let func_offset = ext_import.functions.get(func_name)
                            .copied()
                            .ok_or_else(|| {
                                println!("DEBUG: Function '{}' not found in external interface", func_name);
                                VMError::InvalidScript
                            })?;
                        println!("DEBUG: Function '{}' offset: {}", func_name, func_offset);
                        // Reverting to u16 to match the VM's expectation
                        emitter.emit_u16(func_offset);
                        emitter.emit_u8(args.len() as u8);
                        
                        return Ok(());
                    } else {
                        println!("DEBUG: FAILURE! Module '{}' NOT found in external_imports", module_name);
                    }
                } else {
                     println!("DEBUG: parse_qualified_name returned None for '{}'", name);
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

        // Step 2: Emit program ID (bottom of stack)
        let program_id_bytes = self.parse_program_id(&interface_info.program_id)?;
        emitter.emit_const_pubkey(&program_id_bytes)?;

        // Step 3: Serialize and emit instruction data (discriminator + data args)
        // Now handling both literals and variables via dynamic opcode generation
        self.emit_instruction_data_construction(
            emitter,
            interface_method,
            &data_arg_indices,
            args,
        )?;

        // Step 4: Emit account indices in REVERSE order
        // VM pops them in reverse, so we emit reversed to reconstruct original order
        for &account_idx in account_indices.iter().rev() {
            emitter.emit_const_u8(account_idx)?;
        }

        // Step 5: Emit account count
        emitter.emit_const_u8(account_indices.len() as u8)?;

        // Step 6: Emit INVOKE opcode
        emitter.emit_opcode(INVOKE);

        Ok(())
    }

    /// Check if a TypeNode represents an account-meta parameter.
    /// Only explicit `Account` parameters are emitted as account metas.
    /// `pubkey` parameters are serialized into instruction data.
    fn is_account_meta_type(type_node: &TypeNode) -> bool {
        matches!(type_node, TypeNode::Account)
    }

    /// Check if a field_type string represents an account type
    fn is_account_type_str(field_type: &str) -> bool {
        field_type == "Account"
            || field_type == "account"
            || field_type.starts_with("Account<")
    }

    /// Resolve an account argument to its parameter index.
    /// Account arguments must be simple identifiers that resolve to function parameters
    /// of Account type. Returns the parameter index if valid.
    fn resolve_account_argument(&self, arg: &AstNode) -> Result<u8, VMError> {
        match arg {
            AstNode::Identifier(name) => {
                // Look up in local symbol table (function parameters)
                if let Some(field_info) = self.local_symbol_table.get(name) {
                    // Validate it's an account type
                    if !Self::is_account_type_str(&field_info.field_type) {
                        return Err(VMError::TypeMismatch);
                    }

                    Ok(super::super::account_utils::account_index_from_param_offset(
                        field_info.offset,
                    ))
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
        for (idx, (param_type, arg)) in interface_method
            .parameters
            .iter()
            .zip(args.iter())
            .enumerate()
        {
            if Self::is_account_meta_type(param_type) {
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
        // PUSH_ARRAY_LITERAL expects the number of stack values, not final byte length.
        let mut element_count = 0;

        // 1. Emit discriminator bytes
        let discriminator_bytes = interface_method
            .discriminator_bytes
            .clone()
            .unwrap_or_else(|| vec![interface_method.discriminator]);
        
        for byte in discriminator_bytes {
            emitter.emit_const_u8(byte)?;
            element_count += 1;
        }

        // 2. Emit each data argument
        for &arg_idx in data_arg_indices {
            let param_type = if let Some(param) = interface_method.parameters.get(arg_idx) {
                param
            } else {
                return Err(VMError::InvalidParameterCount);
            };

            let arg = &args[arg_idx];
            let values_emitted = self.emit_argument_serialization(emitter, param_type, arg)?;
            element_count += values_emitted;
        }

        // 3. Create the array ref from the stack values
        emitter.emit_opcode(PUSH_ARRAY_LITERAL);
        // PUSH_ARRAY_LITERAL takes a u8 element count.
        if element_count > 255 {
             println!("DEBUG: Instruction element count too large ({}) for PUSH_ARRAY_LITERAL", element_count);
             return Err(VMError::InvalidOperation);
        }
        emitter.emit_u8(element_count as u8);

        Ok(())
    }

    /// Emit opcodes to serialize a single argument onto the stack as bytes (Little Endian).
    /// Returns number of bytes emitted.
    fn emit_argument_serialization<T: OpcodeEmitter>(
        &mut self,
        emitter: &mut T,
        param_type: &TypeNode,
        arg: &AstNode,
    ) -> Result<usize, VMError> {
        // Handle Literals efficiently if possible, otherwise use dynamic logic
        match (param_type, arg) {
            (TypeNode::Primitive(name), AstNode::Literal(val)) if name == "u8" => {
                if let Value::U8(v) = val {
                     emitter.emit_const_u8(*v)?;
                    Ok(1) // one stack value
                } else {
                    Err(VMError::TypeMismatch)
                }
            }
             (TypeNode::Primitive(name), AstNode::Literal(val)) if name == "u64" => {
                if let Value::U64(v) = val {
                    let bytes = v.to_le_bytes();
                    for b in bytes {
                        emitter.emit_const_u8(b)?;
                    }
                    Ok(8) // eight u8 stack values
                } else {
                     Err(VMError::TypeMismatch)
                }
             }
             // For variables (Identifiers) or other expressions
             (TypeNode::Primitive(name), _) if name == "u8" => {
                 // Generate code to put u8 value on stack
                 self.generate_ast_node(emitter, arg)?;
                 // Check if we need to mask? Assuming generated value is u64 but holding u8, or proper u8 type logic
                 // Five VM usually works with Value enums. Value::U8 is a distinct type.
                 // So we just leave it on stack.
                 // PUSH_ARRAY_LITERAL expects Values.
                 Ok(1) // one stack value
             }
             (TypeNode::Primitive(name), _) if name == "u64" => {
                 // Generate code to put u64 value on stack
                  if let AstNode::Literal(val) = arg {
                        let v = val.as_i64()
                            .map(|i| i as u64)
                            .or(val.as_u64())
                            .ok_or(VMError::TypeMismatch)?;
                        emitter.emit_const_u64(v)?;
                    } else {
                        self.generate_ast_node(emitter, arg)?;
                    }
                 
                 // We need to split this u64 into 8 u8s on the stack.
                 // Using temp local strategy
                 let temp_idx = self.field_counter; 
                 self.field_counter += 1;
                 
                 // Store value in temp
                 self.emit_set_local(emitter, temp_idx, "__temp_u64_ser");
                 
                 // Extract 8 bytes (Little Endian)
                 // val % 256, (val/256)%256, ...
                 for _ in 0..8 {
                     // Get current value
                     self.emit_get_local(emitter, temp_idx, "__temp_u64_ser");
                     
                     // Calculate byte: val % 256
                     // val - (val/256 * 256)
                     emitter.emit_opcode(DUP); // val, val
                     emitter.emit_const_u64(256)?; // val, val, 256
                     emitter.emit_opcode(DIV); // val, val/256
                     emitter.emit_opcode(DUP); // val, val/256, val/256 (for next iter update)
                     
                     // Update temp with val/256 for next iteration
                     // Swap not available here.
                     // Saving val/256 to temp NOW is better
                     self.emit_set_local(emitter, temp_idx, "__temp_u64_ser_update"); 
                     // Stack: val, val/256 (SET_LOCAL consumed top)
                     // So stack: val. We need (val/256) for subtraction.
                     
                     // Optimization:
                     // LOAD temp
                     // DUP
                     // PUSH 256
                     // DIV
                     // DUP
                     // SET LOCAL temp (consumed) -> temp is now val/256
                     // PUSH 256
                     // MUL
                     // SUB -> remainder (byte)
                     // Stack has byte.
                     
                     // Implement sequence.
                 }
                 
                 // Loop logic:
                 // We are changing temp inside loop.
                 // And leaving bytes on stack.
                 
                 // Reset temp logic:
                 for _ in 0..8 {
                     self.emit_get_local(emitter, temp_idx, "__temp_u64_ser"); // Val
                     emitter.emit_opcode(DUP); // Val, Val
                     emitter.emit_const_u64(256)?; // Val, Val, 256
                     emitter.emit_opcode(DIV); // Val, Quotient
                     emitter.emit_opcode(DUP); // Val, Quotient, Quotient
                     self.emit_set_local(emitter, temp_idx, "__temp_u64_ser"); // Val, Quotient (temp updated)
                     emitter.emit_const_u64(256)?; // Val, Quotient, 256
                     emitter.emit_opcode(MUL); // Val, Product
                     emitter.emit_opcode(SUB); // Remainder (Byte as U64)
                      
                      // CRITICAL FIX: Cast U64 remainder to U8 so ArrayRef packing treats it as a byte
                      emitter.emit_opcode(five_protocol::opcodes::CAST);
                      emitter.emit_u8(five_protocol::types::U8);
                      
                      // Byte is left on stack as U8. Correct order (Little Endian: byte0 first).
                 }
                 
                 Ok(8) // eight u8 stack values
             }
             (TypeNode::Primitive(name), AstNode::Literal(val)) if name == "pubkey" => {
                 if let Value::Pubkey(pk) = val {
                     emitter.emit_const_pubkey(pk)?;
                    Ok(1) // one pubkey stack value
                 } else {
                     Err(VMError::TypeMismatch)
                 }
             }
             (TypeNode::Primitive(name), _) if name == "pubkey" => {
                 // Keep pubkey as a single stack value. INVOKE array packing expands PUBKEY
                 // typed elements into 32 instruction-data bytes.
                 self.generate_ast_node(emitter, arg)?;
                 Ok(1) // one pubkey stack value
             }
             _ => {
                 eprintln!("DEBUG: unsupported dynamic serialization for {:?}", param_type);
                 Err(VMError::TypeMismatch)
             }
        }
    }
}
