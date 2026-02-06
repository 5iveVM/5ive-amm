export const ParameterEncoder = {
    encode_execute: (funcIdx, params) => {
        // Return dummy fixed size instruction data
        // Discriminator(9) + FuncIdx(u32) + ParamCount(u32)
        return new Uint8Array([9, 0,0,0,0, 0,0,0,0]);
    },
    decode_instruction_data: (data) => {
        return {
            discriminator: 2,
            function_index: 0,
            parameters: []
        };
    }
};

export class FiveVMWasm {
    constructor(bytecode) {}
    execute_partial(input, accounts) {
        return {
            status: () => "Completed",
            error_message: () => undefined,
            has_result_value: true,
            get_result_value: { type: "U64", value: 0n },
            compute_units_used: 100,
            stopped_at_opcode_name: "HALT"
        };
    }
    static validate_bytecode(bytecode) { return true; }
    static get_constants() { return "{}"; }
    get_state() { return "{}"; }
}

export class WasmAccount {
    constructor(key, data, lamports, isWritable, isSigner, owner) {}
}

export function wrap_with_script_header(bytecode) {
    return bytecode;
}
