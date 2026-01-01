/**
 * State Serializer for Five VM Account Data
 *
 * Serializes state account data based on Five account field definitions.
 * Handles conversion of JavaScript types to Five VM bytecode format.
 */
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
export declare class StateSerializer {
    /**
     * Serialize complete state object based on definition
     */
    static serialize(stateDefinition: StateDefinition, data: Record<string, any>, options?: {
        debug?: boolean;
    }): Uint8Array;
    /**
     * Serialize a single field value based on its type
     */
    static serializeField(type: string, value: any, options?: {
        debug?: boolean;
    }): Uint8Array;
    /**
     * Serialize an integer value (u8, u16, u32, u64, i8, i16, i32, i64)
     */
    private static serializeInteger;
    /**
     * Serialize a public key (base58 to 32-byte array)
     */
    private static serializePubkey;
    /**
     * Serialize a UTF-8 string with length prefix
     * Format: u32 length (little-endian) + UTF-8 bytes
     */
    private static serializeString;
    /**
     * Serialize an array of values
     */
    private static serializeArray;
    /**
     * Get default value for a type
     */
    private static getDefaultValue;
    /**
     * Convert byte array to hex string for debugging
     */
    private static toHexString;
    /**
     * Calculate total size of a state based on definition
     */
    static calculateSize(stateDefinition: StateDefinition): number;
    /**
     * Get size of a single field type in bytes
     */
    private static getFieldSize;
}
export default StateSerializer;
//# sourceMappingURL=StateSerializer.d.ts.map