import { beforeAll, beforeEach, describe, expect, it, jest } from '@jest/globals';

const mockGetVMState = jest.fn(async () => ({
  authority: '11111111111111111111111111111111',
  scriptCount: 1,
  deployFeeLamports: 500,
  executeFeeLamports: 250,
  deployFeeBps: 500,
  executeFeeBps: 250,
  feeVaultShardCount: 10,
  vmStateBump: 255,
  isInitialized: true,
}));

jest.unstable_mockModule('../../modules/vm-state.js', () => ({
  getVMState: mockGetVMState,
}));

jest.unstable_mockModule('../../crypto/index.js', () => ({
  RentCalculator: {
    calculateRentExemption: jest.fn(async () => 10_000),
    formatSOL: (value: number) => `${value}`,
  },
}));

let FeesModule: any;

describe('fees module (flat lamports)', () => {
  beforeAll(async () => {
    FeesModule = await import('../../modules/fees.js');
  });

  beforeEach(() => {
    mockGetVMState.mockClear();
  });

  it('returns lamport-denominated fees from VM state', async () => {
    const fees = await FeesModule.getFees({});
    expect(fees.deployFeeLamports).toBe(500);
    expect(fees.executeFeeLamports).toBe(250);
    expect(fees.deployFeeBps).toBe(500);
    expect(fees.executeFeeBps).toBe(250);
  });

  it('uses flat deploy fee lamports in estimate', async () => {
    const estimate = await FeesModule.calculateDeployFee(64, {});
    expect(estimate.basisLamports).toBe(10_000);
    expect(estimate.feeLamports).toBe(500);
    expect(estimate.totalEstimatedCost).toBe(10_500);
  });

  it('uses flat execute fee lamports in estimate', async () => {
    const estimate = await FeesModule.calculateExecuteFee({});
    expect(estimate.basisLamports).toBe(5_000);
    expect(estimate.feeLamports).toBe(250);
    expect(estimate.totalEstimatedCost).toBe(5_250);
  });
});
