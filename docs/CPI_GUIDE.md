# Five CPI (Cross-Program Invocation) Guide

This guide covers Cross-Program Invocation (CPI) in Five, including interface definition, serialization formats, and real-world usage patterns.

For external account state decoding (`Mint`, `TokenAccount`, etc.), see:
- [`ACCOUNT_SERIALIZER_STATE_ACCESS_GUIDE.md`](./ACCOUNT_SERIALIZER_STATE_ACCESS_GUIDE.md)

## Overview

CPI allows Five contracts to invoke instructions on other Solana programs, including:
- **SPL Token** - Mint tokens, transfer, burn, etc.
- **Anchor programs** - Call methods on other Anchor-based contracts
- **Metaplex** - Interact with NFT and compressed token programs
- **Custom programs** - Any Solana program with a known instruction format

Five implements CPI through:
1. **Interface declarations** - Define program boundaries and instruction formats
2. **INVOKE/INVOKE_SIGNED opcodes** - Execute the CPI at runtime
3. **Serialization** - Encode instruction data in Borsh or Bincode format
4. **Account management** - Partition accounts and data at compile time

Important distinction:
- Interface `@serializer(...)` controls CPI instruction-data encoding.
- Account `@serializer(...)` controls account-state decoding for typed field access.

## Interface Declarations

### Basic Syntax

Interfaces define the boundary between your Five contract and external programs:

```five
interface MyProgram @program("11111111111111111111111111111111") {
    my_instruction @discriminator(0) (
        account1: pubkey,
        account2: pubkey,
        data_arg: u64
    );
}
```

### Attributes

#### @program(address)
Specifies the program ID that this interface calls. Required on all interfaces.

```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    // ...
}
```

#### @serializer(format)
Specifies the instruction data encoding format. Supported values:
- `bincode` - **Default**. Used for SPL/native program compatibility in Five MVP
- `borsh` - Used by Anchor programs and selected Solana programs
- `raw` - Custom binary format (advanced, currently limited)

```five
interface MyLegacyProgram @program("...") @serializer(bincode) {
    // ...
}
```

#### @discriminator(value)
Specifies how to identify the instruction. Can be:
- **Single u8** - Appended to instruction data (0-255)
- **Multiple bytes** - For Anchor's 8-byte discriminators

```five
// u8 discriminator (appended after data args)
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// Anchor 8-byte discriminator (prepended to data args)
interface AnchorProgram @program("...") {
    my_instruction @discriminator([0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF, 0x00]) (
        value: u64
    );
}
```

## Account vs Data Parameters

CPI divides parameters into two categories:

### Account Parameters (Pubkeys)
Accounts that the external program will interact with. Listed as instruction accounts in the CPI call.

```five
pub transfer(
    source: account,      // Account param - listed separately
    dest: account,        // Account param
    amount: u64           // Data param - encoded in instruction data
) {
    ExternalProgram.transfer(source, dest, amount);
}
```

**Key Points:**
- Derived from parameter type `pubkey` or `account`
- Must be resolved at compile time
- Can refer to local state or function parameters
- Accounts are passed by index in the INVOKE instruction
- Maximum 16 accounts per call (Solana limit)

### Data Parameters
Values encoded into the instruction data buffer.

**Supported Data Types:**
- `u8`, `u16`, `u32`, `u64` - Unsigned integers (network byte order)
- `bool` - Boolean (0x00 or 0x01)
- `pubkey` - 32-byte public key
- `string` - UTF-8 string with length prefix

**MVP Limitation: Literals Only**
Data arguments must be **compile-time constants**. Function parameters and expressions are not yet supported:

```five
// ✅ Works - literal data args
SPLToken.mint_to(mint, to, authority, 1000);

// ❌ Fails - variable data arg (MVP limitation)
let amount: u64 = 1000;
SPLToken.mint_to(mint, to, authority, amount);

// ❌ Fails - expression data arg
SPLToken.mint_to(mint, to, authority, 500 + 500);
```

## Serialization Formats

### Borsh (Optional)

Borsh is the standard format used by Anchor and most Solana programs.

**Format:**
- Discriminator (u8, appended after data)
- Data args in declaration order, serialized per Borsh spec
- All integers in little-endian format

**Example:**
```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// Call
SPLToken.mint_to(mint_key, to_key, auth_key, 1000);

// Encodes to (hex):
// [32 bytes: mint_key]
// [32 bytes: to_key]
// [32 bytes: auth_key]
// [8 bytes: 1000 in little-endian] (e8 03 00 00 00 00 00 00)
// [1 byte: discriminator 7] (07)
```

**Borsh Type Encoding:**
- `u8`: 1 byte
- `u16`: 2 bytes (LE)
- `u32`: 4 bytes (LE)
- `u64`: 8 bytes (LE)
- `pubkey`: 32 bytes (unchanged)
- `bool`: 1 byte (0x00 or 0x01)
- `string`: 4-byte length (LE) + UTF-8 bytes

### Bincode

Bincode is used by legacy Solana programs and some custom implementations.

**Format:**
- Identical data ordering and type encoding as Borsh for numeric types
- Discriminator treated the same way
- Primarily used for compatibility with non-Anchor programs

```five
interface LegacyProgram @program("...") @serializer(bincode) {
    // Same format as Borsh for numeric types
}
```

### Raw Serializer

For custom binary formats. Currently limited in the MVP.

```five
interface CustomProgram @program("...") @serializer(raw) {
    // Not recommended - limited support in current version
}
```

## Invoking External Programs

### Basic CPI Call

```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    payer: account @signer
) {
    SPLToken.mint_to(mint, to, authority, 1000);
}
```

### INVOKE vs INVOKE_SIGNED

#### INVOKE
Used when the authority account is a signer in the current transaction.

```five
// Authority is a signer in this transaction
pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,  // Must sign current tx
    payer: account @signer
) {
    // Simple INVOKE
    SPLToken.mint_to(mint, to, authority, 1000);
}
```

#### INVOKE_SIGNED
Used when the authority is a PDA (Program Derived Address) that your contract controls.

```five
pub mint_from_pda(
    mint: account @mut,
    to: account @mut,
    pda_authority: account,        // PDA (not a signer)
    payer: account @signer
) {
    // INVOKE_SIGNED with PDA seeds
    // Seeds: ["mint_authority"]
    SPLToken.mint_to(mint, to, pda_authority, 1000);
}
```

**Note:** INVOKE_SIGNED support is implemented in the VM but requires bytecode generation support in the compiler. Check current version for availability.

## Real-World Examples

### SPL Token Mint

```five
use SPLToken;

interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    mint_to @discriminator(7) (
        mint: pubkey,
        to: pubkey,
        authority: pubkey,
        amount: u64
    );
}

pub mint_tokens(
    mint: account @mut,
    to: account @mut,
    authority: account @signer,
    amount: u64
) {
    SPLToken.mint_to(mint, to, authority, 1000);
}
```

### Anchor Program Call

```five
interface CounterProgram @program("CounterProgramIdHere...") {
    increment @discriminator([0xAA, 0x12, 0x34, 0x56, 0x78, 0xAB, 0xCD, 0xEF]) (
        counter: pubkey,
        user: pubkey,
        amount: u64
    );
}

pub increment_remote(
    counter: account @mut,
    user: account @signer,
    amount: u64
) {
    CounterProgram.increment(counter, user, amount);
}
```

### PDA Authority with INVOKE_SIGNED

```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") {
    burn @discriminator(8) (
        token_account: pubkey,
        mint: pubkey,
        authority: pubkey,
        amount: u64
    );
}

// PDA: Derived from ["treasury", program_id]
mut treasury_balance: u64;

pub burn_from_treasury(
    token_account: account @mut,
    mint: account @mut,
    treasury_pda: account,        // Our PDA (not a signer)
    amount: u64
) {
    // INVOKE_SIGNED with PDA authority
    SPLToken.burn(token_account, mint, treasury_pda, amount);
}
```

## Account Constraints

**Note:** Account constraints are parsed but not enforced at runtime in the current MVP. This is a known limitation.

```five
pub transfer(
    from: account @mut @signer,    // Must be writable and must sign
    to: account @mut,              // Must be writable
    amount: u64
) {
    // Constraints are recognized by parser but not validated at VM runtime
    ExternalProgram.transfer(from, to, amount);
}
```

**Current Behavior:**
- `@mut` - Parsed, not enforced
- `@signer` - Parsed, not enforced
- `@init(payer=X, space=N)` - Parsed, not enforced
- Account constraint validation happens at instruction validation level (outside VM)

## Stack Contract Format

Five encodes interfaces in a stack contract (`.stack`) file for on-chain verification:

```json
{
  "interfaces": [
    {
      "program": "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
      "methods": [
        {
          "name": "mint_to",
          "discriminator": 7,
          "account_params": 3,
          "data_types": ["u64"],
          "serializer": "bincode"
        }
      ]
    }
  ]
}
```

This prevents:
- **Bytecode substitution attacks** - If bytecode modifies interface definitions, it's rejected
- **Interface drift** - Ensures called interfaces match deployment expectations
- **Type safety** - Verifies data arg types match serialization format

## Troubleshooting

### "Unknown interface MyProgram"
**Cause:** Interface not declared or not imported.

**Fix:**
```five
// Declare the interface before using it
interface MyProgram @program("...") {
    my_method @discriminator(0) (...);
}

pub my_function(...) {
    MyProgram.my_method(...);  // Now valid
}
```

### "Parameter count mismatch"
**Cause:** Calling interface method with wrong number of arguments.

**Fix:** Count both account and data parameters:
```five
interface SPLToken @program("...") {
    mint_to @discriminator(7) (
        mint: pubkey,           // param 1
        to: pubkey,             // param 2
        authority: pubkey,      // param 3
        amount: u64             // param 4
    );
}

// Must pass all 4 arguments
SPLToken.mint_to(mint, to, authority, 1000);  // ✅ Correct

// ❌ Wrong: Only 3 arguments
SPLToken.mint_to(mint, to, authority);
```

### "Data argument must be literal"
**Cause:** Trying to pass a variable or expression as a data argument (MVP limitation).

**Fix:** Use literal values:
```five
// ❌ Fails - MVP limitation
let amount: u64 = 1000;
SPLToken.mint_to(mint, to, authority, amount);

// ✅ Works - literal
SPLToken.mint_to(mint, to, authority, 1000);
```

### "Import verification failed"
**Cause:** Stack contract interfaces don't match bytecode interfaces.

**Fix:** Rebuild from source or verify program IDs match deployment.

## Performance Considerations

### Instruction Size
Maximum instruction data: **32 bytes** (Solana limit after account list)

For Borsh encoding:
- 3 pubkey accounts (96 bytes) + 4 bytes data = 100 bytes total
- Accounts are passed separately, only data counts toward 32-byte limit
- Plan data args carefully to stay under limit

### Stack Usage
Five VM uses a 64-byte temp buffer for intermediate values. CPI calls are efficient:
- No per-argument stack allocation
- Account indices are 1-byte pointers
- Data is encoded directly in instruction buffer

### On-Chain Compute
CPI adds ~15,000 compute units per call (varies by target program). Budget accordingly.

## Known Limitations (MVP)

| Feature | Status | Impact |
|---------|--------|--------|
| Literal data arguments | ✅ Works | Can't pass variables |
| Account parameters | ✅ Works | Works as expected |
| Bincode serialization | ✅ Works | Default, well-tested |
| Borsh serialization | ✅ Works | Anchor program support |
| CPI return data | ❌ Not implemented | Can't capture return values |
| Dynamic data args | ❌ Not implemented | Can only use compile-time constants |
| Account constraint enforcement | ❌ Not implemented | Constraints parsed but not validated |
| Account-state decoding metadata | ✅ Works | Type + param serializer precedence supported |

## Testing Your CPIs

### Local Execution

Test CPI code locally before deploying:

```bash
# Compile to bytecode
five compile my_contract.v -o my_contract.five

# Execute locally (mocks external calls)
five local execute my_contract.v 0
```

### On-Chain Testing

For real CPI validation, deploy to devnet and test with real programs:

```bash
# Deploy to devnet
solana config set -u devnet
five deploy my_contract.five --url devnet

# Execute on devnet (requires account setup)
five execute <SCRIPT_ACCOUNT> -f 0
```

See `five-templates/cpi-examples/` for complete integration test examples.
For typed external account reads and serializer precedence tests, see:
- `five-templates/cpi-integration-tests/test-localnet.mjs`
- `five-templates/cpi-integration-tests/test-spl-state-read.v`

## Further Reading

- **Protocol Spec:** See `five-protocol/OPCODE_SPEC.md` for INVOKE/INVOKE_SIGNED opcodes
- **VM Implementation:** See `five-vm-mito/src/handlers/system/invoke.rs` for low-level details
- **Compiler Source:** See `five-dsl-compiler/src/interface_serializer.rs` for Borsh/Bincode encoding
- **Tests:** See `five-dsl-compiler/tests/lib.rs` (lines 2628-2766) for CPI compiler tests
- **VM Tests:** See `five-vm-mito/tests/*cpi*.rs` for VM and integration tests
