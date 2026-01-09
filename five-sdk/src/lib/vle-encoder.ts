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

      // Fallback: Inline load for Node.js
      if (!wasmModule && typeof process !== 'undefined') {
        console.log("[DEBUG VLE] (SRC) Attempting inline WASM load...");
        try {
          const fs = await import('fs');
          const path = await import('path');
          const url = await import('url');

          const __dirname = path.dirname(url.fileURLToPath(import.meta.url));
          // Assuming running from dist/lib/vle-encoder.js, assets are in ../assets/vm/
          const bgPath = path.resolve(__dirname, '../assets/vm/five_vm_wasm_bg.js');
          const wasmPath = path.resolve(__dirname, '../assets/vm/five_vm_wasm_bg.wasm');

          if (fs.existsSync(bgPath) && fs.existsSync(wasmPath)) {
            console.log("[DEBUG VLE] Found assets at:", bgPath);
            const bg = await import(bgPath);
            const bytes = fs.readFileSync(wasmPath);
            const mod = new WebAssembly.Module(bytes);
            const instance = new WebAssembly.Instance(mod, { "./five_vm_wasm_bg.js": bg });

            if (bg.__wbg_set_wasm) {
              bg.__wbg_set_wasm(instance.exports);
              wasmModule = bg;
              console.log("[DEBUG VLE] Inline load SUCCESS!");
            } else {
              console.error("[DEBUG VLE] bg module missing __wbg_set_wasm export");
            }
          } else {
            console.error("[DEBUG VLE] Assets not found at expected path:", bgPath);
          }
        } catch (err) {
          console.error("[DEBUG VLE] Inline load FAILED:", err);
          throw err;
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

    const encodeVleU32 = (value: number): Buffer => {
      const bytes: number[] = [];
      let remaining = value >>> 0;
      while (remaining >= 0x80) {
        bytes.push((remaining & 0x7f) | 0x80);
        remaining >>>= 7;
      }
      bytes.push(remaining);
      return Buffer.from(bytes);
    };

    const encodeVleU64 = (value: number | bigint): Buffer => {
      const bytes: number[] = [];
      let remaining = BigInt(value);
      if (remaining < BigInt(0)) {
        throw new Error("Negative values not supported for unsigned VLE");
      }
      while (remaining >= BigInt(0x80)) {
        bytes.push(Number((remaining & BigInt(0x7f)) | BigInt(0x80)));
        remaining >>= BigInt(7);
      }
      bytes.push(Number(remaining));
      return Buffer.from(bytes);
    };

    const usesTypedParams = paramValues.some(({ param }) => {
      const type = this.normalizeType(param);
      const isTyped = this.isBytesParam(param) ||
        type === 'pubkey' ||
        this.isAccountParam(param);

      return isTyped;
    });

    if (options && (options as any).debug) {
      console.log(`[VLE] usesTypedParams: ${usesTypedParams}`);
      console.log(`[VLE] Parameters:`, paramValues.map(p => ({
        name: p.param.name,
        type: p.param.type || (p.param as any).param_type,
        normalized: this.normalizeType(p.param),
        isAccount: this.isAccountParam(p.param)
      })));
    }

    if (!usesTypedParams) {
      // Use PURE VLE compression - encode only values without type information
      const simpleValues = paramValues.map(({ value }) => value);
      try {
        const encoded = wasmModule.ParameterEncoder.encode_execute_vle(functionIndex, simpleValues);
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

    const parts: Buffer[] = [];
    // IMPORTANT: functionIndex is NOT included here.
    // FiveSDK.ts's encodeExecuteInstruction will prepended discriminator(9) and functionIndex.

    // For typed params, we prepend the sentinel and the actual count.
    // FiveSDK.ts expects this format to detect typed mode.
    parts.push(encodeVleU32(TYPED_PARAM_SENTINEL)); // Signal typed params mode
    parts.push(encodeVleU32(normalizedParameters.length)); // Actual param count

    for (const { param, value } of paramValues) {
      parts.push(this.encodeTypedParam(param, value, encodeVleU32, encodeVleU64, options));
    }

    const finalBuffer = Buffer.concat(parts);
    if (options.debug) {
      console.log(`[VLE] Protocol buffer generated:`, {
        usesTypedParams,
        length: finalBuffer.length,
        hex: finalBuffer.toString('hex')
      });
    }

    return finalBuffer;
  }

  private static encodeTypedParam(
    param: ParameterDefinition,
    value: any,
    encodeVleU32: (value: number) => Buffer,
    encodeVleU64: (value: number | bigint) => Buffer,
    options?: any,
  ): Buffer {
    const typeName = this.normalizeType(param);
    const typeId = this.getTypeId(typeName);

    if (typeId === TYPE_IDS.string) {
      let bytes: Buffer;
      if (typeof value === 'string') {
        bytes = Buffer.from(value, 'utf8');
      } else if (value instanceof Uint8Array) {
        bytes = Buffer.from(value);
      } else {
        throw new Error(`Invalid value for string/bytes parameter: ${param.name}`);
      }
      const lenBytes = encodeVleU32(bytes.length);
      return Buffer.concat([Buffer.from([typeId]), lenBytes, bytes]);
    }

    if (typeId === TYPE_IDS.bool) {
      const boolValue = value ? 1 : 0;
      const valBytes = encodeVleU32(boolValue);
      return Buffer.concat([Buffer.from([typeId]), valBytes]);
    }

    // Handle pubkey type - decode base58 to bytes, encode as STRING since VM rejects PUBKEY type
    // VM's typed param parsing explicitly returns TypeMismatch for PUBKEY type (line 458-459 in utils.rs)
    // Pubkeys are sent as length-prefixed binary data, same format as STRING
    if (typeId === TYPE_IDS.pubkey) {
      let bytes: Buffer;
      if (typeof value === 'string') {
        // Base58 decode the pubkey string
        try {
          const bs58 = require('bs58');
          bytes = Buffer.from(bs58.decode(value));
        } catch {
          // Fallback: try to decode as base58 using dynamic import
          // If bs58 is not available, treat as UTF-8 bytes (for testing)
          bytes = Buffer.from(value, 'utf8');
        }
      } else if (value instanceof Uint8Array) {
        bytes = Buffer.from(value);
      } else if (value && typeof value === 'object') {
        // Handle Solana PublicKey objects
        if (typeof (value as any).toBuffer === 'function') {
          bytes = (value as any).toBuffer();
        } else if (typeof (value as any).toBytes === 'function') {
          bytes = Buffer.from((value as any).toBytes());
        } else {
          throw new Error(`Invalid object for pubkey parameter: ${param.name}`);
        }
      } else {
        throw new Error(`Invalid value for pubkey parameter: ${param.name}`);
      }

      if (bytes.length !== 32) {
        throw new Error(`Invalid pubkey length for parameter ${param.name}: expected 32 bytes, got ${bytes.length}`);
      }
      // Encode as PUBKEY type (TYPE_IDS.pubkey) without length header
      return Buffer.concat([Buffer.from([TYPE_IDS.pubkey]), bytes]);
    }

    if (typeId === TYPE_IDS.account) {
      // For account parameters, we expect the FiveSDK to have already mapped
      // PublicKey objects to account indices. At this point, value should be numeric.
      const accountIdx = Number(value);
      if (isNaN(accountIdx)) {
        throw new Error(`Invalid account index for parameter ${param.name}: expected number, got ${typeof value} (${value}). Account mapping should have been done by FiveSDK.`);
      }
      if (accountIdx < 0) {
        throw new Error(`Invalid account index for parameter ${param.name}: negative index ${accountIdx}`);
      }
      if ((options as any)?.debug) {
        console.log(`[VLE] Encoding account parameter ${param.name} as index ${accountIdx}`);
      }
      return Buffer.concat([Buffer.from([typeId]), encodeVleU32(accountIdx)]);
    }

    // For numbers/bigints (u8...u64, i8...i64)
    let valBytes: Buffer;
    if (typeof value === 'bigint') {
      valBytes = encodeVleU64(value);
    } else {
      const numberValue = Number(value);
      if (!Number.isFinite(numberValue)) {
        throw new Error(`Invalid numeric value for parameter: ${param.name}`);
      }
      if (numberValue < 0) {
        // For simplicity in this fix, assume non-negative for VLE (Five VM uses u64 mostly)
        // or implement signed VLE if needed. Protocol uses unsigned VLE for params mostly.
        throw new Error(`Negative values not supported in TS VLE encoder yet: ${param.name}`);
      }
      valBytes = encodeVleU64(numberValue);
    }
    return Buffer.concat([Buffer.from([typeId]), valBytes]);
  }

  private static isAccountParam(param: ParameterDefinition): boolean {
    if ((param as any).isAccount || (param as any).is_account) {
      return true;
    }
    const type = this.normalizeType(param);
    return type === 'account';
  }

  private static isBytesParam(param: ParameterDefinition): boolean {
    const type = this.normalizeType(param);
    return type === 'string' || type === 'bytes';
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
