/**
 * Jest setup file for WASM tests
 */

// Mock performance if not available in test environment
if (!global.performance) {
  global.performance = {
    now: () => Date.now(),
  } as any;
}

// Mock console methods for cleaner test output
global.console = {
  ...console,
  // Keep error and warn for debugging
  log: jest.fn(),
  debug: jest.fn(),
  info: jest.fn(),
};