jest.mock('chalk', () => {
  const passthrough = (s: string) => s;
  return {
    __esModule: true,
    default: {
      bold: passthrough,
      green: passthrough,
      red: passthrough,
      gray: passthrough,
      cyan: passthrough,
      yellow: passthrough,
      magenta: passthrough,
      magentaBright: passthrough,
      white: passthrough,
      hex: () => passthrough,
    },
  };
});

jest.mock('ora', () => {
  const spinner = {
    start: () => spinner,
    succeed: () => spinner,
    fail: () => spinner,
    stop: () => spinner,
    text: '',
  };
  return () => spinner;
});

jest.mock('@5ive-tech/sdk', () => ({
  FiveSDK: {
    createDeploymentTransaction: jest.fn(),
  },
}), { virtual: true });

import { FiveSDK } from '@5ive-tech/sdk';
import {
  __deriveFallbackReason,
  __isTransactionSizeError,
  __regularDeployFitsTransaction,
} from '../deploy.js';

describe('deploy auto mode helpers', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('detects transaction size errors', () => {
    expect(__isTransactionSizeError('Transaction too large: 1400 > 1232')).toBe(true);
    expect(__isTransactionSizeError('packet data too large')).toBe(true);
    expect(__isTransactionSizeError('rpc timeout')).toBe(false);
  });

  it('derives tx_too_large fallback reason from size failures', () => {
    expect(__deriveFallbackReason('encoded transaction exceeds limit')).toBe('tx_too_large');
    expect(__deriveFallbackReason('simulation failed')).toBe('simulation_failed');
  });

  it('marks regular deploy as non-fit when serialized tx exceeds safe limit', async () => {
    (FiveSDK.createDeploymentTransaction as jest.Mock).mockResolvedValue({
      transaction: {
        partialSign: jest.fn(),
        serialize: jest.fn(() => Buffer.alloc(1300)),
      },
    });

    const result = await __regularDeployFitsTransaction(
      new Uint8Array([1, 2, 3]),
      { simulateTransaction: jest.fn() } as any,
      { publicKey: { toString: () => 'deployer' } } as any,
      { bytecode: new Uint8Array([1]), network: 'devnet' } as any,
      {},
    );

    expect(result.fits).toBe(false);
    expect(result.reason).toBe('tx_too_large');
  });

  it('marks regular deploy as non-fit when simulation fails', async () => {
    const simulateTransaction = jest.fn().mockResolvedValue({
      value: { err: { InstructionError: [0, 'InvalidInstructionData'] } },
    });
    (FiveSDK.createDeploymentTransaction as jest.Mock).mockResolvedValue({
      transaction: {
        partialSign: jest.fn(),
        serialize: jest.fn(() => Buffer.alloc(800)),
      },
    });

    const result = await __regularDeployFitsTransaction(
      new Uint8Array([1, 2, 3]),
      { simulateTransaction } as any,
      { publicKey: { toString: () => 'deployer' } } as any,
      { bytecode: new Uint8Array([1]), network: 'devnet' } as any,
      {},
    );

    expect(result.fits).toBe(false);
    expect(result.reason).toBe('simulation_failed');
  });
});
