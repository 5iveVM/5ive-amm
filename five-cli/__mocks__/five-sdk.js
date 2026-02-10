module.exports = {
  FiveSDK: {
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
    validateBytecode: jest.fn().mockResolvedValue({ success: true }),
    deployToSolana: jest.fn().mockResolvedValue({
      success: true,
      programId: 'mock-program',
      transactionId: 'tx',
      deploymentCost: 0
    }),
    executeLocal: jest.fn().mockResolvedValue({
      success: true,
      result: { value: 0, type: 'u64' }
    })
  }
};
