import { beforeEach, describe, expect, it, jest, afterEach } from '@jest/globals';
import { PublicKey } from '@solana/web3.js';
import { FiveSDK } from '../../FiveSDK.js';
import { ProgramIdResolver } from '../../config/ProgramIdResolver.js';
import { FIVE_VM_PROGRAM_ID } from '../../types.js';
import { TestUtils, TestConstants } from '../setup.js';

describe('Five SDK Integration Tests', () => {
  let mockCompiler: {
    compile: any;
    compileFile: any;
    getFunctionNames: any;
  };

  beforeEach(() => {
    jest.clearAllMocks();

    mockCompiler = {
      compile: jest.fn(),
      compileFile: jest.fn(),
      getFunctionNames: jest.fn(),
    };

    // Keep tests deterministic and avoid real WASM/compiler initialization.
    (FiveSDK as any).compiler = mockCompiler;
    (FiveSDK as any).parameterEncoder = {};

    // Set default program ID to the canonical Five VM program ID for tests
    ProgramIdResolver.setDefault(FIVE_VM_PROGRAM_ID);
  });

  afterEach(() => {
    ProgramIdResolver.clearDefault();
  });

  describe('Script Compilation Workflow', () => {
    it('passes current DSL source through to the public compiler unchanged', async () => {
      const source = `account Counter {\n  authority: pubkey;\n}\n\npub init(counter: Counter @mut, authority: account @signer) {\n  counter.authority = authority.ctx.key;\n}`;

      mockCompiler.compile.mockResolvedValue({
        success: true,
        bytecode: TestConstants.SAMPLE_BYTECODE,
        abi: {
          functions: [
            { name: 'init', index: 0, parameters: [], visibility: 'public' },
          ],
        },
      });
      mockCompiler.getFunctionNames.mockResolvedValue([
        { name: 'init', function_index: 0 },
      ]);

      const result = await FiveSDK.compile(source);

      expect(result.success).toBe(true);
      expect(mockCompiler.compile).toHaveBeenCalledWith(source, {});
    });

    it('compiles source and preserves normalized function names', async () => {
      mockCompiler.compile.mockResolvedValue({
        success: true,
        bytecode: TestConstants.SAMPLE_BYTECODE,
        abi: {
          functions: [
            { name: 'transfer', index: 1, parameters: [], visibility: 'public' },
          ],
        },
      });
      mockCompiler.getFunctionNames.mockResolvedValue([
        { name: 'transfer', function_index: 1 },
      ]);

      const result = await FiveSDK.compile('fn transfer() {}');

      expect(result.success).toBe(true);
      expect(result.bytecode?.length).toBe(TestConstants.SAMPLE_BYTECODE.length);
      expect(result.publicFunctionNames).toEqual(['transfer']);
      expect(mockCompiler.compile).toHaveBeenCalledTimes(1);
      expect(mockCompiler.getFunctionNames).toHaveBeenCalledTimes(1);
    });

    it('passes through compileFile options', async () => {
      mockCompiler.compileFile.mockResolvedValue({
        success: true,
        bytecode: TestConstants.SAMPLE_BYTECODE,
      });

      const filePath = 'src/examples/basic-usage.v';
      const result = await FiveSDK.compileFile(filePath, { optimize: true });

      expect(result.success).toBe(true);
      expect(mockCompiler.compileFile).toHaveBeenCalledWith(filePath, { optimize: true });
    });

    it('preserves account metadata fields in compiled fiveFile ABI parameters', async () => {
      mockCompiler.compile.mockResolvedValue({
        success: true,
        bytecode: TestConstants.SAMPLE_BYTECODE,
        abi: {
          functions: [
            {
              name: 'initialize',
              index: 0,
              parameters: [
                {
                  name: 'counter',
                  type: 'Counter',
                  param_type: 'Counter',
                  is_account: true,
                  attributes: ['mut', 'init'],
                  optional: false,
                },
                {
                  name: 'owner',
                  type: 'account',
                  is_account: true,
                  attributes: ['signer'],
                },
                {
                  name: 'amount',
                  type: 'u64',
                  is_account: false,
                  attributes: [],
                  optional: true,
                },
              ],
              visibility: 'public',
            },
          ],
        },
      });
      mockCompiler.getFunctionNames.mockResolvedValue([
        { name: 'initialize', function_index: 0 },
      ]);

      const result = await FiveSDK.compile('fn initialize() {}');
      const params = result.fiveFile?.abi?.functions?.[0]?.parameters;

      expect(params).toHaveLength(3);
      expect(params[0]).toMatchObject({
        name: 'counter',
        type: 'Counter',
        param_type: 'Counter',
        is_account: true,
        optional: false,
      });
      expect(params[0].attributes).toEqual(['mut', 'init']);
      expect(params[1]).toMatchObject({
        name: 'owner',
        type: 'account',
        is_account: true,
      });
      expect(params[1].attributes).toEqual(['signer']);
      expect(params[2]).toMatchObject({
        name: 'amount',
        type: 'u64',
        is_account: false,
        optional: true,
      });
      expect(params[2].attributes).toEqual([]);
    });
  });

  describe('Deployment Instruction Generation', () => {
    it('generates deploy instruction with canonical program id and discriminator', async () => {
      const bytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 1, 2, 3, 4, 5]);
      const deployer = TestConstants.TEST_USER_PUBKEY;

      const result = await FiveSDK.generateDeployInstruction(bytecode, deployer);

      expect(result.instruction.programId).toBe(FIVE_VM_PROGRAM_ID);
      expect(result.requiredSigners).toEqual([deployer]);
      expect(result.bytecodeSize).toBe(bytecode.length);
      expect(result.instruction.accounts.length).toBeGreaterThanOrEqual(4);

      const raw = Buffer.from(result.instruction.data, 'base64');
      expect(raw[0]).toBe(8);

      const derivedSeed = result.setupInstructions?.createScriptAccount?.seed;
      expect(typeof derivedSeed).toBe('string');
      expect(derivedSeed).toHaveLength(32);

      const expectedScriptAccount = await PublicKey.createWithSeed(
        new PublicKey(deployer),
        derivedSeed,
        new PublicKey(FIVE_VM_PROGRAM_ID)
      );
      expect(result.scriptAccount).toBe(expectedScriptAccount.toBase58());
    });

    it('encodes export metadata bytes into deploy instruction payload', async () => {
      const bytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 1, 2, 3, 4, 5, 6]);
      const deployer = TestConstants.TEST_USER_PUBKEY;

      const result = await FiveSDK.generateDeployInstruction(bytecode, deployer, {
        exportMetadata: {
          methods: ['transfer'],
          interfaces: [{ name: 'TokenOps', methodMap: { transfer: 'transfer_checked' } }],
        } as any,
      });

      const raw = Buffer.from(result.instruction.data, 'base64');
      expect(raw[0]).toBe(8);
      const bytecodeLen = raw.readUInt32LE(1);
      expect(bytecodeLen).toBe(bytecode.length);
      const permissions = raw[5];
      expect(permissions).toBe(0);
      const metadataLen = raw.readUInt32LE(6);
      expect(metadataLen).toBeGreaterThan(0);
      // metadata starts with "5EXP"
      expect(raw[10]).toBe(0x35);
      expect(raw[11]).toBe(0x45);
      expect(raw[12]).toBe(0x58);
      expect(raw[13]).toBe(0x50);
      const bytecodeStart = 10 + metadataLen;
      expect(raw.slice(bytecodeStart, bytecodeStart + bytecode.length)).toEqual(Buffer.from(bytecode));
    });

    it('honors a validated scriptAccount + scriptSeed override', async () => {
      const bytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 4, 4, 4, 4, 4]);
      const deployer = TestConstants.TEST_USER_PUBKEY;
      const scriptSeed = '0123456789abcdef0123456789abcdef';
      const scriptAccount = (
        await PublicKey.createWithSeed(
          new PublicKey(deployer),
          scriptSeed,
          new PublicKey(FIVE_VM_PROGRAM_ID)
        )
      ).toBase58();

      const result = await FiveSDK.generateDeployInstruction(bytecode, deployer, {
        scriptAccount,
        scriptSeed,
      });

      expect(result.scriptAccount).toBe(scriptAccount);
      expect(result.setupInstructions?.createScriptAccount?.seed).toBe(scriptSeed);
    });

    it('rejects scriptAccount overrides without a matching scriptSeed', async () => {
      const bytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 5, 5, 5, 5, 5]);
      const deployer = TestConstants.TEST_USER_PUBKEY;

      await expect(
        FiveSDK.generateDeployInstruction(bytecode, deployer, {
          scriptAccount: TestConstants.TEST_SCRIPT_ACCOUNT,
        }),
      ).rejects.toThrow('options.scriptSeed is required');

      await expect(
        FiveSDK.generateDeployInstruction(bytecode, deployer, {
          scriptAccount: TestConstants.TEST_SCRIPT_ACCOUNT,
          scriptSeed: 'fedcba9876543210fedcba9876543210',
        }),
      ).rejects.toThrow('options.scriptAccount does not match');
    });
  });

  describe('Execution Instruction Generation', () => {
    it('encodes execute payload and resolves function index from ABI', async () => {
      const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
      const accounts = [TestConstants.TEST_USER_PUBKEY];

      const result = await FiveSDK.generateExecuteInstruction(
        scriptAccount,
        'transfer',
        [123, true],
        accounts,
        undefined,
        {
          abi: {
            functions: [
              {
                name: 'transfer',
                index: 7,
                parameters: [
                  { name: 'amount', type: 'u64', is_account: false },
                  { name: 'flag', type: 'bool', is_account: false },
                ],
              },
            ],
          },
          computeUnitLimit: 150_000,
          payerAccount: TestConstants.TEST_USER_PUBKEY,
        },
      );

      expect(result.instruction.programId).toBe(FIVE_VM_PROGRAM_ID);
      expect(result.parameters.function).toBe('transfer');
      expect(result.parameters.count).toBe(2);
      expect(result.estimatedComputeUnits).toBe(150_000);
      expect(result.instruction.accounts[0].pubkey).toBe(scriptAccount);
      expect(result.instruction.accounts.length).toBeGreaterThanOrEqual(2);

      const raw = Buffer.from(result.instruction.data, 'base64');
      expect(raw[0]).toBe(9);
      expect(raw.readUInt32LE(4)).toBe(7);
      expect(raw.readUInt32LE(8)).toBe(2);
    });

    it('falls back to numeric index 0 when function name cannot be resolved', async () => {
      const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;

      const result = await FiveSDK.generateExecuteInstruction(
        scriptAccount,
        'missing_function',
        [],
        [TestConstants.TEST_USER_PUBKEY],
        undefined,
        {
          abi: { functions: [{ name: 'known', index: 5, parameters: [] }] },
          payerAccount: TestConstants.TEST_USER_PUBKEY,
        },
      );

      const raw = Buffer.from(result.instruction.data, 'base64');
      expect(raw.readUInt32LE(4)).toBe(0);
      expect(result.parameters.function).toBe('missing_function');
    });

    it('estimates compute units from function index and parameter count when not provided', async () => {
      const result = await FiveSDK.generateExecuteInstruction(
        TestConstants.TEST_SCRIPT_ACCOUNT,
        20,
        [1, 2, 3, 4, 5],
        [TestConstants.TEST_USER_PUBKEY],
        undefined,
        {
          abi: {
            functions: [
              { name: 'f', index: 20, parameters: [] },
            ],
          },
          payerAccount: TestConstants.TEST_USER_PUBKEY,
        },
      );

      expect(result.estimatedComputeUnits).toBe(5_500);
    });
  });

  describe('Script Metadata Operations', () => {
    it('gets metadata using accountFetcher contract', async () => {
      const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
      const accountData = TestUtils.createTestScriptAccountData(TestConstants.SAMPLE_BYTECODE);
      const accountFetcher = {
        getAccountData: jest.fn(async () => ({
          data: accountData,
          owner: FIVE_VM_PROGRAM_ID,
          lamports: 1_000_000,
        })),
      };

      const result = await FiveSDK.getScriptMetadataWithConnection(scriptAccount, accountFetcher);

      expect(result.address).toBe(scriptAccount);
      expect(result.bytecode.length).toBe(TestConstants.SAMPLE_BYTECODE.length);
      expect(accountFetcher.getAccountData).toHaveBeenCalledWith(scriptAccount);
    });

    it('caches metadata results and avoids duplicate fetches', async () => {
      const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
      const accountData = TestUtils.createTestScriptAccountData(TestConstants.SAMPLE_BYTECODE);
      const accountFetcher = {
        getAccountData: jest.fn(async () => ({
          data: accountData,
          owner: FIVE_VM_PROGRAM_ID,
          lamports: 1_000_000,
        })),
      };

      const first = await FiveSDK.getCachedScriptMetadata(scriptAccount, accountFetcher, 60_000);
      const second = await FiveSDK.getCachedScriptMetadata(scriptAccount, accountFetcher, 60_000);

      expect(first.address).toBe(scriptAccount);
      expect(second.address).toBe(scriptAccount);
      expect(accountFetcher.getAccountData).toHaveBeenCalledTimes(1);
    });
  });
});
