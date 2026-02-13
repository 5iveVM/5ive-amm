"use client";

import { GlassCard } from "@/components/ui/glass-card";
import { ThemeToggle } from "@/components/ui/ThemeToggle";
import { ArrowRight, Book, Code, Cpu, GitBranch, Link2, Shield, Terminal, Wrench } from "lucide-react";
import DocsEditor from "@/components/editor/DocsEditor";
import { CodeBlock } from "@/components/ui/code-block";
import { useState } from "react";
import Link from "next/link";

const DATA_TYPES = [
    {
        type: "u8..u128",
        desc: "Unsigned integers for counters, balances, and supply math.",
        code: `pub bump_counter(value: u8) -> u8 {
    return value + 1;
}

pub add_supply(a: u128, b: u128) -> u128 {
    return a + b;
}`,
    },
    {
        type: "i8..i64",
        desc: "Signed integers for deltas and directional arithmetic.",
        code: `pub apply_delta(balance: i64, delta: i64) -> i64 {
    return balance + delta;
}`,
    },
    {
        type: "bool",
        desc: "Feature flags, guards, and branch control.",
        code: `pub can_execute(is_paused: bool, has_auth: bool) -> bool {
    return !is_paused && has_auth;
}`,
    },
    {
        type: "pubkey",
        desc: "Authorities, ownership, and account identity.",
        code: `account Config {
    authority: pubkey;
}`,
    },
    {
        type: "string<N>",
        desc: "Sized strings for account-safe text fields.",
        code: `account Metadata {
    name: string<32>;
    symbol: string<16>;
    uri: string<128>;
}`,
    },
    {
        type: "[T; N]",
        desc: "Fixed-size arrays for structured collections.",
        code: `account GuardSet {
    guardians: [pubkey; 5];
}`,
    },
    {
        type: "Optional fields",
        desc: "Optional account fields via ? syntax.",
        code: `account Profile {
    authority: pubkey;
    nickname?: string<32>;
}`,
    },
];

const LANGUAGE_PATTERNS = [
    {
        name: "Account Params + Constraints",
        desc: "Patterns used heavily in templates and runtime harness scripts.",
        code: `pub settle(
    source: account @mut,
    destination: account @mut,
    owner: account @signer,
    amount: u64
) {
    require(amount > 0);
}`,
    },
    {
        name: "Control Flow",
        desc: "If/while flows are used in production templates (AMM/bench patterns).",
        code: `pub accumulate(limit: u64) -> u64 {
    let mut i: u64 = 0;
    let mut total: u64 = 0;
    while (i < limit) {
        total = total + i;
        i = i + 1;
    }
    return total;
}`,
    },
    {
        name: "Option + Result Types",
        desc: "Generic return types are supported for expressive APIs.",
        code: `interface TypeShapes @program("11111111111111111111111111111111") {
    maybe_balance @discriminator(1)(found: bool, balance: u64) -> Option<u64>;
    validate_amount @discriminator(2)(amount: u64) -> Result<bool, string>;
}`,
    },
];

const QUICK_START_SNIPPET = `account Counter {
    value: u64;
    authority: pubkey;
}

pub init_counter(counter: Counter @mut @init, authority: account @signer) {
    counter.value = 0;
    counter.authority = authority.key;
}

pub increment(counter: Counter @mut, authority: account @signer) {
    require(counter.authority == authority.key);
    counter.value = counter.value + 1;
}`;

const SDK_INSTALL_SNIPPET = `npm install @5ive-tech/sdk @solana/web3.js`;

const SDK_INTERACTION_SNIPPET = `import { readFile } from "node:fs/promises";
import { Connection, PublicKey, Transaction, TransactionInstruction, sendAndConfirmTransaction } from "@solana/web3.js";
import { FiveSDK, FiveProgram } from "@5ive-tech/sdk";

const connection = new Connection(process.env.RPC_URL!, "confirmed");
const fiveFileText = await readFile("./build/counter.five", "utf8");
const { abi } = await FiveSDK.loadFiveFile(fiveFileText);

const program = FiveProgram.fromABI(process.env.SCRIPT_ACCOUNT!, abi, {
  fiveVMProgramId: process.env.FIVE_VM_PROGRAM_ID!,
  vmStateAccount: process.env.FIVE_VM_STATE!,
  feeReceiverAccount: process.env.FIVE_FEE_RECEIVER!,
});

const serializedIx = await program
  .function("transfer")
  .accounts({
    from: fromTokenAccount,
    to: toTokenAccount,
    authority: wallet.publicKey,
  })
  .args({ amount: 100 })
  .instruction();

const ix = new TransactionInstruction({
  programId: new PublicKey(serializedIx.programId),
  keys: serializedIx.keys.map((key) => ({
    pubkey: new PublicKey(key.pubkey),
    isSigner: key.isSigner,
    isWritable: key.isWritable,
  })),
  data: Buffer.from(serializedIx.data, "base64"),
});

const tx = new Transaction().add(ix);
await sendAndConfirmTransaction(connection, tx, [wallet], { skipPreflight: false });`;

const CLI_INSTALL_SNIPPET = `npm install -g @5ive-tech/cli`;

const CLI_WORKFLOW_SNIPPET = `5ive init my-app
cd my-app
5ive build
5ive test
5ive deploy ./build/my-app.five --target devnet`;



// Verified syntax: external function import from a 5IVE bytecode account.
const EXTERNAL_IMPORT_SNIPPET = `use "11111111111111111111111111111111"::{transfer};

pub settle(
    from: account @mut,
    to: account @mut,
    owner: account @signer
) {
    transfer(from, to, owner, 50);
}`;



// Verified example: mirrors interface/CPI compiler tests.
const INTERFACE_CPI_SNIPPET = `interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    transfer @discriminator(3) (
        from: account,
        to: account,
        authority: account,
        amount: u64
    );
}

pub cpi_transfer(from: account @mut, to: account @mut, authority: account @signer) {
    SPLToken.transfer(from, to, authority, 50);
}`;

const ANCHOR_INTERFACE_SNIPPET = `@anchor
interface AnchorToken @program("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw") {
    mint_to(
        mint: Account,
        to: Account,
        authority: Account,
        amount: u64
    );
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer
) {
    AnchorToken.mint_to(mint, to, authority, 1000);
}`;

const SERIALIZER_INTERFACE_SNIPPET = `interface AnchorTokenComparison @program("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw") @serializer(borsh) {
    mint_to @discriminator([0xF1, 0x22, 0x30, 0xBA, 0x25, 0xB3, 0x7B, 0xC0]) (
        mint: Account,
        destination: Account,
        authority: Account,
        amount: u64
    );
}

pub mint_manual(
    mint: account @mut,
    dest: account @mut,
    auth: account @signer
) {
    AnchorTokenComparison.mint_to(mint, dest, auth, 1000);
}`;

const SECURITY_SNIPPET = `account Vault {
    authority: pubkey;
    total_assets: u64;
}

pub withdraw(vault: Vault @mut, authority: account @signer, amount: u64) {
    require(vault.authority == authority.key);
    require(amount > 0);
    require(vault.total_assets > amount || vault.total_assets == amount);
    vault.total_assets = vault.total_assets - amount;
}`;

function DataTypesSection() {
    const [active, setActive] = useState(DATA_TYPES[0]);

    return (
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_1.4fr] gap-6">
            <div className="space-y-2">
                {DATA_TYPES.map((item) => (
                    <button
                        key={item.type}
                        onClick={() => setActive(item)}
                        className={`w-full text-left p-3 rounded-lg border transition-all ${active.type === item.type
                            ? "bg-rose-pine-surface border-rose-pine-iris/50"
                            : "bg-white/5 border-white/5 hover:bg-white/10"
                            }`}
                    >
                        <code className="font-bold text-rose-pine-iris text-sm">{item.type}</code>
                        <p className="text-xs text-rose-pine-muted mt-1">{item.desc}</p>
                    </button>
                ))}
            </div>
            <DocsEditor filename={`${active.type}.v`} code={active.code} height="260px" />
        </div>
    );
}

function LanguagePatternsSection() {
    const [active, setActive] = useState(LANGUAGE_PATTERNS[0]);

    return (
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_1.4fr] gap-6">
            <div className="space-y-2">
                {LANGUAGE_PATTERNS.map((item) => (
                    <button
                        key={item.name}
                        onClick={() => setActive(item)}
                        className={`w-full text-left p-3 rounded-lg border transition-all ${active.name === item.name
                            ? "bg-rose-pine-surface border-rose-pine-iris/50"
                            : "bg-white/5 border-white/5 hover:bg-white/10"
                            }`}
                    >
                        <span className="font-bold text-rose-pine-iris text-sm">{item.name}</span>
                        <p className="text-xs text-rose-pine-muted mt-1">{item.desc}</p>
                    </button>
                ))}
            </div>
            <DocsEditor filename={`pattern_${active.name.toLowerCase().replace(/\s+/g, "_")}.v`} code={active.code} height="260px" />
        </div>
    );
}

function PathDecisionSection() {
    return (
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <GlassCard className="p-5 border-l-4 border-l-rose-pine-iris">
                <h4 className="font-semibold text-rose-pine-text">External bytecode call (non-CPI)</h4>
                <p className="text-sm text-rose-pine-subtle mt-2">
                    Use address imports and call imported public functions directly through 5IVE&apos;s external composition path (non-CPI).
                </p>
            </GlassCard>
            <GlassCard className="p-5 border-l-4 border-l-rose-pine-foam">
                <h4 className="font-semibold text-rose-pine-text">Interface CPI call</h4>
                <p className="text-sm text-rose-pine-subtle mt-2">
                    Use interface definitions with <code className="text-rose-pine-iris">@program</code> when invoking
                    non-5IVE Solana programs through CPI semantics.
                </p>
            </GlassCard>
        </div>
    );
}

export default function DocsPage() {
    return (
        <div className="min-h-screen bg-rose-pine-base text-rose-pine-text font-sans selection:bg-rose-pine-love/30 flex flex-col">
            <header className="fixed top-6 left-1/2 transform -translate-x-1/2 z-50 flex items-center justify-between px-6 py-3 rounded-full border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.12)] w-[90%] max-w-5xl">
                <div className="flex items-center gap-4">
                    <Link href="/" className="font-black text-xl tracking-tighter bg-gradient-to-b from-white via-[#c4a7e7] to-[#eb6f92] bg-clip-text text-transparent">
                        5IVE
                    </Link>
                    <span className="hidden sm:inline-block px-2 py-0.5 rounded-full bg-rose-pine-surface border border-rose-pine-hl-low text-[10px] font-bold uppercase tracking-wider text-rose-pine-subtle">
                        DOCS
                    </span>
                </div>

                <nav className="hidden md:flex items-center gap-8 text-sm font-medium text-rose-pine-muted">
                    <Link href="/" className="hover:text-rose-pine-text transition-colors">Home</Link>
                    <a href="#quick-start" className="hover:text-rose-pine-text transition-colors">Quick Start</a>
                    <a href="/ide" className="hover:text-rose-pine-text transition-colors">IDE</a>
                </nav>

                <div className="flex items-center gap-4">
                    <ThemeToggle />
                    <a href="https://github.com/five-org" target="_blank" className="text-rose-pine-muted hover:text-white transition-colors" rel="noreferrer">
                        <span className="sr-only">GitHub</span>
                        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
                        </svg>
                    </a>
                </div>
            </header>

            <main className="flex-1 pt-24 pb-20 px-6 max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-[260px_1fr] gap-10">
                <aside className="hidden lg:block sticky top-24 h-[calc(100vh-8rem)] overflow-y-auto">
                    <nav className="space-y-6">
                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">Overview</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#dsl" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Book className="w-3 h-3" /> Introduction</a>
                                <a href="#quick-start" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Terminal className="w-3 h-3" /> Quick Start</a>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">DSL Guide</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#language-essentials" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Code className="w-3 h-3" /> Essentials</a>
                                <a href="#imports-external" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Link2 className="w-3 h-3" /> Imports + External Calls</a>
                                <a href="#interfaces-cpi" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><GitBranch className="w-3 h-3" /> Interfaces + CPI</a>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">Runtime</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#security-model" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Shield className="w-3 h-3" /> Security Model</a>
                                <a href="#execution-model" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Cpu className="w-3 h-3" /> Execution + Cost</a>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">Toolchain</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#sdk" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Code className="w-3 h-3" /> 5ive SDK</a>
                                <a href="#cli" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1"><Terminal className="w-3 h-3" /> 5ive CLI</a>
                            </div>
                        </div>
                    </nav>
                </aside>

                <div className="space-y-16">
                    <section id="dsl" className="space-y-6">
                        <div className="space-y-2">
                            <h1 className="text-4xl font-black tracking-tighter text-rose-pine-text">5IVE DSL</h1>
                            <p className="text-xl text-rose-pine-muted font-light">Tear down the mainnet wall. Build the moat.</p>
                        </div>
                        <GlassCard className="p-6 border-l-4 border-l-rose-pine-iris space-y-3">
                            <p className="text-rose-pine-text/90 leading-relaxed">
                                5IVE is a DSL and VM toolchain for secure Solana applications. Contracts can compose with other 5IVE bytecode accounts through
                                a native external-call path without CPI, while still supporting interface-based CPI for non-5IVE programs.
                            </p>
                            <p className="text-sm text-rose-pine-subtle">
                                Product thesis: make mainnet deployable apps economically accessible, then compound defensibility through a template moat and app-store-style distribution surface.
                            </p>
                            <PathDecisionSection />
                        </GlassCard>
                    </section>



                    <section id="quick-start" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-foam/10 text-rose-pine-foam"><Terminal className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Quick Start</h2>
                        </div>
                        <p className="text-rose-pine-text">Start with an account schema, an initializer, and explicit authority checks.</p>
                        <DocsEditor filename="quick_start_counter.v" code={QUICK_START_SNIPPET} height="360px" />
                    </section>

                    <section id="language-essentials" className="space-y-8">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-rose/10 text-rose-pine-rose"><Code className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Language Essentials</h2>
                        </div>
                        <p className="text-sm text-rose-pine-subtle">
                            These examples reflect real usage across 5IVE templates and BPF-CU runtime harness scripts.
                        </p>

                        <div className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Data Types</h3>
                            <DataTypesSection />
                        </div>

                        <div className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Core Patterns</h3>
                            <LanguagePatternsSection />
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <GlassCard className="p-5">
                                <h4 className="font-semibold text-rose-pine-text">Accounts</h4>
                                <p className="text-sm text-rose-pine-subtle mt-2">Use <code className="text-rose-pine-iris">account</code> blocks to define state layout and runtime constraints.</p>
                            </GlassCard>
                            <GlassCard className="p-5">
                                <h4 className="font-semibold text-rose-pine-text">Instructions</h4>
                                <p className="text-sm text-rose-pine-subtle mt-2">Use <code className="text-rose-pine-iris">pub</code> instructions for entrypoints and module helpers for shared logic.</p>
                            </GlassCard>
                            <GlassCard className="p-5">
                                <h4 className="font-semibold text-rose-pine-text">Constraints</h4>
                                <p className="text-sm text-rose-pine-subtle mt-2"><code className="text-rose-pine-iris">@signer</code>, <code className="text-rose-pine-iris">@mut</code>, and <code className="text-rose-pine-iris">@init</code> enforce account behavior and initialization safety.</p>
                            </GlassCard>
                        </div>
                    </section>

                    <section id="imports-external" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-iris/10 text-rose-pine-iris"><Link2 className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Imports + External Bytecode Calls</h2>
                        </div>

                        <GlassCard className="p-6 space-y-4">
                            <p className="text-sm text-rose-pine-subtle">
                                Use <code className="text-rose-pine-iris">use</code> with a deployed 5IVE bytecode account address. Imported functions can be called unqualified through 5IVE&apos;s non-CPI external-call path.
                                For non-5IVE programs like SPL Token, use the interface/CPI path below.
                            </p>
                            <DocsEditor filename="external_import_non_cpi.v" code={EXTERNAL_IMPORT_SNIPPET} height="280px" />
                        </GlassCard>

                        <GlassCard className="p-6 space-y-4 border-l-4 border-l-rose-pine-gold">
                            <h4 className="text-sm font-semibold text-rose-pine-gold">Import verification</h4>
                            <p className="text-sm text-rose-pine-subtle">
                                5IVE embeds import verification metadata at compile time and validates account identity at runtime before external execution. Unauthorized bytecode substitution is rejected.
                            </p>
                            <div className="bg-rose-pine-base/50 rounded-lg p-4 text-xs text-rose-pine-subtle space-y-1">
                                <p>Compile time: import metadata stored in script bytecode.</p>
                                <p>Runtime: account address checked before external call dispatch.</p>
                                <p>Failure mode: unauthorized substitution returns runtime error.</p>
                            </div>
                        </GlassCard>


                    </section>

                    <section id="interfaces-cpi" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-love/10 text-rose-pine-love"><GitBranch className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Interfaces + CPI</h2>
                        </div>

                        <GlassCard className="p-6 space-y-4">
                            <h4 className="font-semibold text-rose-pine-text">Standard Interface</h4>
                            <p className="text-sm text-rose-pine-subtle">
                                Interfaces declare non-5IVE program methods and discriminators. Use this path when true Solana CPI is required.
                            </p>
                            <DocsEditor filename="interface_cpi.v" code={INTERFACE_CPI_SNIPPET} height="320px" />
                        </GlassCard>

                        <GlassCard className="p-6 space-y-4 border-l-4 border-l-rose-pine-love">
                            <h4 className="font-semibold text-rose-pine-love">Anchor Interface</h4>
                            <p className="text-sm text-rose-pine-subtle">
                                Use <code className="text-rose-pine-iris">@anchor</code> to automatically handle 8-byte discriminators and
                                <code className="text-rose-pine-iris">Account</code> type mappings for Anchor programs.
                            </p>
                            <DocsEditor filename="anchor_interface.v" code={ANCHOR_INTERFACE_SNIPPET} height="320px" />
                        </GlassCard>

                        <GlassCard className="p-6 space-y-4 border-l-4 border-l-rose-pine-gold">
                            <h4 className="font-semibold text-rose-pine-gold">Custom Serializers</h4>
                            <p className="text-sm text-rose-pine-subtle">
                                Use <code className="text-rose-pine-iris">@serializer(borsh)</code> or <code className="text-rose-pine-iris">@serializer(bincode)</code>
                                to control parameter encoding. You can also specify explicit discriminator bytes.
                            </p>
                            <DocsEditor filename="serializer_interface.v" code={SERIALIZER_INTERFACE_SNIPPET} height="320px" />
                        </GlassCard>
                    </section>

                    <section id="security-model" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-gold/10 text-rose-pine-gold"><Shield className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Security Model</h2>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                            <GlassCard className="p-6">
                                <h4 className="font-semibold text-rose-pine-text">Constraint checks</h4>
                                <p className="text-sm text-rose-pine-subtle mt-2">Use signer and mutability constraints on instruction parameters. Gate privileged actions with explicit authority checks.</p>
                                <div className="mt-4">
                                    <DocsEditor filename="security_constraints.v" code={SECURITY_SNIPPET} height="250px" />
                                </div>
                            </GlassCard>
                            <GlassCard className="p-6">
                                <h4 className="font-semibold text-rose-pine-text">External call security</h4>
                                <ul className="text-sm text-rose-pine-subtle mt-2 space-y-2">
                                    <li>Imported external calls are resolved by the VM using import metadata.</li>
                                    <li>Constraint enforcement still applies when entering external bytecode functions.</li>
                                    <li>Prefer explicit bytecode account parameters for predictable binding.</li>
                                </ul>
                            </GlassCard>
                        </div>
                    </section>

                    <section id="execution-model" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-foam/10 text-rose-pine-foam"><Cpu className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Execution + Cost Model</h2>
                        </div>

                        <GlassCard className="p-6 space-y-4 border-l-4 border-l-rose-pine-foam">
                            <p className="text-sm text-rose-pine-subtle">
                                External bytecode calls avoid CPI framing costs but are not free. They still pay for runtime validation, stack management, and account handling.
                            </p>
                            <p className="text-sm text-rose-pine-subtle">
                                Use CU measurements from the runtime harness to compare internal calls, external bytecode calls, and interface CPI for your exact workload.
                            </p>
                            <p className="text-sm text-rose-pine-subtle">
                                Long-term direction: package richer app surfaces and template libraries into compact on-chain footprints, including the 10MB account form factor target.
                            </p>
                            <a href="/ide" className="inline-flex items-center gap-2 text-rose-pine-foam font-semibold hover:gap-3 transition-all">
                                Try snippets in the IDE <ArrowRight size={16} />
                            </a>
                        </GlassCard>
                    </section>

                    <section id="sdk" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-iris/10 text-rose-pine-iris"><Code className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">5ive SDK</h2>
                        </div>
                        <GlassCard className="p-6 space-y-4">
                            <p className="text-sm text-rose-pine-subtle">
                                Use the SDK to load <code className="text-rose-pine-iris">.five</code> artifacts, build typed instruction payloads, and send transactions with explicit account wiring.
                            </p>
                            <div className="space-y-2">
                                <h3 className="text-sm font-semibold text-rose-pine-foam uppercase tracking-wider">Install</h3>
                                <CodeBlock
                                    code={SDK_INSTALL_SNIPPET}
                                    language="shell"
                                />
                            </div>
                            <div className="space-y-2">
                                <h3 className="text-sm font-semibold text-rose-pine-foam uppercase tracking-wider">Interaction Flow</h3>
                                <CodeBlock
                                    filename="sdk_interaction.ts"
                                    code={SDK_INTERACTION_SNIPPET}
                                    language="typescript"
                                />
                            </div>
                        </GlassCard>
                    </section>

                    <section id="cli" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-foam/10 text-rose-pine-foam"><Terminal className="w-6 h-6" /></div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">5ive CLI</h2>
                        </div>
                        <GlassCard className="p-6 space-y-4">
                            <p className="text-sm text-rose-pine-subtle">
                                Use the CLI for project scaffolding, build/test loops, and deployment. Keep the command surface focused on stable workflows.
                            </p>
                            <div className="space-y-2">
                                <h3 className="text-sm font-semibold text-rose-pine-foam uppercase tracking-wider">Install</h3>
                                <CodeBlock
                                    code={CLI_INSTALL_SNIPPET}
                                    language="shell"
                                />
                            </div>
                            <div className="space-y-2">
                                <h3 className="text-sm font-semibold text-rose-pine-foam uppercase tracking-wider">Core Workflow</h3>
                                <CodeBlock
                                    filename="cli_workflow.sh"
                                    code={CLI_WORKFLOW_SNIPPET}
                                    language="shell"
                                />
                            </div>
                        </GlassCard>
                    </section>
                </div>
            </main>

            <footer className="py-8 border-t border-rose-pine-hl-low/20 text-center text-sm text-rose-pine-muted relative z-10">
                <p>© 2026 5ive Tech. All rights reserved.</p>
            </footer>
        </div>
    );
}
