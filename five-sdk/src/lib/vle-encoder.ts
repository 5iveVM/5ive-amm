/**
 * VLE (Variable Length Encoding) Encoder for Five VM Protocol
 * 
 * Uses the existing WASM VLE and Parameter encoders for protocol-compliant
 * encoding of execute instructions.
 * 
 * Execute instruction format:
 * [discriminator(2), vle_function_index, vle_param_count, ...vle_parameters]
 */

import { getWasmModule } from '../wasm/loader.js';

let wasmModule: any = null;

export interface ParameterDefinition {
  name: string;
  type: string;
  isAccount?: boolean;
  is_account?: boolean;
  param_type?: string;
}

export interface ParameterValue {
  [key: string]: any;
}

/**
 * Type mapping for VLE encoding - matches Five protocol types
 */
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

const TYPED_PARAM_SENTINEL = 0x80;

/**
 * VLE Encoder utility class that uses WASM module for protocol compliance
 */
export class VLEEncoder {
  /**
   * Get type ID for VLE encoding
   */
  static getTypeId(type: string): number {
    const typeId = TYPE_IDS[type.toLowerCase()];
    if (typeId === undefined) {
      throw new Error(`Unknown type for VLE encoding: ${type}`);
    }
    return typeId;
  }

  /**
   * Encode execute instruction data using WASM ParameterEncoder
   */
  static async encodeExecuteVLE(
    functionIndex: number,
    parameters: ParameterDefinition[] = [],
    values: ParameterValue = {},
    retry: boolean = true,
    options: any = {}
  ): Promise<Buffer> {
    // ⚡ Normalize parameter types before encoding to handle custom types (Mint, TokenAccount, etc.)
    const normalizedParameters = parameters.map(p => ({
      ...p,
      type: this.normalizeType(p)
    }));

    // Load WASM module using shared loader
    if (!wasmModule) {
      try {
        wasmModule = await getWasmModule();

        // Check if loaded module is valid
        if (wasmModule && wasmModule.ParameterEncoder) {
          try {
            // Health check: Try to encode empty params
            wasmModule.ParameterEncoder.encode_execute_vle(0, []);
          } catch (e: any) {
            console.warn("[VLE] Module validation failed, falling back:", e.message);
            wasmModule = null; // Force retry with inline loader
          }
        } else {
          wasmModule = null;
        }
      } catch (e) {
        // Silently ignore loader errors and try fallback
        wasmModule = null;
      }

      // Fallback: Import the proper wasm-pack generated module for Node.js
      if (!wasmModule && typeof process !== 'undefined') {
        console.log("[DEBUG VLE] (SRC) Attempting wasm-pack module import...");
        try {
          const fs = await import('fs');
          const path = await import('path');
          const url = await import('url');

          const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
          // Import the wasm-pack generated entry point (five_vm_wasm.js)
          // which already handles all the WASM initialization
          const moduleEntryPath = path.resolve(__dirname, '../assets/vm/five_vm_wasm.js');

          if (fs.existsSync(moduleEntryPath)) {
            console.log("[DEBUG VLE] Found WASM module at:", moduleEntryPath);
            const wasmMod = await import(moduleEntryPath);

            // The wasm-pack module exports the initialized module directly
            if (wasmMod && wasmMod.ParameterEncoder) {
              wasmModule = wasmMod;
              console.log("[DEBUG VLE] WASM module imported and initialized successfully!");
            } else {
              console.error("[DEBUG VLE] WASM module missing expected exports");
            }
          } else {
            console.error("[DEBUG VLE] WASM module not found at:", moduleEntryPath);
          }
        } catch (err) {
          console.error("[DEBUG VLE] Module import FAILED:", err);
          // Don't throw - let it fall through to error handling below
        }
      }
    }

    // Do not filter out account parameters. 
    // Even if the VM handles them specially via AccountRef, they should still be 
    // part of the parameter list for correct parameter counting and displacement.
    const filteredParams = normalizedParameters;
    const paramValues = filteredParams.map(param => {
      const value = values[param.name];
      if (value === undefined || value === null) {
        throw new Error(`Missing value for parameter: ${param.name}`);
      }
      return { param, value };
    });

    if (options && (options as any).debug) {
      console.log(`[VLE] Parameters:`, paramValues.map(p => ({
        name: p.param.name,
        type: p.param.type || (p.param as any).param_type,
        normalized: this.normalizeType(p.param),
        isAccount: this.isAccountParam(p.param)
      })));
    }

    // Always use WASM encoder for all parameter types (accounts, pubkeys, strings, etc.)
    // The WASM encoder handles mixed-type parameters correctly via VLE encoding
    // This avoids the complexity and unreliability of manual typed params handling

    // Build parameter array with metadata for WASM encoder to distinguish account vs pubkey types
    const paramArray = paramValues.map(({ param, value }) => {
      const normalizedType = this.normalizeType(param);
      // Create object with type information so WASM encoder can distinguish accounts from pubkeys
      if (normalizedType === 'account') {
        // Mark account parameters with type metadata
        return { __type: 'account', value };
      }
      return value;
    });

    try {
      const encoded = wasmModule.ParameterEncoder.encode_execute_vle(functionIndex, paramArray);
      if (options && (options as any).debug) {
        const buf = Buffer.from(encoded);
        console.log(`[VLE] WASM encoded ${paramArray.length} parameters: ${buf.length} bytes`);
      }
      return Buffer.from(encoded);
    } catch (e) {
      console.warn("[VLE] Encode failed via WASM:", e);
      if (retry) {
        console.warn("[VLE] Reloading WASM module and retrying...");
        wasmModule = null; // Force reload
        return this.encodeExecuteVLE(functionIndex, parameters, values, false, options);
      }
      throw e;
    }
  }

  private static isAccountParam(param: ParameterDefinition): boolean {
    if ((param as any).isAccount || (param as any).is_account) {
      return true;
    }
    const type = this.normalizeType(param);
    return type === 'account';
  }

  private static normalizeType(param: ParameterDefinition): string {
    if (param.isAccount || param.is_account) {
      return 'account';
    }
    const type = (param.type || param.param_type || '').toString().trim().toLowerCase();

    // Special handling for common account-backed types in the DSL
    if (['mint', 'tokenaccount'].includes(type)) {
      return 'account';
    }

    return type;
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
