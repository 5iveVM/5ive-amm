
import { FiveProgram } from '../../program/FiveProgram.js';
import { TypeGenerator } from '../../program/TypeGenerator.js';
import { ProgramAccount } from '../../program/ProgramAccount.js';
import { ScriptABI } from '../../metadata/index.js';
import * as borsh from 'borsh';
import { PublicKey } from '@solana/web3.js';
import { TestUtils, TestConstants } from '../setup.js';
import { jest, describe, it, expect } from '@jest/globals';

describe('New SDK Features Full Test', () => {
    // 1. Complex ABI for testing
    const complexABI: ScriptABI = {
        name: 'ComplexContract',
        functions: [
            {
                name: 'process_data',
                index: 0,
                parameters: [
                    { name: 'data', type: 'MyStruct', is_account: false, attributes: [] }
                ],
                return_type: null,
                is_public: true, // legacy format
                visibility: 'public', // new format
                bytecode_offset: 0
            } as any
        ],
        types: [
            {
                name: 'MyStruct',
                structure: 'struct',
                fields: [
                    { name: 'count', type: 'u64' },
                    { name: 'label', type: 'string' }
                ]
            }
        ]
    };

    describe('Type Generation', () => {
        it('should generate properly typed interfaces', () => {
            const generator = new TypeGenerator(complexABI);
            const typeDefs = generator.generate();

            // Verify basic structure
            expect(typeDefs).toContain('export interface ComplexContractProgram');
            expect(typeDefs).toContain('process_data(params: Process_dataParams): FunctionBuilder;');

            // Verify struct mappings (depending on implementation, might be 'any' or specfic)
            // Ideally should contain interface for MyStruct params
            expect(typeDefs).toContain('export interface Process_dataParams');
        });
    });

    describe('Robust State Decoding (ProgramAccount)', () => {
        it('should decode complex structs using generated Borsh schema', async () => {
            // 1. Construct valid Borsh buffer for MyStruct { count: 123456789, label: "Hello Five" }
            const count = BigInt(123456789);
            const label = "Hello Five";

            // u64 (8) + string len (4) + string bytes
            const bufferSize = 8 + 4 + label.length;
            const buffer = Buffer.alloc(bufferSize);

            buffer.writeBigUInt64LE(count, 0);
            buffer.writeUInt32LE(label.length, 8);
            buffer.write(label, 12);

            // 2. Mock fetcher
            const mockFetcher = {
                getAccountData: jest.fn().mockResolvedValue({
                    address: 'TestAccountAddress',
                    data: new Uint8Array(buffer),
                    owner: TestConstants.FIVE_VM_PROGRAM_ID,
                    lamports: 1000
                } as any),
                getMultipleAccountsData: jest.fn() as any
            };

            // 3. Create Program & Account
            const program = new FiveProgram('ScriptAddr', complexABI, { fetcher: mockFetcher as any });
            const account = program.account('MyStruct');

            // 4. Fetch and Decode
            const decoded = await account.fetch('TestAccountAddress');

            // 5. Verify
            // Depending on how u64 is decoded (BigInt or Number), check accordingly.
            // ProgramAccount.ts readU64 (simpleDecode) uses Number fallback if > 2^53? 
            // But if it uses Borsh, it returns BN or BigInt usually?
            // Expect loosely or check type.

            if (typeof decoded.count === 'bigint') {
                expect(decoded.count).toBe(count);
            } else {
                // BN or other
                expect(decoded.count.toString()).toBe(count.toString());
            }
            expect(decoded.label).toBe(label);
        });
    });

    describe('FiveProgram.load()', () => {
        it('should load program from chain metadata', async () => {
            // 1. Create a script account buffer with our complexABI embedded
            // We need to override createTestScriptAccountData to inject OUR abi.
            // But TestUtils helps.
            // We can manually create the buffer following the structure in TestUtils.

            // Or reuse TestUtils but patch the ABI JSON
            // TestUtils.createTestScriptAccountData puts ABI at the end.
            const scriptData = TestUtils.createTestScriptAccountData(TestConstants.SAMPLE_BYTECODE);
            // This has "test_contract" ABI.

            const mockConnection = {
                getAccountInfo: jest.fn().mockResolvedValue({
                    data: scriptData,
                    owner: { toBase58: () => TestConstants.FIVE_VM_PROGRAM_ID },
                    lamports: 1000000,
                    executable: false
                } as any),
                getMultipleAccountsInfo: jest.fn()
            };

            // 2. Call load
            // Generate guaranteed valid base58 address for 32 bytes
            const scriptAddr = new PublicKey(new Uint8Array(32).fill(1)).toBase58();
            const program = await FiveProgram.load(scriptAddr, mockConnection);

            // 3. Verify
            expect(program).toBeInstanceOf(FiveProgram);
            expect(program.getFunctions()).toContain('test'); // From TestConstants.SAMPLE_ABI
            expect(program.getFunctions()).toContain('add');
        });
    });
});
