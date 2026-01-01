/**
 * Five VM WASM Integration
 * 
 * Real integration with Five VM WASM bindings for script execution,
 * partial execution, and bytecode analysis.
 */

import { 
  VMExecutionOptions, 
  VMExecutionResult,
  AccountInfo,
  Logger,
  CLIError
} from '../types.js';
import { existsSync, readFileSync } from 'fs';
import { dirname, resolve } from 'path';
import { fileURLToPath } from 'url';
import { ConfigManager } from '../config/ConfigManager.js';

// Real Five VM WASM imports
let FiveVMWasm: any;
let WasmAccount: any;
let ParameterEncoder: any;
let wrap_with_script_header: undefined | ((bytecode: Uint8Array) => Uint8Array);

const SCRIPT_HEADER_LEN = 64; // ScriptHeader::LEN (see five-protocol)
const OPTIMIZED_HEADER_LEN = 7; // OptimizedHeader V2 size (magic + features + counts)
const FIVE_MAGIC = [0x35, 0x49, 0x56, 0x45];

export class FiveVM {
  private vm: any = null;
  private logger: Logger;
  private initialized = false;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  /**
   * Initialize the VM with real Five VM WASM module
   */
  async initialize(): Promise<void> {
    try {
      console.log('[DEBUG] Starting VM WASM initialization...');

      // Try multiple candidate locations for the WASM bundle to be robust
      const cfg = await ConfigManager.getInstance().get();
      const prefer = cfg.wasm?.loader || 'auto';
      const configured = Array.isArray(cfg.wasm?.modulePaths) ? cfg.wasm!.modulePaths! : [];
      const nodeCandidates = [
        '../../five_vm_wasm.js',
        '../five_vm_wasm.js',
      ];
      const bundlerCandidates = [
        '../../assets/vm/five_vm_wasm.js',
        '../assets/vm/five_vm_wasm.js',
      ];
      let candidates: string[] = [];
      candidates.push(...configured);
      if (prefer === 'node') {
        candidates.push(...nodeCandidates);
      } else if (prefer === 'bundler') {
        candidates.push(...bundlerCandidates);
      } else {
        candidates.push(...nodeCandidates, ...bundlerCandidates);
      }

      let wasmModule: any | null = null;
      const tried: Array<{ path: string; error: unknown }> = [];

      for (const candidate of candidates) {
        try {
          // Dynamic import of Five VM WASM bindings
          // Note: ESM import is resolved relative to this file
          // eslint-disable-next-line no-await-in-loop
          let mod: any | null = null;
          try {
            mod = await import(candidate as string);
          } catch (esmErr) {
            // If ESM import fails and path looks like our CJS asset, try createRequire
            try {
              const { createRequire } = await import('module');
              const here = dirname(fileURLToPath(import.meta.url));
              const abs = resolve(here, candidate);
              const req = createRequire(import.meta.url);
              // eslint-disable-next-line @typescript-eslint/no-var-requires
              mod = req(abs);
            } catch (cjsErr) {
              tried.push({ path: candidate, error: cjsErr });
              mod = null;
            }
          }
          // If initSync is available, prefer initializing with local file bytes to avoid fetch/file URL issues
          if (mod && typeof (mod as any).initSync === 'function') {
            try {
              const here = dirname(fileURLToPath(import.meta.url));
              const wasmFiles = [
                resolve(here, '../five_vm_wasm_bg.wasm'),             // dist/five_vm_wasm_bg.wasm
                resolve(here, '../../five_vm_wasm_bg.wasm'),          // five-cli/five_vm_wasm_bg.wasm (unlikely)
                resolve(here, '../assets/vm/five_vm_wasm_bg.wasm'),   // dist/assets/vm
                resolve(here, '../../assets/vm/five_vm_wasm_bg.wasm') // assets/vm
              ];
              for (const wf of wasmFiles) {
                if (existsSync(wf)) {
                  // eslint-disable-next-line no-await-in-loop
                  (mod as any).initSync(readFileSync(wf));
                  break;
                }
              }
            } catch (syncErr) {
              tried.push({ path: candidate, error: syncErr });
            }
          }
          // Initialize node-friendly wasm-pack bundle if it exposes a default init (fallback)
          if (mod && typeof (mod as any).default === 'function') {
            try {
              // eslint-disable-next-line no-await-in-loop
              await (mod as any).default();
            } catch (initErr) {
              // Don't continue yet; allow export validation below to decide
              tried.push({ path: candidate, error: initErr });
            }
          }
          if (mod && mod.FiveVMWasm && mod.WasmAccount && mod.ParameterEncoder) {
            wasmModule = mod;
            console.log(`[WASM VM] Loaded module from: ${candidate}`);
            break;
          }
          tried.push({ path: candidate, error: 'Missing expected exports' });
        } catch (e) {
          tried.push({ path: candidate, error: e });
        }
      }

      if (!wasmModule) {
        const attempted = tried
          .map(t => `  - ${t.path}: ${t.error instanceof Error ? t.error.message : String(t.error)}`)
          .join('\n');
        throw this.createVMError(
          `Failed to load WASM VM: Five VM WASM modules not found.\nAttempted:\n${attempted}\nPlease run \"npm run build:wasm\" to build assets.`,
        );
      }

      FiveVMWasm = wasmModule.FiveVMWasm;
      WasmAccount = wasmModule.WasmAccount;
      ParameterEncoder = wasmModule.ParameterEncoder;
      if (typeof (wasmModule as any).wrap_with_script_header !== 'function') {
        const attempted = tried
          .map(t => `  - ${t.path}: ${t.error instanceof Error ? t.error.message : String(t.error)}`)
          .join('\n');
        throw this.createVMError(
          `WASM VM missing required export wrap_with_script_header.\nAttempted:\n${attempted}\nPlease rebuild WASM assets: npm -C five-wasm run build`
        );
      }
      wrap_with_script_header = (wasmModule as any).wrap_with_script_header;
      console.log('[WASM VM] Using Rust header wrapper for ScriptHeader generation');

      this.initialized = true;
    } catch (error) {
      throw this.createVMError(
        'Five VM WASM modules not found. Please run "npm run build:wasm" to build the required WebAssembly modules.',
        error as Error
      );
    }
  }

  /**
   * Check if we're in WASM-only execution mode (vs network deployment)
   */
  private isWasmOnlyExecution(): boolean {
    // Check command line arguments to determine execution context
    const args = process.argv;
    
    // If called with "local" subcommand, always use WASM-only mode
    if (args.includes('local')) {
      return true;
    }
    
    // Check for test-runner.sh or similar local testing scripts
    const scriptName = process.argv[1];
    if (scriptName && (scriptName.includes('test-runner') || scriptName.includes('local'))) {
      return true;
    }
    
    // For now, default to WASM-only execution for all cases
    // TODO: Add proper network deployment detection when deployment flow is implemented
    return true;
  }

  private hasFiveMagic(data: Uint8Array): boolean {
    if (data.length < FIVE_MAGIC.length) {
      return false;
    }
    return FIVE_MAGIC.every((byte, index) => data[index] === byte);
  }

  private looksLikeScriptHeader(data: Uint8Array): boolean {
    if (data.length < SCRIPT_HEADER_LEN) {
      return false;
    }

    if (!this.hasFiveMagic(data)) {
      return false;
    }

    const encodedLen = data[4] + (data[5] << 8) + (data[6] << 16);
    const payloadLen = data.length - SCRIPT_HEADER_LEN;
    return encodedLen === payloadLen;
  }

  private looksLikeOptimizedHeader(data: Uint8Array): boolean {
    if (data.length < OPTIMIZED_HEADER_LEN) {
      return false;
    }

    if (!this.hasFiveMagic(data)) {
      return false;
    }

    return !this.looksLikeScriptHeader(data);
  }

  /**
   * Execute bytecode using real Five VM
   */
  async execute(options: VMExecutionOptions): Promise<VMExecutionResult> {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    const startTime = Date.now();

    try {
      // Context-aware bytecode handling
      let scriptData: Uint8Array;
      
      const hasScriptHeader = this.looksLikeScriptHeader(options.bytecode);
      const hasOptimizedHeader = this.looksLikeOptimizedHeader(options.bytecode);

      if (hasScriptHeader || hasOptimizedHeader) {
        scriptData = options.bytecode;
        if (hasOptimizedHeader) {
          console.log(
            `[WASM VM] Detected optimized header (${options.bytecode.length} bytes); executing without re-wrapping`
          );
        }
      } else if (this.isWasmOnlyExecution()) {
        console.log('[WASM VM] Legacy bytecode detected; wrapping for WASM-only execution');
        if (!wrap_with_script_header) {
          throw this.createVMError('WASM header wrapping function not available');
        }
        const wrapped = wrap_with_script_header(options.bytecode);
        scriptData = new Uint8Array(wrapped);
      } else {
        // Network execution requires proper deployment
        throw this.createVMError(
          'Network execution requires deployed bytecode with ScriptHeader. Use deployment flow for localnet/devnet/mainnet.'
        );
      }

      // Create VM instance with script data
      this.vm = new FiveVMWasm(scriptData);
      
      this.logger.debug(`Executing script data (${scriptData.length} bytes)`);

      // Convert accounts to WASM format
      const wasmAccounts = this.convertAccountsToWasm(options.accounts || []);

      // Prepare input data (VLE encoded if needed)
      const inputData = options.inputData || new Uint8Array(0);

      // Execute with partial execution support
      const result = this.vm.execute_partial(inputData, wasmAccounts);

      const executionTime = Date.now() - startTime;

      // Convert WASM result to our format
      let resultValue = null;
      let success = false;
      let status = 'Failed';
      let errorMessage = undefined;

      // Parse the result based on WASM output format
      this.logger.debug(`VM result type: ${typeof result}, value: ${JSON.stringify(result)}`);
      
      if (typeof result === 'string') {
        // Handle string result like "Ok(Some(U64(2)))" or "Err(StackError)"
        if (result.startsWith('Ok(')) {
          success = true;
          status = 'Completed';
          
          // Extract value from Ok(Some(ValueType(value))) format
          const u64Match = result.match(/Ok\(Some\(U64\((\d+)\)\)\)/);
          const u8Match = result.match(/Ok\(Some\(U8\((\d+)\)\)\)/);
          const i64Match = result.match(/Ok\(Some\(I64\((-?\d+)\)\)\)/);
          const boolMatch = result.match(/Ok\(Some\(Bool\((true|false)\)\)\)/);
          
          if (u64Match) {
            resultValue = { type: 'U64', value: parseInt(u64Match[1]) };
          } else if (u8Match) {
            resultValue = { type: 'U8', value: parseInt(u8Match[1]) };
          } else if (i64Match) {
            resultValue = { type: 'I64', value: parseInt(i64Match[1]) };
          } else if (boolMatch) {
            resultValue = { type: 'Bool', value: boolMatch[1] === 'true' };
          } else if (result === 'Ok(None)') {
            resultValue = null;
          } else {
            // Fallback: try to extract any numeric value for backward compatibility
            const fallbackMatch = result.match(/Ok\(Some\(\w+\((\d+)\)\)\)/);
            if (fallbackMatch) {
              resultValue = parseInt(fallbackMatch[1]);
            }
          }
          
          this.logger.debug(`Parsed result value: ${JSON.stringify(resultValue)}`);
        } else if (result.startsWith('Err(')) {
          success = false;
          status = 'Failed';
          errorMessage = result.replace('Err(', '').replace(')', '');
        }
      } else {
        // Handle object result format  
        this.logger.debug(`Object result properties: ${Object.getOwnPropertyNames(result)}`);
        this.logger.debug(`Object result methods: ${Object.getOwnPropertyNames(Object.getPrototypeOf(result))}`);
        
        // Use getter methods for WASM object properties  
        const resultStatus = typeof result.status === 'function' ? result.status() : result.status;
        success = resultStatus === 'Completed';
        status = resultStatus || 'Completed'; // Default to Completed for partial execution
        
        const resultErrorMessage = typeof result.error_message === 'function' ? result.error_message() : result.error_message;
        errorMessage = resultErrorMessage;
        
        // Access WASM getter properties directly (not function calls)
        this.logger.debug(`has_result_value type: ${typeof result.has_result_value}, value: ${result.has_result_value}`);
        this.logger.debug(`get_result_value type: ${typeof result.get_result_value}, value: ${result.get_result_value}`);
        
        if (result.has_result_value) {
          const rawValue = result.get_result_value;
          // Handle BigInt values from WASM
          if (typeof rawValue === 'bigint') {
            resultValue = Number(rawValue);
            this.logger.debug(`Got BigInt result value, converted to number: ${resultValue}`);
          } else {
            resultValue = rawValue;
            this.logger.debug(`Got result value from WASM getter: ${JSON.stringify(resultValue)}`);
          }
        } else if (result.result_value !== undefined) {
          resultValue = result.result_value;
          this.logger.debug(`Got result value from result_value: ${JSON.stringify(resultValue)}`);
        }
        
        // Try accessing properties directly as they might be getters
        try {
          if (resultValue === null && result.result_value !== undefined) {
            resultValue = result.result_value;
            this.logger.debug(`Got result value from direct property access: ${JSON.stringify(resultValue)}`);
          }
        } catch (e) {
          this.logger.debug(`Failed to access result_value property: ${e}`);
        }
      }

      const vmResult: VMExecutionResult = {
        success,
        result: resultValue,
        computeUnitsUsed: (typeof result === 'object' ? result.compute_units_used : 0) || 0,
        executionTime,
        logs: [],
        status,
        stoppedAt: typeof result === 'object' ? result.stopped_at_opcode_name : undefined,
        errorMessage
      };

      this.logger.debug(`VM execution completed in ${executionTime}ms with status: ${status}`);
      return vmResult;

    } catch (error) {
      const executionTime = Date.now() - startTime;
      return {
        success: false,
        error: {
          type: 'ExecutionError',
          message: error instanceof Error ? error.message : 'Unknown VM error',
          instructionPointer: 0,
          stackTrace: [],
          errorCode: -1
        },
        executionTime,
        logs: []
      };
    }
  }

  /**
   * Execute with function parameters using proper VLE encoding
   */
  async executeFunction(
    bytecode: Uint8Array,
    functionIndex: number,
    parameters: Array<{ type: string; value: any }>,
    accounts?: AccountInfo[]
  ): Promise<VMExecutionResult> {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    try {
      // Debug parameter encoding
      console.log(`[WASM VM] executeFunction called:`);
      console.log(`  Function index: ${functionIndex}`);
      console.log(`  Parameters:`, parameters);
      console.log(`  Parameter count: ${parameters.length}`);
      
      // Use proper VLE encoding that MitoVM expects
      console.log(`[WASM VM] Using proper VLE parameter encoding with function index`);
      
      // Convert parameters to VLE encoder format - force complex encoding for type prefixes
      const paramDefinitions = parameters.map((p, i) => ({
        name: `param${i}`,
        type: p.type
      }));
      
      const paramValues: any = {};
      parameters.forEach((p, i) => {
        paramValues[`param${i}`] = p.value;
      });
      
      console.log(`[WASM VM] VLE parameter definitions:`, paramDefinitions);
      console.log(`[WASM VM] VLE parameter values:`, paramValues);
      
      // Import VLE encoder and force complex encoding that includes type bytes
      const { VLEEncoder } = await import('../lib/vle-encoder.js');
      
      // Use pure VLE compression - encode only parameter values without types
      console.log(`[WASM VM] Using pure VLE compression for parameters`);
      
      // ENGINEERING INTEGRITY FIX: Use proper instruction format with discriminator + function index
      // The complete instruction format is: [discriminator(2), function_index(VLE), param_count(VLE), param1(VLE), param2(VLE)]
      
      const simpleValues = parameters.map(param => param.value);
      
      console.log(`[WASM VM] Pure VLE encoding with values:`, simpleValues);
      if (!ParameterEncoder || !ParameterEncoder.encode_execute_vle) {
        throw new Error('ParameterEncoder WASM binding not loaded');
      }
      const rawVLEParams = ParameterEncoder.encode_execute_vle(functionIndex, simpleValues);
      console.log(`[WASM VM] Raw VLE params (param_count + values):`, Array.from(rawVLEParams));
      
      // Use FiveSDK to create proper instruction format: [discriminator(2), function_index(VLE), ...rawVLEParams]
      const { FiveSDK } = await import('five-sdk');
      const properInstructionData = (FiveSDK as any).encodeExecuteInstruction(functionIndex, new Uint8Array(rawVLEParams));
      
      console.log(`[WASM VM] Complete instruction data:`, Array.from(properInstructionData));
      console.log(`[WASM VM] Complete instruction data (hex):`, Buffer.from(properInstructionData).toString('hex'));
      
      return await this.execute({
        bytecode,
        inputData: properInstructionData,
        accounts: accounts || []
      });
    } catch (error) {
      console.error(`[WASM VM] VLE parameter encoding failed:`, error);
      console.error(`[WASM VM] Encoder input was:`, { functionIndex, parameters });
      console.error(`[WASM VM] Error details:`, error);
      throw this.createVMError('Function execution failed', error as Error);
    }
  }

  /**
   * Get VM state information
   */
  async getState(): Promise<any> {
    if (!this.vm) {
      throw this.createVMError('No VM instance available');
    }

    try {
      const state = this.vm.get_state();
      return JSON.parse(state);
    } catch (error) {
      throw this.createVMError('Failed to get VM state', error as Error);
    }
  }

  /**
   * Validate bytecode before execution
   */
  async validateBytecode(bytecode: Uint8Array): Promise<{ valid: boolean; error?: string }> {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    try {
      const valid = FiveVMWasm.validate_bytecode(bytecode);
      return { valid };
    } catch (error) {
      return { 
        valid: false, 
        error: error instanceof Error ? error.message : 'Unknown validation error' 
      };
    }
  }

  /**
   * Get VM constants and opcodes
   */
  getVMConstants(): any {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    try {
      const constants = FiveVMWasm.get_constants();
      return JSON.parse(constants);
    } catch (error) {
      throw this.createVMError('Failed to get VM constants', error as Error);
    }
  }

  /**
   * Convert accounts to WASM format
   */
  private convertAccountsToWasm(accounts: AccountInfo[]): any[] {
    const wasmAccounts = [];

    for (const account of accounts) {
      try {
        const wasmAccount = new WasmAccount(
          account.key,
          account.data || new Uint8Array(0),
          account.lamports || 0,
          account.isWritable || false,
          account.isSigner || false,
          account.owner || new Uint8Array(32)
        );
        wasmAccounts.push(wasmAccount);
      } catch (error) {
        this.logger.warn(`Failed to convert account ${account.key}: ${error}`);
      }
    }

    return wasmAccounts;
  }

  /**
   * Create a standardized VM error
   */
  private createVMError(message: string, cause?: Error): CLIError {
    const error = new Error(message) as CLIError;
    error.name = 'VMError';
    error.code = 'VM_ERROR';
    error.category = 'wasm';
    error.exitCode = 1;
    
    if (cause) {
      error.details = {
        cause: cause.message,
        stack: cause.stack
      };
    }
    
    return error;
  }

  /**
   * Get VM capabilities and version info
   */
  getVMInfo(): { version: string; features: string[] } {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    // Return basic info since real VM doesn't expose this directly
    return {
      version: '1.0.0',
      features: [
        'partial-execution',
        'system-call-detection', 
        'vle-parameter-encoding',
        'account-simulation',
        'bytecode-validation'
      ]
    };
  }

  /**
   * Check if VM is ready for execution
   */
  isReady(): boolean {
    return this.initialized;
  }

  /**
   * Clean up VM resources
   */
  cleanup(): void {
    this.vm = null;
    this.logger.debug('VM resources cleaned up');
  }
}
