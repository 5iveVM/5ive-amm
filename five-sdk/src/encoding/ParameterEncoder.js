// Parameter encoding for Five VM.
import { EncodedParameter, ParameterEncodingOptions, ParameterEncodingError, FiveType, FiveFunction, FiveParameter } from '../types.js';
const TYPE_IDS = {
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
    debug;
    constructor(debug = false) {
        this.debug = debug;
        if (this.debug) {
            console.log('[ParameterEncoder] Initialized');
        }
    }
    async encodeParameterData(parameters = [], functionSignature) {
        if (this.debug) {
            console.log(`[ParameterEncoder] Encoding parameter data: params=${parameters.length}`);
        }
        const encodedData = await this.encodeParametersInternal(parameters, functionSignature);
        if (this.debug) {
            console.log(`[ParameterEncoder] Encoded parameters: ${encodedData.length} bytes, hex: ${encodedData.toString('hex')}`);
        }
        return encodedData;
    }
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
            const targetType = (paramDef === null || paramDef === void 0 ? void 0 : paramDef.type) || this.inferType(value);
            const encodedParam = this.encodeParameter(value, targetType, i);
            encoded.push(encodedParam);
        }
        if (this.debug) {
            console.log(`[ParameterEncoder] Encoded ${encoded.length} parameters successfully`);
        }
        return encoded;
    }
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
    async encodeParametersInternal(parameters, functionSignature) {
        try {
            const { BytecodeEncoder } = await import('../lib/bytecode-encoder.js');
            const params = parameters.map((value, index) => {
                var _a;
                const paramDef = functionSignature === null || functionSignature === void 0 ? void 0 : functionSignature.parameters[index];
                return {
                    name: (paramDef === null || paramDef === void 0 ? void 0 : paramDef.name) || `param_${index}`,
                    type: ((_a = paramDef === null || paramDef === void 0 ? void 0 : paramDef.type) !== null && _a !== void 0 ? _a : this.inferTypeString(value))
                };
            });
            const values = {};
            params.forEach((param, index) => {
                values[param.name] = parameters[index];
            });
            const encoded = await BytecodeEncoder.encodeExecute(0, params, values);
            return Buffer.from(encoded);
        }
        catch (error) {
            throw new Error(`Parameter encoding failed: ${error instanceof Error ? error.message : String(error)}`);
        }
    }
    encodeParameter(value, type, index) {
        const coercedValue = this.coerceValue(value, type);
        const typeId = TYPE_IDS[type];
        return {
            type: typeId,
            value: coercedValue
        };
    }
    inferType(value) {
        if (typeof value === 'boolean') {
            return 'bool';
        }
        if (typeof value === 'string') {
            return 'string';
        }
        if (typeof value === 'number') {
            return Number.isInteger(value) && value >= 0 ? 'u64' : 'i64';
        }
        if (typeof value === 'bigint') {
            return value >= 0 ? 'u64' : 'i64';
        }
        if (value instanceof Uint8Array || value instanceof Buffer) {
            return 'bytes';
        }
        if (Array.isArray(value)) {
            return 'array';
        }
        return 'string';
    }
    inferTypeString(value) {
        const type = this.inferType(value);
        return type;
    }
    coerceToU8(value) {
        const num = Number(value);
        if (num < 0 || num > 255 || !Number.isInteger(num)) {
            throw new Error(`Value ${value} is not a valid u8`);
        }
        return num;
    }
    coerceToU16(value) {
        const num = Number(value);
        if (num < 0 || num > 65535 || !Number.isInteger(num)) {
            throw new Error(`Value ${value} is not a valid u16`);
        }
        return num;
    }
    coerceToU32(value) {
        const num = Number(value);
        if (num < 0 || num > 4294967295 || !Number.isInteger(num)) {
            throw new Error(`Value ${value} is not a valid u32`);
        }
        return num;
    }
    coerceToU64(value) {
        try {
            const big = BigInt(value);
            if (big < 0n) {
                throw new Error(`Value ${value} is not a valid u64`);
            }
            return big;
        }
        catch {
            throw new Error(`Value ${value} is not a valid u64`);
        }
    }
    coerceToI8(value) {
        const num = Number(value);
        if (num < -128 || num > 127 || !Number.isInteger(num)) {
            throw new Error(`Value ${value} is not a valid i8`);
        }
        return num;
    }
    coerceToI16(value) {
        const num = Number(value);
        if (num < -32768 || num > 32767 || !Number.isInteger(num)) {
            throw new Error(`Value ${value} is not a valid i16`);
        }
        return num;
    }
    coerceToI32(value) {
        const num = Number(value);
        if (num < -2147483648 || num > 2147483647 || !Number.isInteger(num)) {
            throw new Error(`Value ${value} is not a valid i32`);
        }
        return num;
    }
    coerceToI64(value) {
        try {
            const big = BigInt(value);
            return big;
        }
        catch {
            throw new Error(`Value ${value} is not a valid i64`);
        }
    }
    coerceToBool(value) {
        if (typeof value === 'boolean') {
            return value;
        }
        if (value === 0 || value === 1) {
            return Boolean(value);
        }
        if (value === 'true' || value === 'false') {
            return value === 'true';
        }
        throw new Error(`Value ${value} is not a valid bool`);
    }
    coerceToString(value) {
        if (typeof value === 'string') {
            return value;
        }
        return String(value);
    }
    coerceToPubkey(value) {
        if (typeof value === 'string') {
            return value;
        }
        if (value instanceof Uint8Array || value instanceof Buffer) {
            return value;
        }
        throw new Error(`Value ${value} is not a valid pubkey`);
    }
    coerceToBytes(value) {
        if (value instanceof Uint8Array || value instanceof Buffer) {
            return value;
        }
        if (typeof value === 'string') {
            return Buffer.from(value, 'utf8');
        }
        throw new Error(`Value ${value} is not valid bytes`);
    }
    coerceToArray(value) {
        if (!Array.isArray(value)) {
            throw new Error(`Value ${value} is not a valid array`);
        }
        return value;
    }
}
