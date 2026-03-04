import { describe, expect, it, jest } from "@jest/globals";

const readFile = jest.fn(async () => JSON.stringify({
  accounts: {
    sample_account: { owner: "system", lamports: 1, data_len: 0 },
  },
  tests: {
    test_add: {
      parameters: [1, 2],
      expected: { success: true },
    },
  },
}));
const stat = jest.fn(async () => ({ isFile: () => true, isDirectory: () => false }));

jest.unstable_mockModule("fs/promises", () => ({
  readFile,
  readdir: jest.fn(),
  stat,
}));

let TestDiscovery: any;

describe("TestDiscovery fixture compatibility", () => {
  it("ignores fixture-shaped .test.json files without warning", async () => {
    const warn = jest.spyOn(console, "warn").mockImplementation(() => {});
    ({ TestDiscovery } = await import("../../testing/TestDiscovery.js"));

    const discovered = await TestDiscovery.discoverTests("/tmp/main.test.json");

    expect(discovered).toEqual([]);
    expect(warn).not.toHaveBeenCalled();
    warn.mockRestore();
  });
});
