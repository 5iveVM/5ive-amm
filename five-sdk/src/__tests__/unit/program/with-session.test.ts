import { describe, expect, it } from '@jest/globals';
import { FiveProgram } from '../../../program/FiveProgram.js';
import { SessionManager } from '../../../program/SessionManager.js';

describe('FiveProgram.withSession', () => {
  it('returns a program instance with session config applied', () => {
    const abi: any = {
      name: 'Game',
      functions: [
        {
          name: 'play_move',
          index: 0,
          parameters: [
            { name: 'session', is_account: true, attributes: ['session'] },
            { name: 'delegate', is_account: true, attributes: ['signer'] },
          ],
        },
      ],
    };

    const base = FiveProgram.fromABI(
      '11111111111111111111111111111111',
      abi,
      {},
    );

    // Minimal SessionManager stub compatible with the option type.
    const manager = {
      defaultTtlSlots: 3000,
    } as unknown as SessionManager;

    const withSession = base.withSession({
      manager,
      mode: 'auto',
      sessionAccountByFunction: { play_move: 'Sess1111111111111111111111111111111111' },
      delegateAccountByFunction: { play_move: 'Delg1111111111111111111111111111111111' },
    });

    expect(withSession).toBeInstanceOf(FiveProgram);
    expect(withSession).not.toBe(base);
  });
});
