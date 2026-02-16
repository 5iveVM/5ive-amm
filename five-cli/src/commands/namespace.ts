// Namespace command.

import { readFile, writeFile } from "fs/promises";
import { join } from "path";
import { parse as parseToml, stringify as stringifyToml } from "@iarna/toml";
import { Connection, Keypair } from "@solana/web3.js";

import { CommandContext, CommandDefinition } from "../types.js";
import { ConfigManager } from "../config/ConfigManager.js";
import { VmClusterConfigResolver } from "../config/VmClusterConfigResolver.js";
import { loadProjectConfig } from "../project/ProjectLoader.js";
import { section, success as uiSuccess, error as uiError, keyValue } from "../utils/cli-ui.js";
import { FiveSDK, ProgramIdResolver } from "@5ive-tech/sdk";

type NamespaceLock = {
  version?: number;
  packages?: any[];
  namespaces?: Array<{ namespace: string; address: string; updated_at?: string }>;
  namespace_tlds?: Array<{ symbol: string; domain: string; owner: string; registered_at?: string }>;
  namespace_manager?: { script_account: string; treasury_account?: string; updated_at?: string };
};

const SYMBOLS = new Set(["!", "@", "#", "$", "%"]);

function canonicalizeNamespace(input: string): {
  symbol: string;
  domain: string;
  subprogram?: string;
  canonical: string;
} {
  const trimmed = input.trim();
  if (trimmed.length < 2) {
    throw new Error("namespace is too short");
  }
  const symbol = trimmed[0];
  if (!SYMBOLS.has(symbol)) {
    throw new Error("namespace symbol must be one of ! @ # $ %");
  }
  const path = trimmed.slice(1);
  const parts = path.split("/");
  if (parts.length < 1 || parts.length > 2) {
    throw new Error("namespace must be !domain or !domain/subprogram");
  }
  const valid = (value: string) =>
    value.length > 0 && /^[a-z0-9-]+$/i.test(value);
  if (!valid(parts[0])) {
    throw new Error("domain must be alphanumeric + hyphen");
  }
  if (parts[1] && !valid(parts[1])) {
    throw new Error("subprogram must be alphanumeric + hyphen");
  }
  const domain = parts[0].toLowerCase();
  const subprogram = parts[1]?.toLowerCase();
  const canonical = subprogram
    ? `${symbol}${domain}/${subprogram}`
    : `${symbol}${domain}`;
  return { symbol, domain, subprogram, canonical };
}

async function readLockfile(rootDir: string): Promise<NamespaceLock> {
  const path = join(rootDir, "five.lock");
  try {
    const content = await readFile(path, "utf8");
    return parseToml(content) as NamespaceLock;
  } catch {
    return { version: 1, packages: [], namespaces: [], namespace_tlds: [] };
  }
}

async function writeLockfile(rootDir: string, lock: NamespaceLock): Promise<void> {
  const path = join(rootDir, "five.lock");
  await writeFile(path, stringifyToml(lock), "utf8");
}

async function loadSignerKeypair(keypairPath: string): Promise<Keypair> {
  const path = keypairPath.startsWith("~/")
    ? keypairPath.replace("~", process.env.HOME || "")
    : keypairPath;
  const content = await readFile(path, "utf8");
  const secret = Uint8Array.from(JSON.parse(content));
  return Keypair.fromSecretKey(secret);
}

function resolveManagerScriptAccount(
  options: any,
  projectContext: Awaited<ReturnType<typeof loadProjectConfig>>,
  lock: NamespaceLock,
): string | undefined {
  return (
    options.manager ||
    projectContext?.config?.namespaceManager ||
    process.env.FIVE_NAMESPACE_MANAGER ||
    lock.namespace_manager?.script_account
  );
}

export const namespaceCommand: CommandDefinition = {
  name: "namespace",
  description: "Manage 5NS namespace registrations and bindings",
  aliases: ["ns"],
  options: [
    {
      flags: "--script <pubkey>",
      description: "Script account to bind to namespace",
      required: false,
    },
    {
      flags: "--owner <pubkey>",
      description: "Override owner for local registration checks",
      required: false,
    },
    {
      flags: "--project <path>",
      description: "Project directory (default: cwd)",
      required: false,
    },
    {
      flags: "--manager <script-account>",
      description: "Namespace manager script account address",
      required: false,
    },
    {
      flags: "--program-id <pubkey>",
      description: "Override 5IVE VM program ID for PDA derivation",
      required: false,
    },
    {
      flags: "--local",
      description: "Use lockfile-only mode (skip on-chain manager calls)",
      defaultValue: false,
    },
  ],
  arguments: [
    { name: "action", description: "register | bind | resolve", required: true },
    { name: "namespace", description: "@domain or @domain/subprogram", required: true },
  ],
  examples: [
    { command: "5ive namespace register @5ive-tech", description: "Register top-level namespace in local cache" },
    { command: "5ive namespace bind @5ive-tech/program --script <pubkey>", description: "Bind namespace to script account" },
    { command: "5ive namespace resolve @5ive-tech/program", description: "Resolve namespace from lockfile cache" },
  ],
  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const action = (args[0] || "").toLowerCase();
    const nsInput = args[1];
    if (!action || !nsInput) {
      throw new Error("usage: five namespace <register|bind|resolve> <namespace>");
    }
    const projectContext = await loadProjectConfig(options.project, process.cwd());
    const rootDir = projectContext?.rootDir || options.project || process.cwd();
    const lock = await readLockfile(rootDir);
    lock.namespaces ||= [];
    lock.namespace_tlds ||= [];
    const useLocalOnly = Boolean(options.local);
    const managerScriptAccount = resolveManagerScriptAccount(options, projectContext, lock);

    const configManager = ConfigManager.getInstance();
    const config = await configManager.applyOverrides({
      target: projectContext?.config.cluster as any,
      network: projectContext?.config.rpcUrl,
      keypair: projectContext?.config.keypairPath,
    });

    // Resolve program ID with precedence: CLI flag → project config → config file (per-target) → vm constants
    const configuredProgramId = await configManager.getProgramId(config.target as any);
    const vmProgramId = options.programId || projectContext?.config.programId || configuredProgramId || VmClusterConfigResolver.loadClusterConfig({
      cluster: VmClusterConfigResolver.fromCliTarget(config.target as any),
    }).programId;
    let signer: Keypair | undefined;
    let connection: Connection | undefined;

    const ensureSigner = async (): Promise<Keypair> => {
      if (!signer) {
        signer = await loadSignerKeypair(config.keypairPath);
      }
      return signer;
    };

    const ensureConnection = (): Connection => {
      if (!connection) {
        const rpcUrl = config.networks[config.target].rpcUrl;
        connection = new Connection(rpcUrl, "confirmed");
      }
      return connection;
    };

    const signerOwner = (await ensureSigner()).publicKey.toBase58();
    const owner = useLocalOnly ? (options.owner || signerOwner) : signerOwner;

    if (action === "register") {
      const parsed = canonicalizeNamespace(nsInput);
      if (parsed.subprogram) {
        throw new Error("register expects top-level namespace like @domain");
      }
      if (useLocalOnly || !managerScriptAccount) {
        const localOwner = options.owner || signerOwner;
        const existing = lock.namespace_tlds.find(
          (entry) => entry.symbol === parsed.symbol && entry.domain === parsed.domain,
        );
        if (existing && existing.owner !== localOwner) {
          throw new Error(`namespace ${parsed.canonical} already registered to ${existing.owner}`);
        }
        if (!existing) {
          lock.namespace_tlds.push({
            symbol: parsed.symbol,
            domain: parsed.domain,
            owner: localOwner,
            registered_at: new Date().toISOString(),
          });
        }
        await writeLockfile(rootDir, lock);
        console.log(uiSuccess(`Registered ${parsed.canonical} (local lockfile)`));
        console.log(keyValue("Owner", localOwner));
        if (!managerScriptAccount && !useLocalOnly) {
          console.log(uiError("No namespace manager configured; used local fallback."));
        }
        return;
      }

      const onChain = await FiveSDK.registerNamespaceTldOnChain(parsed.canonical, {
        managerScriptAccount,
        connection: ensureConnection(),
        signerKeypair: await ensureSigner(),
        fiveVMProgramId: vmProgramId,
        debug: context.options.debug,
      });

      const existing = lock.namespace_tlds.find(
        (entry) => entry.symbol === parsed.symbol && entry.domain === parsed.domain,
      );
      if (existing) {
        existing.owner = owner;
        existing.registered_at = existing.registered_at || new Date().toISOString();
      } else {
        lock.namespace_tlds.push({
          symbol: parsed.symbol,
          domain: parsed.domain,
          owner,
          registered_at: new Date().toISOString(),
        });
      }
      lock.namespace_manager = {
        script_account: managerScriptAccount,
        treasury_account: onChain.treasuryAccount,
        updated_at: new Date().toISOString(),
      };
      await writeLockfile(rootDir, lock);

      console.log(uiSuccess(`Registered ${parsed.canonical} on-chain`));
      console.log(keyValue("Owner", owner));
      console.log(keyValue("TLD Account", onChain.tldAddress));
      if (onChain.transactionId) {
        console.log(keyValue("Transaction", onChain.transactionId));
      }
      return;
    }

    if (action === "bind") {
      const parsed = canonicalizeNamespace(nsInput);
      if (!parsed.subprogram) {
        throw new Error("bind expects namespace with subprogram like @domain/subprogram");
      }
      const script = options.script;
      if (!script) {
        throw new Error("--script <pubkey> is required for bind");
      }
      const tld = lock.namespace_tlds.find(
        (entry) => entry.symbol === parsed.symbol && entry.domain === parsed.domain,
      );
      if (tld && tld.owner !== owner) {
        throw new Error(`only namespace owner can bind subprograms (owner: ${tld.owner})`);
      }

      if (!useLocalOnly && managerScriptAccount) {
        const onChain = await FiveSDK.bindNamespaceOnChain(parsed.canonical, script, {
          managerScriptAccount,
          connection: ensureConnection(),
          signerKeypair: await ensureSigner(),
          fiveVMProgramId: vmProgramId,
          debug: context.options.debug,
        });
        lock.namespace_manager = {
          script_account: managerScriptAccount,
          treasury_account: lock.namespace_manager?.treasury_account,
          updated_at: new Date().toISOString(),
        };
        console.log(uiSuccess(`Bound ${parsed.canonical} on-chain`));
        console.log(keyValue("Binding Account", onChain.bindingAddress));
        if (onChain.transactionId) {
          console.log(keyValue("Transaction", onChain.transactionId));
        }
      } else if (!managerScriptAccount && !useLocalOnly) {
        console.log(uiError("No namespace manager configured; used local fallback."));
      }

      const idx = lock.namespaces.findIndex((entry) => entry.namespace === parsed.canonical);
      const value = {
        namespace: parsed.canonical,
        address: script,
        updated_at: new Date().toISOString(),
      };
      if (idx >= 0) {
        lock.namespaces[idx] = value;
      } else {
        lock.namespaces.push(value);
      }
      await writeLockfile(rootDir, lock);
      console.log(uiSuccess(`Bound ${parsed.canonical}`));
      console.log(keyValue("Script", script));
      return;
    }

    if (action === "resolve") {
      const parsed = canonicalizeNamespace(nsInput);
      if (!parsed.subprogram) {
        throw new Error("resolve expects namespace with subprogram like @domain/subprogram");
      }
      if (!useLocalOnly && managerScriptAccount) {
        try {
          const onChain = await FiveSDK.resolveNamespaceOnChain(parsed.canonical, {
            managerScriptAccount,
            connection: ensureConnection(),
            signerKeypair: await ensureSigner(),
            fiveVMProgramId: vmProgramId,
            debug: context.options.debug,
          });
          if (onChain.resolvedScript) {
            const idx = lock.namespaces.findIndex((entry) => entry.namespace === parsed.canonical);
            const value = {
              namespace: parsed.canonical,
              address: onChain.resolvedScript,
              updated_at: new Date().toISOString(),
            };
            if (idx >= 0) lock.namespaces[idx] = value;
            else lock.namespaces.push(value);
            lock.namespace_manager = {
              script_account: managerScriptAccount,
              treasury_account: lock.namespace_manager?.treasury_account,
              updated_at: new Date().toISOString(),
            };
            await writeLockfile(rootDir, lock);

            console.log(section("Namespace Resolution"));
            console.log(keyValue("Namespace", parsed.canonical));
            console.log(keyValue("Script", onChain.resolvedScript));
            console.log(keyValue("Binding Account", onChain.bindingAddress));
            if (onChain.transactionId) {
              console.log(keyValue("Transaction", onChain.transactionId));
            }
            return;
          }
        } catch (e) {
          if (context.options.debug) {
            console.log(uiError(`On-chain resolve failed: ${e instanceof Error ? e.message : String(e)}`));
          }
        }
      }

      const match = lock.namespaces.find((entry) => entry.namespace === parsed.canonical);
      if (!match) {
        console.log(uiError(`No local binding for ${parsed.canonical}`));
        process.exitCode = 1;
        return;
      }
      console.log(section("Namespace Resolution"));
      console.log(keyValue("Namespace", parsed.canonical));
      console.log(keyValue("Script", match.address));
      return;
    }

    throw new Error("action must be one of: register, bind, resolve");
  },
};
