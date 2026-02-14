# AGENTS_REFERENCE.md - 5IVE Practical Reference

This reference is for agents that do not have direct access to the 5IVE monorepo internals.
Use with `./AGENTS.md` and `./AGENTS_CHECKLIST.md`.

## 1) Core Surfaces

1. Source language: `.v`
2. Build artifact: `.five` (bytecode + ABI)
3. CLI: `@5ive-tech/cli` commands `5ive` or `five`
4. SDK: `@5ive-tech/sdk`

## 2) Compiler-Critical Syntax

### Account declarations

```v
account Vault {
    authority: pubkey;
    balance: u64;
    status: u8;
}
```

Rule: every account field must end with `;`.

### Signers and key extraction

```v
pub update_authority(
    state: Vault @mut,
    authority: account @signer,
    next_authority: pubkey
) {
    require(state.authority == authority.key);
    state.authority = next_authority;
}
```

Rules:
1. signer params are `account @signer`
2. use `.key` when comparing or assigning pubkeys from account params

### Init attribute order

Canonical order for initialized account params:

`Type @mut @init(payer=name, space=bytes) @signer`

```v
pub initialize(
    state: Vault @mut @init(payer=creator, space=128) @signer,
    creator: account @mut @signer
) {
    state.authority = creator.key;
    state.balance = 0;
    state.status = 1;
}
```

### Return types and locals

```v
pub quote(amount: u64, fee_bps: u64) -> u64 {
    let mut result: u64 = amount;
    result = result - ((amount * fee_bps) / 10000);
    return result;
}
```

Rules:
1. functions returning values must use `-> ReturnType`
2. locals are immutable unless declared with `let mut`

## 3) Built-ins and Units

Compiler-aligned signatures:
1. `get_clock() -> u64`
2. `derive_pda(seed1, seed2, ...) -> (pubkey, u8)`
3. `derive_pda(seed1, seed2, ..., bump: u8) -> pubkey`

Recommended unit standards:
1. time in seconds
2. USD price scale `1e6`
3. rate scale `1e9` (or `1e12`, but stay consistent per contract)

## 4) CPI Rules

1. Interface uses `@program("...")` with valid base58 program ID.
2. Anchor CPI: use `@anchor` and do not add manual discriminator.
3. Non-anchor CPI: use single-byte `@discriminator(N)`.
4. Interface account params use `Account`, not `pubkey`.
5. Invoke interface methods with dot notation: `Iface.method(...)`.
6. Pass account params directly in CPI calls, not `.key`.
7. CPI-writable accounts must be `account @mut` in caller signature.

## 5) Build and Test Commands

```bash
5ive build
5ive test --sdk-runner
5ive test --filter "test_*" --verbose
```

Discovery behavior:
1. test functions can be named `pub test_*`
2. `.v` tests and `.test.json` suites are supported by `5ive test`

## 6) Debugging Loop for Weak Error Messages

When compiler errors are unclear, use this fixed loop:
1. Keep the requested contract scope intact.
2. Compile and capture the first failing file/line.
3. Check parser-critical items first:
- account field semicolons
- init attribute order
- signer type and `.key` usage
- `let` vs `let mut`
4. Recompile immediately after each small fix.
5. If still failing, isolate one instruction block, fix it, then merge back.
6. Do not downgrade to a simplified contract unless the user requests it.

## 7) five.toml and Program ID Resolution

On-chain command precedence (`deploy`, `execute`, `namespace`):
1. `--program-id`
2. `five.toml [deploy].program_id`
3. current CLI config target/program
4. `FIVE_PROGRAM_ID`

Never deploy/execute with ambiguous target or program ID.

## 8) SDK Client Pattern

Use this pattern for clients:

```ts
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { FiveSDK } from "@5ive-tech/sdk";
import fs from "node:fs";

const connection = new Connection("http://127.0.0.1:8899", "confirmed");
const payer = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(fs.readFileSync("./payer.json", "utf8")))
);

const programId = new PublicKey("REPLACE_WITH_PROGRAM_ID");
const artifact = fs.readFileSync("./build/main.five");

const sdk = new FiveSDK(connection, payer);
const program = await sdk.loadProgram({
  programId,
  bytecode: artifact,
});

const sig = await program
  .method("initialize")
  .accounts({
    state: new PublicKey("REPLACE_STATE"),
    authority: payer.publicKey,
  })
  .args({})
  .rpc();

console.log("signature", sig);
```

Client debugging checks:
1. method name must exactly match ABI
2. required accounts must all be provided
3. args shape/order must match ABI
4. signer/payer must be funded and correct
5. print and inspect transaction logs on failure

## 9) Delivery Checklist Summary

Before declaring done:
1. `.five` built
2. tests green
3. deploy/execute verified when requested (`meta.err == null`)
4. signature and compute units recorded
5. client snippet or script included when requested
