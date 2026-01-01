export interface NormalizedABIFunction {
    name: string;
    index: number;
    parameters: Array<{
        name: string;
        type: string;
        optional?: boolean;
    }>;
    returnType?: string;
    accounts?: any[];
    visibility?: 'public' | 'private';
}
/**
 * Normalize ABI function definitions that may be emitted as either arrays (FIVEABI)
 * or maps (SimpleABI) into a consistent array format.
 */
export declare function normalizeAbiFunctions(abiFunctions: unknown): NormalizedABIFunction[];
//# sourceMappingURL=abi.d.ts.map