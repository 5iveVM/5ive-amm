/**
 * Five Parameter Encoder
 *
 * Handles parameter encoding for Five VM:
 * - VLE (Variable Length Encoding) for efficient bytecode
 * - Type coercion based on ABI information
 * - Parameter validation and error handling
 * - Integration with existing VLE encoder
 */
import { EncodedParameter, ParameterEncodingOptions, FiveType, FiveFunction } from '../types.js';
/**
 * Parameter encoder for Five VM execution
 */
export declare class ParameterEncoder {
    private debug;
    constructor(debug?: boolean);
    /**
     * Encode parameter data only (no instruction discriminators)
     */
    encodeParameterData(parameters?: any[], functionSignature?: FiveFunction): Promise<Buffer>;
    /**
     * Encode parameters with ABI-driven type coercion
     */
    encodeParametersWithABI(parameters: any[], functionSignature: FiveFunction, options?: ParameterEncodingOptions): EncodedParameter[];
    /**
     * Coerce value to specific Five VM type
     */
    coerceValue(value: any, targetType: FiveType): any;
    /**
     * Use existing VLE encoder for parameter data only
     */
    private encodeParametersVLE;
    /**
     * Manual parameter encoding fallback (parameters only)
     */
    private encodeParametersManual;
    /**
     * Encode individual parameter
     */
    private encodeParameter;
    /**
     * Infer Five VM type from JavaScript value
     */
    private inferType;
    /**
     * Infer type as string for VLE encoder compatibility
     */
    private inferTypeString;
    private coerceToU8;
    private coerceToU16;
    private coerceToU32;
    private coerceToU64;
    private coerceToI8;
    private coerceToI16;
    private coerceToI32;
    private coerceToI64;
    private coerceToBool;
    private coerceToString;
    private coerceToPubkey;
    private coerceToBytes;
    private coerceToArray;
    /**
     * Encode u32 value using Variable Length Encoding
     */
    private encodeVLEU32;
    /**
     * Encode value based on type
     */
    private encodeValue;
}
//# sourceMappingURL=ParameterEncoder.d.ts.map