// Bytecode encoder for the Five VM protocol.

import { getWasmModule } from '../wasm/loader.js';

let wasmModule: any = null;

export interface ParameterDefinition {
  name: string;
  type: string;
  isAccount?: boolean;
  is_account?: boolean;
  param_type?: string;
  maxLen?: number;
}

export interface ParameterValue {
  [key: string]: any;
}

// Type mapping for encoding - matches Five protocol types.
const TYPE_IDS: Record<string, number> = {
  'u8': 1,
  'u16': 2,
  'u32': 3,
  'u64': 4,
  'i8': 5,
  'i16': 6,
  'i32': 7,
  'i64': 8,
  'bool': 9,
  'pubkey': 10,
  'string': 11,
  'account': 12,
  'bytes': 11,
  'array': 13
};

export class BytecodeEncoder {
  /**
   * Parse a raw ABI type string into a canonical execute-encoding type.
   * Supports sized string syntax like string<32>.
   */
  static parseTypeSpec(rawType: string): { baseType: string; maxLen?: number } {
    const normalized = (rawType || '').toString().trim().toLowerCase();

    const sizedStringMatch = normalized.match(/^string\s*<\s*(\d+)\s*>$/);
    if (sizedStringMatch) {
      return {
        baseType: 'string',
        maxLen: Number.parseInt(sizedStringMatch[1], 10),
      };
    }

    return { baseType: normalized };
  }

  /**
   * Get type ID for encoding.
   */
  static getTypeId(type: string): number {
    const typeId = TYPE_IDS[type.toLowerCase()];
    if (typeId === undefined) {
      throw new Error(`Unknown type for encoding: ${type}`);
    }
    return typeId;
  }

  /**
   * Encode a 32-bit unsigned integer (Little Endian).
   */
  static encodeU32(value: number): Uint8Array {
      const buffer = new Uint8Array(4);
      buffer[0] = value & 0xff;
      buffer[1] = (value >> 8) & 0xff;
      buffer[2] = (value >> 16) & 0xff;
      buffer[3] = (value >>> 24) & 0xff;
      return buffer;
  }

  /**
   * Encode execute instruction data using WASM ParameterEncoder.
   */
  static async encodeExecute(
    functionIndex: number,
    parameters: ParameterDefinition[] = [],
    values: ParameterValue = {},
    retry: boolean = true,
    options: any = {}
  ): Promise<Buffer> {
    // Normalize parameter types before encoding to handle custom types (Mint, TokenAccount, etc.).
    const normalizedParameters = parameters.map(p => {
      const typeSpec = this.getCanonicalTypeSpec(p);
      return {
        ...p,
        type: typeSpec.baseType,
        maxLen: typeSpec.maxLen,
      };
    });

    // Load WASM module using shared loader
    if (!wasmModule) {
      try {
        wasmModule = await getWasmModule();

        // Check if loaded module is valid.
        if (wasmModule && wasmModule.ParameterEncoder) {
          try {
            // Health check: try to encode empty params.
            wasmModule.ParameterEncoder.encode_execute(0, []);
          } catch (e: any) {
            console.warn("[BytecodeEncoder] Module validation failed, falling back:", e.message);
            wasmModule = null; // Force retry with inline loader
          }
        } else {
          wasmModule = null;
        }
      } catch (e) {
        // Silently ignore loader errors and try fallback
        wasmModule = null;
      }

      // Fallback: import the wasm-pack generated module for Node.js.
      if (!wasmModule && typeof process !== 'undefined') {
        console.log("[DEBUG] (SRC) Attempting wasm-pack module import...");
        try {
          const fs = await import('fs');
          const path = await import('path');
          const url = await import('url');

          const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
          // Import the wasm-pack generated entry point (five_vm_wasm.js)
          // which already handles all the WASM initialization.
          const moduleEntryPath = path.resolve(__dirname, '../assets/vm/five_vm_wasm.js');

          if (fs.existsSync(moduleEntryPath)) {
            console.log("[DEBUG] Found WASM module at:", moduleEntryPath);
            const wasmMod = await import(moduleEntryPath);

            // The wasm-pack module exports the initialized module directly.
            if (wasmMod && wasmMod.ParameterEncoder) {
              wasmModule = wasmMod;
              console.log("[DEBUG] WASM module imported and initialized successfully!");
            } else {
              console.error("[DEBUG] WASM module missing expected exports");
            }
          } else {
            console.error("[DEBUG] WASM module not found at:", moduleEntryPath);
          }
        } catch (err) {
          console.error("[DEBUG] Module import FAILED:", err);
          // Don't throw - let it fall through to error handling below.
        }
      }
    }

    const filteredParams = normalizedParameters;
    const paramValues = filteredParams.map(param => {
      const value = values[param.name];
      if (value === undefined || value === null) {
        throw new Error(`Missing value for parameter: ${param.name}`);
      }
      return { param, value };
    });

    if (options && (options as any).debug) {
      console.log(`[BytecodeEncoder] Parameters:`, paramValues.map(p => ({
        name: p.param.name,
        type: p.param.type || (p.param as any).param_type,
        normalized: this.getCanonicalTypeSpec(p.param).baseType,
        maxLen: this.getCanonicalTypeSpec(p.param).maxLen,
        isAccount: this.isAccountParam(p.param)
      })));
    }

    // Build parameter array with full metadata for fixed-size WASM encoder
    const paramArray = paramValues.map(({ param, value }) => {
      const canonicalType = this.getCanonicalTypeSpec(param);
      // Pass full parameter definition for type-aware encoding
      return {
        name: param.name,
        type: canonicalType.baseType,
        param_type: canonicalType.baseType,
        maxLen: canonicalType.maxLen,
        isAccount: canonicalType.baseType === 'account',
        is_account: canonicalType.baseType === 'account',
        value: value
      };
    });

    try {
      // Debug logging to understand parameter encoding
      console.log(`[BytecodeEncoder] About to encode with paramArray:`, JSON.stringify(paramArray.map(p => ({
        name: (p as any).name || 'unknown',
        type: (p as any).type || 'unknown',
        hasValue: (p as any).value !== undefined
      }))));

      const encoded = wasmModule.ParameterEncoder.encode_execute(functionIndex, paramArray);
      if (options && (options as any).debug) {
        const buf = Buffer.from(encoded);
        console.log(`[BytecodeEncoder] WASM encoded ${paramArray.length} parameters: ${buf.length} bytes`);
      }
      console.log(`[BytecodeEncoder] Encoded result: ${Buffer.from(encoded).toString('hex')}`);
      return Buffer.from(encoded);
    } catch (e) {
      console.warn("[BytecodeEncoder] Encode failed via WASM:", e);
      if (retry) {
        console.warn("[BytecodeEncoder] Reloading WASM module and retrying...");
        wasmModule = null; // Force reload.
        return this.encodeExecute(functionIndex, parameters, values, false, options);
      }
      throw e;
    }
  }

  private static isAccountParam(param: ParameterDefinition): boolean {
    if ((param as any).isAccount || (param as any).is_account) {
      return true;
    }
    const type = this.getCanonicalTypeSpec(param).baseType;
    return type === 'account';
  }

  private static getCanonicalTypeSpec(param: ParameterDefinition): { baseType: string; maxLen?: number } {
    if (param.isAccount || param.is_account) {
      return { baseType: 'account' };
    }

    const rawType = (param.type || param.param_type || '').toString().trim();
    const parsed = this.parseTypeSpec(rawType);

    // Special handling for common account-backed types in the DSL
    if (['mint', 'tokenaccount'].includes(parsed.baseType)) {
      return { baseType: 'account' };
    }

    return parsed;
  }

  private static normalizeType(param: ParameterDefinition): string {
    return this.getCanonicalTypeSpec(param).baseType;
  }

  /**
   * Parse parameter definitions from JSON string
   */
  static parseParameters(parametersJson: string): { definitions: ParameterDefinition[], values: ParameterValue } {
    try {
      const parsed = JSON.parse(parametersJson);

      if (Array.isArray(parsed)) {
        // Array format: [{ name: "a", type: "u64", value: 123 }, ...]
        const definitions: ParameterDefinition[] = [];
        const values: ParameterValue = {};

        for (const param of parsed) {
          if (!param.name || !param.type || param.value === undefined) {
            throw new Error('Parameter must have name, type, and value fields');
          }
          definitions.push({ name: param.name, type: param.type });
          values[param.name] = param.value;
        }

        return { definitions, values };
      } else {
        throw new Error('Object format parameters require type definitions. Use array format: [{ name: "a", type: "u64", value: 123 }]');
      }
    } catch (error) {
      throw new Error(`Invalid parameters JSON: ${error instanceof Error ? error.message : 'Unknown error'}`);
    }
  }

  /**
   * Validate WASM module availability
   */
  static async validateWasmModule(): Promise<{ available: boolean; error?: string }> {
    try {
      if (!wasmModule) {
        wasmModule = await getWasmModule();
      }

      if (!wasmModule.ParameterEncoder) {
        return { available: false, error: 'ParameterEncoder not found in WASM module' };
      }
      return { available: true };
    } catch (error) {
      return {
        available: false,
        error: `WASM module not available: ${error instanceof Error ? error.message : 'Unknown error'}`
      };
    }
  }

  /**
   * Create typed parameter definitions with validation
   */
  static createTypedParameters(params: Array<{
    name: string;
    type: string;
    value: any;
    optional?: boolean;
  }>): { definitions: ParameterDefinition[], values: ParameterValue } {
    const definitions: ParameterDefinition[] = [];
    const values: ParameterValue = {};

    for (const param of params) {
      if (!TYPE_IDS[param.type.toLowerCase()]) {
        throw new Error(`Unknown parameter type: ${param.type}`);
      }

      if (!param.optional && (param.value === undefined || param.value === null)) {
        throw new Error(`Missing value for required parameter: ${param.name}`);
      }

      definitions.push({ name: param.name, type: param.type });
      if (param.value !== undefined) {
        values[param.name] = param.value;
      }
    }

    return { definitions, values };
  }
}
