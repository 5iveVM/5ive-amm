/**
 * State Serializer for Five VM Account Data
 *
 * Serializes state account data based on Five account field definitions.
 * Handles conversion of JavaScript types to Five VM bytecode format.
 */

import { Base58Utils } from '../crypto/index.js';

export interface StateFieldDefinition {
  name: string;
  type: string;
}

export interface StateDefinition {
  name: string;
  fields: StateFieldDefinition[];
}

/**
 * Serializes state account data to Five VM bytecode format
 */
export class StateSerializer {
  /**
   * Serialize complete state object based on definition
   */
  static serialize(
    stateDefinition: StateDefinition,
    data: Record<string, any>,
    options: { debug?: boolean } = {}
  ): Uint8Array {
    if (options.debug) {
      console.log(
        `[StateSerializer] Serializing ${stateDefinition.name} with ${stateDefinition.fields.length} fields`
      );
    }

    // Collect all field buffers
    let totalSize = 0;
    const fieldBuffers: Uint8Array[] = [];

    for (const field of stateDefinition.fields) {
      const value = data[field.name];

      if (value === undefined) {
        if (options.debug) {
          console.warn(
            `[StateSerializer] Field "${field.name}" not provided, using default value for type ${field.type}`
          );
        }
        // Use default value based on type
        const defaultValue = this.getDefaultValue(field.type);
        const buffer = this.serializeField(field.type, defaultValue, options);
        fieldBuffers.push(buffer);
        totalSize += buffer.length;
      } else {
        const buffer = this.serializeField(field.type, value, options);
        fieldBuffers.push(buffer);
        totalSize += buffer.length;
      }
    }

    // Concatenate all field buffers in order
    const result = new Uint8Array(totalSize);
    let offset = 0;

    for (const buffer of fieldBuffers) {
      result.set(buffer, offset);
      offset += buffer.length;
    }

    if (options.debug) {
      console.log(
        `[StateSerializer] Serialized to ${totalSize} bytes, hex: ${this.toHexString(result)}`
      );
    }

    return result;
  }

  /**
   * Serialize a single field value based on its type
   */
  static serializeField(
    type: string,
    value: any,
    options: { debug?: boolean } = {}
  ): Uint8Array {
    // Normalize type name (remove whitespace, convert to lowercase)
    const normalizedType = type.toLowerCase().trim();

    if (options.debug) {
      console.log(`[StateSerializer] Serializing field type="${normalizedType}" value="${value}"`);
    }

    // Handle integer types
    if (normalizedType === 'u8' || normalizedType === 'u16' ||
        normalizedType === 'u32' || normalizedType === 'u64' ||
        normalizedType === 'i8' || normalizedType === 'i16' ||
        normalizedType === 'i32' || normalizedType === 'i64') {
      return this.serializeInteger(normalizedType, value);
    }

    // Handle boolean type
    if (normalizedType === 'bool' || normalizedType === 'boolean') {
      return new Uint8Array([value ? 1 : 0]);
    }

    // Handle public key type
    if (normalizedType === 'pubkey' || normalizedType === 'publickey' ||
        normalizedType === 'publickey' || normalizedType === 'account') {
      return this.serializePubkey(value);
    }

    // Handle string type (UTF-8 encoded with length prefix)
    if (normalizedType === 'string') {
      return this.serializeString(value);
    }

    // Handle array types
    if (normalizedType.endsWith('[]')) {
      const elementType = normalizedType.slice(0, -2);
      return this.serializeArray(elementType, value, options);
    }

    throw new Error(`Unsupported type: ${type}`);
  }

  /**
   * Serialize an integer value (u8, u16, u32, u64, i8, i16, i32, i64)
   */
  private static serializeInteger(type: string, value: any): Uint8Array {
    let numValue: number | bigint;

    if (typeof value === 'string') {
      // Try parsing as BigInt first for 64-bit values
      try {
        numValue = BigInt(value);
      } catch {
        numValue = parseInt(value, 10);
      }
    } else {
      numValue = value;
    }

    // Determine size based on type
    let size: number;
    let isSigned: boolean;

    switch (type) {
      case 'u8':
        size = 1;
        isSigned = false;
        break;
      case 'u16':
        size = 2;
        isSigned = false;
        break;
      case 'u32':
        size = 4;
        isSigned = false;
        break;
      case 'u64':
        size = 8;
        isSigned = false;
        break;
      case 'i8':
        size = 1;
        isSigned = true;
        break;
      case 'i16':
        size = 2;
        isSigned = true;
        break;
      case 'i32':
        size = 4;
        isSigned = true;
        break;
      case 'i64':
        size = 8;
        isSigned = true;
        break;
      default:
        throw new Error(`Unsupported integer type: ${type}`);
    }

    const buffer = new Uint8Array(size);
    const view = new DataView(buffer.buffer);

    // Write value in little-endian format
    if (size === 8 && typeof numValue === 'bigint') {
      if (isSigned) {
        view.setBigInt64(0, numValue, true);
      } else {
        view.setBigUint64(0, numValue, true);
      }
    } else {
      const numericValue = typeof numValue === 'bigint' ? Number(numValue) : numValue;

      switch (type) {
        case 'u8':
          view.setUint8(0, numericValue);
          break;
        case 'u16':
          view.setUint16(0, numericValue, true);
          break;
        case 'u32':
          view.setUint32(0, numericValue, true);
          break;
        case 'u64':
          view.setBigUint64(0, BigInt(numericValue), true);
          break;
        case 'i8':
          view.setInt8(0, numericValue);
          break;
        case 'i16':
          view.setInt16(0, numericValue, true);
          break;
        case 'i32':
          view.setInt32(0, numericValue, true);
          break;
        case 'i64':
          view.setBigInt64(0, BigInt(numericValue), true);
          break;
      }
    }

    return buffer;
  }

  /**
   * Serialize a public key (base58 to 32-byte array)
   */
  private static serializePubkey(value: string | Uint8Array): Uint8Array {
    if (value instanceof Uint8Array) {
      if (value.length !== 32) {
        throw new Error(
          `Invalid pubkey: expected 32 bytes, got ${value.length}`
        );
      }
      return value;
    }

    try {
      // Assume base58-encoded public key string
      const decoded = Base58Utils.decode(value);

      if (decoded.length !== 32) {
        throw new Error(
          `Invalid pubkey: expected 32 bytes after decoding, got ${decoded.length}`
        );
      }

      return decoded;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to decode pubkey: ${errorMessage}`);
    }
  }

  /**
   * Serialize a UTF-8 string with length prefix
   * Format: u32 length (little-endian) + UTF-8 bytes
   */
  private static serializeString(value: string): Uint8Array {
    const encoder = new TextEncoder();
    const stringBytes = encoder.encode(value);
    const length = stringBytes.length;

    // Create buffer: 4 bytes for length + string bytes
    const buffer = new Uint8Array(4 + length);
    const view = new DataView(buffer.buffer);

    // Write length as u32 in little-endian
    view.setUint32(0, length, true);

    // Write string bytes
    buffer.set(stringBytes, 4);

    return buffer;
  }

  /**
   * Serialize an array of values
   */
  private static serializeArray(
    elementType: string,
    values: any[],
    options: { debug?: boolean } = {}
  ): Uint8Array {
    const elementBuffers: Uint8Array[] = [];
    let totalSize = 4; // 4 bytes for array length

    // First, serialize all elements
    for (const value of values) {
      const buffer = this.serializeField(elementType, value, options);
      elementBuffers.push(buffer);
      totalSize += buffer.length;
    }

    // Create result buffer with length prefix
    const result = new Uint8Array(totalSize);
    const view = new DataView(result.buffer);

    // Write array length as u32 in little-endian
    view.setUint32(0, values.length, true);

    // Write all elements
    let offset = 4;
    for (const buffer of elementBuffers) {
      result.set(buffer, offset);
      offset += buffer.length;
    }

    return result;
  }

  /**
   * Get default value for a type
   */
  private static getDefaultValue(type: string): any {
    const normalizedType = type.toLowerCase().trim();

    if (normalizedType === 'bool' || normalizedType === 'boolean') {
      return false;
    }

    if (normalizedType === 'u8' || normalizedType === 'u16' ||
        normalizedType === 'u32' || normalizedType === 'u64' ||
        normalizedType === 'i8' || normalizedType === 'i16' ||
        normalizedType === 'i32' || normalizedType === 'i64') {
      return 0;
    }

    if (normalizedType === 'pubkey' || normalizedType === 'publickey' ||
        normalizedType === 'account') {
      return '11111111111111111111111111111111';  // Default system program
    }

    if (normalizedType === 'string') {
      return '';
    }

    if (normalizedType.endsWith('[]')) {
      return [];
    }

    return null;
  }

  /**
   * Convert byte array to hex string for debugging
   */
  private static toHexString(bytes: Uint8Array): string {
    return Array.from(bytes)
      .map(byte => byte.toString(16).padStart(2, '0'))
      .join('');
  }

  /**
   * Calculate total size of a state based on definition
   */
  static calculateSize(stateDefinition: StateDefinition): number {
    let totalSize = 0;

    for (const field of stateDefinition.fields) {
      totalSize += this.getFieldSize(field.type);
    }

    return totalSize;
  }

  /**
   * Get size of a single field type in bytes
   */
  private static getFieldSize(type: string): number {
    const normalizedType = type.toLowerCase().trim();

    switch (normalizedType) {
      case 'u8':
      case 'i8':
      case 'bool':
        return 1;
      case 'u16':
      case 'i16':
        return 2;
      case 'u32':
      case 'i32':
        return 4;
      case 'u64':
      case 'i64':
        return 8;
      case 'pubkey':
      case 'publickey':
      case 'account':
        return 32;
      case 'string':
        // Variable-length, return minimum (4 bytes for length)
        return 4;
      default:
        if (normalizedType.endsWith('[]')) {
          // Variable-length array
          return 4; // Minimum for length prefix
        }
        throw new Error(`Unsupported type: ${type}`);
    }
  }
}

export default StateSerializer;
