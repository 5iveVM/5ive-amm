module.exports = {
  normalizeAbiFunctions: (abiFunctions) => {
    if (!abiFunctions) return [];
    const functionsArray = Array.isArray(abiFunctions)
      ? abiFunctions
      : Object.entries(abiFunctions).map(([name, func]) => ({ name, ...(func || {}) }));

    return functionsArray.map((func, idx) => {
      const rawParameters = Array.isArray(func.parameters) ? func.parameters : [];
      const existingNames = new Set(rawParameters.map((param) => param.name));
      const accountParameters = Array.isArray(func.accounts)
        ? func.accounts
            .map((account, accountIdx) => ({
              name: account.name ?? `account${accountIdx}`,
              type: 'pubkey',
              param_type: 'pubkey',
              optional: false,
              is_account: true,
              isAccount: true,
              attributes: [
                ...(account.writable ? ['mut'] : []),
                ...(account.signer ? ['signer'] : []),
              ],
            }))
            .filter((param) => !existingNames.has(param.name))
        : [];

      return {
        name: func.name ?? `function_${idx}`,
        index: typeof func.index === 'number' ? func.index : idx,
        parameters: [
          ...accountParameters,
          ...rawParameters.map((param) => ({
            name: param.name,
            type: param.type ?? param.param_type ?? '',
            param_type: param.param_type,
            optional: param.optional ?? false,
            is_account: param.is_account ?? param.isAccount ?? false,
            isAccount: param.isAccount ?? param.is_account ?? false,
            attributes: Array.isArray(param.attributes) ? [...param.attributes] : [],
          })),
        ],
        accounts: func.accounts ?? [],
        visibility: func.visibility ?? 'public',
        returnType: func.returnType ?? func.return_type,
      };
    });
  },
  TypeGenerator: class {
    constructor(abi) {
      this.abi = abi;
    }
    generate() {
      return `// generated for ${Array.isArray(this.abi?.functions) ? this.abi.functions.length : 0} functions\n`;
    }
  },
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
