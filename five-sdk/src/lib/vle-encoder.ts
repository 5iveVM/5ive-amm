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
    values: ParameterValue = {}
  ): Promise<Buffer> {

    // Load WASM module using shared loader
    if (!wasmModule) {
      wasmModule = await getWasmModule();
    }

    const filteredParams = parameters.filter(param => !this.isAccountParam(param));
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

    const usesTypedParams = paramValues.some(({ param }) => this.isBytesParam(param));

    if (!usesTypedParams) {
      // Use PURE VLE compression - encode only values without type information
      const simpleValues = paramValues.map(({ value }) => value);
      const encoded = wasmModule.ParameterEncoder.encode_execute_vle(functionIndex, simpleValues);
      return Buffer.from(encoded);
    }

    const parts: Buffer[] = [];
    parts.push(encodeVleU32(TYPED_PARAM_SENTINEL));
    parts.push(encodeVleU32(TYPED_PARAM_SENTINEL));

    for (const { param, value } of paramValues) {
      parts.push(this.encodeTypedParam(param, value, encodeVleU32));
    }

    return Buffer.concat(parts);
  }

  private static encodeTypedParam(
    param: ParameterDefinition,
    value: any,
    encodeVleU32: (value: number) => Buffer,
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

    const numberValue = Number(value);
    if (!Number.isFinite(numberValue) || numberValue < 0 || numberValue > 0xFFFF_FFFF) {
      throw new Error(`Invalid numeric value for parameter: ${param.name}`);
    }
    const valBytes = encodeVleU32(Math.floor(numberValue));
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
    return (
      (param.type || param.param_type || '').toString().trim().toLowerCase()
    );
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
