
import { ScriptABI, TypeDefinition } from '../metadata/index.js';

export class BorshSchemaGenerator {
    constructor(private abi: ScriptABI) { }

    public generate(typeDef: TypeDefinition): Map<Function, any> {
        const map = new Map<Function, any>();
        this.processType(typeDef, map);
        return map;
    }

    private processType(typeDef: TypeDefinition, map: Map<Function, any>) {
        // Dynamic class creation for the schema
        const StructClass = function () { };
        Object.defineProperty(StructClass, 'name', { value: typeDef.name });

        if (typeDef.structure === 'struct') {
            const fields = typeDef.fields?.map(field => {
                return [field.name, this.mapType(field.type)];
            });

            map.set(StructClass, { kind: 'struct', fields });
        } else if (typeDef.structure === 'enum') {
            // Enum handling
            const values = typeDef.variants?.map(v => {
                // Safe cast or optional access as v matches structure
                // Based on metadata definition, variants have { name, value? } but not explicit 'fields' in interface?
                // We will map simple scalar enums
                return [v.name, undefined];
            });

            map.set(StructClass, { kind: 'enum', values });
        }
    }

    private mapType(type: string): string | any {
        switch (type) {
            case 'u8': return 'u8';
            case 'u16': return 'u16';
            case 'u32': return 'u32';
            case 'u64': return 'u64';
            case 'u128': return 'u128';
            case 'i8': return 'i8';
            case 'i16': return 'i16';
            case 'i32': return 'i32';
            case 'i64': return 'i64';
            case 'i128': return 'i128';
            case 'bool': return 'u8'; // Borsh standard for bool
            case 'string': return 'string';
            case 'pubkey': return [32]; // Fixed array of 32 bytes
            default:
                // Handle vectors: Vec<T>
                if (type.startsWith('Vec<')) {
                    const inner = type.slice(4, -1);
                    return [this.mapType(inner)];
                }
                // Handle arrays: [T; N]
                // Handle option: Option<T>
                return type; // Assume it's a known struct name
        }
    }
}
