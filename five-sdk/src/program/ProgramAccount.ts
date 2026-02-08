/**
 * ProgramAccount - Typed state fetching for Five VM
 *
 * Provides typed access to account state based on ABI definitions.
 *
 * Usage:
 * ```typescript
 * const myAccount = program.account('MyStruct');
 * const state = await myAccount.fetch(publicKey);
 * console.log(state.someField);
 * ```
 */

import { AccountFetcher, AccountData } from '../metadata/index.js';
import type { ScriptABI, TypeDefinition } from '../metadata/index.js';
import { BorshSchemaGenerator } from './BorshSchemaGenerator.js';
import * as borsh from 'borsh';

export class ProgramAccount {
    constructor(
        private structName: string,
        private abi: ScriptABI,
        private fetcher?: AccountFetcher
    ) { }

    /**
     * Fetch and decode account state
     *
     * @param address - Account address to fetch
     * @returns Decoded account state object
     */
    async fetch(address: string): Promise<any> {
        if (!this.fetcher) {
            throw new Error('Account fetcher not provided. Cannot fetch account data.');
        }

        const accountData = await this.fetcher.getAccountData(address);
        if (!accountData) {
            return null;
        }

        return this.decode(accountData.data);
    }

    /**
     * Decode raw account data based on ABI
     *
     * @param data - Raw byte array
     * @returns Decoded JavaScript object
     */
    decode(data: Uint8Array): any {
        // 1. Find struct definition in ABI
        const structDef = this.findStructDefinition(this.structName);
        if (!structDef) {
            // Fallback: if no types defined, return raw data
            return { data };
        }

        // 2. Decode based on fields
        // NOTE: This uses the robust BorshSchemaGenerator to map ABI types to runtime schema
        try {
            const generator = new BorshSchemaGenerator(this.abi);

            // We need to construct a class that matches the schema for borsh to deserialize into
            // The generator effectively returns a map where keys are these classes
            const schema = generator.generate(structDef);

            // Find the class constructor from the map keys
            const SchemaClass = Array.from(schema.keys())[0] as any;

            if (!SchemaClass) {
                throw new Error(`Failed to generate schema for ${this.structName}`);
            }

            // Fallback: Manual deserialization using BinaryReader
            // This bypasses potential issues with borsh.deserialize returning empty objects for dynamic classes
            try {
                const reader = new borsh.BinaryReader(Buffer.from(data));
                const result = new SchemaClass();
                const structSchema = schema.get(SchemaClass);

                if (structSchema && structSchema.kind === 'struct') {
                    for (const [fieldName, fieldType] of structSchema.fields) {
                        result[fieldName] = this.readField(reader, fieldType);
                    }
                    return result;
                }
            } catch (manualErr) {
            }

            // Fallback to library if manual failed (though manual is preferred now)
            return borsh.deserialize(schema as any, SchemaClass, Buffer.from(data));
        } catch (e) {
            console.error(`Borsh decoding failed for ${this.structName}:`, e);
            // Fallback to simple decode if borsh fails (e.g. mismatch)
            return this.simpleDecode(data, structDef);
        }
    }

    private readField(reader: any, fieldType: any): any {
        if (typeof fieldType === 'string') {
            switch (fieldType) {
                case 'u8': return reader.readU8();
                case 'u16': return reader.readU16();
                case 'u32': return reader.readU32();
                case 'u64': return reader.readU64();
                case 'u128': return reader.readU128();
                case 'i8': return reader.readU8();
                case 'i16': return reader.readU16();
                case 'i32': return reader.readU32();
                case 'i64': return reader.readU64();
                case 'i128': return reader.readU128();
                case 'bool': return reader.readU8() !== 0;
                case 'string': return reader.readString();
            }
        }

        if (Array.isArray(fieldType)) {
            // Fixed array [length] or [Type] (Vec)
            if (typeof fieldType[0] === 'number') {
                return reader.readFixedArray(fieldType[0]);
            }
        }
        return null;
    }

    private findStructDefinition(name: string): TypeDefinition | undefined {
        if (!this.abi.types) return undefined;
        return this.abi.types.find(t => t.name === name && t.structure === 'struct');
    }

    private simpleDecode(data: Uint8Array, structDef: TypeDefinition): any {
        const result: any = {};
        let offset = 0;

        // Basic discriminator check (8 bytes for Five accounts) could go here
        // offset += 8;

        if (!structDef.fields) return result;

        for (const field of structDef.fields) {
            // Very basic decoding logic for primitive types
            // A robust implementation would use a proper deserialization library
            if (offset >= data.length) break;

            switch (field.type) {
                case 'u8':
                case 'bool':
                    result[field.name] = data[offset];
                    offset += 1;
                    break;
                case 'u32':
                case 'i32':
                    result[field.name] = this.readU32(data, offset);
                    offset += 4;
                    break;
                case 'u64':
                case 'i64':
                    // Read as Number for simplicity, BigInt for precision
                    const low = this.readU32(data, offset);
                    const high = this.readU32(data, offset + 4);
                    result[field.name] = low + (high * 0x100000000); // Danger: precision loss > 2^53
                    offset += 8;
                    break;
                case 'Pubkey':
                case 'address':
                    if (offset + 32 <= data.length) {
                        // We return base58 string in a real implementation
                        // Here we just slice because we don't have bs58 dep in this file
                        result[field.name] = data.slice(offset, offset + 32);
                        offset += 32;
                    }
                    break;
                default:
                    // Skip unknown variable-length types to avoid reading garbage
                    // console.warn(`Skipping decoding of complex type ${field.type}`);
                    break;
            }
        }

        return result;
    }

    private readU32(data: Uint8Array, offset: number): number {
        return (
            data[offset] |
            (data[offset + 1] << 8) |
            (data[offset + 2] << 16) |
            (data[offset + 3] << 24)
        ) >>> 0;
    }
}
