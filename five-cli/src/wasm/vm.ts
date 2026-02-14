// Five VM WASM integration.

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

  async initialize(): Promise<void> {
    try {
      console.log('[DEBUG] Starting VM WASM initialization...');

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

  private isWasmOnlyExecution(): boolean {
    const args = process.argv;
    
    if (args.includes('local')) {
      return true;
    }
    
    const scriptName = process.argv[1];
    if (scriptName && (scriptName.includes('test-runner') || scriptName.includes('local'))) {
      return true;
    }
    
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

  async execute(options: VMExecutionOptions): Promise<VMExecutionResult> {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    const startTime = Date.now();

    try {
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

      this.vm = new FiveVMWasm(scriptData);
      
      this.logger.debug(`Executing script data (${scriptData.length} bytes)`);

      const wasmAccounts = this.convertAccountsToWasm(options.accounts || []);

      // Prepare input data
      const inputData = options.inputData || new Uint8Array(0);

      const result = this.vm.execute_partial(inputData, wasmAccounts);

      const executionTime = Date.now() - startTime;

      let resultValue = null;
      let success = false;
      let status = 'Failed';
      let errorMessage = undefined;

      this.logger.debug(`VM result type: ${typeof result}, value: ${JSON.stringify(result)}`);
      
      if (typeof result === 'string') {
        if (result.startsWith('Ok(')) {
          success = true;
          status = 'Completed';
          
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
        this.logger.debug(`Object result properties: ${Object.getOwnPropertyNames(result)}`);
        this.logger.debug(`Object result methods: ${Object.getOwnPropertyNames(Object.getPrototypeOf(result))}`);
        
        const resultStatus = typeof result.status === 'function' ? result.status() : result.status;
        success = resultStatus === 'Completed';
        status = resultStatus || 'Completed'; // Default to Completed for partial execution
        
        const resultErrorMessage = typeof result.error_message === 'function' ? result.error_message() : result.error_message;
        errorMessage = resultErrorMessage;
        
        this.logger.debug(`has_result_value type: ${typeof result.has_result_value}, value: ${result.has_result_value}`);
        this.logger.debug(`get_result_value type: ${typeof result.get_result_value}, value: ${result.get_result_value}`);
        
        if (result.has_result_value) {
          const rawValue = result.get_result_value;
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

  // Execute with fixed-width parameter encoding.
  // Uses ParameterEncoder.encode_execute which produces packed bytes.
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
      console.log(`[WASM VM] executeFunction called:`);
      console.log(`  Function index: ${functionIndex}`);
      console.log(`  Parameters:`, parameters);
      console.log(`  Parameter count: ${parameters.length}`);
      
      if (!ParameterEncoder || !ParameterEncoder.encode_execute) {
        throw new Error('ParameterEncoder WASM binding not loaded or missing encode_execute');
      }

      const simpleValues = parameters.map(param => param.value);
      const rawParams = ParameterEncoder.encode_execute(functionIndex, simpleValues);

      const { BytecodeEncoder } = await import('@5ive-tech/sdk');
      const discriminator = new Uint8Array([9]);
      const functionIndexBytes = BytecodeEncoder.encodeU32(functionIndex);
      const instructionData = new Uint8Array(
        discriminator.length + functionIndexBytes.length + rawParams.length
      );
      instructionData.set(discriminator, 0);
      instructionData.set(functionIndexBytes, discriminator.length);
      instructionData.set(rawParams, discriminator.length + functionIndexBytes.length);

      return await this.execute({
        bytecode,
        inputData: instructionData,
        accounts: accounts || []
      });
    } catch (error) {
      console.error(`[WASM VM] Parameter encoding failed:`, error);
      console.error(`[WASM VM] Encoder input was:`, { functionIndex, parameters });
      console.error(`[WASM VM] Error details:`, error);
      throw this.createVMError('Function execution failed', error as Error);
    }
  }

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

  getVMInfo(): { version: string; features: string[] } {
    if (!this.initialized) {
      throw this.createVMError('VM not initialized');
    }

    return {
      version: '1.0.0',
      features: [
        'partial-execution',
        'system-call-detection', 
        'fixed-parameter-encoding',
        'account-simulation',
        'bytecode-validation'
      ]
    };
  }

  isReady(): boolean {
    return this.initialized;
  }

  cleanup(): void {
    this.vm = null;
    this.logger.debug('VM resources cleaned up');
  }
}
