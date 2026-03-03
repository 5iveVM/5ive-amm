// Five VM WASM integration.

import {
  VMExecutionOptions,
  VMExecutionResult,
  AccountInfo,
  Logger,
  CLIError
} from '../types.js';

import {
  SCRIPT_ACCOUNT_HEADER_LEN,
  SCRIPT_BYTECODE_HEADER_V1_LEN,
} from '../constants/headers.js';
import { getWasmModule } from './loader.js';

let FiveVMWasm: any;
let WasmAccount: any;
let ParameterEncoder: any;
let wrap_with_script_header: undefined | ((bytecode: Uint8Array) => Uint8Array);

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
      this.logger.debug('[DEBUG] Starting VM WASM initialization...');

      const wasmModule = await getWasmModule();

      FiveVMWasm = wasmModule.FiveVMWasm;
      WasmAccount = wasmModule.WasmAccount;
      ParameterEncoder = wasmModule.ParameterEncoder;

      if (typeof (wasmModule as any).wrap_with_script_header !== 'function') {
        // Warning only? Or throw?
        this.logger.warn('WASM VM missing wrap_with_script_header');
      } else {
        wrap_with_script_header = (wasmModule as any).wrap_with_script_header;
      }

      this.initialized = true;
    } catch (error) {
      throw this.createVMError(
        'Five VM WASM modules not found. Please run "npm run build:wasm".',
        error as Error
      );
    }
  }

  private isWasmOnlyExecution(): boolean {
    // Check basic env
    if (typeof process !== 'undefined' && process.argv) {
      if (process.argv.includes('local')) return true;
    }
    return true;
  }

  private hasFiveMagic(data: Uint8Array): boolean {
    if (data.length < FIVE_MAGIC.length) return false;
    return FIVE_MAGIC.every((byte, index) => data[index] === byte);
  }

  private looksLikeScriptAccountHeader(data: Uint8Array): boolean {
    if (data.length < SCRIPT_ACCOUNT_HEADER_LEN) return false;
    if (!this.hasFiveMagic(data)) return false;
    const encodedLen = data[4] + (data[5] << 8) + (data[6] << 16);
    const payloadLen = data.length - SCRIPT_ACCOUNT_HEADER_LEN;
    return encodedLen === payloadLen;
  }

  private looksLikeScriptBytecodeHeaderV1(data: Uint8Array): boolean {
    if (data.length < SCRIPT_BYTECODE_HEADER_V1_LEN) return false;
    if (!this.hasFiveMagic(data)) return false;
    return !this.looksLikeScriptAccountHeader(data);
  }

  async execute(options: VMExecutionOptions): Promise<VMExecutionResult> {
    if (!this.initialized) await this.initialize();
    if (!this.initialized) throw this.createVMError('VM not initialized');

    const startTime = Date.now();

    try {
      let scriptData: Uint8Array;
      const hasScriptAccountHeader = this.looksLikeScriptAccountHeader(options.bytecode);
      const hasScriptBytecodeHeaderV1 = this.looksLikeScriptBytecodeHeaderV1(options.bytecode);

      if (hasScriptAccountHeader || hasScriptBytecodeHeaderV1) {
        scriptData = options.bytecode;
      } else {
        if (!wrap_with_script_header) {
          // Fallback if binding missing
          scriptData = options.bytecode;
        } else {
          const wrapped = wrap_with_script_header(options.bytecode);
          scriptData = new Uint8Array(wrapped);
        }
      }

      // Create VM instance
      this.vm = new FiveVMWasm(scriptData);

      const wasmAccounts = this.convertAccountsToWasm(options.accounts || []);
      const inputData = options.inputData || new Uint8Array(0);

      // Execute
      const result = this.vm.execute_partial(inputData, wasmAccounts);
      const executionTime = Date.now() - startTime;

      return this.parseVMResult(result, executionTime);

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

  private parseVMResult(result: any, executionTime: number): VMExecutionResult {
    let resultValue = null;
    let success = false;
    let status = 'Failed';
    let errorMessage = undefined;

  // Handle simple string results.
    if (typeof result === 'string') {
      if (result.startsWith('Ok(')) {
        success = true;
        status = 'Completed';
        // Regex parsing for basic types
        const u64Match = result.match(/Ok\(Some\(U64\((\d+)\)\)\)/);
        if (u64Match) resultValue = { type: 'U64', value: parseInt(u64Match[1]) };
        // ... (Can expand primitive parsing if needed, but maintaining minimal logic here)
      } else if (result.startsWith('Err(')) {
        success = false;
        status = 'Failed';
        errorMessage = result;
      }
    } else {
      // Handle Object result (WASM binding struct)
      const resultStatus = typeof result.status === 'function' ? result.status() : result.status;
      success = resultStatus === 'Completed';
      status = resultStatus || 'Completed';

      const resultErrorMessage = typeof result.error_message === 'function' ? result.error_message() : result.error_message;
      errorMessage = resultErrorMessage;

      if (result.has_result_value) {
        const raw = result.get_result_value;
        resultValue = typeof raw === 'bigint' ? Number(raw) : raw;
      } else if (result.result_value !== undefined) {
        resultValue = result.result_value;
      }
    }

    return {
      success,
      result: resultValue,
      computeUnitsUsed: typeof result === 'object' ? result.compute_units_used : 0,
      executionTime,
      logs: [],
      status,
      stoppedAt: typeof result === 'object' ? result.stopped_at_opcode_name : undefined,
      errorMessage
    };
  }

  async executeFunction(
    bytecode: Uint8Array,
    functionIndex: number,
    parameters: Array<{ type: string; value: any }>,
    accounts?: AccountInfo[]
  ): Promise<VMExecutionResult> {
    if (!this.initialized) await this.initialize();

    // Import BytecodeEncoder
    const { BytecodeEncoder } = await import('../lib/bytecode-encoder.js');

    // Pass typed parameters through to WASM encoder
    const typedParams = parameters.map(param => ({
      type: param.type,
      value: param.value
    }));

    if (!ParameterEncoder || !ParameterEncoder.encode_execute) {
      throw new Error('ParameterEncoder WASM binding not loaded or missing encode_execute');
    }
    // Use WASM binding to encode parameters (returns fixed-size encoded params)
    const rawParams = ParameterEncoder.encode_execute(functionIndex, typedParams);

    // EXECUTE_INSTRUCTION is 9 (matches on-chain protocol)
    const discriminator = new Uint8Array([9]);

    // Encode function index as u32 little endian
    const functionIndexBytes = BytecodeEncoder.encodeU32(functionIndex);

    // Assemble: [discriminator, function_index(u32), param_count(u32), rawParams...]

    // Construct instruction:
    const functionIndexArr = BytecodeEncoder.encodeU32(functionIndex); // 4 bytes
    const paramCountArr = BytecodeEncoder.encodeU32(parameters.length);

    const properInstructionData = new Uint8Array(
      discriminator.length + functionIndexArr.length + paramCountArr.length + rawParams.length
    );
    properInstructionData.set(discriminator, 0);
    properInstructionData.set(functionIndexArr, discriminator.length);
    properInstructionData.set(paramCountArr, discriminator.length + functionIndexArr.length);
    properInstructionData.set(rawParams, discriminator.length + functionIndexArr.length + paramCountArr.length);

    return await this.execute({
      bytecode,
      inputData: properInstructionData,
      accounts: accounts || []
    });
  }

  async getState(): Promise<any> {
    if (!this.vm) throw this.createVMError('No VM instance available');
    return JSON.parse(this.vm.get_state());
  }

  async validateBytecode(bytecode: Uint8Array): Promise<{ valid: boolean; error?: string }> {
    if (!this.initialized) await this.initialize();
    try {
      const valid = FiveVMWasm.validate_bytecode(bytecode);
      return { valid };
    } catch (error) {
      return { valid: false, error: String(error) };
    }
  }

  getVMConstants(): any {
    if (!this.initialized) throw this.createVMError('VM not initialized');
    return JSON.parse(FiveVMWasm.get_constants());
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
        this.loggerwarn(`Failed to convert account ${account.key}: ${error}`);
      }
    }
    return wasmAccounts;
  }

  private loggerwarn(msg: string) {
    if (this.logger && this.logger.warn) this.logger.warn(msg);
    else console.warn(msg);
  }

  private createVMError(message: string, cause?: Error): CLIError {
    const error = new Error(message) as CLIError;
    error.name = 'VMError';
    error.code = 'VM_ERROR';
    error.category = 'wasm';

    if (cause) {
      error.details = { cause: cause.message };
    }
    return error;
  }

  getVMInfo() {
    return { version: '1.0.0', features: ['wasm', 'sdk'] };
  }

  isReady() { return this.initialized; }
  cleanup() { this.vm = null; }
}
