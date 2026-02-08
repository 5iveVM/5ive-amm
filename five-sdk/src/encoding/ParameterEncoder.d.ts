// Parameter encoding for Five VM.
import { EncodedParameter, ParameterEncodingOptions, FiveType, FiveFunction } from '../types.js';
export declare class ParameterEncoder {
    private debug;
    constructor(debug?: boolean);
    encodeParameterData(parameters?: any[], functionSignature?: FiveFunction): Promise<Buffer>;
    encodeParametersWithABI(parameters: any[], functionSignature: FiveFunction, options?: ParameterEncodingOptions): EncodedParameter[];
    coerceValue(value: any, targetType: FiveType): any;
    private encodeParametersInternal;
    private encodeParameter;
    private inferType;
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
}
//# sourceMappingURL=ParameterEncoder.d.ts.map
