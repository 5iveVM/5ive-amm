import { mkdtemp, writeFile, readFile } from "fs/promises";
import { join } from "path";
import { tmpdir } from "os";
import { Keypair } from "@solana/web3.js";

jest.mock("chalk", () => {
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

const setNamespaceSymbolPriceOnChain = jest.fn();
const getNamespaceSymbolPriceOnChain = jest.fn();

const applyOverrides = jest.fn();
const getProgramId = jest.fn();

jest.mock("@5ive-tech/sdk", () => ({
  FiveSDK: {
    setNamespaceSymbolPriceOnChain,
    getNamespaceSymbolPriceOnChain,
  },
}));

jest.mock("../../config/ConfigManager.js", () => ({
  ConfigManager: {
    getInstance: () => ({
      applyOverrides,
      getProgramId,
    }),
    getTargetPrefix: () => "[devnet]",
  },
}));

jest.mock("../../project/ProjectLoader.js", () => ({
  loadProjectConfig: jest.fn().mockResolvedValue(null),
}));

import { namespaceCommand } from "../namespace.js";

const logger = {
  debug: jest.fn(),
  info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn(),
};

function createContext() {
  return {
    config: { rootDir: process.cwd() },
    logger,
    wasmManager: null,
    options: { debug: false, verbose: false },
  };
}

async function createTempProject(): Promise<{ root: string; keypairPath: string }> {
  const root = await mkdtemp(join(tmpdir(), "five-cli-namespace-"));
  const keypair = Keypair.generate();
  const keypairPath = join(root, "id.json");
  await writeFile(keypairPath, JSON.stringify(Array.from(keypair.secretKey)));
  return { root, keypairPath };
}

describe("namespace command price actions", () => {
  beforeEach(() => {
    jest.clearAllMocks();
    getProgramId.mockResolvedValue(undefined);
  });

  it("rejects unsupported symbol for set-price", async () => {
    const { root, keypairPath } = await createTempProject();
    applyOverrides.mockResolvedValue({
      target: "devnet",
      networks: { devnet: { rpcUrl: "http://127.0.0.1:8899" } },
      keypairPath,
      showConfig: false,
    });

    await expect(
      namespaceCommand.handler(["set-price", "^", "1000"], { project: root, manager: "mgr111" }, createContext() as any),
    ).rejects.toThrow("symbol must be one of ! @ # $ %");
  });

  it("rejects non-integer lamports for set-price", async () => {
    const { root, keypairPath } = await createTempProject();
    applyOverrides.mockResolvedValue({
      target: "devnet",
      networks: { devnet: { rpcUrl: "http://127.0.0.1:8899" } },
      keypairPath,
      showConfig: false,
    });

    await expect(
      namespaceCommand.handler(["set-price", "$", "10.5"], { project: root, manager: "mgr111" }, createContext() as any),
    ).rejects.toThrow("set-price requires a positive integer lamports value");
  });

  it("requires manager for price actions", async () => {
    const { root, keypairPath } = await createTempProject();
    applyOverrides.mockResolvedValue({
      target: "devnet",
      networks: { devnet: { rpcUrl: "http://127.0.0.1:8899" } },
      keypairPath,
      showConfig: false,
    });

    await expect(
      namespaceCommand.handler(["get-price", "$"], { project: root }, createContext() as any),
    ).rejects.toThrow("--manager <script-account> is required for namespace price actions");
  });

  it("calls SDK set price and persists manager metadata", async () => {
    const { root, keypairPath } = await createTempProject();
    applyOverrides.mockResolvedValue({
      target: "devnet",
      networks: { devnet: { rpcUrl: "http://127.0.0.1:8899" } },
      keypairPath,
      showConfig: false,
    });
    setNamespaceSymbolPriceOnChain.mockResolvedValue({
      transactionId: "tx-set-price",
      symbol: "$",
      priceLamports: 10_000_000_000,
    });

    await namespaceCommand.handler(
      ["set-price", "$", "10000000000"],
      { project: root, manager: "mgr111" },
      createContext() as any,
    );

    expect(setNamespaceSymbolPriceOnChain).toHaveBeenCalledWith(
      "$",
      10_000_000_000,
      expect.objectContaining({
        managerScriptAccount: "mgr111",
      }),
    );

    const lock = await readFile(join(root, "five.lock"), "utf8");
    expect(lock).toContain("namespace_manager");
    expect(lock).toContain("mgr111");
  });

  it("calls SDK get price", async () => {
    const { root, keypairPath } = await createTempProject();
    applyOverrides.mockResolvedValue({
      target: "devnet",
      networks: { devnet: { rpcUrl: "http://127.0.0.1:8899" } },
      keypairPath,
      showConfig: false,
    });
    getNamespaceSymbolPriceOnChain.mockResolvedValue({
      transactionId: "tx-get-price",
      symbol: "$",
      priceLamports: 10_000_000_000,
      priceSol: 10,
    });

    await namespaceCommand.handler(
      ["get-price", "$"],
      { project: root, manager: "mgr111" },
      createContext() as any,
    );

    expect(getNamespaceSymbolPriceOnChain).toHaveBeenCalledWith(
      "$",
      expect.objectContaining({
        managerScriptAccount: "mgr111",
      }),
    );
  });
});
