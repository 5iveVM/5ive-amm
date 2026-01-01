/**
 * Five Parameter Encoder
 *
 * Handles parameter encoding for Five VM:
 * - VLE (Variable Length Encoding) for efficient bytecode
 * - Type coercion based on ABI information
 * - Parameter validation and error handling
 * - Integration with existing VLE encoder
 */
import { EncodedParameter, ParameterEncodingOptions, ParameterEncodingError, FiveType, FiveFunction, FiveParameter } from '../types.js';
/**
 * VLE Type ID mapping (matches Five VM protocol)
 */
const VLE_TYPE_IDS = {
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
/**
 * Parameter encoder for Five VM execution
 */
export class ParameterEncoder {
    debug;
    constructor(debug = false) {
        this.debug = debug;
        if (this.debug) {
            console.log('[ParameterEncoder] Initialized');
        }
    }
    // ==================== Pure Parameter Encoding ====================
    /**
     * Encode parameter data only (no instruction discriminators)
     */
    async encodeParameterData(parameters = [], functionSignature) {
        if (this.debug) {
            console.log(`[ParameterEncoder] Encoding parameter data: params=${parameters.length}`);
        }
        try {
            // Use existing VLE encoder if available
            const vleData = await this.encodeParametersVLE(parameters, functionSignature);
            if (this.debug) {
                console.log(`[ParameterEncoder] Encoded parameters: ${vleData.length} bytes, hex: ${vleData.toString('hex')}`);
            }
            return vleData;
        }
        catch (error) {
            // Fallback to manual encoding if VLE encoder fails
            if (this.debug) {
                console.log(`[ParameterEncoder] VLE encoding failed, using manual encoding: ${error}`);
            }
            return this.encodeParametersManual(parameters);
        }
    }
    /**
     * Encode parameters with ABI-driven type coercion
     */
    encodeParametersWithABI(parameters, functionSignature, options = {}) {
        if (this.debug) {
            console.log(`[ParameterEncoder] Encoding ${parameters.length} parameters with ABI guidance`);
        }
        const encoded = [];
        for (let i = 0; i < parameters.length; i++) {
            const value = parameters[i];
            const paramDef = functionSignature.parameters[i];
            if (!paramDef && options.strict) {
                throw new ParameterEncodingError(`Parameter ${i} provided but function only expects ${functionSignature.parameters.length} parameters`, { functionName: functionSignature.name, parameterIndex: i });
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
    /**
     * Coerce value to specific Five VM type
     */
    coerceValue(value, targetType) {
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
        }
        catch (error) {
            throw new ParameterEncodingError(`Failed to coerce value ${JSON.stringify(value)} to ${targetType}: ${error instanceof Error ? error.message : 'Unknown error'}`, { value, targetType });
        }
    }
    // ==================== Private Methods ====================
    /**
     * Use existing VLE encoder for parameter data only
     */
    async encodeParametersVLE(parameters, functionSignature) {
        try {
            // Import existing VLE encoder
            const { VLEEncoder } = await import('../../lib/vle-encoder.js');
            // Convert parameters to VLE format
            const vleParams = parameters.map((value, index) => {
                const paramDef = functionSignature?.parameters[index];
                return {
                    name: paramDef?.name || `param_${index}`,
                    type: paramDef?.type || this.inferTypeString(value)
                };
            });
            const values = {};
            vleParams.forEach((param, index) => {
                values[param.name] = parameters[index];
            });
            // Encode parameters only; function index is handled by the SDK when building instruction data
            const encoded = await VLEEncoder.encodeExecuteVLE(0, vleParams, values);
            return Buffer.from(encoded);
        }
        catch (error) {
            throw new Error(`VLE parameter encoding failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
        }
    }
    /**
     * Manual parameter encoding fallback (parameters only)
     */
    encodeParametersManual(parameters) {
        const parts = [];
        // Encode parameter count as VLE
        parts.push(this.encodeVLEU32(parameters.length));
        // Encode each parameter
        for (const param of parameters) {
            const type = this.inferType(param);
            const typeId = VLE_TYPE_IDS[type];
            const encodedParam = this.encodeParameter(param, type, 0);
            // Add type and value
            parts.push(Buffer.from([typeId]));
            parts.push(this.encodeValue(encodedParam.value, type));
        }
        return Buffer.concat(parts);
    }
    /**
     * Encode individual parameter
     */
    encodeParameter(value, type, index) {
        const coercedValue = this.coerceValue(value, type);
        const typeId = VLE_TYPE_IDS[type];
        return {
            type: typeId,
            value: coercedValue
        };
    }
    /**
     * Infer Five VM type from JavaScript value
     */
    inferType(value) {
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
     * Infer type as string for VLE encoder compatibility
     */
    inferTypeString(value) {
        const type = this.inferType(value);
        return type;
    }
    // ==================== Type Coercion Methods ====================
    coerceToU8(value) {
        const num = Number(value);
        if (!Number.isInteger(num) || num < 0 || num > 255) {
            throw new Error(`Value ${value} cannot be coerced to u8 (0-255)`);
        }
        return num;
    }
    coerceToU16(value) {
        const num = Number(value);
        if (!Number.isInteger(num) || num < 0 || num > 65535) {
            throw new Error(`Value ${value} cannot be coerced to u16 (0-65535)`);
        }
        return num;
    }
    coerceToU32(value) {
        const num = Number(value);
        if (!Number.isInteger(num) || num < 0 || num > 4294967295) {
            throw new Error(`Value ${value} cannot be coerced to u32 (0-4294967295)`);
        }
        return num;
    }
    coerceToU64(value) {
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
    coerceToI8(value) {
        const num = Number(value);
        if (!Number.isInteger(num) || num < -128 || num > 127) {
            throw new Error(`Value ${value} cannot be coerced to i8 (-128 to 127)`);
        }
        return num;
    }
    coerceToI16(value) {
        const num = Number(value);
        if (!Number.isInteger(num) || num < -32768 || num > 32767) {
            throw new Error(`Value ${value} cannot be coerced to i16 (-32768 to 32767)`);
        }
        return num;
    }
    coerceToI32(value) {
        const num = Number(value);
        if (!Number.isInteger(num) || num < -2147483648 || num > 2147483647) {
            throw new Error(`Value ${value} cannot be coerced to i32 (-2^31 to 2^31-1)`);
        }
        return num;
    }
    coerceToI64(value) {
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
    coerceToBool(value) {
        if (typeof value === 'boolean') {
            return value;
        }
        if (typeof value === 'string') {
            const lower = value.toLowerCase();
            if (lower === 'true' || lower === '1')
                return true;
            if (lower === 'false' || lower === '0')
                return false;
            throw new Error(`String "${value}" cannot be coerced to boolean`);
        }
        if (typeof value === 'number') {
            return value !== 0;
        }
        throw new Error(`Value ${value} cannot be coerced to boolean`);
    }
    coerceToString(value) {
        return String(value);
    }
    coerceToPubkey(value) {
        if (typeof value === 'string' && value.length === 44) {
            return value; // Assume base58 encoded pubkey
        }
        throw new Error(`Value ${value} cannot be coerced to pubkey`);
    }
    coerceToBytes(value) {
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
    coerceToArray(value) {
        if (Array.isArray(value)) {
            return value;
        }
        throw new Error(`Value ${value} cannot be coerced to array`);
    }
    // ==================== VLE Encoding Utilities ====================
    /**
     * Encode u32 value using Variable Length Encoding
     */
    encodeVLEU32(value) {
        if (value < 128) {
            return Buffer.from([value]);
        }
        else if (value < 16384) {
            return Buffer.from([
                (value & 0x7F) | 0x80,
                (value >> 7) & 0x7F
            ]);
        }
        else {
            return Buffer.from([
                (value & 0x7F) | 0x80,
                ((value >> 7) & 0x7F) | 0x80,
                (value >> 14) & 0x7F
            ]);
        }
    }
    /**
     * Encode value based on type
     */
    encodeValue(value, type) {
        switch (type) {
            case 'u8':
            case 'i8':
                return Buffer.from([value]);
            case 'u16':
            case 'i16':
                const buf16 = Buffer.allocUnsafe(2);
                buf16.writeUInt16LE(value, 0);
                return buf16;
            case 'u32':
            case 'i32':
                const buf32 = Buffer.allocUnsafe(4);
                buf32.writeUInt32LE(value, 0);
                return buf32;
            case 'u64':
            case 'i64':
                const buffer = Buffer.allocUnsafe(8);
                if (typeof value === 'bigint') {
                    buffer.writeBigUInt64LE(value, 0);
                }
                else {
                    buffer.writeUInt32LE(value, 0);
                    buffer.writeUInt32LE(0, 4);
                }
                return buffer;
            case 'bool':
                return Buffer.from([value ? 1 : 0]);
            case 'string':
                const str = Buffer.from(value, 'utf8');
                const len = this.encodeVLEU32(str.length);
                return Buffer.concat([len, str]);
            default:
                throw new Error(`Cannot encode value for type: ${type}`);
        }
    }
}
//# sourceMappingURL=ParameterEncoder.js.map