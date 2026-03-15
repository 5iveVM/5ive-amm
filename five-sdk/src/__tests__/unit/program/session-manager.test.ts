import { describe, it, expect } from '@jest/globals';
import { SessionManager } from '../../../program/SessionManager.js';
import { FiveProgram } from '../../../program/FiveProgram.js';

describe('SessionManager', () => {
  it('produces stable scope hashes regardless of function order', () => {
    const a = SessionManager.scopeHashForFunctions(['play', 'hit', 'stand']);
    const b = SessionManager.scopeHashForFunctions(['stand', 'play', 'hit']);
    expect(a).toEqual(b);
  });

  it('enforces canonical session manager on mainnet by default', () => {
    const abi: any = { name: 'Session', functions: [] };
    const program = FiveProgram.fromABI(
      '11111111111111111111111111111111',
      abi,
      { fiveVMProgramId: '2DXiYbzfSMwkDSxc9aWEaW7XgJjkNzGdADfRN4FbxMNN' },
    );

    expect(() =>
      new SessionManager(program, 3000, {
        identity: {
          cluster: 'mainnet',
          scriptAccount: '9xQeWvG816bUx9EPfB6PW5fNtx4AQm5fpu6vXcQ5s4gW',
          codeHash: '11111111111111111111111111111111',
          version: 1,
          status: 'active',
        },
      }),
    ).toThrow(/canonical session_v1 service/i);
  });

  it('allows mainnet override only behind allowUnsafeOverride', () => {
    const abi: any = { name: 'Session', functions: [] };
    const program = FiveProgram.fromABI(
      '11111111111111111111111111111111',
      abi,
      { fiveVMProgramId: '2DXiYbzfSMwkDSxc9aWEaW7XgJjkNzGdADfRN4FbxMNN' },
    );

    expect(
      () =>
        new SessionManager(program, 3000, {
          identity: {
            cluster: 'mainnet',
            scriptAccount: '11111111111111111111111111111111',
            codeHash: '11111111111111111111111111111111',
            version: 1,
            status: 'active',
          },
          enforceCanonical: false,
        }),
    ).toThrow(/allowUnsafeOverride/);

    expect(
      () =>
        new SessionManager(program, 3000, {
          identity: {
            cluster: 'mainnet',
            scriptAccount: '11111111111111111111111111111111',
            codeHash: '11111111111111111111111111111111',
            version: 1,
            status: 'active',
          },
          enforceCanonical: false,
          allowUnsafeOverride: true,
        }),
    ).not.toThrow();
  });
});
