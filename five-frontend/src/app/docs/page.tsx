"use client";

import { GlassCard } from "@/components/ui/glass-card";
import { ThemeToggle } from "@/components/ui/ThemeToggle";
import { Book, Code, Shield, Terminal, ArrowRight, ChevronRight, FileText } from "lucide-react";
import DocsEditor from "@/components/editor/DocsEditor";
import { useState } from "react";
import { cn } from "@/lib/utils";

const DATA_TYPES = [
    { type: "u8", desc: "Unsigned 8-bit integer (0 to 255)", code: `pub run() -> u8 {\n    let val: u8 = 255;\n    // val = val + 1; // Overflow!\n    return val;\n}` },
    { type: "u64", desc: "Unsigned 64-bit integer. Standard for amounts.", code: `pub run() -> u64 {\n    let balance: u64 = 1_000_000;\n    let supply: u64 = 500_000;\n    let total = balance + supply;\n    return total;\n}` },
    { type: "u128", desc: "Unsigned 128-bit integer. For high precision.", code: `pub run() -> u128 {\n    let big_num: u128 = 340282366920938463463374607431768211455;\n    return big_num;\n}` },
    { type: "bool", desc: "Boolean value (true or false).", code: `pub run() -> bool {\n    let is_active: bool = true;\n    if (is_active) {\n        return true;\n    }\n    return false;\n}` },
    { type: "string", desc: "UTF-8 encoded string.", code: `pub run() -> string {\n    let message: string = "Hello 5IVE";\n    return message;\n}` },
    { type: "pubkey", desc: "Solana public key (32 bytes).", code: `pub run() -> pubkey {\n    // User's wallet address\n    let owner: pubkey = Pubkey::new_unique();\n    return owner;\n}` },
    { type: "[T; N]", desc: "Fixed-size array.", code: `pub run() -> u8 {\n    let data: [u8; 4] = [1, 2, 3, 4];\n    let first = data[0];\n    return first;\n}` },
];

function DataTypesSection() {
    const [active, setActive] = useState(DATA_TYPES[1]); // Default u64

    return (
        <div className="grid grid-cols-1 lg:grid-cols-[1fr_1.5fr] gap-6">
            <div className="space-y-2">
                {DATA_TYPES.map((item) => (
                    <button
                        key={item.type}
                        onClick={() => setActive(item)}
                        className={cn(
                            "w-full text-left p-3 rounded-lg border transition-all flex flex-col gap-1",
                            active.type === item.type
                                ? "bg-rose-pine-surface border-rose-pine-iris/50 shadow-lg shadow-rose-pine-iris/10"
                                : "bg-white/5 border-white/5 hover:bg-white/10"
                        )}
                    >
                        <code className={cn(
                            "font-bold text-sm",
                            active.type === item.type ? "text-rose-pine-iris" : "text-rose-pine-love"
                        )}>{item.type}</code>
                        <p className="text-xs text-rose-pine-muted">{item.desc}</p>
                    </button>
                ))}
            </div>

            <div className="h-full min-h-[400px]">
                <DocsEditor
                    key={active.type}
                    filename="example.v"
                    code={active.code}
                    height="500px"
                />
            </div>
        </div>
    );
}

function AccountArchitectureSection() {
    const [activeTab, setActiveTab] = useState<'standard' | 'global'>('standard');

    return (
        <div className="space-y-6">
            <div className="flex bg-rose-pine-surface/30 border border-rose-pine-hl-low/50 p-1 rounded-lg w-fit">
                <button
                    onClick={() => setActiveTab('standard')}
                    className={cn(
                        "px-4 py-2 rounded-md text-sm font-medium transition-all",
                        activeTab === 'standard' ? "bg-rose-pine-surface text-rose-pine-text shadow-sm" : "text-rose-pine-subtle hover:text-rose-pine-text"
                    )}
                >
                    Standard Pattern
                </button>
                <button
                    onClick={() => setActiveTab('global')}
                    className={cn(
                        "px-4 py-2 rounded-md text-sm font-medium transition-all",
                        activeTab === 'global' ? "bg-rose-pine-surface text-rose-pine-text shadow-sm" : "text-rose-pine-subtle hover:text-rose-pine-text"
                    )}
                >
                    Global State Pattern
                </button>
            </div>

            {activeTab === 'standard' ? (
                <div className="space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
                    <GlassCard className="p-6 border-l-4 border-l-rose-pine-foam">
                        <h4 className="font-bold text-rose-pine-text mb-2">Program + State Accounts (Recommended)</h4>
                        <p className="text-sm text-rose-pine-text leading-relaxed mb-4">
                            In this pattern, the program logic is stateless. Users invoke the program to create and modify
                            separate state accounts. This allows one program to manage millions of user accounts.
                        </p>
                        <DocsEditor
                            filename="standard_pattern.v"
                            height="380px"
                            code={`// 1. Define the State Schema
account Counter {
    value: u64;
    owner: pubkey;
}

// 2. Initialize a NEW account
// @init creates the account. @signer pays for it.
pub init_counter(payer: account @signer, new_account: account @init, state: Counter @init) {
    state.value = 0;
    state.owner = payer.key;
}

// 3. Modify that specific account
pub increment(@mut counter: Counter) {
    counter.value = counter.value + 1;
}`}
                        />
                    </GlassCard>
                </div>
            ) : (
                <div className="space-y-4 animate-in fade-in slide-in-from-bottom-2 duration-300">
                    <GlassCard className="p-6 border-l-4 border-l-rose-pine-gold">
                        <h4 className="font-bold text-rose-pine-text mb-2">Global State Pattern (Singleton)</h4>
                        <p className="text-sm text-rose-pine-text leading-relaxed mb-4">
                            For simple protocols, the program itself can hold state. The variables are defined
                            at the top level, and the `init` block runs once upon deployment.
                        </p>
                        <DocsEditor
                            filename="global_pattern.v"
                            height="300px"
                            code={`// Global State Variables
mut admin: pubkey;
mut paused: bool;

// Implicit initialization block (runs on deploy)
init {
    // Set initial values
    paused = false;
    // admin = tx.signer; // Set to deployer
}

// Update global state
pub set_paused(status: bool) {
    // In real app, check if signer is admin
    paused = status;
}`}
                        />
                    </GlassCard>
                </div>
            )}
        </div>
    );
}



export default function DocsPage() {
    return (
        <div className="min-h-screen bg-rose-pine-base text-rose-pine-text font-sans selection:bg-rose-pine-love/30 flex flex-col">
            {/* Command Capsule Header (Docs Version) */}
            <header className="fixed top-6 left-1/2 transform -translate-x-1/2 z-50 flex items-center justify-between px-6 py-3 rounded-full border border-[var(--glass-border)] bg-[var(--glass-bg)] backdrop-blur-2xl shadow-[0_8px_32px_rgba(0,0,0,0.12)] w-[90%] max-w-5xl transition-all duration-500 hover:shadow-[0_8px_40px_rgba(0,0,0,0.2)] hover:border-white/10">
                <div className="flex items-center gap-4">
                    <a href="/" className="font-black text-xl tracking-tighter bg-gradient-to-b from-white via-[#c4a7e7] to-[#eb6f92] bg-clip-text text-transparent hover:opacity-80 transition-opacity">
                        5IVE
                    </a>
                    <span className="hidden sm:inline-block px-2 py-0.5 rounded-full bg-rose-pine-surface border border-rose-pine-hl-low text-[10px] font-bold uppercase tracking-wider text-rose-pine-subtle">
                        DOCS
                    </span>
                </div>

                <nav className="hidden md:flex items-center gap-8 text-sm font-medium text-rose-pine-muted">
                    <a href="/" className="hover:text-rose-pine-text transition-colors relative group">
                        Home
                        <span className="absolute -bottom-1 left-0 w-0 h-[1px] bg-rose-pine-rose transition-all group-hover:w-full" />
                    </a>
                    {/* Docs specific links or just reuse main ones if appropriate, simplified for now */}
                    <a href="#quick-start" className="hover:text-rose-pine-text transition-colors relative group">
                        Quick Start
                        <span className="absolute -bottom-1 left-0 w-0 h-[1px] bg-rose-pine-love transition-all group-hover:w-full" />
                    </a>
                    <a href="/ide" className="hover:text-rose-pine-text transition-colors relative group">
                        IDE
                        <span className="absolute -bottom-1 left-0 w-0 h-[1px] bg-rose-pine-gold transition-all group-hover:w-full" />
                    </a>
                </nav>

                <div className="flex items-center gap-4">
                    <ThemeToggle />
                    <a href="https://github.com/five-org" target="_blank" className="text-rose-pine-muted hover:text-white transition-colors">
                        <span className="sr-only">GitHub</span>
                        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                            <path fillRule="evenodd" d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z" clipRule="evenodd" />
                        </svg>
                    </a>
                </div>
            </header>

            <main className="flex-1 pt-24 pb-20 px-6 max-w-7xl mx-auto w-full grid grid-cols-1 lg:grid-cols-[240px_1fr] gap-10">
                {/* Sidebar Navigation */}
                <aside className="hidden lg:block sticky top-24 h-[calc(100vh-8rem)] overflow-y-auto">
                    <nav className="space-y-6">
                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">Getting Started</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#introduction" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1 transition-colors">
                                    <Book className="w-3 h-3" /> Introduction
                                </a>
                                <a href="#quick-start" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1 transition-colors">
                                    <Terminal className="w-3 h-3" /> Quick Start
                                </a>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">Language Reference</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#types" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1 transition-colors">
                                    <Code className="w-3 h-3" /> Data Types
                                </a>
                                <a href="#structures" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1 transition-colors">
                                    <FileText className="w-3 h-3" /> Structures
                                </a>
                                <a href="#instructions" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1 transition-colors">
                                    <ArrowRight className="w-3 h-3" /> Instructions
                                </a>
                            </div>
                        </div>

                        <div className="space-y-2">
                            <h4 className="text-xs uppercase tracking-widest text-rose-pine-muted font-bold">Security</h4>
                            <div className="flex flex-col space-y-1">
                                <a href="#security-rules" className="flex items-center gap-2 text-sm text-rose-pine-text/80 hover:text-rose-pine-text py-1 transition-colors">
                                    <Shield className="w-3 h-3" /> Security Rules
                                </a>
                            </div>
                        </div>
                    </nav>
                </aside>

                {/* Main Content */}
                <div className="space-y-16">
                    {/* Introduction */}
                    <section id="introduction" className="space-y-6">
                        <div className="space-y-2">
                            <h1 className="text-4xl font-black tracking-tighter text-rose-pine-text">5IVE DSL</h1>
                            <p className="text-xl text-rose-pine-muted font-light">
                                The safest way to build on Solana.
                            </p>
                        </div>
                        <GlassCard className="p-6 border-l-4 border-l-rose-pine-iris">
                            <p className="text-rose-pine-text/90 leading-relaxed">
                                5IVE is a domain-specific language designed for writing secure, predictable, and efficient smart contracts on the Solana blockchain.
                                It abstracts away low-level complexity while enforcing strict security boundaries at compile time.
                            </p>
                        </GlassCard>
                    </section>

                    {/* Quick Start */}
                    <section id="quick-start" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-foam/10 text-rose-pine-foam">
                                <Terminal className="w-6 h-6" />
                            </div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Quick Start</h2>
                        </div>

                        <p className="text-rose-pine-text">
                            Here is a simple counter program. It defines a state account and instructions to modify it.
                        </p>

                        <DocsEditor
                            filename="counter.v"
                            height="380px"
                            code={`// Define an account to hold state
account StateAccount {
    count: u64;
}

// Initialize the account
pub initialize(@init state: StateAccount) {
    state.count = 0;
}

// Increment the counter
pub increment(@mut state: StateAccount) {
    state.count = state.count + 1;
}

// Read the current count
pub get_count(state: StateAccount) -> u64 {
    return state.count;
}`}
                        />
                    </section>

                    {/* Language Reference */}
                    <section id="reference" className="space-y-10">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-rose/10 text-rose-pine-rose">
                                <Code className="w-6 h-6" />
                            </div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Language Reference</h2>
                        </div>

                        <div id="types" className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Data Types</h3>
                            <DataTypesSection />
                        </div>

                        <div id="account-architecture" className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Account Architecture</h3>
                            <AccountArchitectureSection />
                        </div>

                        <div id="state" className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Accounts & Instructions</h3>
                            <GlassCard className="p-6 space-y-4">
                                <div className="space-y-2">
                                    <h4 className="font-bold text-rose-pine-text">Account Definition</h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Define the structure of your on-chain data accounts.
                                    </p>
                                    <DocsEditor
                                        filename="state.v"
                                        height="120px"
                                        code={`account UserProfile {
    active: bool;
    score: u64;
}`}
                                    />
                                </div>

                                <div className="space-y-2">
                                    <h4 className="font-bold text-rose-pine-text">Instructions</h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Use <code className="text-rose-pine-iris">pub</code> to expose instructions. Helpers can be private.
                                    </p>
                                    <DocsEditor
                                        filename="instructions.v"
                                        height="140px"
                                        code={`pub update_score(@mut user: UserProfile, point: u64) {
    user.score = user.score + point;
}`}
                                    />
                                </div>
                            </GlassCard>
                        </div>

                        <div id="interfaces" className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Interfaces & CPI</h3>
                            <GlassCard className="p-6 space-y-4">
                                <p className="text-sm text-rose-pine-subtle">
                                    Define interfaces to interact with external programs via Cross-Program Invocation (CPI).
                                </p>
                                <DocsEditor
                                    filename="interfaces.v"
                                    height="280px"
                                    code={`// 1. Define the Interface (usually imported)
interface Vault {
    deposit @discriminator(1) (amount: u64)
}

// 2. Use it in your instruction
// This compiles to a standard Solana CPI (invoke)
pub instruction call_vault(v: Vault) {
    v.deposit(100); 
}`}
                                />
                            </GlassCard>
                        </div>

                        <div id="constraints" className="space-y-4">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Constraints & Safety</h3>
                            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <GlassCard className="p-6 space-y-4">
                                    <h4 className="font-bold text-rose-pine-text flex items-center gap-2">
                                        <code className="text-rose-pine-iris">@mut</code>
                                        <span className="text-xs font-normal text-rose-pine-subtle">Mutable Account</span>
                                    </h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Required to modify an account's data. Without this, accounts are read-only.
                                    </p>
                                    <DocsEditor
                                        filename="mutability.v"
                                        height="160px"
                                        code={`account Counter { val: u64; }

pub inc(@mut c: Counter) {
    c.val = c.val + 1;
}`}
                                    />
                                </GlassCard>

                                <GlassCard className="p-6 space-y-4">
                                    <h4 className="font-bold text-rose-pine-text flex items-center gap-2">
                                        <code className="text-rose-pine-iris">@signer</code>
                                        <span className="text-xs font-normal text-rose-pine-subtle">Signer Check</span>
                                    </h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Ensures the transaction was signed by this account. Essential for authority checks.
                                    </p>
                                    <DocsEditor
                                        filename="signer.v"
                                        height="160px"
                                        code={`pub withdraw(owner: account @signer) {
    // Only 'owner' can authorize this
    // Code executes only if signed
}`}
                                    />
                                </GlassCard>

                                <GlassCard className="p-6 space-y-4">
                                    <h4 className="font-bold text-rose-pine-text flex items-center gap-2">
                                        <code className="text-rose-pine-iris">@init</code>
                                        <span className="text-xs font-normal text-rose-pine-subtle">Initialize Account</span>
                                    </h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Creates a new account. Typically requires a funded <code className="text-rose-pine-iris">@signer</code> to pay for rent.
                                    </p>
                                    <DocsEditor
                                        filename="init.v"
                                        height="160px"
                                        code={`account Data { v: u64; }

pub new(payer: account @signer, d: account @init) {
    d.v = 0;
}`}
                                    />
                                </GlassCard>

                                <GlassCard className="p-6 space-y-4 border-l-4 border-l-rose-pine-iris">
                                    <h4 className="font-bold text-rose-pine-text flex items-center gap-2">
                                        <span className="text-xs font-normal text-rose-pine-subtle">On-Chain Fees</span>
                                    </h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Deploy and execute can include native SOL fees configured by the VM admin (basis points of rent and the standard Solana tx fee).
                                        The IDE cost estimate includes this deploy fee when it can read the on-chain configuration.
                                    </p>
                                </GlassCard>

                                <GlassCard className="p-6 space-y-4">
                                    <h4 className="font-bold text-rose-pine-text flex items-center gap-2">
                                        <code className="text-rose-pine-iris">@requires</code>
                                        <span className="text-xs font-normal text-rose-pine-subtle">Pre-condition</span>
                                    </h4>
                                    <p className="text-sm text-rose-pine-subtle">
                                        Runtime check. If the condition is false, the transaction aborts.
                                    </p>
                                    <DocsEditor
                                        filename="requires.v"
                                        height="160px"
                                        code={`pub deposit(amount: u64 @requires(amount > 0)) {
    // Amount is guaranteed > 0
}`}
                                    />
                                </GlassCard>
                            </div>
                        </div>

                        <div id="imports" className="space-y-6">
                            <h3 className="text-lg font-medium text-rose-pine-foam">Zero-Cost Imports & Import Verification</h3>

                            {/* Basic Imports */}
                            <GlassCard className="p-6 space-y-4">
                                <h4 className="text-sm font-semibold text-rose-pine-text">Importing Other Bytecode</h4>
                                <p className="text-sm text-rose-pine-subtle">
                                    Importing other Five bytecode accounts allows you to call their internal functions directly, without the overhead of CPI (Cross-Program Invocation). This is much cheaper and faster than traditional Solana program calls.
                                </p>
                                <DocsEditor
                                    filename="imports.v"
                                    height="220px"
                                    code={`// Direct import of trusted Five bytecode account
use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub calculate(x: u64) -> u64 {
    // Direct internal call to imported bytecode (zero CPI overhead)
    // Functions in imported bytecode are called via CALL_EXTERNAL opcode
    return x * x;
}`}
                                />
                            </GlassCard>

                            {/* Import Verification Security */}
                            <GlassCard className="p-6 space-y-4 border-l-4 border-l-rose-pine-gold">
                                <h4 className="text-sm font-semibold text-rose-pine-gold">🔒 Import Verification (Security Feature)</h4>
                                <p className="text-sm text-rose-pine-subtle">
                                    When you declare an import, Five automatically embeds verification metadata in your bytecode. At runtime, the Five VM verifies that the account being called matches your declared import address—preventing attackers from substituting a different bytecode account.
                                </p>
                                <div className="bg-rose-pine-base/50 rounded-lg p-4 text-xs space-y-2 text-rose-pine-subtle">
                                    <p><span className="text-rose-pine-gold font-semibold">Compile-time:</span> Import address stored in bytecode metadata with FEATURE_IMPORT_VERIFICATION flag</p>
                                    <p><span className="text-rose-pine-gold font-semibold">Runtime:</span> VM verifies account address matches before CALL_EXTERNAL execution</p>
                                    <p><span className="text-rose-pine-gold font-semibold">Result:</span> Unauthorized bytecode invocation rejected with error</p>
                                </div>
                                <DocsEditor
                                    filename="import_verification.v"
                                    height="260px"
                                    code={`// Secure import with automatic verification
use "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

pub mint_tokens(mint: account, dest: account, amount: u64) {
    // VM verifies that the account at this index is the declared
    // TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA before executing
    // If account != declared address, call is rejected with
    // UnauthorizedBytecodeInvocation error

    // This prevents an attacker from passing a different bytecode
    // account and hijacking your program flow
}

// Backward compatible: Old bytecode without imports
// still works (accepts any account)`}
                                />
                            </GlassCard>

                            {/* Verification Benefits */}
                            <GlassCard className="p-6 space-y-4">
                                <h4 className="text-sm font-semibold text-rose-pine-text">Why Import Verification Matters</h4>
                                <ul className="text-sm text-rose-pine-subtle space-y-2">
                                    <li className="flex gap-2">
                                        <span className="text-rose-pine-love">✓</span>
                                        <span><strong>Prevents Bytecode Substitution Attacks:</strong> Attacker cannot swap your imported bytecode for a malicious one</span>
                                    </li>
                                    <li className="flex gap-2">
                                        <span className="text-rose-pine-love">✓</span>
                                        <span><strong>Zero Runtime Cost for Valid Imports:</strong> Single 32-byte address comparison (&lt;1μs)</span>
                                    </li>
                                    <li className="flex gap-2">
                                        <span className="text-rose-pine-love">✓</span>
                                        <span><strong>Transparent Security:</strong> Verification happens automatically—no code changes needed</span>
                                    </li>
                                    <li className="flex gap-2">
                                        <span className="text-rose-pine-love">✓</span>
                                        <span><strong>Backward Compatible:</strong> Existing bytecode without imports continues to work</span>
                                    </li>
                                    <li className="flex gap-2">
                                        <span className="text-rose-pine-love">✓</span>
                                        <span><strong>Future PDA Support:</strong> Metadata format supports PDA-derived bytecode accounts</span>
                                    </li>
                                </ul>
                            </GlassCard>
                        </div>
                    </section>

                    {/* Security */}
                    <section id="security-rules" className="space-y-6">
                        <div className="flex items-center gap-3">
                            <div className="p-2 rounded-lg bg-rose-pine-gold/10 text-rose-pine-gold">
                                <Shield className="w-6 h-6" />
                            </div>
                            <h2 className="text-2xl font-bold text-rose-pine-text">Security Rules</h2>
                        </div>

                        <p className="text-rose-pine-text">
                            5IVE enforces strict security rules at compile time to prevent common exploits.
                        </p>

                        <div className="grid gap-6">
                            <GlassCard className="p-6 border-l-4 border-l-rose-pine-love" hoverEffect>
                                <div className="flex items-start gap-4">
                                    <div className="bg-rose-pine-love/20 p-2 rounded text-rose-pine-love font-black font-mono">01</div>
                                    <div className="space-y-2">
                                        <h3 className="font-bold text-rose-pine-text">Read-Only External Fields</h3>
                                        <p className="text-sm text-rose-pine-subtle mb-3">
                                            You can read fields from imported contracts, but you cannot modify them directly.
                                            This prevents unauthorized state changes.
                                        </p>
                                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4 text-xs font-mono">
                                            <div className="p-3 rounded bg-red-500/10 border border-red-500/20">
                                                <span className="text-red-400 font-bold block mb-1">❌ Incorrect</span>
                                                balance = 100;
                                            </div>
                                            <div className="p-3 rounded bg-emerald-500/10 border border-emerald-500/20">
                                                <span className="text-emerald-500 font-bold block mb-1">✅ Correct</span>
                                                let x = balance;
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </GlassCard>

                            <GlassCard className="p-6 border-l-4 border-l-rose-pine-love" hoverEffect>
                                <div className="flex items-start gap-4">
                                    <div className="bg-rose-pine-love/20 p-2 rounded text-rose-pine-love font-black font-mono">02</div>
                                    <div className="space-y-2">
                                        <h3 className="font-bold text-rose-pine-text">Explicit Function Calls</h3>
                                        <p className="text-sm text-rose-pine-subtle mb-3">
                                            To modify an external contract's state, you must call one of its public instructions.
                                        </p>
                                        <div className="p-3 rounded bg-rose-pine-surface/50 border border-rose-pine-hl-low font-mono text-xs text-rose-pine-iris">
                                            token.transfer(recipient, amount);
                                        </div>
                                    </div>
                                </div>
                            </GlassCard>
                        </div>
                    </section>
                </div>
            </main>

            {/* Simple Footer */}
            <footer className="py-8 border-t border-rose-pine-hl-low/20 text-center text-sm text-rose-pine-muted">
                <p>© 2025 5IVE Org. All rights reserved.</p>
            </footer>
        </div>
    );
}
