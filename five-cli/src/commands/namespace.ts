// Namespace command.

import { readFile, writeFile } from "fs/promises";
import { join } from "path";
import { parse as parseToml, stringify as stringifyToml } from "@iarna/toml";
import { Keypair } from "@solana/web3.js";

import { CommandContext, CommandDefinition } from "../types.js";
import { ConfigManager } from "../config/ConfigManager.js";
import { section, success as uiSuccess, error as uiError, keyValue } from "../utils/cli-ui.js";

type NamespaceLock = {
  version?: number;
  packages?: any[];
  namespaces?: Array<{ namespace: string; address: string; updated_at?: string }>;
  namespace_tlds?: Array<{ symbol: string; domain: string; owner: string; registered_at?: string }>;
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

async function inferOwnerFromConfig(context: CommandContext): Promise<string> {
  const config = await ConfigManager.getInstance().get();
  const keypairPath = config.keypairPath;
  if (!keypairPath) {
    throw new Error("no configured keypair path");
  }
  const path = keypairPath.startsWith("~/")
    ? keypairPath.replace("~", process.env.HOME || "")
    : keypairPath;
  const content = await readFile(path, "utf8");
  const secret = Uint8Array.from(JSON.parse(content));
  return Keypair.fromSecretKey(secret).publicKey.toBase58();
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
  ],
  arguments: [
    { name: "action", description: "register | bind | resolve", required: true },
    { name: "namespace", description: "@domain or @domain/subprogram", required: true },
  ],
  examples: [
    { command: "five namespace register @5ive-tech", description: "Register top-level namespace in local cache" },
    { command: "five namespace bind @5ive-tech/program --script <pubkey>", description: "Bind namespace to script account" },
    { command: "five namespace resolve @5ive-tech/program", description: "Resolve namespace from lockfile cache" },
  ],
  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const action = (args[0] || "").toLowerCase();
    const nsInput = args[1];
    if (!action || !nsInput) {
      throw new Error("usage: five namespace <register|bind|resolve> <namespace>");
    }
    const rootDir = options.project || process.cwd();
    const lock = await readLockfile(rootDir);
    lock.namespaces ||= [];
    lock.namespace_tlds ||= [];

    if (action === "register") {
      const parsed = canonicalizeNamespace(nsInput);
      if (parsed.subprogram) {
        throw new Error("register expects top-level namespace like @domain");
      }
      const owner = options.owner || (await inferOwnerFromConfig(context));
      const existing = lock.namespace_tlds.find(
        (entry) => entry.symbol === parsed.symbol && entry.domain === parsed.domain,
      );
      if (existing && existing.owner !== owner) {
        throw new Error(`namespace ${parsed.canonical} already registered to ${existing.owner}`);
      }
      if (!existing) {
        lock.namespace_tlds.push({
          symbol: parsed.symbol,
          domain: parsed.domain,
          owner,
          registered_at: new Date().toISOString(),
        });
      }
      await writeLockfile(rootDir, lock);
      console.log(uiSuccess(`Registered ${parsed.canonical} (local lockfile)`));
      console.log(keyValue("Owner", owner));
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
      const owner = options.owner || (await inferOwnerFromConfig(context));
      const tld = lock.namespace_tlds.find(
        (entry) => entry.symbol === parsed.symbol && entry.domain === parsed.domain,
      );
      if (!tld) {
        throw new Error(`top-level namespace ${parsed.symbol}${parsed.domain} is not registered`);
      }
      if (tld.owner !== owner) {
        throw new Error(`only namespace owner can bind subprograms (owner: ${tld.owner})`);
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

