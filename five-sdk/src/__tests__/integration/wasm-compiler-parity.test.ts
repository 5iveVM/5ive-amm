import { describe, expect, it } from '@jest/globals';
import { BytecodeCompiler } from '../../compiler/BytecodeCompiler.js';
import { FiveSDK } from '../../FiveSDK.js';
import { FiveProgram } from '../../program/FiveProgram.js';

describe('WASM Compiler Parity (real compiler)', () => {
  it('compiles current DSL account metadata syntax', async () => {
    const compiler = new BytecodeCompiler();
    const source = `account Counter {
  authority: pubkey;
}

pub init_counter(
  counter: Counter @mut @init(payer=authority, space=96),
  authority: account @mut @signer
) {
  counter.authority = authority.ctx.key;
}

pub assert_authority(
  counter: Counter,
  authority: account @signer
) -> bool {
  require(counter.authority == authority.ctx.key);
  return true;
}
`;

    const result = await compiler.compile({ filename: 'ctx-key.v', content: source }, {
      target: 'vm',
    });

    expect(result.success).toBe(true);
    expect(result.bytecode).toBeDefined();
    expect(result.bytecode!.length).toBeGreaterThan(0);
    expect(result.abi?.functions?.map((fn: any) => fn.name)).toEqual(
      expect.arrayContaining(['init_counter', 'assert_authority']),
    );
  });

  it('compiles lowercase authored types and current runtime metadata syntax', async () => {
    const compiler = new BytecodeCompiler();
    const source = `use std::builtins;

account ClockState {
  last_seen: u64;
}

pub stamp(state: ClockState @mut, authority: account @signer) -> u64 {
  require(authority.ctx.key != 0);
  state.last_seen = authority.ctx.lamports;
  return state.last_seen;
}
`;

    const result = await compiler.compile({ filename: 'stdlib-lowercase.v', content: source }, {
      target: 'vm',
    });

    expect(result.success).toBe(true);
    expect(result.bytecode).toBeDefined();
    expect(result.bytecode!.length).toBeGreaterThan(0);
    expect(result.abi?.functions?.map((fn: any) => fn.name)).toEqual(
      expect.arrayContaining(['stamp']),
    );
  });

  it('compiles current stdlib spl_token module syntax used by public examples', async () => {
    const compiler = new BytecodeCompiler();
    const source = `use std::interfaces::spl_token;

pub transfer_one(
  source: account @mut,
  destination: account @mut,
  authority: account @signer
) {
  spl_token::SPLToken::transfer(source, destination, authority, 1);
}
`;

    const result = await compiler.compile({ filename: 'native-interface.v', content: source }, {
      target: 'vm',
    });

    expect(result.success).toBe(true);
    expect(result.bytecode).toBeDefined();
    expect(result.bytecode!.length).toBeGreaterThan(0);
    expect(result.abi?.functions?.map((fn: any) => fn.name)).toEqual(
      expect.arrayContaining(['transfer_one']),
    );
  });

  it('uses compiler-generated ABI in the public FiveSDK compile result', async () => {
    const source = `account Counter {
  authority: pubkey;
}

pub init_counter(counter: Counter @mut, authority: account @signer) {
  counter.authority = authority.ctx.key;
}

pub has_authority(counter: Counter, authority: account @signer) -> bool {
  require(counter.authority == authority.ctx.key);
  return true;
}
`;

    const result = await FiveSDK.compile({ filename: 'generated-abi.v', content: source }, {
      target: 'vm',
    });

    expect(result.success).toBe(true);
    expect(result.fiveFile).toBeDefined();
    expect(result.fiveFile?.abi?.functions?.map((fn: any) => fn.name)).toEqual(
      expect.arrayContaining(['init_counter', 'has_authority']),
    );

    const authorityParam = result.fiveFile?.abi?.functions?.find((fn: any) => fn.name === 'init_counter')
      ?.parameters?.find((param: any) => param.name === 'authority');
    expect(authorityParam).toMatchObject({
      name: 'authority',
      is_account: true,
    });
  });

  it('builds FiveProgram function builders from compiler-generated ABI', async () => {
    const source = `account Counter {
  authority: pubkey;
  seen: u64;
}

pub init_counter(counter: Counter @mut, authority: account @signer) {
  counter.authority = authority.ctx.key;
  counter.seen = authority.ctx.lamports;
}

pub has_authority(counter: Counter, authority: account @signer) -> bool {
  require(counter.authority == authority.ctx.key);
  return true;
}
`;

    const compiled = await FiveSDK.compile({ filename: 'five-program-generated-abi.v', content: source }, {
      target: 'vm',
    });

    expect(compiled.success).toBe(true);
    expect(compiled.fiveFile?.abi).toBeDefined();

    const program = FiveProgram.fromABI(
      'So11111111111111111111111111111111111111112',
      compiled.fiveFile!.abi as any,
    );

    expect(program.getFunctions()).toEqual(
      expect.arrayContaining(['init_counter', 'has_authority']),
    );

    const builder = program
      .function('init_counter')
      .accounts({
        counter: 'SysvarRent111111111111111111111111111111111',
        authority: 'SysvarC1ock11111111111111111111111111111111',
      });

    expect(builder.getFunctionDef().name).toBe('init_counter');
    expect(builder.getAccounts()).toMatchObject({
      counter: 'SysvarRent111111111111111111111111111111111',
      authority: 'SysvarC1ock11111111111111111111111111111111',
    });

    const instruction = await builder
      .payer('SysvarC1ock11111111111111111111111111111111')
      .instruction();
    expect(instruction.programId).toBeDefined();
    expect(instruction.keys.some((key) => key.pubkey === 'SysvarRent111111111111111111111111111111111')).toBe(true);
    expect(instruction.keys.some((key) => key.pubkey === 'SysvarC1ock11111111111111111111111111111111')).toBe(true);
  });

  it('round-trips compiler-generated .five artifacts through loadFiveFile and ABI consumers', async () => {
    const source = `account Counter {
  authority: pubkey;
  seen: u64;
}

pub init_counter(counter: Counter @mut, authority: account @signer) {
  counter.authority = authority.ctx.key;
  counter.seen = authority.ctx.lamports;
}
`;

    const compiled = await FiveSDK.compile({ filename: 'roundtrip-generated-abi.v', content: source }, {
      target: 'vm',
    });

    expect(compiled.success).toBe(true);
    expect(compiled.fiveFile).toBeDefined();

    const serialized = JSON.stringify(compiled.fiveFile);
    const loaded = await FiveSDK.loadFiveFile(serialized);

    expect(Array.from(loaded.bytecode)).toEqual(Array.from(FiveSDK.extractBytecode(compiled.fiveFile!)));
    expect(loaded.abi).toEqual(compiled.fiveFile!.abi);

    const program = FiveProgram.fromABI(
      'So11111111111111111111111111111111111111112',
      loaded.abi,
    );

    expect(program.getFunctions()).toEqual(
      expect.arrayContaining(['init_counter']),
    );

    const builder = program
      .function('init_counter')
      .accounts({
        counter: 'SysvarRent111111111111111111111111111111111',
        authority: 'SysvarC1ock11111111111111111111111111111111',
      });

    const instruction = await builder
      .payer('SysvarC1ock11111111111111111111111111111111')
      .instruction();
    expect(instruction.keys.length).toBeGreaterThan(0);
    expect(instruction.keys.some((key) => key.pubkey === 'SysvarRent111111111111111111111111111111111')).toBe(true);
  });
});
