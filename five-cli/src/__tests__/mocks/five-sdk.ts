export const FiveSDK = {
  compile: jest.fn().mockResolvedValue({
    success: true,
    fiveFile: {},
    bytecode: new Uint8Array(),
    metadata: {}
  }),
  compileModules: jest.fn().mockResolvedValue({
    success: true,
    fiveFile: {},
    bytecode: new Uint8Array(),
    metadata: {}
  }),
  validateBytecode: jest.fn().mockResolvedValue({ success: true, valid: true }),
  deployToSolana: jest.fn().mockResolvedValue({
    success: true,
    programId: 'mock-program',
    transactionId: 'tx',
    deploymentCost: 0
  }),
  executeOnSolana: jest.fn().mockResolvedValue({
    success: true,
    result: 0,
    computeUnitsUsed: 0,
    cost: 0
  }),
  executeLocally: jest.fn().mockResolvedValue({
    success: true,
    result: 0
  }),
  execute: jest.fn().mockResolvedValue({
    success: true,
    result: 0
  }),
  loadFiveFile: jest.fn().mockResolvedValue({
    bytecode: new Uint8Array(),
    abi: {}
  }),
  deployLargeProgramOptimizedToSolana: jest.fn().mockResolvedValue({ success: true }),
  deployLargeProgramToSolana: jest.fn().mockResolvedValue({ success: true }),
  executeScriptAccount: jest.fn().mockResolvedValue({ success: true })
};

export class FiveTestRunner {
  async discoverTestSuites() {
    return [];
  }
  async runTestSuites() {
    return [];
  }
}

export const TestDiscovery = {
  discoverTests: jest.fn().mockResolvedValue([])
};

export const compileScript = jest.fn();
export const executeLocally = jest.fn();
export const compileAndExecuteLocally = jest.fn();
