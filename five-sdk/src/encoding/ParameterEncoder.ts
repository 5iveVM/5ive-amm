// Parameter encoding for Five VM.

import {
  EncodedParameter,
  ParameterEncodingOptions,
  ParameterEncodingError,
  FiveType,
  FiveFunction,
  FiveParameter
} from '../types.js';

// Type ID mapping (matches Five VM protocol).
const TYPE_IDS: Record<FiveType, number> = {
  'u8': 1,
  'u16': 2,
  'u32': 3,
  'u64': 4,
  'i8': 5,
  'i16': 6,
  'i32': 7,
  'i64': 8,
  'bool': 9,
  'string': 11,
  'pubkey': 10,
  'bytes': 12,
  'array': 13
};

export class ParameterEncoder {
  private debug: boolean;

  constructor(debug: boolean = false) {
    this.debug = debug;

    if (this.debug) {
      console.log('[ParameterEncoder] Initialized');
    }
  }

  // ==================== Pure Parameter Encoding ====================

  async encodeParameterData(
    parameters: any[] = [],
    functionSignature?: FiveFunction
  ): Promise<Buffer> {
    if (this.debug) {
      console.log(`[ParameterEncoder] Encoding parameter data: params=${parameters.length}`);
    }

    const encodedData = await this.encodeParametersInternal(parameters, functionSignature);

    if (this.debug) {
      console.log(`[ParameterEncoder] Encoded parameters: ${encodedData.length} bytes, hex: ${encodedData.toString('hex')}`);
    }

    return encodedData;
  }

  encodeParametersWithABI(
    parameters: any[],
    functionSignature: FiveFunction,
    options: ParameterEncodingOptions = {}
  ): EncodedParameter[] {
    if (this.debug) {
      console.log(`[ParameterEncoder] Encoding ${parameters.length} parameters with ABI guidance`);
    }

    const encoded: EncodedParameter[] = [];

    for (let i = 0; i < parameters.length; i++) {
      const value = parameters[i];
      const paramDef = functionSignature.parameters[i];

      if (!paramDef && options.strict) {
        throw new ParameterEncodingError(
          `Parameter ${i} provided but function only expects ${functionSignature.parameters.length} parameters`,
          { functionName: functionSignature.name, parameterIndex: i }
        );
      }

      // Use ABI type if available, otherwise infer
      const targetType = paramDef?.type || this.inferType(value);
      const encodedParam = this.encodeParameter(value, targetType, i);

      encoded.push(encodedParam);
    }

    if (this.debug) {
      console.log(`[ParameterEncoder] Encoded ${encoded.length} parameters successfully`);
    }

    return encoded;
  }

  // ==================== Type Coercion ====================

  /// Coerce value to a Five VM type.
  coerceValue(value: any, targetType: FiveType): any {
    if (this.debug) {
      console.log(`[ParameterEncoder] Coercing value ${JSON.stringify(value)} to ${targetType}`);
    }

    try {
      switch (targetType) {
        case 'u8':
          return this.coerceToU8(value);
        case 'u16':
          return this.coerceToU16(value);
        case 'u32':
          return this.coerceToU32(value);
        case 'u64':
          return this.coerceToU64(value);
        case 'i8':
          return this.coerceToI8(value);
        case 'i16':
          return this.coerceToI16(value);
        case 'i32':
          return this.coerceToI32(value);
        case 'i64':
          return this.coerceToI64(value);
        case 'bool':
          return this.coerceToBool(value);
        case 'string':
          return this.coerceToString(value);
        case 'pubkey':
          return this.coerceToPubkey(value);
        case 'bytes':
          return this.coerceToBytes(value);
        case 'array':
          return this.coerceToArray(value);
        default:
          throw new Error(`Unsupported type: ${targetType}`);
      }
    } catch (error) {
      throw new ParameterEncodingError(
        `Failed to coerce value ${JSON.stringify(value)} to ${targetType}: ${error instanceof Error ? error.message : 'Unknown error'}`,
        { value, targetType }
      );
    }
  }

  // ==================== Private Methods ====================

  /// Use BytecodeEncoder for parameter data only.
  private async encodeParametersInternal(parameters: any[], functionSignature?: FiveFunction): Promise<Buffer> {
    try {
      // Import BytecodeEncoder
      const { BytecodeEncoder } = await import('../lib/bytecode-encoder.js');

      // Convert parameters to format expected by encoder
      const params = parameters.map((value, index) => {
        const paramDef = functionSignature?.parameters[index];
        return {
          name: paramDef?.name || `param_${index}`,
          type: paramDef?.type || this.inferTypeString(value)
        };
      });

      const values: Record<string, any> = {};
      params.forEach((param, index) => {
        values[param.name] = parameters[index];
      });

      // Encode parameters only; function index is handled by the SDK when building instruction data
      const encoded = await BytecodeEncoder.encodeExecute(0, params, values);
      return Buffer.from(encoded);

    } catch (error) {
      throw new Error(`Parameter encoding failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  /// Encode individual parameter.
  private encodeParameter(value: any, type: FiveType, index: number): EncodedParameter {
    const coercedValue = this.coerceValue(value, type);
    const typeId = TYPE_IDS[type];

    return {
      type: typeId,
      value: coercedValue
    };
  }

  /**
   * Infer Five VM type from JavaScript value
   */
  private inferType(value: any): FiveType {
    if (typeof value === 'boolean') {
      return 'bool';
    }
    if (typeof value === 'string') {
      return 'string';
    }
    if (typeof value === 'number') {
      // Default to u64 for positive integers, i64 for negative
      return Number.isInteger(value) && value >= 0 ? 'u64' : 'i64';
    }
    if (typeof value === 'bigint') {
      return value >= 0 ? 'u64' : 'i64';
    }
    if (Array.isArray(value)) {
      return 'array';
    }

    return 'string'; // Default fallback
  }

  /**
   * Infer type as string
   */
  private inferTypeString(value: any): string {
    const type = this.inferType(value);
    return type;
  }

  // ==================== Type Coercion Methods ====================

  private coerceToU8(value: any): number {
    const num = Number(value);
    if (!Number.isInteger(num) || num < 0 || num > 255) {
      throw new Error(`Value ${value} cannot be coerced to u8 (0-255)`);
    }
    return num;
  }

  private coerceToU16(value: any): number {
    const num = Number(value);
    if (!Number.isInteger(num) || num < 0 || num > 65535) {
      throw new Error(`Value ${value} cannot be coerced to u16 (0-65535)`);
    }
    return num;
  }

  private coerceToU32(value: any): number {
    const num = Number(value);
    if (!Number.isInteger(num) || num < 0 || num > 4294967295) {
      throw new Error(`Value ${value} cannot be coerced to u32 (0-4294967295)`);
    }
    return num;
  }

  private coerceToU64(value: any): bigint | number {
    if (typeof value === 'bigint') {
      if (value < BigInt(0) || value > BigInt('18446744073709551615')) {
        throw new Error(`Value ${value} cannot be coerced to u64 (0-2^64-1)`);
      }
      return value;
    }

    const num = Number(value);
    if (!Number.isInteger(num) || num < 0) {
      throw new Error(`Value ${value} cannot be coerced to u64`);
    }
    return num;
  }

  private coerceToI8(value: any): number {
    const num = Number(value);
    if (!Number.isInteger(num) || num < -128 || num > 127) {
      throw new Error(`Value ${value} cannot be coerced to i8 (-128 to 127)`);
    }
    return num;
  }

  private coerceToI16(value: any): number {
    const num = Number(value);
    if (!Number.isInteger(num) || num < -32768 || num > 32767) {
      throw new Error(`Value ${value} cannot be coerced to i16 (-32768 to 32767)`);
    }
    return num;
  }

  private coerceToI32(value: any): number {
    const num = Number(value);
    if (!Number.isInteger(num) || num < -2147483648 || num > 2147483647) {
      throw new Error(`Value ${value} cannot be coerced to i32 (-2^31 to 2^31-1)`);
    }
    return num;
  }

  private coerceToI64(value: any): bigint | number {
    if (typeof value === 'bigint') {
      if (value < BigInt('-9223372036854775808') || value > BigInt('9223372036854775807')) {
        throw new Error(`Value ${value} cannot be coerced to i64 (-2^63 to 2^63-1)`);
      }
      return value;
    }

    const num = Number(value);
    if (!Number.isInteger(num)) {
      throw new Error(`Value ${value} cannot be coerced to i64`);
    }
    return num;
  }

  private coerceToBool(value: any): boolean {
    if (typeof value === 'boolean') {
      return value;
    }
    if (typeof value === 'string') {
      const lower = value.toLowerCase();
      if (lower === 'true' || lower === '1') return true;
      if (lower === 'false' || lower === '0') return false;
      throw new Error(`String "${value}" cannot be coerced to boolean`);
    }
    if (typeof value === 'number') {
      return value !== 0;
    }
    throw new Error(`Value ${value} cannot be coerced to boolean`);
  }

  private coerceToString(value: any): string {
    return String(value);
  }

  private coerceToPubkey(value: any): string {
    if (typeof value === 'string' && value.length === 44) {
      return value; // Assume base58 encoded pubkey
    }
    throw new Error(`Value ${value} cannot be coerced to pubkey`);
  }

  private coerceToBytes(value: any): Uint8Array {
    if (value instanceof Uint8Array) {
      return value;
    }
    if (Array.isArray(value)) {
      return new Uint8Array(value);
    }
    if (typeof value === 'string') {
      // Assume hex string
      return new Uint8Array(Buffer.from(value, 'hex'));
    }
    throw new Error(`Value ${value} cannot be coerced to bytes`);
  }

  private coerceToArray(value: any): any[] {
    if (Array.isArray(value)) {
      return value;
    }
    throw new Error(`Value ${value} cannot be coerced to array`);
  }
}
