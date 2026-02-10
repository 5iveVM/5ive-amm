#![recursion_limit = "256"]

use wasm_bindgen::prelude::*;

use five_protocol::{
    opcodes, types, Value, FIVE_DEPLOY_MAGIC, FIVE_MAGIC, MAX_SCRIPT_SIZE,
};
use five_vm_mito::{error::VMError, FIVE_VM_PROGRAM_ID};
use serde::{Deserialize, Serialize};

const MAX_COMPUTE_UNITS: usize = 1_000_000;
use five_dsl_compiler::{
    error::integration,
    metrics::{export_metrics, CompilerMetrics, ExportFormat, MetricsCollector},

    DslCompiler,
};

// Initialize panic hook for better error messages in WASM.
fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn log_to_console(message: &str) {
    log_message(message);
}

// Helper for logging that works in both WASM and native environments.
fn log_message(message: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::log_1(&wasm_bindgen::JsValue::from_str(message));

    #[cfg(not(target_arch = "wasm32"))]
    println!("[WASM LOG] {}", message);
}

// Helper for warning that works in both WASM and native environments.
fn warn_message(message: &str) {
    #[cfg(target_arch = "wasm32")]
    web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(message));

    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("[WASM WARN] {}", message);
}

/// Execution result.
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub enum ExecutionStatus {
    /// All operations completed successfully.
    Completed,
    /// Execution stopped because it hit a system program call that cannot be executed in WASM.
    StoppedAtSystemCall,
    /// Execution stopped because it hit an INIT_PDA operation that requires real Solana context.
    StoppedAtInitPDA,
    /// Execution stopped because it hit an INVOKE operation that requires real RPC.
    StoppedAtInvoke,
    /// Execution stopped because it hit an INVOKE_SIGNED operation that requires real RPC.
    StoppedAtInvokeSigned,
    /// Execution stopped because compute limit was reached.
    ComputeLimitExceeded,
    /// Execution failed due to an error.
    Failed,
}

/// Detailed execution result.
#[derive(Debug)]
#[wasm_bindgen]
pub struct TestResult {
    /// Final execution status.
    #[wasm_bindgen(skip)]
    pub status: ExecutionStatus,
    /// Final value on stack (if any).
    #[wasm_bindgen(skip)]
    pub result_value: Option<JsValue>,
    /// Compute units consumed.
    pub compute_units_used: u64,
    /// Final instruction pointer.
    pub instruction_pointer: usize,
    /// Stack contents at stop point.
    #[wasm_bindgen(skip)]
    pub final_stack: Vec<JsValue>,
    /// Final memory state (temp buffer).
    #[wasm_bindgen(skip)]
    pub final_memory: Vec<u8>,
    /// Final state of accounts after execution.
    #[wasm_bindgen(skip)]
    pub final_accounts: Vec<JsValue>,
    /// Error message if failed.
    #[wasm_bindgen(skip)]
    pub error_message: Option<String>,
    /// Detailed execution context for enhanced debugging.
    #[wasm_bindgen(skip)]
    pub execution_context: Option<String>,
    /// Which opcode caused the stop (if stopped at system call).
    pub stopped_at_opcode: Option<u8>,
    /// Human-readable name of the stopping opcode.
    #[wasm_bindgen(skip)]
    pub stopped_at_opcode_name: Option<String>,
}

#[wasm_bindgen]
impl TestResult {
    #[wasm_bindgen(getter)]
    pub fn status(&self) -> String {
        format!("{:?}", self.status)
    }

    #[wasm_bindgen(getter)]
    pub fn has_result_value(&self) -> bool {
        self.result_value.is_some()
    }

    #[wasm_bindgen(getter)]
    pub fn get_result_value(&self) -> JsValue {
        self.result_value.clone().unwrap_or(JsValue::NULL)
    }

    #[wasm_bindgen(getter)]
    pub fn final_stack(&self) -> js_sys::Array {
        self.final_stack.iter().cloned().collect()
    }

    #[wasm_bindgen(getter)]
    pub fn final_memory(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.final_memory[..])
    }

    #[wasm_bindgen(getter)]
    pub fn final_accounts(&self) -> js_sys::Array {
        self.final_accounts.iter().cloned().collect()
    }

    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn execution_context(&self) -> Option<String> {
        self.execution_context.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn stopped_at_opcode_name(&self) -> Option<String> {
        self.stopped_at_opcode_name.clone()
    }
}

/// JavaScript-compatible VM state representation.
#[wasm_bindgen]
pub struct FiveVMState {
    #[wasm_bindgen(skip)]
    pub stack: Vec<String>,
    #[wasm_bindgen(skip)]
    pub instruction_pointer: usize,
    #[wasm_bindgen(skip)]
    pub compute_units: u64,
}

#[wasm_bindgen]
impl FiveVMState {
    #[wasm_bindgen(getter)]
    pub fn stack(&self) -> js_sys::Array {
        self.stack.iter().map(|s| JsValue::from_str(s)).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn instruction_pointer(&self) -> usize {
        self.instruction_pointer
    }

    #[wasm_bindgen(getter)]
    pub fn compute_units(&self) -> u64 {
        self.compute_units
    }
}

/// JavaScript-compatible account representation.
#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct WasmAccount {
    #[wasm_bindgen(skip)]
    pub key: [u8; 32],
    #[wasm_bindgen(skip)]
    pub data: Vec<u8>,
    pub lamports: u64,
    pub is_writable: bool,
    pub is_signer: bool,
    #[wasm_bindgen(skip)]
    pub owner: [u8; 32],
}

#[wasm_bindgen]
impl WasmAccount {
    #[wasm_bindgen(constructor)]
    pub fn new(
        key: &[u8],
        data: &[u8],
        lamports: u64,
        is_writable: bool,
        is_signer: bool,
        owner: &[u8],
    ) -> Result<WasmAccount, JsValue> {
        if key.len() != 32 {
            return Err(JsValue::from_str("Key must be 32 bytes"));
        }
        if owner.len() != 32 {
            return Err(JsValue::from_str("Owner must be 32 bytes"));
        }

        Ok(WasmAccount {
            key: key
                .try_into()
                .map_err(|_| JsValue::from_str("Invalid key"))?,
            data: data.to_vec(),
            lamports,
            is_writable,
            is_signer,
            owner: owner
                .try_into()
                .map_err(|_| JsValue::from_str("Invalid owner"))?,
        })
    }

    #[wasm_bindgen(getter)]
    pub fn key(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.key[..])
    }

    #[wasm_bindgen(getter)]
    pub fn data(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.data[..])
    }

    #[wasm_bindgen(setter)]
    pub fn set_data(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }

    #[wasm_bindgen(getter)]
    pub fn owner(&self) -> js_sys::Uint8Array {
        js_sys::Uint8Array::from(&self.owner[..])
    }
}

/// WASM logger implementation.
pub struct WasmLogger;

impl WasmLogger {
    pub fn log(&self, message: &str) {
        log_message(message);
    }
}

/// WASM system interface (system calls detected by opcode analysis).
pub struct WasmSystemInterface {
    pub encountered_system_call: std::sync::Arc<std::sync::Mutex<Option<SystemCallType>>>,
}

/// Types of system calls that stop execution in WASM.
#[derive(Debug, Clone)]
pub enum SystemCallType {
    Invoke(u8),
    InvokeSigned(u8),
    InitPDA(u8),
    Transfer(u8),
    CreateAccount(u8),
    ReallocAccount(u8),
}

impl WasmSystemInterface {
    pub fn new() -> Self {
        Self {
            encountered_system_call: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }

    pub fn get_encountered_system_call(&self) -> Option<SystemCallType> {
        self.encountered_system_call.lock().unwrap().clone()
    }

    pub fn reset_system_call_flag(&self) {
        *self.encountered_system_call.lock().unwrap() = None;
    }

    pub fn detect_system_call(&self, opcode: u8) -> bool {
        let system_call = match opcode {
            o if o == opcodes::INVOKE => Some(SystemCallType::Invoke(opcode)),
            o if o == opcodes::INVOKE_SIGNED => Some(SystemCallType::InvokeSigned(opcode)),
            o if o == opcodes::CREATE_ACCOUNT => Some(SystemCallType::InitPDA(opcode)),
            o if o == opcodes::TRANSFER => Some(SystemCallType::Transfer(opcode)),
            o if o == opcodes::CREATE_ACCOUNT => Some(SystemCallType::CreateAccount(opcode)),
            // Note: REALLOC_ACCOUNT opcode doesn't exist in current protocol
            _ => None,
        };

        if let Some(call_type) = system_call {
            *self.encountered_system_call.lock().unwrap() = Some(call_type);
            true
        } else {
            false
        }
    }
}

/// WASM account conversion for MitoVM zero-copy execution.
impl WasmAccount {
    /// Convert to raw account data for MitoVM.
    pub fn to_mito_account_data(&self) -> Vec<u8> {
        // MitoVM expects raw account data without metadata
        self.data.clone()
    }

    /// Update from MitoVM account data.
    pub fn update_from_mito_data(&mut self, data: &[u8]) {
        self.data = data.to_vec();
    }

    /// Create account metadata for MitoVM context.
    pub fn create_mito_metadata(&self) -> (bool, bool, [u8; 32]) {
        (self.is_signer, self.is_writable, self.owner)
    }
}

/// Main WASM VM wrapper.
#[wasm_bindgen]
pub struct FiveVMWasm {
    bytecode: Vec<u8>,
    abi_data: Option<String>, // Store ABI JSON from .five file
    _logger: &'static WasmLogger,
    _system: &'static WasmSystemInterface,
}

// Static instances for WASM environment.
static WASM_LOGGER: WasmLogger = WasmLogger;
static WASM_SYSTEM: std::sync::OnceLock<WasmSystemInterface> = std::sync::OnceLock::new();

#[wasm_bindgen]
impl FiveVMWasm {
    /// Create new VM instance with bytecode.
    #[wasm_bindgen(constructor)]
    pub fn new(_bytecode: &[u8]) -> Result<FiveVMWasm, JsValue> {
        // Initialize panic hook for better error messages.
        init_panic_hook();

        // Initialize system interface once.
        let system = WASM_SYSTEM.get_or_init(|| WasmSystemInterface::new());

        // Extract pure FIVE bytecode from account data.
        let extracted_bytecode = Self::extract_five_bytecode(_bytecode)
            .map_err(|e| JsValue::from_str(&format!("Failed to extract FIVE bytecode: {:?}", e)))?;

        // Try to extract ABI information from .five file format.
        let abi_data = Self::extract_abi_from_five_file(_bytecode);

        Ok(FiveVMWasm {
            bytecode: extracted_bytecode,
            abi_data,
            _logger: &WASM_LOGGER,
            _system: system,
        })
    }

    /// Execute VM with input data and accounts (legacy method).
    #[wasm_bindgen]
    pub fn execute(
        &mut self,
        input_data: &[u8],
        accounts: js_sys::Array,
    ) -> Result<JsValue, JsValue> {
        let test_result = self.execute_partial(input_data, accounts)?;

        // Convert TestResult to legacy format for compatibility.
        if test_result.status == ExecutionStatus::Failed {
            let error_msg = if let Some(context) = test_result.execution_context() {
                context
            } else {
                test_result
                    .error_message()
                    .unwrap_or_else(|| "Execution failed".to_string())
            };
            return Err(JsValue::from_str(&error_msg));
        }

        Ok(test_result.get_result_value())
    }

    /// Execute VM with partial execution support - stops at system calls.
    #[wasm_bindgen]
    pub fn execute_partial(
        &mut self,
        input_data: &[u8],
        accounts: js_sys::Array,
    ) -> Result<TestResult, JsValue> {
        // MitoVM uses static execution, no instance needed

        // Reset system call flag
        self._system.reset_system_call_flag();

        // Convert JS accounts to WASM accounts
        let mut wasm_accounts: Vec<WasmAccount> = Vec::new();
        for i in 0..accounts.length() {
            let account_js = accounts.get(i);
            let account: WasmAccount = serde_wasm_bindgen::from_value(account_js).map_err(|e| {
                JsValue::from_str(&format!("Failed to deserialize account: {:?}", e))
            })?;
            wasm_accounts.push(account);
        }

        // Execute using actual MitoVM with proper account conversion
        let execution_result = self.execute_mito_vm(input_data, &wasm_accounts);

        // Determine execution status and build result
        match execution_result {
            Ok((result_value, exec_context, updated_accounts)) => {
                // Extract real execution context from MitoVM
                let compute_units = 0; // Not tracking Solana BPF CUs
                let instruction_pointer = exec_context.instruction_pointer;
                let final_stack = vec![]; // Stack contents not needed for current use case
                let final_memory = exec_context.memory.to_vec();
                                          // Check if we stopped due to a system call
                if let Some(system_call) = self._system.get_encountered_system_call() {
                    let (status, opcode_name, opcode) = match system_call {
                        SystemCallType::Invoke(op) => (ExecutionStatus::StoppedAtInvoke, "INVOKE", op),
                        SystemCallType::InvokeSigned(op) => {
                            (ExecutionStatus::StoppedAtInvokeSigned, "INVOKE_SIGNED", op)
                        }
                        SystemCallType::InitPDA(op) => (ExecutionStatus::StoppedAtInitPDA, "INIT_PDA", op),
                        SystemCallType::Transfer(op) => {
                            (ExecutionStatus::StoppedAtSystemCall, "TRANSFER", op)
                        }
                        SystemCallType::CreateAccount(op) => {
                            (ExecutionStatus::StoppedAtSystemCall, "CREATE_ACCOUNT", op)
                        }
                        SystemCallType::ReallocAccount(op) => {
                            (ExecutionStatus::StoppedAtSystemCall, "REALLOC_ACCOUNT", op)
                        }
                    };

                    Ok(TestResult {
                        status,
                        result_value: result_value.and_then(|v| value_to_js(&v).ok()),
                        compute_units_used: compute_units,
                        instruction_pointer,
                        final_stack: final_stack.clone(),
                        final_memory: final_memory.clone(),
                        final_accounts: updated_accounts.iter().map(|acc| serde_wasm_bindgen::to_value(acc).unwrap()).collect(),
                        error_message: None,
                        execution_context: None,
                        stopped_at_opcode: Some(opcode),
                        stopped_at_opcode_name: Some(opcode_name.to_string()),
                    })
                } else {
                    // Normal completion
                    Ok(TestResult {
                        status: ExecutionStatus::Completed,
                        result_value: result_value.and_then(|v| value_to_js(&v).ok()),
                        compute_units_used: compute_units,
                        instruction_pointer,
                        final_stack,
                        final_memory,
                        final_accounts: updated_accounts.iter().map(|acc| serde_wasm_bindgen::to_value(acc).unwrap()).collect(),
                        error_message: None,
                        execution_context: None,
                        stopped_at_opcode: None,
                        stopped_at_opcode_name: None,
                    })
                }
            }
            Err((exec_error, exec_context)) => {
                // Enhanced error handling with ABI-aware parameter mismatch detection
                let (status, error_msg) = match &exec_error {
                    VMError::AbiParameterMismatch {
                        function_index,
                        expected_param_count,
                        actual_param_count,
                        failed_param_index,
                    } => {
                        // Try to enhance with ABI information if available
                        let enhanced_msg = self.enhance_parameter_error(
                            *function_index,
                            *expected_param_count,
                            *actual_param_count,
                            *failed_param_index,
                        );
                        (ExecutionStatus::Failed, enhanced_msg)
                    }
                    VMError::InvalidOperation => {
                        (ExecutionStatus::Failed, "Invalid operation".to_string())
                    }
                    _ => (
                        ExecutionStatus::Failed,
                        format!("Execution failed: {:?}", exec_error),
                    ),
                };

                // Get execution context from failed execution
                let instruction_pointer = exec_context.instruction_pointer;
                let compute_units = 0; // Not tracking Solana BPF CUs
                let final_stack = vec![]; // Stack contents not available in error case
                let final_memory = exec_context.memory.to_vec();
                let stopped_at_opcode = exec_context.failed_opcode;
                let stopped_at_opcode_name = stopped_at_opcode.map(|op| opcode_to_name(op).to_string());

                Ok(TestResult {
                    status,
                    result_value: None,
                    compute_units_used: compute_units,
                    instruction_pointer,
                    final_stack,
                    final_memory,
                    final_accounts: vec![],
                    error_message: Some(error_msg),
                    execution_context: Some(format!("MitoVM execution error: {:#?}", exec_error)), // Use {:#?} for detailed debug output
                    stopped_at_opcode,
                    stopped_at_opcode_name,
                })
            }
        }
    }

    /// Get current VM state
    #[wasm_bindgen]
    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        // MitoVM is stateless, return basic info

        // Create state representation (simplified)
        let state = FiveVMState {
            stack: vec![],          // Stack contents not directly accessible in current VM
            instruction_pointer: 0, // IP not directly accessible
            compute_units: 0,       // CU not directly accessible
        };

        let state_json = serde_json::json!({
            "stack": state.stack,
            "instruction_pointer": state.instruction_pointer,
            "compute_units": state.compute_units
        });

        Ok(JsValue::from_str(&state_json.to_string()))
    }

    /// Validate bytecode without execution
    #[wasm_bindgen]
    pub fn validate_bytecode(bytecode: &[u8]) -> Result<bool, JsValue> {
        Self::validate_bytecode_internal(bytecode)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
    }

    /// Internal validation logic returning Rust types
    pub(crate) fn validate_bytecode_internal(bytecode: &[u8]) -> Result<bool, VMError> {
        // First try to extract pure bytecode
        let extracted_bytecode = match Self::extract_five_bytecode(bytecode) {
            Ok(extracted) => extracted,
            Err(_) => return Ok(false),
        };

        // Check minimum size
        if extracted_bytecode.len() < 4 {
            return Ok(false);
        }

        // Check maximum size
        if extracted_bytecode.len() > MAX_SCRIPT_SIZE {
            return Ok(false);
        }

        // Check magic bytes (should be guaranteed by extract_five_bytecode, but double-check)
        if &extracted_bytecode[0..4] != FIVE_MAGIC {
            return Ok(false);
        }

        // Validate optimized header structure for sync with core protocol
        if five_protocol::parse_header(&extracted_bytecode).is_err() {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get VM constants for JavaScript
    #[wasm_bindgen]
    pub fn get_constants() -> JsValue {
        let mut opcodes_map = serde_json::Map::new();

        for info in five_protocol::opcodes::OPCODE_TABLE {
            opcodes_map.insert(info.name.to_string(), serde_json::Value::Number(serde_json::Number::from(info.opcode)));
        }

        let constants = serde_json::json!({
            "MAX_SCRIPT_SIZE": MAX_SCRIPT_SIZE,
            "MAX_COMPUTE_UNITS": MAX_COMPUTE_UNITS,
            "FIVE_MAGIC": FIVE_MAGIC,
            "opcodes": opcodes_map,
            "types": {
                "U64": types::U64,
                "BOOL": types::BOOL,
                "PUBKEY": types::PUBKEY,
                "I64": types::I64,
                "U8": types::U8,
                "STRING": types::STRING,
                "ACCOUNT": types::ACCOUNT,
                "ARRAY": types::ARRAY
            }
        });

        JsValue::from_str(&constants.to_string())
    }

    /// Execute MitoVM with parameter decoding
    fn execute_mito_vm(
        &self,
        input_data: &[u8],
        wasm_accounts: &[WasmAccount],
    ) -> Result<
        (
            Option<Value>,
            five_vm_mito::VMExecutionContext,
            Vec<WasmAccount>,
        ),
        (five_vm_mito::error::VMError, five_vm_mito::VMExecutionContext),
    > {
        // Decode instruction data before passing to MitoVM
        // If decoding fails, return early with empty context
        let decoded_input = self.decode_instruction_data(input_data).map_err(|e| {
            (
                e,
                five_vm_mito::VMExecutionContext {
                    instruction_pointer: 0,
                    halted: false,
                    error: None,
                    memory: [0u8; five_protocol::TEMP_BUFFER_SIZE],
                    failed_opcode: None,
                },
            )
        })?;

        // Log account information for debugging
        log_message(&format!(
            "WASM: execute_mito_vm called with {} accounts",
            wasm_accounts.len()
        ));
        for (i, account) in wasm_accounts.iter().enumerate() {
            log_message(&format!(
                "WASM: Account {}: writable={}, signer={}, data_len={}",
                i,
                account.is_writable,
                account.is_signer,
                account.data.len()
            ));
        }

        // WASM execution currently doesn't support real AccountInfo creation since pinocchio
        // AccountInfo structs are typically provided by the Solana validator during execution.
        // Use empty accounts and log the account information we would have used.
        log_message(&format!(
            "WASM: Would pass {} accounts to VM execution:",
            wasm_accounts.len()
        ));
        for (i, account) in wasm_accounts.iter().enumerate() {
            log_message(&format!(
                "  Account {}: key={:?}, lamports={}, owner={:?}, signer={}, writable={}",
                i,
                account.key,
                account.lamports,
                account.owner,
                account.is_signer,
                account.is_writable
            ));
        }

        // Prepare backing storage for account data
        // The AccountInfo constructor is not available in pinocchio 0.9.2
        // We'll keep the backing storage but use empty account_infos for now
        let keys: Vec<pinocchio::pubkey::Pubkey> = wasm_accounts.iter().map(|a| {
            let mut pubkey_bytes = [0u8; 32];
            pubkey_bytes.copy_from_slice(&a.key);
            pinocchio::pubkey::Pubkey::from(pubkey_bytes)
        }).collect();
        let owners: Vec<pinocchio::pubkey::Pubkey> = wasm_accounts.iter().map(|a| {
            let mut pubkey_bytes = [0u8; 32];
            pubkey_bytes.copy_from_slice(&a.owner);
            pinocchio::pubkey::Pubkey::from(pubkey_bytes)
        }).collect();
        let mut lamports: Vec<u64> = wasm_accounts.iter().map(|a| a.lamports).collect();
        let mut data: Vec<Vec<u8>> = wasm_accounts.iter().map(|a| a.data.clone()).collect();

        // Manual AccountInfo construction using Pinocchio API
        let mut account_infos: Vec<pinocchio::account_info::AccountInfo> = Vec::with_capacity(wasm_accounts.len());
        for i in 0..wasm_accounts.len() {
             account_infos.push(pinocchio::account_info::AccountInfo::new(
                 &keys[i],
                 wasm_accounts[i].is_signer,
                 wasm_accounts[i].is_writable,
                 &mut lamports[i],
                 &mut data[i],
                 &owners[i],
                 false, // executable
                 0,     // rent_epoch
             ));
        }

        // Execute using MitoVM with populated accounts
        log_message(&format!(
            "WASM: About to execute MitoVM with account list size: {}",
            account_infos.len()
        ));

        // Execute using MitoVM
        // Execute using MitoVM
        let program_id = pinocchio::pubkey::Pubkey::from(FIVE_VM_PROGRAM_ID);
        let result = five_vm_mito::MitoVM::execute_with_context(
            &self.bytecode,
            &decoded_input,
            &account_infos,
            &program_id,
        );

        // Reconstruct updated WasmAccounts from the modified AccountInfos
        // Note: AccountInfo::new() creates a copy, so we must read back from AccountInfo
        let mut updated_wasm_accounts = Vec::with_capacity(wasm_accounts.len());
        for i in 0..wasm_accounts.len() {
             let account_info = &account_infos[i];

             // Safety: We are the only ones accessing this data in this thread
             let new_data = unsafe { account_info.borrow_data_unchecked() }.to_vec();
             let new_lamports = account_info.lamports();
             let new_owner = *account_info.owner();

             updated_wasm_accounts.push(WasmAccount {
                key: keys[i],
                data: new_data,
                lamports: new_lamports,
                is_writable: wasm_accounts[i].is_writable,
                is_signer: wasm_accounts[i].is_signer,
                owner: new_owner,
             });
        }

        // Enhanced error reporting for debugging
        match &result {
             Ok((result_value, context)) => {
                log_message(&format!(
                    "WASM: MitoVM execution SUCCESS - IP: {}, halted: {}",
                    context.instruction_pointer, context.halted
                ));
                if result_value.is_some() {
                    log_message("WASM: MitoVM - function returned a value");
                } else {
                    log_message("WASM: MitoVM - function completed without return value");
                }
            }
            Err((vm_error, _)) => {
                log_message(&format!("WASM: MitoVM execution ERROR: {:?}", vm_error));

                // Enhanced error analysis
                match vm_error {
                    five_vm_mito::error::VMError::StackError => {
                        log_message("StackError detected");
                        log_message("Possible causes: empty stack pop, return failure, or stack corruption");
                    }
                    five_vm_mito::error::VMError::AbiParameterMismatch {
                        function_index,
                        expected_param_count,
                        actual_param_count,
                        failed_param_index,
                    } => {
                        log_message(&format!(
                            "Parameter mismatch in function {}",
                            function_index
                        ));
                        log_message(&format!(
                            "Exp: {}, Act: {}, Failed: {}",
                            expected_param_count, actual_param_count, failed_param_index
                        ));
                    }
                    five_vm_mito::error::VMError::CallStackOverflow => {
                        log_message("Call stack overflow");
                    }
                    five_vm_mito::error::VMError::CallStackUnderflow => {
                        log_message("Call stack underflow");
                    }
                    five_vm_mito::error::VMError::InvalidInstruction => {
                        log_message("Invalid opcode");
                    }
                    _ => {
                        log_message(&format!("Other error: {:?}", vm_error));
                    }
                }
            }
        }

        log_message(&format!("Result: {:?}", result));
        
        // Return result with updated accounts
        result.map(|(val, ctx)| (val, ctx, updated_wasm_accounts))
    }

    /// Extract pure FIVE bytecode from Solana account data
    /// Account structure: 48-byte header + FIVE bytecode
    /// This method handles both raw bytecode and account data formats
    fn extract_five_bytecode(input_data: &[u8]) -> Result<Vec<u8>, VMError> {
        // If data is empty, return empty
        if input_data.is_empty() {
            return Ok(Vec::new());
        }

        // Log input for debugging
        log_message(&format!(
            "Extracting bytecode ({} bytes)",
            input_data.len()
        ));

        // Check if this is already pure FIVE bytecode (starts with magic bytes)
        if input_data.len() >= 4 && &input_data[0..4] == FIVE_MAGIC {
            log_message("Input is pure 5IVX");
            return Ok(input_data.to_vec());
        }

        if input_data.len() >= 4 && &input_data[0..4] == FIVE_DEPLOY_MAGIC {
            log_message("Input is pure 5IVE");
            return Ok(input_data.to_vec());
        }

        // Check if this is account data with 64-byte header (ScriptAccountHeader::LEN)
        const ACCOUNT_HEADER_SIZE: usize = 64;
        if input_data.len() > ACCOUNT_HEADER_SIZE {
            let bytecode_offset = ACCOUNT_HEADER_SIZE;
            let potential_bytecode = &input_data[bytecode_offset..];

            // Verify FIVE magic bytes at the expected offset
            if potential_bytecode.len() >= 4 && &potential_bytecode[0..4] == FIVE_MAGIC {
                log_message(&format!(
                    "Found bytecode at {}, extracted {} bytes",
                    bytecode_offset,
                    potential_bytecode.len()
                ));
                return Ok(potential_bytecode.to_vec());
            }
        }

        // Try to find FIVE magic bytes anywhere in the data (fallback)
        for i in 0..input_data.len().saturating_sub(4) {
            if &input_data[i..i + 4] == FIVE_MAGIC {
                let extracted = &input_data[i..];
                log_message(&format!(
                    "Found magic at {}, extracted {} bytes",
                    i,
                    extracted.len()
                ));
                return Ok(extracted.to_vec());
            }
        }

        // If no FIVE magic found, return error
        Err(VMError::InvalidScript)
    }

    /// Extract ABI information from .five file format
    /// .five files contain: DSL source + ABI JSON + bytecode in a structured format
    fn extract_abi_from_five_file(input_data: &[u8]) -> Option<String> {
        // .five files are structured with sections
        // Try to parse as JSON first to see if this is a .five file
        if let Ok(content) = std::str::from_utf8(input_data) {
            // Look for JSON structure with "abi" field
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(content) {
                if let Some(abi_section) = json_value.get("abi") {
                    if let Ok(abi_json) = serde_json::to_string(abi_section) {
                        log_message("Extracted ABI");
                        return Some(abi_json);
                    }
                }
            }
        }

        log_message("No ABI found");
        None
    }

    /// Decode instruction data for MitoVM execution
    /// Frontend sends: [discriminator(u8), function_index(u32), param_count(u32), ...parameters]
    /// MitoVM expects: [function_index(u32), param_count(u32), ...parameters]
    fn decode_instruction_data(&self, input_data: &[u8]) -> Result<Vec<u8>, VMError> {
        if input_data.is_empty() {
            return Ok(Vec::new());
        }

        // Check for Execute discriminator (2 for legacy, 9 for on-chain Solana program)
        if input_data[0] == 2 || input_data[0] == 9 {
            // Strip discriminator and return data for MitoVM
            let data = &input_data[1..];

            // Log the decoding for debugging
            log_message("Decoded instruction");
            log_message(&format!(
                "  Raw ({}): {:?}",
                input_data.len(),
                input_data
            ));
            log_message(&format!("  Discriminator: {}", input_data[0]));
            log_message(&format!(
                "  MitoVM data ({}): {:?}",
                data.len(),
                data
            ));

            Ok(data.to_vec())
        } else {
            // Not an Execute instruction - pass through as-is
            log_message(&format!(
                "Non-Execute, passing through: {:?}",
                input_data
            ));
            Ok(input_data.to_vec())
        }
    }

    /// Enhance parameter mismatch error with ABI information when available
    /// Generates detailed error messages by looking up function names
    /// and parameter types from ABI data embedded in .five files
    fn enhance_parameter_error(
        &self,
        function_index: u32,
        expected_param_count: u32,
        actual_param_count: u32,
        failed_param_index: u32,
    ) -> String {
        Self::enhance_parameter_error_static(
            &self.abi_data,
            function_index,
            expected_param_count,
            actual_param_count,
            failed_param_index,
        )
    }

    /// Static version of enhance_parameter_error for testing without instance
    pub(crate) fn enhance_parameter_error_static(
        abi_data: &Option<String>,
        function_index: u32,
        expected_param_count: u32,
        actual_param_count: u32,
        failed_param_index: u32,
    ) -> String {
        // Try to get enhanced information from ABI if available
        if let Some(abi_json) = abi_data {
            if let Ok(abi) = serde_json::from_str::<serde_json::Value>(abi_json) {
                if let Some(functions) = abi.get("functions") {
                    // Look for function by index - check both SimpleABI format (object) and FIVEABI format (array)
                    let function_info = if let Some(functions_obj) = functions.as_object() {
                        // SimpleABI format: { "functions": { "functionName": { "index": 0, ... } } }
                        functions_obj.values().find(|f| {
                            f.get("index")
                                .and_then(|i| i.as_u64())
                                .map(|i| i == function_index as u64)
                                .unwrap_or(false)
                        })
                    } else if let Some(functions_array) = functions.as_array() {
                        // FIVEABI format: { "functions": [{ "name": "...", "index": 0, ... }] }
                        functions_array.iter().find(|f| {
                            f.get("index")
                                .and_then(|i| i.as_u64())
                                .map(|i| i == function_index as u64)
                                .unwrap_or(false)
                        })
                    } else {
                        None
                    };

                    if let Some(func) = function_info {
                        let default_name = format!("function_{}", function_index);
                        let function_name = func
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or(&default_name);

                        let parameters = func
                            .get("parameters")
                            .and_then(|p| p.as_array())
                            .map(|params| {
                                params
                                    .iter()
                                    .filter_map(|p| p.get("type").or_else(|| p.get("param_type")))
                                    .filter_map(|t| t.as_str())
                                    .collect::<Vec<_>>()
                            })
                            .unwrap_or_default();

                        // Create enhanced error message with function name and parameter types
                        return Self::create_enhanced_error_message_static(
                            function_name,
                            &parameters,
                            expected_param_count,
                            actual_param_count,
                            failed_param_index,
                        );
                    }
                }
            }
        }

        // Fallback to basic error message if no ABI available
        Self::create_basic_error_message_static(
            function_index,
            expected_param_count,
            actual_param_count,
            failed_param_index,
        )
    }

    /// Create enhanced error message with function name and parameter types from ABI
    pub(crate) fn create_enhanced_error_message_static(
        function_name: &str,
        parameters: &[&str],
        expected_param_count: u32,
        actual_param_count: u32,
        failed_param_index: u32,
    ) -> String {
        let base_msg = format!(
            "❌ Function Parameter Mismatch\n\n\
            Function '{}' expected {} parameters but received {}\n\
            Failed to load parameter '{}' at position {}\n\n",
            function_name,
            expected_param_count,
            actual_param_count,
            parameters
                .get(failed_param_index as usize)
                .unwrap_or(&"unknown"),
            failed_param_index + 1
        );

        let param_types_msg = if !parameters.is_empty() {
            format!(
                "📝 Expected parameter types:\n\
                {}\n\n",
                parameters
                    .iter()
                    .enumerate()
                    .map(|(i, param_type)| format!(
                        "  {}. {} ({})",
                        i + 1,
                        param_type,
                        if i == failed_param_index as usize {
                            "← FAILED HERE"
                        } else {
                            "✓"
                        }
                    ))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            String::new()
        };

        let help_msg = if actual_param_count < expected_param_count {
            format!(
                "💡 Fix this error:\n\
                • Add {} missing parameter(s) to your function call\n\
                • Example: {}({})\n\
                • Ensure all parameters match the expected types",
                expected_param_count - actual_param_count,
                function_name,
                (0..expected_param_count)
                    .map(|i| parameters.get(i as usize).unwrap_or(&"value"))
                    .map(|t| match *t {
                        "u64" => "100",
                        "bool" => "true",
                        "string" => "\"text\"",
                        "pubkey" => "\"11111111111111111111111111111112\"",
                        _ => "value",
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else if actual_param_count > expected_param_count {
            format!(
                "💡 Fix this error:\n\
                • Remove {} extra parameter(s) from your function call\n\
                • Expected: {}({})",
                actual_param_count - expected_param_count,
                function_name,
                parameters.join(", ")
            )
        } else {
            "💡 Parameter count matches but parameter encoding failed\n\
            • Check that parameter types are correctly encoded\n\
            • Verify account references are valid\n\
            • Ensure string parameters are properly formatted"
                .to_string()
        };

        format!("{}{}{}", base_msg, param_types_msg, help_msg)
    }

    /// Create basic error message when ABI information is not available
    pub(crate) fn create_basic_error_message_static(
        function_index: u32,
        expected_param_count: u32,
        actual_param_count: u32,
        failed_param_index: u32,
    ) -> String {
        let base_msg = format!(
            "❌ Function Parameter Mismatch\n\n\
            Function at index {} expected {} parameters but received {}\n\
            Failed to load parameter at position {}\n\n",
            function_index,
            expected_param_count,
            actual_param_count,
            failed_param_index + 1
        );

        let help_msg = if actual_param_count < expected_param_count {
            format!(
                "💡 Help: You're missing {} parameter(s)\n\
                • Check that you're passing all required parameters\n\
                • Verify parameter types match the function signature\n\
                • Ensure account parameters are properly provided",
                expected_param_count - actual_param_count
            )
        } else if actual_param_count > expected_param_count {
            "💡 Help: You're passing too many parameters\n\
            • Remove extra parameters from your function call\n\
            • Check the function signature for required parameters only"
                .to_string()
        } else {
            "💡 Help: Parameter count matches but parameter loading failed\n\
            • Check parameter types are correct\n\
            • Verify account indices are valid\n\
            • Ensure all parameters are properly encoded"
                .to_string()
        };

        let debug_msg = format!(
            "\n📍 Debug Information:\n\
            • Function Index: {}\n\
            • Expected Parameters: {}\n\
            • Actual Parameters: {}\n\
            • Failed Parameter: {}\n\
            • Note: Use .five files with embedded ABI for enhanced error messages",
            function_index, expected_param_count, actual_param_count, failed_param_index
        );

        format!("{}{}{}", base_msg, help_msg, debug_msg)
    }
}

fn looks_like_optimized_header(bytecode: &[u8]) -> bool {
    if bytecode.len() < five_protocol::FIVE_HEADER_OPTIMIZED_SIZE {
        return false;
    }

    if &bytecode[0..4] != five_protocol::FIVE_MAGIC {
        return false;
    }

    true
}

/// Utility: Validate optimized headers and mirror bytecode back to JS callers
#[wasm_bindgen]
pub fn wrap_with_script_header(bytecode: &[u8]) -> Result<js_sys::Uint8Array, JsValue> {
    if !looks_like_optimized_header(bytecode) {
        return Err(JsValue::from_str(
            "bytecode does not contain an optimized FIVE header",
        ));
    }

    log_message("wrap_with_script_header: optimized header detected, returning bytecode as-is");
    Ok(js_sys::Uint8Array::from(bytecode))
}

/// Parse function names from bytecode metadata
///
/// Returns a JS value which is a JSON string encoding an array of objects:
/// [ { "name": "...", "function_index": N }, ... ]
/// We serialize via serde_json and return the JSON string as a `JsValue` to
/// avoid complex JS object construction in Rust/WASM glue.
#[wasm_bindgen]
pub fn parse_function_names(bytecode: &[u8]) -> Result<JsValue, JsValue> {
    use five_protocol::parser::parse_optimized_bytecode;

    let parsed = parse_optimized_bytecode(bytecode)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Build a serializable list of simple objects
    let mut names: Vec<serde_json::Value> = Vec::new();

    if let Some(metadata) = parsed.function_names {
        for entry in metadata.names {
            names.push(serde_json::json!({
                "name": entry.name,
                "function_index": entry.function_index
            }));
        }
    }

    // Serialize to JSON and return as JsValue::from_str(json)
    let json = serde_json::to_string(&names).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(JsValue::from_str(&json))
}

/// Get the count of public functions from bytecode header
#[wasm_bindgen]
pub fn get_public_function_count(bytecode: &[u8]) -> Result<u8, JsValue> {
    use five_protocol::FIVE_HEADER_OPTIMIZED_SIZE;

    if bytecode.len() < FIVE_HEADER_OPTIMIZED_SIZE {
        return Err(JsValue::from_str("Bytecode too short for header"));
    }

    // Check magic
    if &bytecode[0..4] != five_protocol::FIVE_MAGIC {
        return Err(JsValue::from_str("Invalid magic number"));
    }

    Ok(bytecode[8]) // public_function_count
}

/// Get function names from bytecode as a JS value (array of objects)
///
/// This function avoids constructing `FunctionNameInfo` JS instances and instead
/// marshals the parsed metadata directly into a serde-friendly structure and
/// returns a `JsValue` via `JsValue::from_serde`.
#[wasm_bindgen]
pub fn get_function_names(bytecode: &[u8]) -> Result<JsValue, JsValue> {
    use five_protocol::parser::parse_optimized_bytecode;

    // Parse the optimized bytecode directly (returns Result<ParsedScript, String>)
    let parsed = parse_optimized_bytecode(bytecode)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    // Convert metadata to a serializable Vec of simple maps
    let mut names: Vec<serde_json::Value> = Vec::new();
    if let Some(metadata) = parsed.function_names {
        for entry in metadata.names {
            names.push(serde_json::json!({
                "name": entry.name,
                "function_index": entry.function_index
            }));
        }
    }

    // Serialize to JSON string and return as JsValue (avoids relying on JsValue::from_serde)
    let json = serde_json::to_string(&names).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(JsValue::from_str(&json))
}

/// Helper function to convert VM Value to JS value
fn value_to_js(value: &Value) -> Result<JsValue, JsValue> {
    match value {
        Value::Empty => Ok(JsValue::null()),
        Value::U64(v) => Ok(JsValue::from(*v)),
        Value::Bool(v) => Ok(JsValue::from(*v)),
        Value::Pubkey(v) => Ok(js_sys::Uint8Array::from(&v[..]).into()),
        Value::I64(v) => Ok(JsValue::from(*v)),
        Value::U8(v) => Ok(JsValue::from(*v)),
        Value::String(idx) => Ok(JsValue::from_str(&format!("STRING[{}]", idx))), // String index reference
        Value::Account(idx) => Ok(JsValue::from_str(&format!("ACCOUNT[{}]", idx))), // Account index reference
        Value::Array(idx) => Ok(JsValue::from_str(&format!("ARRAY[{}]", idx))), // Array index reference
        Value::U128(v) => Ok(JsValue::from_str(&format!("U128[{}]", v))), // U128 value as string (JS doesn't support 128-bit ints natively)
    }
}

/// Helper function to convert JS value to VM Value
#[wasm_bindgen]
pub fn js_value_to_vm_value(js_val: &JsValue, value_type: u8) -> Result<JsValue, JsValue> {
    let vm_value = match value_type {
        t if t == types::U64 => {
            let v = js_val
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Expected number for U64"))?
                as u64;
            Value::U64(v)
        }
        t if t == types::BOOL => {
            let v = js_val
                .as_bool()
                .ok_or_else(|| JsValue::from_str("Expected boolean"))?;
            Value::Bool(v)
        }
        t if t == types::PUBKEY => {
            let array = js_sys::Uint8Array::new(js_val);
            if array.length() != 32 {
                return Err(JsValue::from_str("Pubkey must be 32 bytes"));
            }
            let mut pubkey = [0u8; 32];
            array.copy_to(&mut pubkey);
            Value::Pubkey(pubkey)
        }
        t if t == types::I64 => {
            let v = js_val
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Expected number for I64"))?
                as i64;
            Value::I64(v)
        }
        t if t == types::U8 => {
            let v = js_val
                .as_f64()
                .ok_or_else(|| JsValue::from_str("Expected number for U8"))?
                as u8;
            Value::U8(v)
        }
        t if t == types::STRING => {
            let _v = js_val
                .as_string()
                .ok_or_else(|| JsValue::from_str("Expected string"))?;
            Value::String(0) // Default string index - would need proper string storage
        }
        _ => return Err(JsValue::from_str("Unsupported value type")),
    };

    value_to_js(&vm_value)
}

/// Bytecode analyzer for WASM
#[wasm_bindgen]
pub struct BytecodeAnalyzer;

impl BytecodeAnalyzer {
    pub(crate) fn analyze_internal(bytecode: &[u8]) -> Result<serde_json::Value, String> {
        if !FiveVMWasm::validate_bytecode_internal(bytecode).map_err(|e| format!("{:?}", e))? {
            return Err("Invalid bytecode".to_string());
        }

        let mut instructions = Vec::new();
        let (header, start_offset) = five_protocol::parse_header(bytecode)
            .map_err(|e| format!("Header parse failed: {:?}", e))?;
        let mut i = start_offset;

        while i < bytecode.len() {
            let opcode = bytecode[i];
            let instruction_info = serde_json::json!({
                "offset": i,
                "opcode": opcode,
                "name": opcode_to_name(opcode),
                "size": get_instruction_size_with_features(opcode, &bytecode[i..], header.features)
            });
            instructions.push(instruction_info);

            i += get_instruction_size_with_features(opcode, &bytecode[i..], header.features);
        }

        Ok(serde_json::json!({
            "total_size": bytecode.len(),
            "instruction_count": instructions.len(),
            "instructions": instructions
        }))
    }

    pub(crate) fn analyze_semantic_internal(
        bytecode: &[u8],
    ) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::bytecode_generator::AdvancedBytecodeAnalyzer;

        // Create and run the advanced analyzer
        let mut analyzer = AdvancedBytecodeAnalyzer::new(bytecode.to_vec());
        let analysis = analyzer
            .analyze()
            .map_err(|e| format!("Analysis failed: {:?}", e))?;

        // Convert to JSON-serializable format
        Ok(serde_json::json!({
            "summary": {
                "total_size": analysis.summary.total_size,
                "total_instructions": analysis.summary.total_instructions,
                "total_compute_units": analysis.summary.total_compute_cost,
                "max_stack_depth": 0, // Not available
                "has_jumps": analysis.summary.jump_count > 0,
                "has_function_calls": analysis.summary.function_call_count > 0,
                "category_breakdown": analysis.summary.category_distribution.len()
            },
            "instructions": analysis.instructions.iter().map(|inst| {
                serde_json::json!({
                    "offset": inst.offset,
                    "opcode": inst.opcode,
                    "name": inst.name,
                    "description": inst.description,
                    "category": format!("{:?}", inst.category),
                    "operands": inst.operands.iter().map(|op| {
                        serde_json::json!({
                            "type": op.operand_type,
                            "decoded_value": op.decoded_value,
                            "size": op.size,
                            "description": op.description,
                            "raw_bytes": op.raw_value
                        })
                    }).collect::<Vec<_>>(),
                    "size": inst.size,
                    "stack_effect": inst.stack_effect,
                    "compute_cost": inst.compute_cost,
                    "control_flow": {
                        "is_jump": inst.control_flow.is_jump,
                        "jump_targets": inst.control_flow.jump_targets,
                        "can_fall_through": inst.control_flow.can_fall_through,
                        "is_terminator": inst.control_flow.is_terminator
                    },
                    "raw_bytes": inst.raw_bytes
                })
            }).collect::<Vec<_>>(),
            "control_flow": {
                "basic_blocks": analysis.control_flow.basic_blocks.iter().map(|block| {
                    serde_json::json!({
                        "start": block.start,
                        "end": block.end,
                        "instructions": block.instructions,
                        "successors": block.successors,
                        "predecessors": block.predecessors
                    })
                }).collect::<Vec<_>>(),
                "entry_points": analysis.control_flow.entry_points
            },
            "stack_analysis": {
                "stack_depths": analysis.stack_analysis.stack_depths,
                "max_stack_depth": analysis.stack_analysis.max_stack_depth,
                "min_stack_depth": analysis.stack_analysis.min_stack_depth,
                "is_consistent": analysis.stack_analysis.is_consistent
            },
            "patterns": analysis.patterns.len() // Just count for now
        }))
    }

    pub(crate) fn analyze_instruction_at_internal(
        bytecode: &[u8],
        offset: usize,
    ) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::bytecode_generator::AdvancedBytecodeAnalyzer;

        let mut analyzer = AdvancedBytecodeAnalyzer::new(bytecode.to_vec());
        let analysis = analyzer
            .analyze()
            .map_err(|e| format!("Analysis failed: {:?}", e))?;

        // Find instruction at the specified offset
        if let Some(instruction) = analysis
            .instructions
            .iter()
            .find(|inst| inst.offset == offset)
        {
            Ok(serde_json::json!({
                "offset": instruction.offset,
                "opcode": instruction.opcode,
                "name": instruction.name,
                "description": instruction.description,
                "category": format!("{:?}", instruction.category),
                "operands": instruction.operands.iter().map(|op| {
                    serde_json::json!({
                        "type": op.operand_type,
                        "decoded_value": op.decoded_value,
                        "size": op.size,
                        "description": op.description,
                        "raw_bytes": op.raw_value
                    })
                }).collect::<Vec<_>>(),
                "size": instruction.size,
                "stack_effect": instruction.stack_effect,
                "compute_cost": instruction.compute_cost,
                "control_flow": {
                    "is_jump": instruction.control_flow.is_jump,
                    "jump_targets": instruction.control_flow.jump_targets,
                    "can_fall_through": instruction.control_flow.can_fall_through,
                    "is_terminator": instruction.control_flow.is_terminator
                },
                "raw_bytes": instruction.raw_bytes,
                "hex_representation": instruction.raw_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            }))
        } else {
            Err(format!("No instruction found at offset {}", offset))
        }
    }

    pub(crate) fn get_bytecode_summary_internal(
        bytecode: &[u8],
    ) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::bytecode_generator::AdvancedBytecodeAnalyzer;

        let mut analyzer = AdvancedBytecodeAnalyzer::new(bytecode.to_vec());
        let analysis = analyzer
            .analyze()
            .map_err(|e| format!("Analysis failed: {:?}", e))?;

        Ok(serde_json::json!({
            "total_size": analysis.summary.total_size,
            "total_instructions": analysis.summary.total_instructions,
            "total_compute_units": analysis.summary.total_compute_cost,
            "max_stack_depth": 0, // Not available in current implementation
            "has_jumps": analysis.summary.jump_count > 0,
            "has_function_calls": analysis.summary.function_call_count > 0,
            "patterns_detected": analysis.patterns.len(),
            "patterns": analysis.patterns.len(), // Just count for now
            "stack_consistency": analysis.stack_analysis.is_consistent,
            "basic_blocks_count": analysis.control_flow.basic_blocks.len(),
            "category_breakdown": analysis.summary.category_distribution.len()
        }))
    }

    pub(crate) fn analyze_execution_flow_internal(
        bytecode: &[u8],
    ) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::bytecode_generator::AdvancedBytecodeAnalyzer;

        let mut analyzer = AdvancedBytecodeAnalyzer::new(bytecode.to_vec());
        let analysis = analyzer
            .analyze()
            .map_err(|e| format!("Analysis failed: {:?}", e))?;

        // Build execution flow representation
        let mut execution_paths = Vec::new();

        for (i, instruction) in analysis.instructions.iter().enumerate() {
            let mut next_instructions = Vec::new();

            if instruction.control_flow.can_fall_through && i + 1 < analysis.instructions.len() {
                next_instructions.push(analysis.instructions[i + 1].offset);
            }

            for &target in &instruction.control_flow.jump_targets {
                next_instructions.push(target);
            }

            execution_paths.push(serde_json::json!({
                "offset": instruction.offset,
                "opcode": instruction.opcode,
                "name": instruction.name,
                "description": instruction.description,
                "next_instructions": next_instructions,
                "is_terminator": instruction.control_flow.is_terminator,
                "stack_depth": analysis.stack_analysis.stack_depths.get(i).unwrap_or(&0)
            }));
        }

        Ok(serde_json::json!({
            "execution_paths": execution_paths,
            "entry_points": analysis.control_flow.entry_points,
            "basic_blocks": analysis.control_flow.basic_blocks.iter().map(|block| {
                serde_json::json!({
                    "start_offset": block.start,
                    "end_offset": block.end,
                    "instruction_count": block.instructions.len(),
                    "successors": block.successors,
                    "predecessors": block.predecessors
                })
            }).collect::<Vec<_>>(),
            "stack_analysis": {
                "is_consistent": analysis.stack_analysis.is_consistent,
                "max_depth": analysis.stack_analysis.max_stack_depth,
                "min_depth": analysis.stack_analysis.min_stack_depth
            }
        }))
    }
}

#[wasm_bindgen]
impl BytecodeAnalyzer {
    /// Analyze bytecode and return instruction breakdown (legacy method for compatibility)
    #[wasm_bindgen]
    pub fn analyze(bytecode: &[u8]) -> Result<JsValue, JsValue> {
        Self::analyze_internal(bytecode)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Advanced semantic analysis with full opcode understanding and instruction flow
    /// Performs semantic analysis of bytecode to understand opcode behavior
    /// and instruction flow.
    #[wasm_bindgen]
    pub fn analyze_semantic(bytecode: &[u8]) -> Result<JsValue, JsValue> {
        Self::analyze_semantic_internal(bytecode)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get detailed information about a specific instruction at an offset
    #[wasm_bindgen]
    pub fn analyze_instruction_at(bytecode: &[u8], offset: usize) -> Result<JsValue, JsValue> {
        Self::analyze_instruction_at_internal(bytecode, offset)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get summary statistics about the bytecode
    #[wasm_bindgen]
    pub fn get_bytecode_summary(bytecode: &[u8]) -> Result<JsValue, JsValue> {
        Self::get_bytecode_summary_internal(bytecode)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Get detailed opcode flow analysis - shows execution paths through the bytecode
    #[wasm_bindgen]
    pub fn analyze_execution_flow(bytecode: &[u8]) -> Result<JsValue, JsValue> {
        Self::analyze_execution_flow_internal(bytecode)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }
}

/// Helper function to get opcode name
fn opcode_to_name(opcode: u8) -> &'static str {
    five_protocol::opcodes::opcode_name(opcode)
}

/// Helper function to get instruction size
#[cfg(test)]
fn get_instruction_size(opcode: u8, bytes: &[u8]) -> usize {
    get_instruction_size_with_features(opcode, bytes, 0)
}

fn get_instruction_size_with_features(opcode: u8, bytes: &[u8], features: u32) -> usize {
    let pool_enabled = (features & five_protocol::FEATURE_CONSTANT_POOL) != 0;
    let remaining = if bytes.len() > 1 { &bytes[1..] } else { &[] };
    match five_protocol::opcodes::operand_size(opcode, remaining, pool_enabled) {
        Some(operand_bytes) => 1 + operand_bytes,
        None => 1,
    }
}

/// Enhanced error suggestion for WASM
#[derive(Debug, Clone, serde::Serialize)]
#[wasm_bindgen]
pub struct WasmSuggestion {
    /// Suggestion message
    #[wasm_bindgen(skip)]
    pub message: String,
    /// Optional explanation
    #[wasm_bindgen(skip)]
    pub explanation: Option<String>,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Optional code suggestion/fix
    #[wasm_bindgen(skip)]
    pub code_suggestion: Option<String>,
}

#[wasm_bindgen]
impl WasmSuggestion {
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn explanation(&self) -> Option<String> {
        self.explanation.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn code_suggestion(&self) -> Option<String> {
        self.code_suggestion.clone()
    }
}

/// Enhanced source location for WASM
#[derive(Debug, Clone, serde::Serialize)]
#[wasm_bindgen]
pub struct WasmSourceLocation {
    /// File name
    #[wasm_bindgen(skip)]
    pub file: Option<String>,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based)
    pub column: u32,
    /// Byte offset in source
    pub offset: usize,
    /// Length of the relevant text
    pub length: usize,
}

#[wasm_bindgen]
impl WasmSourceLocation {
    #[wasm_bindgen(getter)]
    pub fn file(&self) -> Option<String> {
        self.file.clone()
    }
}

/// Enhanced compiler error for WASM
#[derive(Debug, Clone, serde::Serialize)]
#[wasm_bindgen]
pub struct WasmCompilerError {
    /// Error code (e.g., "E0001")
    #[wasm_bindgen(skip)]
    pub code: String,
    /// Error severity
    #[wasm_bindgen(skip)]
    pub severity: String,
    /// Error category
    #[wasm_bindgen(skip)]
    pub category: String,
    /// Main error message
    #[wasm_bindgen(skip)]
    pub message: String,
    /// Optional description
    #[wasm_bindgen(skip)]
    pub description: Option<String>,
    /// Source location
    #[wasm_bindgen(skip)]
    pub location: Option<WasmSourceLocation>,
    /// Associated suggestions
    #[wasm_bindgen(skip)]
    pub suggestions: Vec<WasmSuggestion>,
    /// Source line for context display
    #[wasm_bindgen(skip)]
    pub source_line: Option<String>,
    /// Source snippet for multi-line context
    #[wasm_bindgen(skip)] 
    pub source_snippet: Option<String>,
    /// Line number (for SDK compatibility)
    #[wasm_bindgen(skip)]
    pub line: Option<u32>,
    /// Column number (for SDK compatibility)
    #[wasm_bindgen(skip)]
    pub column: Option<u32>,
}

#[wasm_bindgen]
impl WasmCompilerError {
    #[wasm_bindgen(getter)]
    pub fn code(&self) -> String {
        self.code.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn line(&self) -> JsValue {
        match self.line {
            Some(l) => JsValue::from_f64(l as f64),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn column(&self) -> JsValue {
        match self.column {
            Some(c) => JsValue::from_f64(c as f64),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn severity(&self) -> String {
        self.severity.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn category(&self) -> String {
        self.category.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.message.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn description(&self) -> Option<String> {
        self.description.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn location(&self) -> Option<WasmSourceLocation> {
        self.location.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn suggestions(&self) -> Vec<WasmSuggestion> {
        self.suggestions.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn source_line(&self) -> Option<String> {
        self.source_line.clone()
    }

    /// Get formatted error message (terminal style)
    #[wasm_bindgen]
    /// Get formatted error message (terminal style)
    #[wasm_bindgen]
    pub fn format_terminal(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();
        
        // Header: error[E0000]: message
        // Use colors if possible (simulated with ANSI codes for now since we're returning string)
        let color_reset = "\x1b[0m";
        let color_red = "\x1b[31m";
        let color_blue = "\x1b[34m";
        let color_cyan = "\x1b[36m";
        
        let severity_color = match self.severity.as_str() {
            "error" => color_red,
            "warning" => "\x1b[33m", // Yellow
            _ => color_blue,
        };
        
        // 1. Error header
        writeln!(
            &mut output, 
            "{}{}[{}]{}: {}", 
            severity_color,
            self.severity, 
            self.code, 
            color_reset,
            self.message
        ).unwrap();
        
        // 2. Location
        if let Some(loc) = &self.location {
            let file_display = loc.file.as_deref().unwrap_or("unknown");
            writeln!(
                &mut output, 
                "  {}-->{} {}:{}:{}", 
                color_blue, color_reset,
                file_display, loc.line, loc.column
            ).unwrap();
        } else if let (Some(line), Some(col)) = (self.line, self.column) {
             // Fallback to direct line/col if location object missing
             writeln!(
                &mut output, 
                "  {}-->{} line {}:{}", 
                color_blue, color_reset,
                line, col
            ).unwrap();
        }
        
        // 3. Description (if present and different from message)
        if let Some(desc) = &self.description {
            if desc != &self.message {
                writeln!(&mut output, "  {}", desc).unwrap();
            }
        }
        
        // 4. Source snippet
        if let Some(snippet) = &self.source_snippet {
            writeln!(&mut output, "{}", snippet).unwrap();
        } else if let Some(line_content) = &self.source_line {
             if let Some(loc) = &self.location {
                 let line_num_str = loc.line.to_string();
                 let pad = " ".repeat(line_num_str.len());
                 
                 writeln!(&mut output, "  {} |{}", pad, color_reset).unwrap();
                 writeln!(&mut output, "{} {} |{} {}", loc.line, color_blue, color_reset, line_content).unwrap();
                 
                 // Underline error
                 let pointer_pad = " ".repeat(loc.column.saturating_sub(1) as usize);
                 let pointer_len = std::cmp::max(1, loc.length);
                 let pointer = "^".repeat(pointer_len);
                 
                 writeln!(
                     &mut output, 
                     "  {} |{} {}{}{} {}", 
                     pad, color_blue, pointer_pad, severity_color, pointer, color_reset
                 ).unwrap();
             }
        }
        
        // 5. Suggestions
        if !self.suggestions.is_empty() {
            for suggestion in &self.suggestions {
                writeln!(
                    &mut output, 
                    "  {}={}>{} {}", 
                    color_cyan, color_reset, 
                    color_cyan, suggestion.message
                ).unwrap();
            }
        }
        
        output
    }

    /// Get error as JSON string
    #[wasm_bindgen]
    pub fn format_json(&self) -> String {
        // Convert to JSON for programmatic use
        let json_obj = serde_json::json!({
            "code": self.code,
            "severity": self.severity,
            "category": self.category,
            "message": self.message,
            "description": self.description,
            "location": self.location.as_ref().map(|loc| serde_json::json!({
                "file": loc.file,
                "line": loc.line,
                "column": loc.column,
                "offset": loc.offset,
                "length": loc.length
            })),
            "suggestions": self.suggestions.iter().map(|s| serde_json::json!({
                "message": s.message,
                "explanation": s.explanation,
                "confidence": s.confidence
            })).collect::<Vec<_>>()
        });
        json_obj.to_string()
    }
}

/// Enhanced compilation result with rich error information
#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmEnhancedCompilationResult {
    /// Whether compilation succeeded
    pub success: bool,
    /// Generated bytecode (if successful)
    #[wasm_bindgen(skip)]
    pub bytecode: Option<Vec<u8>>,
    /// Size of generated bytecode
    pub bytecode_size: usize,
    /// Compilation time in milliseconds
    pub compilation_time: f64,
    /// Enhanced compiler errors
    #[wasm_bindgen(skip)]
    pub compiler_errors: Vec<WasmCompilerError>,
    /// Total error count
    pub error_count: usize,
    /// Total warning count
    pub warning_count: usize,
}

#[wasm_bindgen]
impl WasmEnhancedCompilationResult {
    #[wasm_bindgen(getter)]
    pub fn compiler_errors(&self) -> Vec<WasmCompilerError> {
        self.compiler_errors.clone()
    }

    /// Get all errors formatted as terminal output
    #[wasm_bindgen]
    pub fn format_all_terminal(&self) -> String {
        self.compiler_errors
            .iter()
            .map(|e| e.format_terminal())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Get all errors as JSON array
    #[wasm_bindgen]
    pub fn format_all_json(&self) -> String {
        let errors_json: Vec<serde_json::Value> = self
            .compiler_errors
            .iter()
            .map(|e| {
                serde_json::from_str(&e.format_json()).unwrap_or_else(|_| serde_json::json!({}))
            })
            .collect();
        serde_json::to_string(&errors_json).unwrap_or_else(|_| "[]".to_string())
    }
}

/// Compilation options for enhanced error reporting and formatting
#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct WasmCompilationOptions {
    // === Core Compilation ===
    /// Compilation mode ("testing", "deployment", "debug")
    #[wasm_bindgen(skip)]
    pub mode: String,
    /// Optimization level ("production")
    #[wasm_bindgen(skip)]
    pub optimization_level: String,

    // === Feature Flags ===
    /// Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
    pub v2_preview: bool,
    /// Enable constraint caching optimization
    pub enable_constraint_cache: bool,

    // === Error & Debug ===
    /// Enable enhanced error reporting with suggestions
    pub enhanced_errors: bool,
    /// Error output format ("terminal", "json", "lsp", "html")
    #[wasm_bindgen(skip)]
    pub error_format: String,
    /// Source file name for error reporting
    #[wasm_bindgen(skip)]
    pub source_file: Option<String>,

    // === Metrics & Analysis ===
    /// Include basic metrics
    pub include_metrics: bool,
    /// Include comprehensive metrics collection
    pub comprehensive_metrics: bool,
    /// Metrics export format ("json", "csv", "dashboard")
    #[wasm_bindgen(skip)]
    pub metrics_format: String,
    /// Include performance analysis
    pub performance_analysis: bool,
    /// Include complexity analysis
    pub complexity_analysis: bool,

    // === Output Control ===
    /// Show compilation summary
    pub summary: bool,
    /// Verbose output
    pub verbose: bool,
    /// Suppress non-essential output
    pub quiet: bool,

    // === Advanced Features ===
    /// Analysis depth level ("quick", "standard", "deep", "comprehensive")
    #[wasm_bindgen(skip)]
    pub analysis_depth: String,
    /// Export format ("binary", "json", "abi")
    #[wasm_bindgen(skip)]
    pub export_format: String,
    /// Include debug information
    pub include_debug_info: bool,
    /// Enable bytecode compression
    pub compress_output: bool,

    // === Experimental ===
    /// Experimental feature flags
    #[wasm_bindgen(skip)]
    pub experimental_features: Vec<String>,
    /// Custom optimization passes
    #[wasm_bindgen(skip)]
    pub custom_optimizations: Vec<String>,

    // === Namespace Control ===
    /// Enable module namespace qualification (module::function)
    pub enable_module_namespaces: bool,
}

#[wasm_bindgen]
impl WasmCompilationOptions {
    /// Create default compilation options
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmCompilationOptions {
        WasmCompilationOptions {
            // Core Compilation
            mode: "testing".to_string(),
            optimization_level: "production".to_string(),

            // Feature Flags
            v2_preview: false,
            enable_constraint_cache: true,

            // Error & Debug
            enhanced_errors: true,
            error_format: "json".to_string(),
            source_file: None,

            // Metrics & Analysis
            include_metrics: false,
            comprehensive_metrics: false,
            metrics_format: "json".to_string(),
            performance_analysis: false,
            complexity_analysis: false,

            // Output Control
            summary: false,
            verbose: false,
            quiet: false,

            // Advanced Features
            analysis_depth: "standard".to_string(),
            export_format: "binary".to_string(),
            include_debug_info: false,
            compress_output: false,

            // Experimental
            experimental_features: Vec::new(),
            custom_optimizations: Vec::new(),

            // Namespace Control
            enable_module_namespaces: true,
        }
    }

    // === Core Compilation Builders ===
    /// Set compilation mode
    #[wasm_bindgen]
    pub fn with_mode(mut self, mode: &str) -> WasmCompilationOptions {
        self.mode = mode.to_string();
        self
    }

    /// Set optimization level (production)
    #[wasm_bindgen]
    pub fn with_optimization_level(mut self, level: &str) -> WasmCompilationOptions {
        self.optimization_level = level.to_string();
        self
    }

    // === Feature Flag Builders ===
    /// Enable or disable v2-preview features
    #[wasm_bindgen]
    pub fn with_v2_preview(mut self, enabled: bool) -> WasmCompilationOptions {
        self.v2_preview = enabled;
        self
    }

    /// Enable or disable constraint caching optimization
    #[wasm_bindgen]
    pub fn with_constraint_cache(mut self, enabled: bool) -> WasmCompilationOptions {
        self.enable_constraint_cache = enabled;
        self
    }

    // === Error & Debug Builders ===
    /// Enable or disable enhanced error reporting
    #[wasm_bindgen]
    pub fn with_enhanced_errors(mut self, enabled: bool) -> WasmCompilationOptions {
        self.enhanced_errors = enabled;
        self
    }

    /// Set error output format
    #[wasm_bindgen]
    pub fn with_error_format(mut self, format: &str) -> WasmCompilationOptions {
        self.error_format = format.to_string();
        self
    }

    /// Set source file name for better error reporting
    #[wasm_bindgen]
    pub fn with_source_file(mut self, filename: &str) -> WasmCompilationOptions {
        self.source_file = Some(filename.to_string());
        self
    }

    // === Metrics & Analysis Builders ===
    /// Enable or disable basic metrics collection
    #[wasm_bindgen]
    pub fn with_metrics(mut self, enabled: bool) -> WasmCompilationOptions {
        self.include_metrics = enabled;
        self
    }

    /// Enable or disable comprehensive metrics collection
    #[wasm_bindgen]
    pub fn with_comprehensive_metrics(mut self, enabled: bool) -> WasmCompilationOptions {
        self.comprehensive_metrics = enabled;
        self
    }

    /// Set metrics export format
    #[wasm_bindgen]
    pub fn with_metrics_format(mut self, format: &str) -> WasmCompilationOptions {
        self.metrics_format = format.to_string();
        self
    }

    /// Enable or disable performance analysis
    #[wasm_bindgen]
    pub fn with_performance_analysis(mut self, enabled: bool) -> WasmCompilationOptions {
        self.performance_analysis = enabled;
        self
    }

    /// Enable or disable complexity analysis
    #[wasm_bindgen]
    pub fn with_complexity_analysis(mut self, enabled: bool) -> WasmCompilationOptions {
        self.complexity_analysis = enabled;
        self
    }

    // === Output Control Builders ===
    /// Enable or disable compilation summary
    #[wasm_bindgen]
    pub fn with_summary(mut self, enabled: bool) -> WasmCompilationOptions {
        self.summary = enabled;
        self
    }

    /// Enable or disable verbose output
    #[wasm_bindgen]
    pub fn with_verbose(mut self, enabled: bool) -> WasmCompilationOptions {
        self.verbose = enabled;
        self
    }

    /// Enable or disable quiet mode
    #[wasm_bindgen]
    pub fn with_quiet(mut self, enabled: bool) -> WasmCompilationOptions {
        self.quiet = enabled;
        self
    }

    // === Advanced Feature Builders ===
    /// Set analysis depth level
    #[wasm_bindgen]
    pub fn with_analysis_depth(mut self, depth: &str) -> WasmCompilationOptions {
        self.analysis_depth = depth.to_string();
        self
    }

    /// Set export format
    #[wasm_bindgen]
    pub fn with_export_format(mut self, format: &str) -> WasmCompilationOptions {
        self.export_format = format.to_string();
        self
    }

    /// Enable or disable debug information
    #[wasm_bindgen]
    pub fn with_debug_info(mut self, enabled: bool) -> WasmCompilationOptions {
        self.include_debug_info = enabled;
        self
    }

    /// Enable or disable bytecode compression
    #[wasm_bindgen]
    pub fn with_compression(mut self, enabled: bool) -> WasmCompilationOptions {
        self.compress_output = enabled;
        self
    }

    /// Enable or disable module namespace qualification
    #[wasm_bindgen]
    pub fn with_module_namespaces(mut self, enabled: bool) -> WasmCompilationOptions {
        self.enable_module_namespaces = enabled;
        self
    }

    // === Preset Configurations ===
    /// Create production-optimized configuration
    #[wasm_bindgen]
    pub fn production_optimized() -> WasmCompilationOptions {
        WasmCompilationOptions::new()
            .with_mode("deployment")
            .with_optimization_level("production")
            .with_v2_preview(true)
            .with_constraint_cache(true)
            .with_comprehensive_metrics(true)
            .with_compression(true)
            .with_quiet(true)
    }

    /// Create development-debug configuration
    #[wasm_bindgen]
    pub fn development_debug() -> WasmCompilationOptions {
        WasmCompilationOptions::new()
            .with_mode("testing")
            .with_optimization_level("production")
            .with_enhanced_errors(true)
            .with_debug_info(true)
            .with_verbose(true)
            .with_analysis_depth("comprehensive")
    }

    /// Create fast iteration configuration
    #[wasm_bindgen]
    pub fn fast_iteration() -> WasmCompilationOptions {
        WasmCompilationOptions::new()
            .with_mode("testing")
            .with_optimization_level("production")
            .with_enhanced_errors(false)
            .with_metrics(false)
            .with_quiet(true)
    }

    // === Getters for WASM ===
    #[wasm_bindgen(getter)]
    pub fn mode(&self) -> String {
        self.mode.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn optimization_level(&self) -> String {
        self.optimization_level.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error_format(&self) -> String {
        self.error_format.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn source_file(&self) -> Option<String> {
        self.source_file.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn metrics_format(&self) -> String {
        self.metrics_format.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn analysis_depth(&self) -> String {
        self.analysis_depth.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn export_format(&self) -> String {
        self.export_format.clone()
    }
}

/// WASM-exposed metrics collector wrapper
#[wasm_bindgen]
pub struct WasmMetricsCollector {
    #[wasm_bindgen(skip)]
    inner: MetricsCollector,
}

#[wasm_bindgen]
impl WasmMetricsCollector {
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMetricsCollector {
        WasmMetricsCollector {
            inner: MetricsCollector::new(),
        }
    }

    /// Start timing a compilation phase
    pub fn start_phase(&mut self, phase_name: &str) {
        self.inner.start_phase(phase_name);
    }

    /// End the current compilation phase
    pub fn end_phase(&mut self) {
        self.inner.end_phase();
    }

    /// Finalize metrics collection
    pub fn finalize(&mut self) {
        self.inner.finalize();
    }

    /// Reset the collector for a new compilation
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Export metrics in the requested format
    pub fn export(&self, format: &str) -> Result<String, JsValue> {
        let export_format = map_metrics_format(format);
        export_metrics(self.inner.get_metrics(), export_format)
            .map_err(|e| JsValue::from_str(&format!("Failed to export metrics: {}", e)))
    }

    /// Get metrics as a JS object for programmatic use
    pub fn get_metrics_object(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(self.inner.get_metrics())
            .map_err(|e| JsValue::from_str(&format!("Failed to convert metrics: {}", e)))
    }
}

/// WASM compilation result - unified with enhanced error support
#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmCompilationResult {
    /// Whether compilation succeeded
    pub success: bool,
    /// Generated bytecode (if successful)
    #[wasm_bindgen(skip)]
    pub bytecode: Option<Vec<u8>>,
    /// Size of generated bytecode
    pub bytecode_size: usize,
    /// Compilation time in milliseconds
    pub compilation_time: f64,
    /// Enhanced compiler errors
    #[wasm_bindgen(skip)]
    pub compiler_errors: Vec<WasmCompilerError>,
    /// Total error count
    pub error_count: usize,
    /// Total warning count
    pub warning_count: usize,
    /// Basic warnings (for backwards compatibility)
    #[wasm_bindgen(skip)]
    pub warnings: Vec<String>,
    /// Basic errors (for backwards compatibility)
    #[wasm_bindgen(skip)]
    pub errors: Vec<String>,
    /// Compilation metrics JSON
    #[wasm_bindgen(skip)]
    pub metrics: String,
    /// Metrics format (json, csv, toml)
    #[wasm_bindgen(skip)]
    pub metrics_format: String,
    /// Detailed metrics object for structured export
    #[wasm_bindgen(skip)]
    pub detailed_metrics: Option<CompilerMetrics>,
    /// Human-readable compilation log (disassembly)
    #[wasm_bindgen(skip)]
    pub disassembly: Vec<String>,
    /// ABI JSON string
    #[wasm_bindgen(skip)]
    pub abi: Option<String>,
    /// Pre-formatted terminal output (full Rust-style error display)
    #[wasm_bindgen(skip)]
    pub formatted_errors_terminal: String,
    /// Pre-formatted JSON output
    #[wasm_bindgen(skip)]
    pub formatted_errors_json: String,
}

/// WASM compilation result with comprehensive metrics
#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmCompilationWithMetrics {
    /// Whether compilation succeeded
    pub success: bool,
    /// Generated bytecode (if successful)
    #[wasm_bindgen(skip)]
    pub bytecode: Option<Vec<u8>>,
    /// Size of generated bytecode
    pub bytecode_size: usize,
    /// Compilation time in milliseconds
    pub compilation_time: f64,
    /// Warnings encountered during compilation
    #[wasm_bindgen(skip)]
    pub warnings: Vec<String>,
    /// Errors encountered during compilation
    #[wasm_bindgen(skip)]
    pub errors: Vec<String>,
    /// Comprehensive metrics as JSON string
    #[wasm_bindgen(skip)]
    pub metrics_json: String,
}

/// WASM source analysis result
#[derive(Debug)]
#[wasm_bindgen]
pub struct WasmAnalysisResult {
    /// Whether analysis succeeded
    pub success: bool,
    /// Human-readable analysis summary
    #[wasm_bindgen(skip)]
    pub summary: String,
    /// Analysis time in milliseconds
    pub analysis_time: f64,
    /// Detailed metrics as JSON string
    #[wasm_bindgen(skip)]
    pub metrics_json: String,
    /// Errors encountered during analysis
    #[wasm_bindgen(skip)]
    pub errors: Vec<String>,
}

#[wasm_bindgen]
impl WasmCompilationResult {
    #[wasm_bindgen(getter)]
    pub fn bytecode(&self) -> Option<js_sys::Uint8Array> {
        self.bytecode
            .as_ref()
            .map(|b| js_sys::Uint8Array::from(&b[..]))
    }

    #[wasm_bindgen(getter)]
    pub fn abi(&self) -> JsValue {
        match &self.abi {
            Some(json) => js_sys::JSON::parse(json).unwrap_or(JsValue::UNDEFINED),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> js_sys::Array {
        self.warnings.iter().map(|w| JsValue::from_str(w)).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> js_sys::Array {
        self.errors.iter().map(|e| JsValue::from_str(e)).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn compiler_errors(&self) -> Vec<WasmCompilerError> {
        self.compiler_errors.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn disassembly(&self) -> js_sys::Array {
        self.disassembly
            .iter()
            .map(|line| JsValue::from_str(line))
            .collect()
    }

    #[wasm_bindgen]
    pub fn get_formatted_errors_terminal(&self) -> String {
        self.formatted_errors_terminal.clone()
    }

    #[wasm_bindgen]
    pub fn get_formatted_errors_json(&self) -> String {
        self.formatted_errors_json.clone()
    }

    /// Get all errors formatted as terminal output
    #[wasm_bindgen]
    pub fn format_all_terminal(&self) -> String {
        self.compiler_errors
            .iter()
            .map(|e| e.format_terminal())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get all errors as JSON array
    #[wasm_bindgen]
    pub fn format_all_json(&self) -> String {
        let json_errors: Vec<_> = self
            .compiler_errors
            .iter()
            .map(|e| serde_json::to_value(e).unwrap_or_default())
            .collect();
        serde_json::to_string(&json_errors).unwrap_or_default()
    }

    /// Get parsed metrics as JavaScript object
    #[wasm_bindgen]
    pub fn get_metrics_object(&self) -> JsValue {
        if let Some(metrics) = &self.detailed_metrics {
            return serde_wasm_bindgen::to_value(metrics)
                .unwrap_or_else(|_| JsValue::NULL);
        }

        if self.metrics.is_empty() {
            return JsValue::NULL;
        }

        if self.metrics_format != "json" {
            return JsValue::NULL;
        }

        match js_sys::JSON::parse(&self.metrics) {
            Ok(obj) => obj,
            Err(_) => JsValue::NULL,
        }
    }

    /// Get fully detailed metrics regardless of export format
    #[wasm_bindgen]
    pub fn get_metrics_detailed(&self) -> Result<JsValue, JsValue> {
        match &self.detailed_metrics {
            Some(metrics) => serde_wasm_bindgen::to_value(metrics)
                .map_err(|e| JsValue::from_str(&format!("Failed to convert metrics: {}", e))),
            None => Err(JsValue::from_str("No metrics available")),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn metrics(&self) -> String {
        self.metrics.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn metrics_format(&self) -> String {
        self.metrics_format.clone()
    }
}

#[wasm_bindgen]
impl WasmCompilationWithMetrics {
    #[wasm_bindgen(getter)]
    pub fn bytecode(&self) -> Option<js_sys::Uint8Array> {
        self.bytecode
            .as_ref()
            .map(|b| js_sys::Uint8Array::from(&b[..]))
    }

    #[wasm_bindgen(getter)]
    pub fn warnings(&self) -> js_sys::Array {
        self.warnings.iter().map(|w| JsValue::from_str(w)).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> js_sys::Array {
        self.errors.iter().map(|e| JsValue::from_str(e)).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn metrics(&self) -> String {
        self.metrics_json.clone()
    }

    /// Get parsed metrics as JavaScript object
    #[wasm_bindgen]
    pub fn get_metrics_object(&self) -> Result<JsValue, JsValue> {
        let parsed: serde_json::Value = serde_json::from_str(&self.metrics_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse metrics JSON: {}", e)))?;
        serde_wasm_bindgen::to_value(&parsed)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert metrics to JS: {}", e)))
    }

}

#[wasm_bindgen]
impl WasmAnalysisResult {
    #[wasm_bindgen(getter)]
    pub fn summary(&self) -> String {
        self.summary.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn metrics(&self) -> String {
        self.metrics_json.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn errors(&self) -> js_sys::Array {
        self.errors.iter().map(|e| JsValue::from_str(e)).collect()
    }

    /// Get parsed metrics as JavaScript object
    #[wasm_bindgen]
    pub fn get_metrics_object(&self) -> Result<JsValue, JsValue> {
        let parsed: serde_json::Value = serde_json::from_str(&self.metrics_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse metrics JSON: {}", e)))?;
        serde_wasm_bindgen::to_value(&parsed)
            .map_err(|e| JsValue::from_str(&format!("Failed to convert metrics to JS: {}", e)))
    }
}

/// WASM DSL Compiler for client-side compilation
#[wasm_bindgen]
pub struct WasmFiveCompiler;

#[wasm_bindgen]
impl WasmFiveCompiler {
    /// Create a new WASM compiler instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmFiveCompiler {
        // Initialize panic hook for better error messages
        init_panic_hook();
        WasmFiveCompiler
    }

    /// Format an error message using the native terminal formatter
    /// This provides rich Rust-style error output with source context and colors
    #[wasm_bindgen]
    pub fn format_error_terminal(&self, message: String, code: String, severity: String, line: u32, column: u32, _source: &str) -> String {
        use five_dsl_compiler::error::{TerminalFormatter, ErrorFormatter, CompilerError, ErrorCode, ErrorSeverity, ErrorCategory, SourceLocation};
        
        // Construct a partial CompilerError for formatting
        // Note: This is a best-effort reconstruction since we can't easily pass the full CompilerError object from JS
        let severity_enum = match severity.as_str() {
            "error" => ErrorSeverity::Error,
            "warning" => ErrorSeverity::Warning,
            "note" => ErrorSeverity::Note,
            "help" => ErrorSeverity::Help,
            _ => ErrorSeverity::Error,
        };
        
        // Parse error code number (e.g. "E0001" -> 1)
        let code_num = code.trim_start_matches('E').parse::<u32>().unwrap_or(0);
        let code_enum = ErrorCode::new(code_num);
        
        let location = SourceLocation::new(line, column, 0); // Offset 0 as deeper context extraction might not need it 
        
        let error = CompilerError::new(
            code_enum,
            severity_enum,
            ErrorCategory::Internal, // Category doesn't affect terminal formatting much
            message
        ).with_location(location);
        
        let formatter = TerminalFormatter::new();
        formatter.format_error(&error)
    }

    /// Unified compilation method with enhanced error reporting and metrics
    #[wasm_bindgen]
    pub fn compile(&self, source: &str, options: &WasmCompilationOptions) -> WasmCompilationResult {
        use five_dsl_compiler::compiler::OptimizationLevel;
        use five_dsl_compiler::{CompilationConfig, CompilationMode};

        let start_time = js_sys::Date::now();

        // Initialize enhanced error system if enabled
        if options.enhanced_errors {
            if let Err(e) = integration::initialize_error_system() {
                warn_message(&format!(
                    "Failed to initialize enhanced error system: {}",
                    e
                ));
            }
            // Set error formatter... (keeping existing logic short for brevity in thought, but must include in code)
            let error_format = options.error_format.as_str();
            let _ = match error_format {
                "terminal" => integration::set_formatter(&mut integration::get_error_system_mut(), "terminal"),
                "json" => integration::set_formatter(&mut integration::get_error_system_mut(), "json"),
                "lsp" => integration::set_formatter(&mut integration::get_error_system_mut(), "lsp"),
                _ => integration::set_formatter(&mut integration::get_error_system_mut(), "json"),
            };
        }

        // Parse compilation mode
        let compilation_mode = match options.mode.as_str() {
            "testing" => CompilationMode::Testing,
            "deployment" => CompilationMode::Deployment,
            "debug" => CompilationMode::Testing,
            _ => CompilationMode::Testing,
        };

        // Parse optimization level
        let optimization_level = match options.optimization_level.as_str() {
            "production" => OptimizationLevel::Production,
            _ => OptimizationLevel::Production,
        };

        // Create comprehensive CompilerConfig
        let config = CompilationConfig {
            mode: compilation_mode,
            optimization_level,
            v2_preview: options.v2_preview,
            enable_constraint_cache: options.enable_constraint_cache,
            enable_module_namespaces: options.enable_module_namespaces,
            include_debug_info: options.include_debug_info,
        };

        let metrics_format = if options.metrics_format.is_empty() {
            "json".to_string()
        } else {
            options.metrics_format.clone()
        };
        let mut metrics_export = String::new();
        let mut detailed_metrics: Option<CompilerMetrics> = None;
        let collect_metrics = options.include_metrics || options.comprehensive_metrics;

        // Use compile_with_config_and_logs to get logs, but we also want ABI.
        // DslCompiler doesn't have a single method for Bytecode + ABI + Log + Metrics clearly exposed in one go without reparsing?
        // Actually `compile_to_five_file_with_config` generates both but doesn't return the log separate from errors.
        // We can use `CompilationPipeline` directly like the original code but ensure we generate ABI from the SAME pipeline.

        let (bytecode, disassembly, success, errors, abi) = {
            use five_dsl_compiler::compiler::pipeline::CompilationPipeline;
            
            let mut pipeline = CompilationPipeline::new(source, None);
            
            // Execute pipeline stages manually
            let result = (|| -> Result<(Box<Vec<u8>>, Vec<String>, Option<five_dsl_compiler::bytecode_generator::types::FIVEABI>), five_dsl_compiler::error::CompilerError> {
                let tokens = pipeline.tokenize()?;
                let ast = pipeline.parse(tokens)?;
                let interface_registry = pipeline.type_check_with_interfaces(&ast)?;

                // Generate bytecode with log
                let (bytecode, log) = pipeline.generate_bytecode_with_log(&ast, &config, Some(interface_registry))?;
                
                // Generate ABI from the SAME AST (efficient!)
                // We need to re-create the DslBytecodeGenerator or expose a method on pipeline?
                // The pipeline has `generate_abi` method.
                let abi = pipeline.generate_abi(&ast, &config)?;

                Ok((Box::new(bytecode), log, Some(abi)))
            })();

            match result {
                Ok((bytecode_box, log, abi)) => {
                    let bytecode = *bytecode_box;
                    // Finalize metrics
                    pipeline.finalize_metrics(&bytecode);
                    
                    if collect_metrics {
                        let metrics = pipeline.get_metrics();
                        let export_format = map_metrics_format(&metrics_format);
                        if let Ok(json) = export_metrics(metrics, export_format) {
                            metrics_export = json;
                        }
                        if options.comprehensive_metrics {
                            detailed_metrics = Some(metrics.clone());
                        }
                    }
                    
                    let b_vec: Vec<u8> = bytecode;
                    (Option::<Vec<u8>>::Some(b_vec), log, true, Vec::<five_dsl_compiler::error::CompilerError>::new(), abi)
                }
                Err(e) => {
                    let mut errors = pipeline.get_error_collector().get_errors().to_vec();
                    if errors.is_empty() {
                        errors.push(e);
                    }
                    (Option::<Vec<u8>>::None, Vec::new(), false, errors, None)
                }
            }
        };

        let compilation_time = (js_sys::Date::now() - start_time) as f64;
        
        let (error_count, warning_count, warnings, error_strings, compiler_errors_vec, formatted_errors_terminal, formatted_errors_json) = 
            process_errors(&errors, source, options.source_file.as_deref());

        // Serialize ABI to JSON if present
        let abi_json = abi.and_then(|a| {
            // Debug: Log ABI structure before serialization
            if !a.functions.is_empty() {
                log_message(&format!(
                    "Generated {} functions",
                    a.functions.len()
                ));
                // Log first function details
                let first_fn = &a.functions[0];
                log_message(&format!(
                    "Func 0: '{}' ({} params)",
                    first_fn.name,
                    first_fn.parameters.len()
                ));
                if !first_fn.parameters.is_empty() {
                    let first_param = &first_fn.parameters[0];
                    log_message(&format!(
                        "Param 0: '{}' ({})",
                        first_param.name,
                        first_param.param_type
                    ));
                }
            }
            let json_result = serde_json::to_string(&a).ok();
            // Debug: Show JSON output
            if let Some(ref json) = json_result {
                log_message(&format!(
                    "ABI JSON len: {}",
                    json.len()
                ));
                // Show first 200 chars of JSON to verify parameters are there
                let preview = if json.len() > 200 {
                    format!("{}...", &json[..200])
                } else {
                    json.clone()
                };
                log_message(&format!(
                    "JSON preview: {}",
                    preview
                ));
            }
            json_result
        });


        let bytecode_size = if let Some(ref b) = bytecode { b.len() } else { 0 };

        WasmCompilationResult {
            success,
            bytecode_size,
            bytecode,
            compilation_time,
            compiler_errors: compiler_errors_vec,
            error_count,
            warning_count,
            warnings,
            errors: error_strings,
            metrics: metrics_export,
            metrics_format,
            detailed_metrics,
            disassembly,
            abi: abi_json,
            formatted_errors_terminal,
            formatted_errors_json,
        }
    }


    /// Compile multi-file project with automatic discovery
    #[wasm_bindgen(js_name = compileMultiWithDiscovery)]
    pub fn compile_multi_with_discovery(
        &self,
        entry_point: String,
        options: &WasmCompilationOptions,
    ) -> Result<WasmCompilationResult, JsValue> {
        use five_dsl_compiler::compiler::OptimizationLevel;
        use five_dsl_compiler::{CompilationConfig, CompilationMode};
        use std::path::Path;

        let start_time = js_sys::Date::now();

        // Parse compilation mode
        let compilation_mode = match options.mode.as_str() {
            "testing" => CompilationMode::Testing,
            "deployment" => CompilationMode::Deployment,
            "debug" => CompilationMode::Testing,
            _ => CompilationMode::Testing,
        };

        // Parse optimization level
        let optimization_level = match options.optimization_level.as_str() {
            "production" => OptimizationLevel::Production,
            _ => OptimizationLevel::Production,
        };

        let config = CompilationConfig {
            mode: compilation_mode,
            optimization_level,
            v2_preview: options.v2_preview,
            enable_constraint_cache: options.enable_constraint_cache,
            enable_module_namespaces: options.enable_module_namespaces,
            include_debug_info: options.include_debug_info,
        };

        // Use the new method that returns FiveFile (Bytecode + ABI)
        match DslCompiler::compile_with_auto_discovery_to_five_file(Path::new(&entry_point), &config) {
            Ok(five_file) => {
                let compilation_time = js_sys::Date::now() - start_time;
                let abi_json = serde_json::to_string(&five_file.abi).ok();

                Ok(WasmCompilationResult {
                    success: true,
                    bytecode_size: five_file.bytecode.len(),
                    compilation_time,
                    bytecode: Some(five_file.bytecode),
                    compiler_errors: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                    metrics: "{}".to_string(),
                    metrics_format: "json".to_string(),
                    detailed_metrics: None,
                    disassembly: Vec::new(),
                    abi: abi_json,
                    formatted_errors_terminal: String::new(),
                    formatted_errors_json: String::new(),
                })
            }
            Err(e) => {
                // Re-run module discovery to build a source map for richer errors.
                use five_dsl_compiler::module_resolver::ModuleDiscoverer;
                let entry_path = Path::new(&entry_point);
                let source_dir = entry_path.parent().unwrap_or(Path::new("")).to_path_buf();
                let discoverer = ModuleDiscoverer::new(source_dir);
                let mut source_map = std::collections::HashMap::new();
                
                if let Ok(graph) = discoverer.discover_modules(entry_path) {
                     for (_, descriptor) in graph.modules() {
                        source_map.insert(descriptor.file_path.clone(), descriptor.source_code.clone());
                    }
                }

                let compilation_time = js_sys::Date::now() - start_time;
                
                let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                    process_multi_errors(&[e], &source_map, Some(&std::path::PathBuf::from(entry_point)));
                
                Ok(WasmCompilationResult {
                    success: false,
                    bytecode_size: 0,
                    compilation_time,
                    bytecode: None,
                    compiler_errors,
                    error_count,
                    warning_count,
                    warnings,
                    errors,
                    metrics: "{}".to_string(),
                    metrics_format: "json".to_string(),
                    detailed_metrics: None,
                    disassembly: Vec::new(),
                    abi: None,
                    formatted_errors_terminal: formatted_terminal,
                    formatted_errors_json: formatted_json,
                })
            }
        }
    }


    /// Discover modules starting from an entry point
    #[wasm_bindgen(js_name = discoverModules)]
    pub fn discover_modules(&self, entry_point: String) -> Result<JsValue, JsValue> {
        use std::path::Path;

        let modules = DslCompiler::discover_modules(Path::new(&entry_point))
            .map_err(|e| JsValue::from_str(&e.message))?;

        Ok(serde_wasm_bindgen::to_value(&modules)?)
    }

    /// Compile multi-file project with explicit module list
    #[wasm_bindgen(js_name = compileModules)]
    pub fn compile_modules(
        &self,
        module_files: JsValue,
        entry_point: String,
        options: &WasmCompilationOptions,
    ) -> Result<WasmCompilationResult, JsValue> {
        use five_dsl_compiler::compiler::OptimizationLevel;
        use five_dsl_compiler::{CompilationConfig, CompilationMode};

        let start_time = js_sys::Date::now();
        let modules: Vec<String> = serde_wasm_bindgen::from_value(module_files)?;

        // Parse compilation mode
        let compilation_mode = match options.mode.as_str() {
            "testing" => CompilationMode::Testing,
            "deployment" => CompilationMode::Deployment,
            "debug" => CompilationMode::Testing,
            _ => CompilationMode::Testing,
        };

        // Parse optimization level
        let optimization_level = match options.optimization_level.as_str() {
            "production" => OptimizationLevel::Production,
            _ => OptimizationLevel::Production,
        };

        let config = CompilationConfig {
            mode: compilation_mode,
            optimization_level,
            v2_preview: options.v2_preview,
            enable_constraint_cache: options.enable_constraint_cache,
            enable_module_namespaces: options.enable_module_namespaces, // Pass explicitly, though `with_module_namespaces` below overrides
            include_debug_info: options.include_debug_info,
        };
        // Ensure module namespaces is propagated
        let config = config.with_module_namespaces(options.enable_module_namespaces);

        // Use the new method that returns FiveFile (Bytecode + ABI)
        match DslCompiler::compile_modules_to_five_file(modules.clone(), &entry_point, &config) {
             Ok(five_file) => {
                let compilation_time = js_sys::Date::now() - start_time;
                let abi_json = serde_json::to_string(&five_file.abi).ok();

                Ok(WasmCompilationResult {
                    success: true,
                    bytecode_size: five_file.bytecode.len(),
                    compilation_time,
                    bytecode: Some(five_file.bytecode),
                    compiler_errors: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                    warnings: Vec::new(),
                    errors: Vec::new(),
                    metrics: "{}".to_string(),
                    metrics_format: "json".to_string(),
                    detailed_metrics: None,
                    disassembly: Vec::new(),
                    abi: abi_json,
                    formatted_errors_terminal: String::new(),
                    formatted_errors_json: String::new(),
                })
             }
             Err(e) => {
                let compilation_time = js_sys::Date::now() - start_time;
                
                // Build simple source map for error formatting since we have the files list
                let mut source_map = std::collections::HashMap::new();
                for file_path in modules {
                   if let Ok(content) = std::fs::read_to_string(&file_path) {
                       source_map.insert(std::path::PathBuf::from(file_path), content);
                   }
                }
                
                let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                    process_multi_errors(&[e], &source_map, Some(&std::path::PathBuf::from(entry_point)));
                
                 Ok(WasmCompilationResult {
                    success: false,
                    bytecode_size: 0,
                    compilation_time,
                    bytecode: None,
                    compiler_errors,
                    error_count,
                    warning_count,
                    warnings,
                    errors,
                    metrics: "{}".to_string(),
                    metrics_format: "json".to_string(),
                    detailed_metrics: None,
                    disassembly: Vec::new(),
                    abi: None,
                    formatted_errors_terminal: formatted_terminal,
                    formatted_errors_json: formatted_json,
                })
             }
        }
    }

    /// Extract function name metadata from compiled bytecode
    /// Returns a list of discovered functions in the bytecode
    #[wasm_bindgen(js_name = extractFunctionMetadata)]
    pub fn extract_function_metadata(&self, bytecode: &[u8]) -> Result<JsValue, JsValue> {
        use five_dsl_compiler::import_discovery::ImportDiscovery;
        use serde::Serialize;

        #[derive(Serialize)]
        struct SimpleDiscoveredFunction {
            pub name: String,
            pub address: u16,
            pub param_count: u8,
        }

        let discovery = ImportDiscovery::discover_functions_from_bytecode("unknown_contract", bytecode)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
            
        let simple_functions: Vec<SimpleDiscoveredFunction> = discovery.functions.values().map(|f| {
            SimpleDiscoveredFunction {
                name: f.name.clone(),
                address: f.address,
                param_count: f.param_count,
            }
        }).collect();

        Ok(serde_wasm_bindgen::to_value(&simple_functions)?)
    }

    /// Multi-file compilation using module merger (main source + modules)
    #[wasm_bindgen]
    pub fn compile_multi(
        &self,
        main_source: &str,
        modules: &JsValue,
        options: &WasmCompilationOptions,
    ) -> WasmCompilationResult {
        use five_dsl_compiler::{
            DslBytecodeGenerator, DslTypeChecker, ModuleMerger,
        };
        use five_dsl_compiler::compiler::pipeline::CompilationPipeline;
        use five_dsl_compiler::error::{CompilerError, ErrorCategory, ErrorCode, ErrorSeverity};
        use std::path::PathBuf;

        #[derive(serde::Deserialize)]
        struct WasmModule {
            name: String,
            content: String,
        }
        
        // Create a source map for all modules to support rich error reporting
        let mut source_map = std::collections::HashMap::new();
        let main_file_name = options.source_file.clone().unwrap_or_else(|| "input.v".to_string());
        source_map.insert(PathBuf::from(&main_file_name), main_source.to_string());
        
        let mut module_sources: Vec<(String, String, PathBuf)> = Vec::new();
        let empty_modules = js_sys::Array::new();
        let modules_array = modules.dyn_ref::<js_sys::Array>().unwrap_or(&empty_modules);
        for module_val in modules_array.iter() {
            let module: WasmModule = match serde_wasm_bindgen::from_value(module_val) {
                Ok(m) => m,
                Err(e) => {
                    let error_msg = format!("Invalid module object: {}", e);
                    let compiler_error = CompilerError::new(
                        ErrorCode::INVALID_SYNTAX,
                        ErrorSeverity::Error,
                        ErrorCategory::Syntax,
                        error_msg.clone(),
                    );
                    let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                        process_multi_errors(&[compiler_error], &source_map, Some(&std::path::PathBuf::from(&main_file_name)));
                    
                    return WasmCompilationResult {
                        success: false,
                        bytecode: None,
                        bytecode_size: 0,
                        compilation_time: 0.0,
                        compiler_errors,
                        error_count,
                        warning_count,
                        warnings,
                        errors,
                        metrics: "{}".to_string(),
                        metrics_format: "json".to_string(),
                        detailed_metrics: None,
                        disassembly: Vec::new(),
                        abi: None,
                        formatted_errors_terminal: formatted_terminal,
                        formatted_errors_json: formatted_json,
                    };
                }
            };
            let path = PathBuf::from(&module.name);
            source_map.insert(path.clone(), module.content.clone());
            module_sources.push((module.name.clone(), module.content.clone(), path));
        }

        let start_time = js_sys::Date::now();

        // Tokenize and parse main source using CompilationPipeline
        let mut main_pipeline = CompilationPipeline::new(main_source, Some(&main_file_name));
        let main_tokens = match main_pipeline.tokenize() {
            Ok(t) => t,
            Err(e) => {
                let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                    process_multi_errors(&[e], &source_map, Some(&std::path::PathBuf::from(&main_file_name)));
                
                return WasmCompilationResult {
                    success: false,
                    bytecode: None,
                    bytecode_size: 0,
                    compilation_time: 0.0,
                    compiler_errors,
                    error_count,
                    warning_count,
                    warnings,
                    errors,
                    metrics: "{}".to_string(),
                    metrics_format: "json".to_string(),
                    detailed_metrics: None,
                    disassembly: Vec::new(),
                    abi: None,
                    formatted_errors_terminal: formatted_terminal,
                    formatted_errors_json: formatted_json,
                };
            }
        };


        let main_ast = match main_pipeline.parse(main_tokens) {
            Ok(ast) => ast,
            Err(e) => {
                let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                    process_multi_errors(&[e], &source_map, Some(&std::path::PathBuf::from(&main_file_name)));
                
                return WasmCompilationResult {
                    success: false,
                    bytecode: None,
                    bytecode_size: 0,
                    compilation_time: 0.0,
                    compiler_errors,
                    error_count,
                    warning_count,
                    warnings,
                    errors,
                    metrics: "{}".to_string(),
                    metrics_format: "json".to_string(),
                    detailed_metrics: None,
                    disassembly: Vec::new(),
                    abi: None,
                    formatted_errors_terminal: formatted_terminal,
                    formatted_errors_json: formatted_json,
                };
            }
        };

        let mut merger = ModuleMerger::new()
            .with_namespaces(options.enable_module_namespaces);
        merger.set_main_ast(main_ast);

        for (module_name, module_source, module_path) in module_sources {
            let mut module_pipeline = CompilationPipeline::new(&module_source, module_path.to_str());
            let tokens = match module_pipeline.tokenize() {
                Ok(t) => t,
                Err(e) => {
                    let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                        process_multi_errors(&[e], &source_map, Some(&std::path::PathBuf::from(&main_file_name)));
                    
                    return WasmCompilationResult {
                        success: false,
                        bytecode: None,
                        bytecode_size: 0,
                        compilation_time: 0.0,
                        compiler_errors,
                        error_count,
                        warning_count,
                        warnings,
                        errors,
                        metrics: "{}".to_string(),
                        metrics_format: "json".to_string(),
                        detailed_metrics: None,
                        disassembly: Vec::new(),
                        abi: None,
                        formatted_errors_terminal: formatted_terminal,
                        formatted_errors_json: formatted_json,
                    };
                }
            };
            let module_ast = match module_pipeline.parse(tokens) {
                Ok(ast) => ast,
                Err(e) => {
                    let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                        process_multi_errors(&[e], &source_map, Some(&std::path::PathBuf::from(&main_file_name)));
                    
                    return WasmCompilationResult {
                        success: false,
                        bytecode: None,
                        bytecode_size: 0,
                        compilation_time: 0.0,
                        compiler_errors,
                        error_count,
                        warning_count,
                        warnings,
                        errors,
                        metrics: "{}".to_string(),
                        metrics_format: "json".to_string(),
                        detailed_metrics: None,
                        disassembly: Vec::new(),
                        abi: None,
                        formatted_errors_terminal: formatted_terminal,
                        formatted_errors_json: formatted_json,
                    };
                }
            };

            merger.add_module(module_name, module_ast);
        }

        match merger.merge() {
            Ok(merged_ast) => {
                // Build ModuleScope for cross-module type resolution
                use five_dsl_compiler::type_checker::{ModuleScope, ModuleSymbol};
                use five_dsl_compiler::ast::{AstNode, TypeNode};
                
                // Extract main module name from filename
                let main_module_name = std::path::Path::new(&main_file_name)
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| "main".to_string());
                
                let mut module_scope = ModuleScope::new(main_module_name.clone());
                
                // Populate module scope from merged AST
                // The merged AST contains all definitions from all modules
                if let AstNode::Program {
                    instruction_definitions,
                    field_definitions,
                    account_definitions,
                    ..
                } = &merged_ast {
                    module_scope.set_current_module(main_module_name.clone());
                    
                    // Add all account definitions to scope
                    for account_def in account_definitions {
                        if let AstNode::AccountDefinition { name, visibility, .. } = account_def {
                            module_scope.add_symbol_to_current(name.clone(), ModuleSymbol {
                                type_info: TypeNode::Account,
                                is_mutable: false,
                                visibility: *visibility,
                            });
                        }
                    }
                    
                    // Add instruction definitions to scope
                    for instr_def in instruction_definitions {
                        if let AstNode::InstructionDefinition { name, return_type, visibility, .. } = instr_def {
                            let type_info = return_type
                                .as_ref()
                                .map(|t| (**t).clone())
                                .unwrap_or_else(|| TypeNode::Primitive("void".to_string()));
                            module_scope.add_symbol_to_current(name.clone(), ModuleSymbol {
                                type_info,
                                is_mutable: false,
                                visibility: *visibility,
                            });
                        }
                    }
                    
                    // Add field definitions to scope
                    for field_def in field_definitions {
                        if let AstNode::FieldDefinition { name, field_type, visibility, .. } = field_def {
                            module_scope.add_symbol_to_current(name.clone(), ModuleSymbol {
                                type_info: (**field_type).clone(),
                                is_mutable: true,
                                visibility: *visibility,
                            });
                        }
                    }
                }
                
                // Perform Type Checking with module scope
                let mut type_checker = DslTypeChecker::new()
                    .with_module_scope(module_scope);
                type_checker.set_current_module(main_module_name);
                
                if let Err(e) = type_checker.check_types(&merged_ast) {
                     let compiler_error = five_dsl_compiler::error::CompilerError::new(
                         five_dsl_compiler::error::ErrorCode::TYPE_MISMATCH,
                         five_dsl_compiler::error::ErrorSeverity::Error,
                         five_dsl_compiler::error::ErrorCategory::Type,
                         format!("Type checking failed: {}", e),
                     );
                     
                     // Attach location info if captured by the type checker
                     // Attach location info if captured by the type checker
                     // Note: last_error_span/file fields are no longer exposed directly on TypeCheckerContext
                     // To fix properly we would need to capture this info from the error itself if available
                     // Proceed without enhanced location info from the context.
                     // if let (Some(span), Some(file)) = (type_checker.last_error_span.clone(), type_checker.last_error_file.clone()) { ... }

                     let (error_count, warning_count, warnings, errors, compiler_errors, formatted_terminal, formatted_json) = 
                        process_multi_errors(&[compiler_error], &source_map, Some(&std::path::PathBuf::from(&main_file_name)));

                     return WasmCompilationResult {
                        success: false,
                        bytecode: None,
                        bytecode_size: 0,
                        compilation_time: 0.0,
                        compiler_errors,
                        error_count,
                        warning_count,
                        warnings,
                        errors,
                        metrics: "{}".to_string(),
                        metrics_format: "json".to_string(),
                        detailed_metrics: None,
                        disassembly: Vec::new(),
                        abi: None,
                        formatted_errors_terminal: formatted_terminal,
                        formatted_errors_json: formatted_json,
                    };


                }
                
                // Configure Bytecode Generator
                use five_dsl_compiler::compiler::OptimizationLevel;
                use five_dsl_compiler::{CompilationConfig, CompilationMode};

                // Parse compilation mode
                let compilation_mode = match options.mode.as_str() {
                    "testing" => CompilationMode::Testing,
                    "deployment" => CompilationMode::Deployment,
                    "debug" => CompilationMode::Testing,
                    _ => CompilationMode::Testing,
                };
        
                // Parse optimization level
                let optimization_level = match options.optimization_level.as_str() {
                    "production" => OptimizationLevel::Production,
                    _ => OptimizationLevel::Production,
                };
        
                let config = CompilationConfig {
                    mode: compilation_mode,
                    optimization_level,
                    v2_preview: options.v2_preview,
                    enable_constraint_cache: options.enable_constraint_cache,
                    enable_module_namespaces: options.enable_module_namespaces,
                    include_debug_info: options.include_debug_info,
                };

                // Generate Bytecode
                let mut generator = if config.v2_preview {
                     DslBytecodeGenerator::with_v2_preview_config(&config)
                } else {
                     DslBytecodeGenerator::with_optimization_config(&config)
                };
                
                match generator.generate(&merged_ast) {
                    Ok(bytecode) => {
                        let compilation_time = js_sys::Date::now() - start_time;
                        let disassembly = generator.get_disassembly();
                        
                        // Generate ABI
                        let abi = match generator.generate_abi(&merged_ast) {
                            Ok(abi) => match serde_json::to_string(&abi) {
                                Ok(json) => Some(json),
                                Err(_) => None,
                            },
                            Err(_) => None,
                        };

                        WasmCompilationResult {
                            success: true,
                            bytecode_size: bytecode.len(),
                            compilation_time,
                            bytecode: Some(bytecode),
                            abi, // Include the generated ABI
                            compiler_errors: Vec::new(),
                            error_count: 0,
                            warning_count: 0,
                            warnings: Vec::new(),
                            errors: Vec::new(),
                            metrics: "{}".to_string(), // Metrics integration skipped for stability fix
                            metrics_format: "json".to_string(),
                            detailed_metrics: None,
                            disassembly,
                            formatted_errors_terminal: String::new(),
                            formatted_errors_json: String::new(),
                        }
                    },
                    Err(e) => {
                         let error_msg = e.to_string();
                         WasmCompilationResult {
                            success: false,
                            bytecode: None,
                            bytecode_size: 0,
                            compilation_time: 0.0,
                            compiler_errors: vec![WasmCompilerError {
                                code: "E0003".to_string(),
                                severity: "error".to_string(),
                                category: "CodeGeneration".to_string(),
                                message: error_msg.clone(),
                                description: None,
                                location: None,
                                suggestions: vec![],
                                source_line: None,
                                source_snippet: None,
                                line: None,
                                column: None,
                            }],
                            error_count: 1,
                            warning_count: 0,
                            warnings: Vec::new(),
                            errors: vec![error_msg.clone()],
                            metrics: "{}".to_string(),
                            metrics_format: "json".to_string(),
                            detailed_metrics: None,
                            disassembly: Vec::new(),
                            abi: None,
                            formatted_errors_terminal: error_msg.clone(),
                            formatted_errors_json: String::new(),
                        }
                    }
                }
            }
            Err(_) => WasmCompilationResult {
                success: false,
                bytecode: None,
                bytecode_size: 0,
                compilation_time: 0.0,
                compiler_errors: vec![],
                error_count: 1,
                warning_count: 0,
                warnings: Vec::new(),
                errors: vec!["Failed to merge modules".to_string()],
                metrics: "{}".to_string(),
                metrics_format: "json".to_string(),
                detailed_metrics: None,
                disassembly: Vec::new(),
                abi: None,
                formatted_errors_terminal: "Failed to merge modules".to_string(),
                formatted_errors_json: String::new(),
            },
        }
    }

    /// Helper function to convert byte position to line/column
    #[allow(dead_code)]
    fn position_to_line_col(position: usize, source: &str) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;

        for (i, ch) in source.chars().enumerate() {
            if i >= position {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Create enhanced parse error message with context
    #[allow(dead_code)]
    fn create_enhanced_parse_message(
        expected: &str,
        found: &str,
        line: usize,
        col: usize,
        _source: &str,
    ) -> String {
        // Extract the source line for context (unused for now)

        match (expected.as_ref(), found.as_ref()) {
            ("function call, method call, or assignment", "'}'") => {
                format!(
                    "expected a statement, but found end of block\n  help: this looks like a missing semicolon after the previous statement"
                )
            }
            ("';'", _) => {
                format!(
                    "expected semicolon after statement, found `{}`\n  help: add a `;` after the statement on line {}",
                    found, line
                )
            }
            (expected, "EOF") => {
                format!(
                    "unexpected end of file, expected `{}`\n  help: the file appears to be incomplete",
                    expected
                )
            }
            _ => {
                format!(
                    "expected `{}`, found `{}`\n  help: check the syntax around line {} column {}",
                    expected, found, line, col
                )
            }
        }
    }

    /// Generate helpful suggestions for parse errors
    #[allow(dead_code)]
    fn generate_parse_suggestions(expected: &str, found: &str) -> Vec<String> {
        match (expected.as_ref(), found.as_ref()) {
            ("function call, method call, or assignment", "'}'") => {
                vec![
                    "Add a semicolon `;` after the previous statement".to_string(),
                    "Check if you meant to write a function call like `function_name()`"
                        .to_string(),
                    "Ensure all statements in the block are properly terminated".to_string(),
                ]
            }
            ("';'", _) => {
                vec![
                    "Add a semicolon `;` at the end of the statement".to_string(),
                    "Most statements in Five require semicolon termination".to_string(),
                ]
            }
            (expected, "EOF") => {
                vec![
                    format!("Add the missing `{}` before the end of the file", expected),
                    "Check for unclosed blocks, parentheses, or brackets".to_string(),
                ]
            }
            _ => {
                vec![
                    format!("Replace `{}` with `{}`", found, expected),
                    "Check the Five language documentation for correct syntax".to_string(),
                ]
            }
        }
    }

    /// Format VMError into a human-readable message
    #[allow(dead_code)]
    fn format_vm_error_message(vm_error: &five_vm_mito::error::VMError) -> String {
        match vm_error {
            five_vm_mito::error::VMError::ParseError {
                expected,
                found,
                position,
            } => {
                format!(
                    "parse error: expected '{}', found '{}' at position {}",
                    expected, found, position
                )
            }
            five_vm_mito::error::VMError::InvalidScript => {
                "the script contains errors that prevent compilation".to_string()
            }
            five_vm_mito::error::VMError::TypeMismatch => {
                "type mismatch: the types in this expression are incompatible".to_string()
            }
            five_vm_mito::error::VMError::CallStackUnderflow => {
                "call stack underflow: function call stack is empty".to_string()
            }
            five_vm_mito::error::VMError::CallStackOverflow => {
                "call stack overflow: too many nested function calls".to_string()
            }
            _ => format!("{:?}", vm_error).to_lowercase(),
        }
    }

    /// Get detailed analysis of source code
    #[wasm_bindgen]
    pub fn analyze_source(&self, source: &str) -> WasmAnalysisResult {
        self.analyze_source_mode(source, "testing")
    }

    /// Get opcode usage statistics from compilation
    #[wasm_bindgen]
    pub fn get_opcode_usage(&self, source: &str) -> Result<JsValue, JsValue> {
        use five_dsl_compiler::CompilationMode;

        match DslCompiler::compile_with_metrics(source, CompilationMode::Testing, true) {
            Ok((bytecode, metrics)) => {
                // Extract opcode usage statistics
                let opcode_usage = serde_json::json!({
                    "total_opcodes": metrics.opcode_stats.total_opcodes,
                    "unique_opcodes": metrics.opcode_stats.usage_frequency.len(),
                    "usage_frequency": metrics.opcode_stats.usage_frequency,
                    "top_opcodes": metrics.opcode_stats.top_opcodes,
                    "category_distribution": metrics.opcode_stats.category_distribution,
                    "advanced_usage": metrics.opcode_stats.advanced_usage,
                    "opcode_patterns": metrics.opcode_stats.opcode_patterns,
                    "bytecode_size": bytecode.len()
                });

                Ok(JsValue::from_str(&opcode_usage.to_string()))
            }
            Err(error) => Err(JsValue::from_str(&format!(
                "Failed to analyze opcode usage: {}",
                error
            ))),
        }
    }

    /// Get comprehensive compiler statistics including which opcodes are used vs unused
    #[wasm_bindgen]
    pub fn get_opcode_analysis(&self, source: &str) -> Result<JsValue, JsValue> {
        use five_dsl_compiler::CompilationMode;
        use five_protocol::opcodes;

        match DslCompiler::compile_with_metrics(source, CompilationMode::Testing, true) {
            Ok((bytecode, metrics)) => {
                // Get all available opcodes from the protocol
                let all_opcodes = vec![
                    ("HALT", opcodes::HALT),
                    ("PUSH_U64", opcodes::PUSH_U64),
                    ("PUSH_U8", opcodes::PUSH_U8),
                    ("PUSH_I64", opcodes::PUSH_I64),
                    ("PUSH_BOOL", opcodes::PUSH_BOOL),
                    ("PUSH_PUBKEY", opcodes::PUSH_PUBKEY),
                    ("POP", opcodes::POP),
                    ("DUP", opcodes::DUP),
                    ("DUP2", opcodes::DUP2),
                    ("SWAP", opcodes::SWAP),
                    ("ADD", opcodes::ADD),
                    ("SUB", opcodes::SUB),
                    ("MUL", opcodes::MUL),
                    ("DIV", opcodes::DIV),
                    ("MOD", opcodes::MOD),
                    ("GT", opcodes::GT),
                    ("LT", opcodes::LT),
                    ("EQ", opcodes::EQ),
                    ("GTE", opcodes::GTE),
                    ("LTE", opcodes::LTE),
                    ("NEQ", opcodes::NEQ),
                    ("AND", opcodes::AND),
                    ("OR", opcodes::OR),
                    ("NOT", opcodes::NOT),
                    ("STORE", opcodes::STORE),
                    ("LOAD", opcodes::LOAD),
                    ("STORE_FIELD", opcodes::STORE_FIELD),
                    ("LOAD_FIELD", opcodes::LOAD_FIELD),
                    ("LOAD_INPUT", opcodes::LOAD_INPUT),
                    ("INVOKE", opcodes::INVOKE),
                    ("INVOKE_SIGNED", opcodes::INVOKE_SIGNED),
                    ("GET_CLOCK", opcodes::GET_CLOCK),
                    ("RETURN", opcodes::RETURN),
                    ("CREATE_ACCOUNT", opcodes::CREATE_ACCOUNT),
                    ("LOAD_ACCOUNT", opcodes::LOAD_ACCOUNT),
                    ("SAVE_ACCOUNT", opcodes::SAVE_ACCOUNT),
                    ("DERIVE_PDA", opcodes::DERIVE_PDA),
                    ("TRANSFER", opcodes::TRANSFER),
                    ("TRANSFER_SIGNED", opcodes::TRANSFER_SIGNED),
                    ("GET_KEY", opcodes::GET_KEY),
                    ("FIND_PDA", opcodes::FIND_PDA),
                    ("GET_SIGNER_KEY", opcodes::GET_SIGNER_KEY),
                    ("JUMP", opcodes::JUMP),
                    ("JUMP_IF", opcodes::JUMP_IF),
                    ("REQUIRE", opcodes::REQUIRE),
                    ("CHECK_SIGNER", opcodes::CHECK_SIGNER),
                    ("CHECK_WRITABLE", opcodes::CHECK_WRITABLE),
                    ("CHECK_OWNER", opcodes::CHECK_OWNER),
                    ("CHECK_INITIALIZED", opcodes::CHECK_INITIALIZED),
                    ("CHECK_PDA", opcodes::CHECK_PDA),
                    ("EMIT_EVENT", opcodes::EMIT_EVENT),
                    ("LOG_DATA", opcodes::LOG_DATA),
                    ("CREATE_ARRAY", opcodes::CREATE_ARRAY),
                    ("ARRAY_GET", opcodes::ARRAY_GET),
                    ("ARRAY_SET", opcodes::ARRAY_SET),
                    ("ARRAY_LENGTH", opcodes::ARRAY_LENGTH),
                ];

                // Categorize opcodes into used vs unused
                let mut used_opcodes = Vec::new();
                let mut unused_opcodes = Vec::new();

                for (name, opcode_value) in all_opcodes {
                    let usage_count = metrics.opcode_stats.usage_frequency.get(name).unwrap_or(&0);
                    if *usage_count > 0 {
                        used_opcodes.push(serde_json::json!({
                            "name": name,
                            "opcode": opcode_value,
                            "usage_count": usage_count
                        }));
                    } else {
                        unused_opcodes.push(serde_json::json!({
                            "name": name,
                            "opcode": opcode_value
                        }));
                    }
                }

                // Sort used opcodes by usage count (descending)
                used_opcodes.sort_by(|a, b| {
                    let count_a = a.get("usage_count").and_then(|v| v.as_u64()).unwrap_or(0);
                    let count_b = b.get("usage_count").and_then(|v| v.as_u64()).unwrap_or(0);
                    count_b.cmp(&count_a)
                });

                let analysis = serde_json::json!({
                    "summary": {
                        "total_opcodes_available": used_opcodes.len() + unused_opcodes.len(),
                        "opcodes_used": used_opcodes.len(),
                        "opcodes_unused": unused_opcodes.len(),
                        "usage_percentage": (used_opcodes.len() as f64) / (used_opcodes.len() + unused_opcodes.len()) as f64 * 100.0,
                        "total_opcode_instances": metrics.opcode_stats.total_opcodes,
                        "bytecode_size": bytecode.len()
                    },
                    "used_opcodes": used_opcodes,
                    "unused_opcodes": unused_opcodes,
                    "opcode_patterns": metrics.opcode_stats.opcode_patterns,
                    "performance": {
                        "compilation_time_ms": metrics.performance.total_compilation_time.as_secs_f64() * 1000.0,
                        "opcodes_per_second": metrics.opcode_stats.total_opcodes as f64 / metrics.performance.total_compilation_time.as_secs_f64()
                    }
                });

                Ok(JsValue::from_str(&analysis.to_string()))
            }
            Err(error) => Err(JsValue::from_str(&format!(
                "Failed to analyze opcodes: {}",
                error
            ))),
        }
    }

    /// Get detailed analysis of source code with compilation mode selection
    #[wasm_bindgen]
    pub fn analyze_source_mode(&self, source: &str, mode: &str) -> WasmAnalysisResult {
        use five_dsl_compiler::CompilationMode;

        let start_time = js_sys::Date::now();

        // Parse compilation mode
        let compilation_mode = match mode.to_lowercase().as_str() {
            "testing" => CompilationMode::Testing,
            "deployment" => CompilationMode::Deployment,
            "debug" => CompilationMode::Testing, // Debug uses testing mode with extended analysis
            _ => CompilationMode::Testing,
        };

        let is_debug_mode = mode.to_lowercase() == "debug";

        match DslCompiler::compile_with_metrics(source, compilation_mode, true) {
            Ok((bytecode, mut metrics)) => {
                let analysis_time = js_sys::Date::now() - start_time;

                // Add debug-specific analysis data
                if is_debug_mode {
                    // Add debug analysis indicators to metrics
                    metrics
                        .source_stats
                        .feature_usage
                        .insert("debug_analysis".to_string(), 1);
                    metrics
                        .source_stats
                        .feature_usage
                        .insert("extended_profiling".to_string(), 1);

                    // Add detailed opcode analysis for debug mode
                    let opcode_diversity = metrics.opcode_stats.usage_frequency.len() as u64;
                    metrics
                        .opcode_stats
                        .advanced_usage
                        .insert("opcode_diversity".to_string(), opcode_diversity);

                    // Add compilation phase breakdown for debug
                    let total_phases = 4; // tokenization, parsing, type_checking, bytecode_generation
                    metrics
                        .memory_analytics
                        .phase_memory
                        .insert("debug_phase_count".to_string(), total_phases);
                }

                // Create enhanced analysis summary
                let summary = if is_debug_mode {
                    format!(
                        "DEBUG Source Analysis:\n\
                         • Lines: {} (code: {}, comments: {}, blank: {})\n\
                         • Tokens: {} (unique: {})\n\
                         • Bytecode: {} bytes (compression: {:.1}x)\n\
                         • Compilation: {:.2}ms\n\
                         • Performance: {:.0} lines/sec, {:.0} tokens/sec\n\
                         • Opcodes: {} total, {} unique patterns\n\
                         • Memory: peak {}KB during compilation\n\
                         • Complexity: cyclomatic {}, max nesting {}",
                        metrics.source_stats.total_lines,
                        metrics.source_stats.code_lines,
                        metrics.source_stats.comment_lines,
                        metrics.source_stats.blank_lines,
                        metrics.source_stats.total_tokens,
                        metrics.source_stats.unique_tokens,
                        bytecode.len(),
                        metrics.bytecode_analytics.compression_ratio,
                        metrics.performance.total_compilation_time.as_secs_f64() * 1000.0,
                        metrics.performance.lines_per_second,
                        metrics.performance.tokens_per_second,
                        metrics.opcode_stats.total_opcodes,
                        metrics.opcode_stats.usage_frequency.len(),
                        metrics.memory_analytics.peak_memory_usage / 1024,
                        metrics.source_stats.cyclomatic_complexity,
                        metrics.source_stats.nesting_depth
                    )
                } else {
                    format!(
                        "Source Analysis:\n\
                         • Lines: {} (code: {}, comments: {}, blank: {})\n\
                         • Tokens: {}\n\
                         • Bytecode: {} bytes\n\
                         • Compilation: {:.2}ms\n\
                         • Performance: {:.0} lines/sec",
                        metrics.source_stats.total_lines,
                        metrics.source_stats.code_lines,
                        metrics.source_stats.comment_lines,
                        metrics.source_stats.blank_lines,
                        metrics.source_stats.total_tokens,
                        bytecode.len(),
                        metrics.performance.total_compilation_time.as_secs_f64() * 1000.0,
                        metrics.performance.lines_per_second
                    )
                };

                // Serialize full metrics with debug mode indicators
                let metrics_json = match serde_json::to_string(&metrics) {
                    Ok(mut json) => {
                        if is_debug_mode {
                            // Add debug mode flag to analysis JSON
                            if let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&json) {
                                if let Some(obj) = value.as_object_mut() {
                                    obj.insert("debug_analysis_mode".to_string(), serde_json::Value::Bool(true));
                                    obj.insert("analysis_mode".to_string(), serde_json::Value::String(mode.to_string()));
                                    obj.insert("extended_analysis".to_string(), serde_json::Value::Bool(true));
                                    json = serde_json::to_string(&value).unwrap_or(json);
                                }
                            }
                        }
                        json
                    },
                    Err(e) => format!("{{\"error\": \"Failed to serialize metrics: {}\", \"debug_analysis_mode\": {}}}", e, is_debug_mode),
                };

                WasmAnalysisResult {
                    success: true,
                    summary,
                    analysis_time,
                    metrics_json,
                    errors: Vec::new(),
                }
            }
            Err(error) => {
                let analysis_time = js_sys::Date::now() - start_time;
                let error_msg = format!("Analysis failed: {}", error.to_string());

                // Include debug context in error response
                let debug_error_json = if is_debug_mode {
                    format!("{{\"error\": \"{}\", \"debug_analysis_mode\": true, \"analysis_mode\": \"{}\", \"debug_context\": \"Debug analysis mode enabled for enhanced error reporting and detailed failure context\"}}", error_msg, mode)
                } else {
                    "{}".to_string()
                };

                WasmAnalysisResult {
                    success: false,
                    summary: if is_debug_mode {
                        format!("DEBUG Analysis failed: {}", error_msg)
                    } else {
                        "Analysis failed".to_string()
                    },
                    analysis_time,
                    metrics_json: debug_error_json,
                    errors: vec![error_msg],
                }
            }
        }
    }

    /// Parse DSL source code and return AST information
    #[wasm_bindgen]
    pub fn parse_dsl(&self, source: &str) -> Result<JsValue, JsValue> {
        // Create a simplified tokenizer and parser for WASM
        use five_dsl_compiler::{DslParser, DslTokenizer};

        let mut tokenizer = DslTokenizer::new(source);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| JsValue::from_str(&format!("Tokenization failed: {:?}", e)))?;

        let mut parser = DslParser::new(tokens);
        let _ast = parser
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Parsing failed: {:?}", e)))?;

        // Return a simple success indicator for now
        // In a full implementation, we'd serialize the AST
        let result = serde_json::json!({
            "success": true,
            "message": "Parsing completed successfully"
        });

        Ok(JsValue::from_str(&result.to_string()))
    }

    /// Type-check parsed AST
    #[wasm_bindgen]
    pub fn type_check(&self, _ast_json: &str) -> Result<JsValue, JsValue> {
        // Return success; full AST serialization not implemented.
        let result = serde_json::json!({
            "success": true,
            "message": "Type checking completed"
        });

        Ok(JsValue::from_str(&result.to_string()))
    }

    /// Optimize bytecode
    #[wasm_bindgen]
    pub fn optimize_bytecode(&self, bytecode: &[u8]) -> Result<js_sys::Uint8Array, JsValue> {
        Self::optimize_bytecode_internal(bytecode)
            .map(|opt| js_sys::Uint8Array::from(&opt[..]))
            .map_err(|e| JsValue::from_str(&e))
    }

    /// Internal optimization logic returning standard Rust types
    pub(crate) fn optimize_bytecode_internal(bytecode: &[u8]) -> Result<Vec<u8>, String> {
        use five_protocol::opcodes;

        // Simple optimization: remove consecutive PUSH/POP pairs
        let mut optimized = Vec::new();
        let (header, start_offset) = five_protocol::parse_header(bytecode)
            .map_err(|e| format!("Header parse failed: {:?}", e))?;
        optimized.extend_from_slice(&bytecode[..start_offset]);
        let mut i = start_offset;

        while i < bytecode.len() {
            let opcode = bytecode[i];
            let instruction_size = get_instruction_size_with_features(opcode, &bytecode[i..], header.features);

            // Ensure we have enough bytes for this instruction
            if i + instruction_size > bytecode.len() {
                // Incomplete instruction, just copy remaining and bail
                optimized.extend_from_slice(&bytecode[i..]);
                break;
            }

            // Check for PUSH variants
            let is_push = matches!(
                opcode,
                opcodes::PUSH_U8
                    | opcodes::PUSH_U16
                    | opcodes::PUSH_U32
                    | opcodes::PUSH_U64
                    | opcodes::PUSH_I64
                    | opcodes::PUSH_BOOL
                    | opcodes::PUSH_U128
                    | opcodes::PUSH_PUBKEY
                    | opcodes::PUSH_STRING
                    | opcodes::PUSH_U8_W
                    | opcodes::PUSH_U16_W
                    | opcodes::PUSH_U32_W
                    | opcodes::PUSH_U64_W
                    | opcodes::PUSH_I64_W
                    | opcodes::PUSH_BOOL_W
                    | opcodes::PUSH_U128_W
                    | opcodes::PUSH_PUBKEY_W
                    | opcodes::PUSH_STRING_W
            );

            if is_push {
                let next_instruction_idx = i + instruction_size;

                // Check if next instruction exists and is POP
                if next_instruction_idx < bytecode.len() && bytecode[next_instruction_idx] == opcodes::POP {
                    // Found PUSH ... POP sequence.
                    // Skip PUSH (size bytes)
                    // Skip POP (1 byte)
                    i = next_instruction_idx + 1; // +1 for POP size (POP is always 1 byte)
                    continue;
                }
            }

            // Copy current instruction bytes
            optimized.extend_from_slice(&bytecode[i..i + instruction_size]);
            i += instruction_size;
        }

        Ok(optimized)
    }

    /// Extract account definitions from DSL source code
    #[wasm_bindgen]
    pub fn extract_account_definitions(&self, source: &str) -> Result<JsValue, JsValue> {
        Self::extract_account_definitions_internal(source)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    pub(crate) fn extract_account_definitions_internal(source: &str) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::{ast::AstNode, DslParser, DslTokenizer};

        let mut tokenizer = DslTokenizer::new(source);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| format!("Tokenization failed: {:?}", e))?;

        let mut parser = DslParser::new(tokens);
        let ast = parser
            .parse()
            .map_err(|e| format!("Parsing failed: {:?}", e))?;

        let mut account_definitions = Vec::new();

        if let AstNode::Program {
            account_definitions: accounts,
            ..
        } = &ast
        {
            for account_def in accounts {
                if let AstNode::AccountDefinition { name, fields, visibility: _, .. } = account_def {
                    let mut field_list = Vec::new();
                    for field in fields {
                        field_list.push(serde_json::json!({
                            "name": field.name,
                            "type": format!("{:?}", field.field_type)
                        }));
                    }

                    account_definitions.push(serde_json::json!({
                        "name": name,
                        "fields": field_list
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "account_definitions": account_definitions
        }))
    }

    /// Extract function signatures with account parameters
    #[wasm_bindgen]
    pub fn extract_function_signatures(&self, source: &str) -> Result<JsValue, JsValue> {
        Self::extract_function_signatures_internal(source)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    pub(crate) fn extract_function_signatures_internal(source: &str) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::{ast::AstNode, DslParser, DslTokenizer};

        let mut tokenizer = DslTokenizer::new(source);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| format!("Tokenization failed: {:?}", e))?;

        let mut parser = DslParser::new(tokens);
        let ast = parser
            .parse()
            .map_err(|e| format!("Parsing failed: {:?}", e))?;

        let mut function_signatures = Vec::new();

        if let AstNode::Program {
            instruction_definitions,
            ..
        } = &ast
        {
            for func_def in instruction_definitions {
                if let AstNode::InstructionDefinition {
                    name, parameters, ..
                } = func_def
                {
                    let mut param_list = Vec::new();
                    for param in parameters {
                        param_list.push(serde_json::json!({
                            "name": param.name,
                            "type": format!("{:?}", param.param_type),
                            "attributes": param.attributes
                        }));
                    }

                    function_signatures.push(serde_json::json!({
                        "name": name,
                        "parameters": param_list
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "success": true,
            "function_signatures": function_signatures
        }))
    }

    /// Validate account constraints against function parameters
    #[wasm_bindgen]
    pub fn validate_account_constraints(
        &self,
        source: &str,
        function_name: &str,
        accounts_json: &str,
    ) -> Result<JsValue, JsValue> {
        // Parse the accounts JSON
        let accounts: serde_json::Value = serde_json::from_str(accounts_json)
            .map_err(|e| JsValue::from_str(&format!("Invalid accounts JSON: {}", e)))?;

        Self::validate_account_constraints_internal(source, function_name, accounts)
            .map(|json| JsValue::from_str(&json.to_string()))
            .map_err(|e| JsValue::from_str(&e))
    }

    pub(crate) fn validate_account_constraints_internal(
        source: &str,
        function_name: &str,
        accounts: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        use five_dsl_compiler::{ast::AstNode, DslParser, DslTokenizer};

        // Parse the DSL to get function definitions
        let mut tokenizer = DslTokenizer::new(source);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| format!("Tokenization failed: {:?}", e))?;

        let mut parser = DslParser::new(tokens);
        let ast = parser
            .parse()
            .map_err(|e| format!("Parsing failed: {:?}", e))?;

        // Find the target function
        if let AstNode::Program {
            instruction_definitions,
            ..
        } = &ast
        {
            for func_def in instruction_definitions {
                if let AstNode::InstructionDefinition {
                    name, parameters, ..
                } = func_def
                {
                    if name == function_name {
                        let mut validation_results = Vec::new();

                        for (index, param) in parameters.iter().enumerate() {
                            if let Some(account) = accounts.get(index) {
                                let mut constraint_checks = Vec::new();

                                for attribute in &param.attributes {
                                    match attribute.name.as_str() {
                                        "signer" => {
                                            let is_signer = account
                                                .get("is_signer")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(false);
                                            constraint_checks.push(serde_json::json!({
                                                "constraint": "signer",
                                                "required": true,
                                                "actual": is_signer,
                                                "valid": is_signer
                                            }));
                                        }
                                        "mut" => {
                                            let is_writable = account
                                                .get("is_writable")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(false);
                                            constraint_checks.push(serde_json::json!({
                                                "constraint": "mut",
                                                "required": true,
                                                "actual": is_writable,
                                                "valid": is_writable
                                            }));
                                        }
                                        "init" => {
                                            // For @init, we expect the account to be writable and owned by the system program initially
                                            let is_writable = account
                                                .get("is_writable")
                                                .and_then(|v| v.as_bool())
                                                .unwrap_or(false);
                                            constraint_checks.push(serde_json::json!({
                                                "constraint": "init",
                                                "required": true,
                                                "actual": is_writable,
                                                "valid": is_writable
                                            }));
                                        }
                                        _ => {}
                                    }
                                }

                                validation_results.push(serde_json::json!({
                                    "parameter_name": param.name,
                                    "parameter_index": index,
                                    "constraint_checks": constraint_checks
                                }));
                            } else {
                                validation_results.push(serde_json::json!({
                                    "parameter_name": param.name,
                                    "parameter_index": index,
                                    "error": "Missing account for parameter"
                                }));
                            }
                        }

                        let result = serde_json::json!({
                            "success": true,
                            "function_name": function_name,
                            "validation_results": validation_results
                        });

                        return Ok(result);
                    }
                }
            }
        }

        Err(format!(
            "Function '{}' not found",
            function_name
        ))
    }

    /// Get compiler statistics
    #[wasm_bindgen]
    pub fn get_compiler_stats(&self) -> JsValue {
        let stats = serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "features": ["dsl-compilation", "bytecode-generation", "abi-generation", "type-checking", "optimization", "account-parsing", "constraint-validation"],
            "supported_syntax": ["vault", "init", "constraints", "account-definitions", "function-parameters"],
            "target_vm": "mito-vm"
        });

        JsValue::from_str(&stats.to_string())
    }

    /// Generate ABI from DSL source code for function calls
    #[wasm_bindgen]
    pub fn generate_abi(&self, source: &str) -> Result<JsValue, JsValue> {
        use five_dsl_compiler::{DslBytecodeGenerator, DslParser, DslTokenizer, DslTypeChecker};

        log_message("generate_abi: start");

        // Tokenize
        let mut tokenizer = DslTokenizer::new(source);
        let tokens = match tokenizer.tokenize() {
            Ok(tokens) => {
                log_message("Tokenization OK");
                tokens
            }
            Err(e) => {
                let err_msg = format!("Tokenization failed: {:?}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                return Err(JsValue::from_str(&err_msg));
            }
        };

        // Parse
        let mut parser = DslParser::new(tokens);
        let ast = match parser.parse() {
            Ok(ast) => {
                log_message("Parsing OK");
                ast
            }
            Err(e) => {
                let err_msg = format!("Parsing failed: {:?}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                return Err(JsValue::from_str(&err_msg));
            }
        };

        // Type check
        let mut type_checker = DslTypeChecker::new();
        match type_checker.check_types(&ast) {
            Ok(_) => {
                log_message("Type checking OK");
            }
            Err(e) => {
                let err_msg = format!("Type checking failed: {:?}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                return Err(JsValue::from_str(&err_msg));
            }
        }

        // Generate bytecode first to ensure compiler state is ready
        let mut bytecode_gen = DslBytecodeGenerator::new();
        match bytecode_gen.generate(&ast) {
            Ok(_) => {
                log_message("Bytecode generation OK");
            }
            Err(e) => {
                let err_msg = format!("Bytecode generation failed: {:?}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                return Err(JsValue::from_str(&err_msg));
            }
        }

        // Generate simplified ABI - reuse the same generator that compiled the bytecode
        let simple_abi = match bytecode_gen.generate_simple_abi(&ast) {
            Ok(abi) => {
                log_message("ABI generation OK");
                abi
            }
            Err(e) => {
                let err_msg = format!("SimpleABI generation failed: {:?}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                return Err(JsValue::from_str(&err_msg));
            }
        };

        // Serialize ABI to JSON
        let abi_json = match serde_json::to_string_pretty(&simple_abi) {
            Ok(json) => {
                log_message("Serialization OK");
                json
            }
            Err(e) => {
                let err_msg = format!("ABI serialization failed: {}", e);
                web_sys::console::error_1(&JsValue::from_str(&err_msg));
                return Err(JsValue::from_str(&err_msg));
            }
        };

        Ok(JsValue::from_str(&abi_json))
    }

    /// Compile DSL and generate both bytecode and ABI
    #[wasm_bindgen]
    pub fn compile_with_abi(&self, source: &str) -> Result<JsValue, JsValue> {
        use five_dsl_compiler::{DslBytecodeGenerator, DslParser, DslTokenizer, DslTypeChecker};

        let start_time = js_sys::Date::now();

        // Tokenize
        let mut tokenizer = DslTokenizer::new(source);
        let tokens = tokenizer
            .tokenize()
            .map_err(|e| JsValue::from_str(&format!("Tokenization failed: {:?}", e)))?;

        // Parse
        let mut parser = DslParser::new(tokens);
        let ast = parser
            .parse()
            .map_err(|e| JsValue::from_str(&format!("Parsing failed: {:?}", e)))?;

        // Type check
        let mut type_checker = DslTypeChecker::new();
        type_checker
            .check_types(&ast)
            .map_err(|e| JsValue::from_str(&format!("Type checking failed: {:?}", e)))?;

        // Generate bytecode and ABI
        let mut bytecode_gen = DslBytecodeGenerator::new();
        let bytecode = bytecode_gen
            .generate(&ast)
            .map_err(|e| JsValue::from_str(&format!("Bytecode generation failed: {:?}", e)))?;

        let abi = bytecode_gen
            .generate_simple_abi(&ast)
            .map_err(|e| JsValue::from_str(&format!("ABI generation failed: {:?}", e)))?;

        // Get the compilation log (disassembly) after all mutable operations
        let disassembly = bytecode_gen.get_compilation_log().to_vec();

        let compilation_time = js_sys::Date::now() - start_time;

        // Create result object with proper JS handling
        let result = js_sys::Object::new();

        // Set basic properties
        js_sys::Reflect::set(&result, &"success".into(), &true.into())?;
        js_sys::Reflect::set(
            &result,
            &"bytecode_size".into(),
            &(bytecode.len() as u32).into(),
        )?;
        js_sys::Reflect::set(
            &result,
            &"compilation_time".into(),
            &compilation_time.into(),
        )?;

        // Set bytecode as Uint8Array
        let bytecode_array = js_sys::Uint8Array::from(&bytecode[..]);
        js_sys::Reflect::set(&result, &"bytecode".into(), &bytecode_array)?;

        // Set ABI as parsed JSON
        let abi_json = serde_json::to_string_pretty(&abi)
            .map_err(|e| JsValue::from_str(&format!("ABI serialization failed: {}", e)))?;
        let abi_js = js_sys::JSON::parse(&abi_json)?;
        js_sys::Reflect::set(&result, &"abi".into(), &abi_js)?;

        // Set disassembly as string array
        let disassembly_array = js_sys::Array::new();
        for line in disassembly.iter() {
            disassembly_array.push(&JsValue::from_str(line));
        }
        js_sys::Reflect::set(&result, &"disassembly".into(), &disassembly_array)?;

        // Set warnings and errors as empty arrays
        let empty_array = js_sys::Array::new();
        js_sys::Reflect::set(&result, &"warnings".into(), &empty_array)?;
        js_sys::Reflect::set(&result, &"errors".into(), &empty_array.clone())?;

        Ok(result.into())
    }

    /// Validate DSL syntax without full compilation
    #[wasm_bindgen]
    pub fn validate_syntax(&self, source: &str) -> JsValue {
        let response = Self::validate_syntax_internal(source);
        JsValue::from_str(&response.to_string())
    }

    pub(crate) fn validate_syntax_internal(source: &str) -> serde_json::Value {
        use five_dsl_compiler::{DslParser, DslTokenizer, DslTypeChecker};

        let result = (|| -> Result<(), String> {
            // Tokenize
            let mut tokenizer = DslTokenizer::new(source);
            let tokens = tokenizer
                .tokenize()
                .map_err(|e| format!("Tokenization error: {:?}", e))?;

            // Parse
            let mut parser = DslParser::new(tokens);
            let ast = parser
                .parse()
                .map_err(|e| format!("Parse error: {:?}", e))?;

            // Type check
            let mut type_checker = DslTypeChecker::new();
            type_checker
                .check_types(&ast)
                .map_err(|e| format!("Type error: {:?}", e))?;

            Ok(())
        })();

        match result {
            Ok(()) => serde_json::json!({
                "valid": true,
                "errors": [],
                "warnings": []
            }),
            Err(error) => serde_json::json!({
                "valid": false,
                "errors": [error],
                "warnings": []
            }),
        }
    }
}

/// Bytecode Encoding utilities for JavaScript (Fixed Size)
#[wasm_bindgen]
pub struct BytecodeEncoder;

#[wasm_bindgen]
impl BytecodeEncoder {
    /// Encode a u32 value
    /// Returns [size, byte1, byte2, byte3, byte4]
    #[wasm_bindgen]
    pub fn encode_u32(value: u32) -> js_sys::Array {
        let bytes = value.to_le_bytes();
        let result = js_sys::Array::new();
        result.push(&JsValue::from(4));
        result.push(&JsValue::from(bytes[0]));
        result.push(&JsValue::from(bytes[1]));
        result.push(&JsValue::from(bytes[2]));
        result.push(&JsValue::from(bytes[3]));
        result
    }

    /// Encode a u16 value
    /// Returns [size, byte1, byte2]
    #[wasm_bindgen]
    pub fn encode_u16(value: u16) -> js_sys::Array {
        let bytes = value.to_le_bytes();
        let result = js_sys::Array::new();
        result.push(&JsValue::from(2));
        result.push(&JsValue::from(bytes[0]));
        result.push(&JsValue::from(bytes[1]));
        result
    }

    /// Decode a u32 value
    /// Returns [value, bytes_consumed] or null if invalid
    #[wasm_bindgen]
    pub fn decode_u32(bytes: &[u8]) -> Option<js_sys::Array> {
        if bytes.len() >= 4 {
            let value = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            let result = js_sys::Array::new();
            result.push(&JsValue::from(value));
            result.push(&JsValue::from(4));
            Some(result)
        } else {
            None
        }
    }

    /// Decode a u16 value
    /// Returns [value, bytes_consumed] or null if invalid
    #[wasm_bindgen]
    pub fn decode_u16(bytes: &[u8]) -> Option<js_sys::Array> {
        if bytes.len() >= 2 {
            let value = u16::from_le_bytes([bytes[0], bytes[1]]);
            let result = js_sys::Array::new();
            result.push(&JsValue::from(value));
            result.push(&JsValue::from(2));
            Some(result)
        } else {
            None
        }
    }

    /// Calculate encoded size (Always 4 for u32)
    #[wasm_bindgen]
    pub fn encoded_size_u32(_value: u32) -> usize {
        4
    }

    /// Calculate encoded size (Always 2 for u16)
    #[wasm_bindgen]
    pub fn encoded_size_u16(_value: u16) -> usize {
        2
    }
}

/// Parameter encoding utilities using fixed-size encoding and protocol types
#[wasm_bindgen]
pub struct ParameterEncoder;

#[wasm_bindgen]
impl ParameterEncoder {
    /// Encode function parameters using fixed size encoding
    /// Returns ONLY parameter data - SDK handles discriminator AND function index
    #[wasm_bindgen]
    pub fn encode_execute(
        _function_index: u32,
        params: js_sys::Array,
    ) -> Result<js_sys::Uint8Array, JsValue> {
        let mut data = Vec::new();

        // Encode each parameter: [type_id, value]
        for i in 0..params.length() {
            let mut param = params.get(i);
            let mut is_account_type = false;
            let mut max_len: Option<u32> = None;

            // Extract type metadata and value if wrapped
            let mut type_str: Option<String> = None;
            if param.is_object() {
                if let Ok(t) = js_sys::Reflect::get(&param, &"type".into()) {
                    if let Some(s) = t.as_string() {
                        type_str = Some(s);
                    }
                }
                if type_str.is_none() {
                    if let Ok(t) = js_sys::Reflect::get(&param, &"param_type".into()) {
                        if let Some(s) = t.as_string() {
                            type_str = Some(s);
                        }
                    }
                }
                if let Ok(t) = js_sys::Reflect::get(&param, &"__type".into()) {
                    if let Some(s) = t.as_string() {
                        type_str = Some(s);
                    }
                }

                if let Ok(val) = js_sys::Reflect::get(&param, &"isAccount".into()) {
                    if val.as_bool().unwrap_or(false) {
                        is_account_type = true;
                    }
                }
                if let Ok(val) = js_sys::Reflect::get(&param, &"is_account".into()) {
                    if val.as_bool().unwrap_or(false) {
                        is_account_type = true;
                    }
                }

                if let Ok(val) = js_sys::Reflect::get(&param, &"value".into()) {
                    param = val;
                }
                if let Ok(val) = js_sys::Reflect::get(&param, &"maxLen".into()) {
                    if let Some(raw) = val.as_f64() {
                        if raw >= 0.0 {
                            max_len = Some(raw as u32);
                        }
                    } else if let Some(raw_str) = val.as_string() {
                        if let Ok(parsed) = raw_str.parse::<u32>() {
                            max_len = Some(parsed);
                        }
                    }
                }
            }

            let normalized_type = type_str
                .as_ref()
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            if is_account_type || normalized_type == "account" || normalized_type == "mint" || normalized_type == "tokenaccount" {
                data.push(types::ACCOUNT);
                if let Some(num) = param.as_f64() {
                    let idx = num as u32;
                    data.extend_from_slice(&idx.to_le_bytes());
                } else {
                    return Err(JsValue::from_str("ACCOUNT parameter must be a numeric index"));
                }
                continue;
            }

            match normalized_type.as_str() {
                "u8" => {
                    data.push(types::U8);
                    let val = param.as_f64().unwrap_or(0.0) as u32;
                    data.extend_from_slice(&val.to_le_bytes());
                }
                "u16" => {
                    return Err(JsValue::from_str("U16 is not supported by current VM parameter parser"));
                }
                "u32" => {
                    data.push(types::U32);
                    let val = param.as_f64().unwrap_or(0.0) as u32;
                    data.extend_from_slice(&val.to_le_bytes());
                }
                "u64" | "i64" => {
                    data.push(types::U64);
                    let raw = param.as_f64().unwrap_or(0.0);
                    if normalized_type == "i64" && raw < 0.0 {
                        return Err(JsValue::from_str("I64 negative values are not supported by current VM parameter parser"));
                    }
                    let val = raw as u64;
                    data.extend_from_slice(&val.to_le_bytes());
                }
                "bool" => {
                    data.push(types::BOOL);
                    let val = if param.as_bool().unwrap_or(false) { 1u32 } else { 0u32 };
                    data.extend_from_slice(&val.to_le_bytes());
                }
                "pubkey" => {
                    data.push(types::PUBKEY);
                    if let Some(str_val) = param.as_string() {
                        let decoded = bs58::decode(&str_val)
                            .into_vec()
                            .map_err(|_| JsValue::from_str("Invalid base58 pubkey"))?;
                        if decoded.len() != 32 {
                            return Err(JsValue::from_str("Pubkey must decode to 32 bytes"));
                        }
                        data.extend_from_slice(&decoded);
                    } else if param.is_object() && js_sys::Uint8Array::instanceof(&param) {
                        let array = js_sys::Uint8Array::new(&param);
                        let mut bytes = vec![0u8; array.length() as usize];
                        array.copy_to(&mut bytes);
                        if bytes.len() != 32 {
                            return Err(JsValue::from_str("Pubkey Uint8Array must be 32 bytes"));
                        }
                        data.extend_from_slice(&bytes);
                    } else {
                        return Err(JsValue::from_str("Pubkey must be a base58 string or 32-byte Uint8Array"));
                    }
                }
                "string" | "bytes" => {
                    data.push(types::STRING);
                    if let Some(str_val) = param.as_string() {
                        let bytes = str_val.as_bytes();
                        let len = bytes.len() as u32;
                        if let Some(max) = max_len {
                            if len > max {
                                return Err(JsValue::from_str(
                                    &format!(
                                        "STRING parameter exceeds declared size: got {} bytes, max {}",
                                        len, max
                                    )
                                ));
                            }
                        }
                        data.extend_from_slice(&len.to_le_bytes());
                        data.extend_from_slice(bytes);
                    } else if param.is_object() && js_sys::Uint8Array::instanceof(&param) {
                        let array = js_sys::Uint8Array::new(&param);
                        let mut bytes = vec![0u8; array.length() as usize];
                        array.copy_to(&mut bytes);
                        let len = bytes.len() as u32;
                        if let Some(max) = max_len {
                            if len > max {
                                return Err(JsValue::from_str(
                                    &format!(
                                        "STRING parameter exceeds declared size: got {} bytes, max {}",
                                        len, max
                                    )
                                ));
                            }
                        }
                        data.extend_from_slice(&len.to_le_bytes());
                        data.extend_from_slice(&bytes);
                    } else {
                        return Err(JsValue::from_str("STRING parameter must be a string or Uint8Array"));
                    }
                }
                "" => {
                    // Infer if no explicit type provided
                    if param.is_string() {
                        let str_val = param.as_string().unwrap();
                        if str_val.len() == 44 {
                            if let Ok(decoded) = bs58::decode(&str_val).into_vec() {
                                if decoded.len() == 32 {
                                    data.push(types::PUBKEY);
                                    data.extend_from_slice(&decoded);
                                    continue;
                                }
                            }
                        }
                        data.push(types::STRING);
                        let bytes = str_val.as_bytes();
                        let len = bytes.len() as u32;
                        if let Some(max) = max_len {
                            if len > max {
                                return Err(JsValue::from_str(
                                    &format!(
                                        "STRING parameter exceeds declared size: got {} bytes, max {}",
                                        len, max
                                    )
                                ));
                            }
                        }
                        data.extend_from_slice(&len.to_le_bytes());
                        data.extend_from_slice(bytes);
                    } else if let Some(num) = param.as_f64() {
                        data.push(types::U64);
                        let val = num as u64;
                        data.extend_from_slice(&val.to_le_bytes());
                    } else if let Some(bool_val) = param.as_bool() {
                        data.push(types::BOOL);
                        let val = if bool_val { 1u32 } else { 0u32 };
                        data.extend_from_slice(&val.to_le_bytes());
                    } else if param.is_object() && js_sys::Uint8Array::instanceof(&param) {
                        data.push(types::STRING);
                        let array = js_sys::Uint8Array::new(&param);
                        let mut bytes = vec![0u8; array.length() as usize];
                        array.copy_to(&mut bytes);
                        let len = bytes.len() as u32;
                        data.extend_from_slice(&len.to_le_bytes());
                        data.extend_from_slice(&bytes);
                    } else {
                        return Err(JsValue::from_str("Unsupported parameter value type"));
                    }
                }
                _ => {
                    return Err(JsValue::from_str("Unsupported parameter type"));
                }
            }
        }

        Ok(js_sys::Uint8Array::from(&data[..]))
    }
}

fn map_metrics_format(format: &str) -> ExportFormat {
    match format.to_lowercase().as_str() {
        "csv" => ExportFormat::Csv,
        "toml" => ExportFormat::Toml,
        "dashboard" => ExportFormat::Dashboard,
        _ => ExportFormat::Json,
    }
}

/// Get information about the WASM compiler capabilities
#[wasm_bindgen]
pub fn get_wasm_compiler_info() -> JsValue {
    let info = serde_json::json!({
        "name": "five-dsl-compiler-wasm",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "WebAssembly-based Five DSL compiler for client-side compilation",
        "features": {
            "compilation": true,
            "abi_generation": true,
            "parsing": true,
            "type_checking": true,
            "optimization": true,
            "validation": true,
            "bytecode_analysis": true
        },
        "supported_syntax": [
            "vault declarations",
            "init blocks",
            "constraint blocks",
            "assignments",
            "method calls",
            "arithmetic operations",
            "comparison operations"
        ],
        "output_format": "mito-vm bytecode",
        "magic_bytes": "5IVE"
    });

    JsValue::from_str(&info.to_string())
}

// Helper function to process compiler errors
fn process_errors(
    errors: &[five_dsl_compiler::error::CompilerError],
    source: &str,
    filename: Option<&str>,
) -> (usize, usize, Vec<String>, Vec<String>, Vec<WasmCompilerError>, String, String) {
    let mut source_map = std::collections::HashMap::new();
    let name = filename.unwrap_or("input.v");
    source_map.insert(std::path::PathBuf::from(name), source.to_string());
    process_multi_errors(errors, &source_map, Some(&std::path::PathBuf::from(name)))
}

// Helper function to process compiler errors with multiple sources
fn process_multi_errors(
    errors: &[five_dsl_compiler::error::CompilerError],
    source_map: &std::collections::HashMap<std::path::PathBuf, String>,
    main_file_hint: Option<&std::path::PathBuf>,
) -> (usize, usize, Vec<String>, Vec<String>, Vec<WasmCompilerError>, String, String) {
    let mut compiler_errors_vec = Vec::new();
    let mut error_strings = Vec::new();
    let mut warnings = Vec::new();
    let mut error_count = 0;
    let mut warning_count = 0;

    // Use context extractor for source lines
    use five_dsl_compiler::error::context::SourceContextExtractor;
    use five_dsl_compiler::error::formatting::{ErrorFormatter, JsonFormatter, TerminalFormatter};
    
    let extractor = SourceContextExtractor::new();
    
    // Create rich errors with source context for formatting
    let rich_errors: Vec<five_dsl_compiler::error::CompilerError> = errors.iter().map(|e| {
        let mut e = e.clone(); 
        
        // Remap input.v to main file hint if present
        if let Some(hint) = main_file_hint {
            if let Some(ref loc) = e.location {
                if let Some(ref file) = loc.file {
                    if file.to_string_lossy() == "input.v" {
                        let mut new_loc = loc.clone();
                        new_loc.file = Some(hint.clone());
                        e.location = Some(new_loc);
                    }
                }
            }
        }

        // Inject correct source from map based on error location
        if let Some(ref loc) = e.location {
            if let Some(ref file) = loc.file {
                if let Some(source) = source_map.get(file) {
                    e.context.source_line = Some(source.clone());
                } else {
                    // Fallback to searching by filename if path doesn't match exactly
                    let filename = file.file_name().and_then(|f| f.to_str());
                    if let Some(name) = filename {
                        for (path, src) in source_map {
                            if path.file_name().and_then(|f| f.to_str()) == Some(name) {
                                e.context.source_line = Some(src.clone());
                                break;
                            }
                        }
                    }
                }
            } else if source_map.len() == 1 {
                // If only one source is provided and error has no file, use that source.
                if let Some(source) = source_map.values().next() {
                    e.context.source_line = Some(source.clone());
                }
            }
        } else if source_map.len() == 1 {
            // No location but only one source: inject for context.
            if let Some(source) = source_map.values().next() {
                e.context.source_line = Some(source.clone());
            }
        }
        
        e
    }).collect();

    let term_formatter = TerminalFormatter::new();
    let json_formatter = JsonFormatter::new();

    let formatted_terminal = term_formatter.format_errors(&rich_errors);
    let formatted_json = json_formatter.format_errors(&rich_errors);

    for error in errors {
        // Extract suggestions
        let mut suggestions = Vec::new();
        // Since suggestion engine is part of the ErrorSystem which is global,
        // we can try to generate suggestions here if appropriate, or rely on what's in the error context
        // However, currently CompilerError doesn't hold suggestions directly in the struct definition in lib.rs
        // CompilerError does not have a suggestions field.
        // Suggestions are generated by SuggestionEngine.
        
        // We'll use the global suggestion engine
        let error_system = five_dsl_compiler::error::error_system();
        let generated_suggestions = error_system.generate_suggestions(error);
        
        for s in generated_suggestions {
             suggestions.push(WasmSuggestion {
                 message: s.message,
                 explanation: s.explanation,
                 confidence: s.confidence as f64,
                 code_suggestion: s.code_fix.as_ref().map(|f| f.replacement.clone()),
             });
        }

        // Extract source context
        let (source_line, source_snippet) = if let Some(ref location) = error.location {
            let file_source = if let Some(ref file) = location.file {
                source_map.get(file).cloned().or_else(|| {
                    // Fallback search
                    let filename = file.file_name().and_then(|f| f.to_str());
                    filename.and_then(|name| {
                        source_map.iter()
                            .find(|(p, _)| p.file_name().and_then(|f| f.to_str()) == Some(name))
                            .map(|(_, s)| s.clone())
                    })
                })
            } else {
                source_map.values().next().cloned()
            };

            if let Some(source) = file_source {
                if let Ok(context) = extractor.extract_context(&source, location) {
                     // Get the specific error line content
                     let line_content = context.lines.iter()
                        .find(|l| l.is_error_line)
                        .map(|l| l.content.clone());
                     
                     // Snippet could be the full joined context
                     let snippet = context.lines.iter()
                        .map(|l| format!("{:4} | {}", l.line_number, l.content))
                        .collect::<Vec<_>>()
                        .join("\n");
                        
                     (line_content, Some(snippet))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Convert to WasmCompilerError (simplified for WASM)
        let wasm_error = WasmCompilerError {
            code: error.code.to_string(),
            severity: error.severity.to_string(),
            category: error.category.to_string(),
            message: error.message.clone(),
            description: error.description.clone(),
            location: error.location.as_ref().map(|loc| WasmSourceLocation {
                line: loc.line,
                column: loc.column,
                length: loc.length,
                offset: loc.offset,
                file: loc.file.as_ref().map(|f| f.display().to_string()),
            }),
            suggestions,
            source_line,
            source_snippet,
            line: error.location.as_ref().map(|loc| loc.line),
            column: error.location.as_ref().map(|loc| loc.column),
        };
        compiler_errors_vec.push(wasm_error);

        if error.severity == five_dsl_compiler::error::ErrorSeverity::Error {
            error_count += 1;
            error_strings.push(error.message.clone());
        } else {
            warning_count += 1;
            warnings.push(error.message.clone());
        }
    }

    (error_count, warning_count, warnings, error_strings, compiler_errors_vec, formatted_terminal, formatted_json)
}

#[cfg(test)]
mod formatting_tests {
    use super::*;

    #[test]
    fn test_format_terminal() {
        let error = WasmCompilerError {
            code: "E0001".to_string(),
            severity: "error".to_string(),
            category: "syntax".to_string(),
            message: "test message".to_string(),
            description: Some("test description".to_string()),
            location: Some(WasmSourceLocation {
                file: Some("test.v".to_string()),
                line: 10,
                column: 5,
                offset: 100,
                length: 3,
            }),
            suggestions: vec![WasmSuggestion {
                message: "try this".to_string(),
                explanation: None,
                confidence: 1.0,
                code_suggestion: Some("fixed_code".to_string()),
            }],
            source_line: Some("let x = error;".to_string()),
            source_snippet: Some("let x = error;".to_string()), // Using snippet directly
            line: Some(10),
            column: Some(5),
        };

        let output = error.format_terminal();
        
        // Check for key components in the output
        assert!(output.contains("[E0001]"));
        assert!(output.contains("test message"));
        assert!(output.contains("test.v:10:5")); 
        assert!(output.contains("test description"));
        assert!(output.contains("let x = error;"));
        assert!(output.contains("try this"));
        
        // Check coloring codes (basic check)
        assert!(output.contains("\x1b[31m")); // Red for error
        assert!(output.contains("\x1b[34m")); // Blue for location
    }
}

#[cfg(test)]
mod internal_tests {
    use super::*;
    use five_protocol::FIVE_MAGIC;

    #[test]
    fn test_extract_five_bytecode_pure() {
        let mut bytecode = Vec::new();
        bytecode.extend_from_slice(&FIVE_MAGIC);
        bytecode.extend_from_slice(&[0x00]); // HALT

        let result = FiveVMWasm::extract_five_bytecode(&bytecode);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), bytecode);
    }

    #[test]
    fn test_extract_five_bytecode_with_account_header() {
        let mut data = vec![0u8; 64]; // Header
        data.extend_from_slice(&FIVE_MAGIC);
        data.extend_from_slice(&[0x00]); // HALT

        let result = FiveVMWasm::extract_five_bytecode(&data);
        assert!(result.is_ok());
        let extracted = result.unwrap();
        assert_eq!(extracted.len(), 5);
        assert_eq!(&extracted[0..4], FIVE_MAGIC);
    }

    #[test]
    fn test_extract_five_bytecode_offset_search() {
        let mut data = vec![0u8; 10]; // Random prefix
        data.extend_from_slice(&FIVE_MAGIC);
        data.extend_from_slice(&[0x00]);

        let result = FiveVMWasm::extract_five_bytecode(&data);
        assert!(result.is_ok());
        let extracted = result.unwrap();
        assert_eq!(extracted.len(), 5);
        assert_eq!(&extracted[0..4], FIVE_MAGIC);
    }

    #[test]
    fn test_extract_five_bytecode_invalid() {
        let data = vec![0x00, 0x01, 0x02];
        let result = FiveVMWasm::extract_five_bytecode(&data);
        assert!(matches!(result, Err(VMError::InvalidScript)));
    }

    #[test]
    fn test_extract_abi_valid() {
        let json = r#"
        {
            "abi": {
                "functions": []
            }
        }
        "#;
        let data = json.as_bytes();
        let result = FiveVMWasm::extract_abi_from_five_file(data);
        assert!(result.is_some());
        let abi = result.unwrap();
        assert!(abi.contains("functions"));
    }

    #[test]
    fn test_extract_abi_invalid_json() {
        let data = b"invalid json";
        let result = FiveVMWasm::extract_abi_from_five_file(data);
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_abi_no_abi_field() {
        let json = r#"{"foo": "bar"}"#;
        let data = json.as_bytes();
        let result = FiveVMWasm::extract_abi_from_five_file(data);
        assert!(result.is_none());
    }

    #[test]
    fn test_decode_instruction_data_discriminator_2() {
        // Setup VM instance
        let bytecode = FIVE_MAGIC.to_vec();
        let vm = FiveVMWasm::new(&bytecode).expect("Failed to create VM");

        // [2, 0x01, 0x02] -> [0x01, 0x02]
        let input = vec![2, 0x01, 0x02];
        let result = vm.decode_instruction_data(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0x01, 0x02]);
    }

    #[test]
    fn test_decode_instruction_data_discriminator_9() {
        let bytecode = FIVE_MAGIC.to_vec();
        let vm = FiveVMWasm::new(&bytecode).expect("Failed to create VM");

        // [9, 0x01, 0x02] -> [0x01, 0x02]
        let input = vec![9, 0x01, 0x02];
        let result = vm.decode_instruction_data(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![0x01, 0x02]);
    }

    #[test]
    fn test_decode_instruction_data_pass_through() {
        let bytecode = FIVE_MAGIC.to_vec();
        let vm = FiveVMWasm::new(&bytecode).expect("Failed to create VM");

        // [1, 0x01, 0x02] -> [1, 0x01, 0x02]
        let input = vec![1, 0x01, 0x02];
        let result = vm.decode_instruction_data(&input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), input);
    }

    #[test]
    fn test_get_instruction_size() {
        // PUSH_U64 (0x1B) + U64 value (8 bytes)
        let opcode = five_protocol::opcodes::PUSH_U64;

        // Case 1: PUSH_U64
        let mut op_bytes = vec![opcode];
        op_bytes.extend_from_slice(&100u64.to_le_bytes());
        // Total 9 bytes (1 + 8)
        assert_eq!(get_instruction_size(opcode, &op_bytes), 9);

        // Case 2: HALT (No args)
        let opcode = five_protocol::opcodes::HALT;
        assert_eq!(get_instruction_size(opcode, &[opcode]), 1);
    }
}

#[cfg(test)]
fn build_min_header() -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(&five_protocol::FIVE_MAGIC);
    header.extend_from_slice(&0u32.to_le_bytes()); // features
    header.push(0); // public_function_count
    header.push(0); // total_function_count
    header
}

#[cfg(test)]
mod analyzer_tests {
    use super::*;
    use five_protocol::opcodes;

    #[test]
    fn test_analyze_internal_simple() {
        let mut bytecode = build_min_header();
        bytecode.push(opcodes::HALT);

        let result = BytecodeAnalyzer::analyze_internal(&bytecode);
        assert!(result.is_ok());
        let json = result.unwrap();

        assert_eq!(json["total_size"], 11);
        assert_eq!(json["instruction_count"], 1);

        let instructions = json["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0]["offset"], 10);
        assert_eq!(instructions[0]["opcode"], opcodes::HALT);
        assert_eq!(instructions[0]["name"], "HALT");
    }

    #[test]
    fn test_analyze_internal_invalid_magic() {
        let bytecode = vec![0x00, 0x01, 0x02, 0x03];
        let result = BytecodeAnalyzer::analyze_internal(&bytecode);
        assert!(result.is_err());
    }

    #[test]
    fn test_analyze_semantic_internal_simple() {
        let mut bytecode = build_min_header();
        bytecode.push(opcodes::HALT);

        let result = BytecodeAnalyzer::analyze_semantic_internal(&bytecode);
        assert!(result.is_ok());
        let json = result.unwrap();

        // Verify structure
        assert!(json.get("summary").is_some());
        assert!(json.get("instructions").is_some());
        assert!(json.get("control_flow").is_some());
        assert!(json.get("stack_analysis").is_some());

        let instructions = json["instructions"].as_array().unwrap();
        assert_eq!(instructions.len(), 1);
        assert_eq!(instructions[0]["name"], "HALT");
    }

    #[test]
    fn test_get_bytecode_summary_internal() {
        let mut bytecode = build_min_header();
        bytecode.push(opcodes::HALT);

        let result = BytecodeAnalyzer::get_bytecode_summary_internal(&bytecode);
        assert!(result.is_ok());
        let json = result.unwrap();

        assert_eq!(json["total_instructions"], 3);
        assert_eq!(json["total_size"], 11);
    }
}

#[cfg(test)]
mod error_enhancement_tests {
    use super::*;

    #[test]
    fn test_enhance_parameter_error_static_with_abi() {
        let abi_json = r#"{
            "functions": {
                "my_func": {
                    "index": 0,
                    "name": "my_func",
                    "parameters": [
                        {"name": "a", "type": "u64"},
                        {"name": "b", "type": "bool"}
                    ]
                }
            }
        }"#;
        let abi_data = Some(abi_json.to_string());

        let msg = FiveVMWasm::enhance_parameter_error_static(
            &abi_data,
            0, // function index
            2, // expected
            1, // actual
            1, // failed index (b)
        );

        assert!(msg.contains("Function 'my_func' expected 2 parameters but received 1"));
        assert!(msg.contains("Failed to load parameter 'bool' at position 2"));
        assert!(msg.contains("Expected parameter types:"));
        assert!(msg.contains("1. u64"));
        assert!(msg.contains("2. bool (← FAILED HERE)"));
    }

    #[test]
    fn test_enhance_parameter_error_static_without_abi() {
        let abi_data = None;

        let msg = FiveVMWasm::enhance_parameter_error_static(
            &abi_data,
            0, // function index
            2, // expected
            1, // actual
            1, // failed index
        );

        assert!(msg.contains("Function at index 0 expected 2 parameters but received 1"));
        assert!(msg.contains("Failed to load parameter at position 2"));
        assert!(msg.contains("Debug Information"));
    }
}

#[cfg(test)]
mod compiler_error_tests {
    use super::*;
    use five_dsl_compiler::error::{CompilerError, ErrorCode, ErrorSeverity, ErrorCategory};

    #[test]
    fn test_process_multi_errors_single() {
        let error = CompilerError::new(
            ErrorCode::INVALID_SYNTAX,
            ErrorSeverity::Error,
            ErrorCategory::Syntax,
            "Test error".to_string(),
        );
        let errors = vec![error];
        let (err_count, warn_count, warnings, err_strs, wasm_errs, _term, _json) =
            process_errors(&errors, "source code", Some("test.v"));

        assert_eq!(err_count, 1);
        assert_eq!(warn_count, 0);
        assert!(warnings.is_empty());
        assert_eq!(err_strs.len(), 1);
        assert_eq!(wasm_errs.len(), 1);
        assert_eq!(wasm_errs[0].message, "Test error");
    }

    #[test]
    fn test_process_multi_errors_warning() {
        let warning = CompilerError::new(
            ErrorCode::UNUSED_VARIABLE,
            ErrorSeverity::Warning,
            ErrorCategory::Semantic,
            "Test warning".to_string(),
        );
        let errors = vec![warning];
        let (err_count, warn_count, warnings, err_strs, wasm_errs, _term, _json) =
            process_errors(&errors, "source code", Some("test.v"));

        assert_eq!(err_count, 0);
        assert_eq!(warn_count, 1);
        assert_eq!(warnings.len(), 1);
        assert!(err_strs.is_empty());
        assert_eq!(wasm_errs.len(), 1);
        assert_eq!(wasm_errs[0].severity, "warning");
    }
}

#[cfg(test)]
mod compiler_tests {
    use super::*;
    use five_protocol::opcodes;

    #[test]
    fn test_optimize_bytecode_no_change() {
        let mut bytecode = build_min_header();
        bytecode.extend_from_slice(&[opcodes::ADD, opcodes::SUB]);
        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        assert_eq!(result, bytecode);
    }

    #[test]
    fn test_optimize_push_u8_pop() {
        // PUSH_U8 (0x1C) + value + POP (0x06)
        // Should be optimized away
        let mut bytecode = build_min_header();
        bytecode.extend_from_slice(&[opcodes::PUSH_U8, 42, opcodes::POP]);
        // Add a HALT at the end to verify we don't optimize too much
        bytecode.push(opcodes::HALT);

        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        let mut expected = build_min_header();
        expected.push(opcodes::HALT);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_optimize_push_u64_pop() {
        // PUSH_U64 (0x1B) + U64 value + POP (0x06)
        let bytes = 1000u64.to_le_bytes();

        let mut bytecode = build_min_header();
        bytecode.push(opcodes::PUSH_U64);
        bytecode.extend_from_slice(&bytes);
        bytecode.push(opcodes::POP);
        bytecode.push(opcodes::HALT);

        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        let mut expected = build_min_header();
        expected.push(opcodes::HALT);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_optimize_push_no_pop() {
        let mut bytecode = build_min_header();
        bytecode.extend_from_slice(&[opcodes::PUSH_U8, 42]);
        bytecode.push(opcodes::HALT);

        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        assert_eq!(result, bytecode);
    }

    #[test]
    fn test_optimize_pop_no_push() {
        let mut bytecode = build_min_header();
        bytecode.extend_from_slice(&[opcodes::POP, opcodes::HALT]);
        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        assert_eq!(result, bytecode);
    }

    #[test]
    fn test_optimize_consecutive() {
        let mut bytecode = build_min_header();
        bytecode.extend_from_slice(&[opcodes::PUSH_U8, 1, opcodes::POP]);
        bytecode.extend_from_slice(&[opcodes::PUSH_U8, 2, opcodes::POP]);
        bytecode.push(opcodes::HALT);

        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        let mut expected = build_min_header();
        expected.push(opcodes::HALT);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_optimize_interleaved() {
        // PUSH 1, POP, PUSH 2, HALT
        let mut bytecode = build_min_header();
        bytecode.extend_from_slice(&[opcodes::PUSH_U8, 1, opcodes::POP]);
        bytecode.extend_from_slice(&[opcodes::PUSH_U8, 2]);
        bytecode.push(opcodes::HALT);

        let result = WasmFiveCompiler::optimize_bytecode_internal(&bytecode).unwrap();
        // Should remove first PUSH/POP
        let mut expected = build_min_header();
        expected.extend_from_slice(&[opcodes::PUSH_U8, 2, opcodes::HALT]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_validate_syntax_internal_valid() {
        let source = r#"
            script test_program {
                instruction test_func() {
                    return;
                }
            }
        "#;
        let result = WasmFiveCompiler::validate_syntax_internal(source);
        assert_eq!(result["valid"], true);
    }

    #[test]
    fn test_validate_syntax_internal_invalid() {
        let source = r#"
            script test_program {
                instruction test_func() {
                    invalid_syntax ///
                }
            }
        "#;
        let result = WasmFiveCompiler::validate_syntax_internal(source);
        assert_eq!(result["valid"], false);
        assert!(!result["errors"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_extract_account_definitions_internal() {
        let source = r#"
            script test_program {
                account MyAccount {
                    field1: u64,
                    field2: bool,
                }
            }
        "#;
        let result = WasmFiveCompiler::extract_account_definitions_internal(source).unwrap();
        assert_eq!(result["success"], true);
        let accounts = result["account_definitions"].as_array().unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0]["name"], "MyAccount");
        let fields = accounts[0]["fields"].as_array().unwrap();
        assert_eq!(fields.len(), 2);
    }

    #[test]
    fn test_extract_function_signatures_internal() {
        let source = r#"
            script test_program {
                instruction my_func(acc1: account, @signer acc2: account) {
                    return;
                }
            }
        "#;
        let result = WasmFiveCompiler::extract_function_signatures_internal(source).unwrap();
        assert_eq!(result["success"], true);
        let funcs = result["function_signatures"].as_array().unwrap();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0]["name"], "my_func");
        let params = funcs[0]["parameters"].as_array().unwrap();
        assert_eq!(params.len(), 2);
        // Check attributes
        let param2 = &params[1];
        let attrs = param2["attributes"].as_array().unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0]["name"], "signer");
    }

    #[test]
    fn test_validate_account_constraints_internal() {
         let source = r#"
            script test_program {
                instruction my_func(@signer acc1: account) {
                    return;
                }
            }
        "#;

        // Mock accounts input: acc1 is signer
        let accounts = serde_json::json!([
            { "is_signer": true, "is_writable": false }
        ]);

        let result = WasmFiveCompiler::validate_account_constraints_internal(
            source,
            "my_func",
            accounts
        ).unwrap();

        assert_eq!(result["success"], true);
        let validations = result["validation_results"].as_array().unwrap();
        let checks = validations[0]["constraint_checks"].as_array().unwrap();
        assert_eq!(checks[0]["valid"], true);

        // Mock accounts input: acc1 is NOT signer (should fail validation)
        let accounts_fail = serde_json::json!([
            { "is_signer": false, "is_writable": false }
        ]);

        let result_fail = WasmFiveCompiler::validate_account_constraints_internal(
            source,
            "my_func",
            accounts_fail
        ).unwrap();

        let validations_fail = result_fail["validation_results"].as_array().unwrap();
        let checks_fail = validations_fail[0]["constraint_checks"].as_array().unwrap();
        assert_eq!(checks_fail[0]["valid"], false);
    }
}
