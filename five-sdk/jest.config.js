const useSolanaMocks = process.env.USE_SOLANA_MOCKS !== '0';

const moduleNameMapper = {
  '^(\\.{1,2}/.*)\\.js$': '$1',
  '^five-sdk$': '<rootDir>/src/__tests__/mocks/five-sdk.ts',
};

if (useSolanaMocks) {
  moduleNameMapper['^@solana/web3.js$'] = '<rootDir>/src/__tests__/mocks/solana-web3.ts';
}

export default {
  preset: 'ts-jest/presets/default-esm', // Use ESM preset
  testEnvironment: 'node',
  // Only run source TypeScript tests; ignore generated JS and declaration artifacts in src.
  testMatch: ['**/?(*.)+(spec|test).ts'],
  modulePathIgnorePatterns: [
    '<rootDir>/dist/',
    '<rootDir>/src/assets/vm/package.json',
    '<rootDir>/src/assets/wasm/package.json',
  ],
  moduleNameMapper,
  transform: {
    '^.+\\.tsx?$': ['ts-jest', {
      useESM: true,
      tsconfig: {
        target: 'es2020',
        module: 'es2020', // Explicitly set module to supports import.meta
        moduleResolution: 'node',
        esModuleInterop: true,
        strict: false
      }
    }],
  },
  extensionsToTreatAsEsm: ['.ts'],
  moduleFileExtensions: ['ts', 'js', 'json', 'node'],
  roots: ['<rootDir>/src'],
};
