# CPI Examples

This directory contains example Five contracts demonstrating Cross-Program Invocation (CPI) to external Solana programs.

## Examples

### 1. spl-token-mint.v
Basic SPL Token interaction - mints tokens from a token mint to a destination account.

**Key Concepts:**
- Interface definition with `@program()` and `@discriminator()`
- Account parameters (mint, to, authority)
- Data parameters (amount as u64 literal)
- Borsh serialization (default)

**Test:**
```bash
npm run test:spl-token-mint
```

### 2. anchor-program-call.v
Calls a custom Anchor program using 8-byte discriminators.

**Key Concepts:**
- Anchor-specific 8-byte discriminator format: `@discriminator([0xAA, 0x12, ...])`
- Mixed account and data parameters
- Borsh serialization for Anchor compatibility

**Note:** Replace `CounterProgramIdHere...` with a real Anchor program ID to test on-chain.

**Test:**
```bash
npm run test:anchor-program
```

### 3. invoke-signed-pda.v
Uses INVOKE_SIGNED with a Program Derived Address (PDA) as authority.

**Key Concepts:**
- PDA authority without direct signer
- INVOKE_SIGNED opcode for delegated authority
- Burning tokens with contract-controlled authority
- Global state tracking

**Note:** This example demonstrates the architecture. Full PDA derivation and INVOKE_SIGNED testing requires localnet setup.

**Test:**
```bash
npm run test:pda-invoke
```

## Building

Compile all examples:

```bash
npm run build
```

Or compile individually:

```bash
five compile spl-token-mint.v -o spl-token-mint.five
five compile anchor-program-call.v -o anchor-program-call.five
five compile invoke-signed-pda.v -o invoke-signed-pda.five
```

## Local Testing

Test locally without on-chain execution:

```bash
five local execute spl-token-mint.v 0
five local execute anchor-program-call.v 0
five local execute invoke-signed-pda.v 0
```

## On-Chain Testing

To test these examples on-chain, you'll need:

1. **Devnet or Localnet Setup**
   ```bash
   solana-test-validator  # For local testing
   # OR
   solana config set -u devnet  # For devnet
   ```

2. **Deploy the contract**
   ```bash
   five deploy spl-token-mint.five --url http://127.0.0.1:8899
   ```

3. **Execute on-chain**
   ```bash
   # Note: Requires proper account setup with actual token mints, etc.
   five execute <SCRIPT_ACCOUNT> -f 0 --params "[...]"
   ```

See the corresponding `.mjs` test files for full integration test examples.

## Architecture Notes

### Account vs Data Parameters

All examples demonstrate the split between:

- **Account Parameters** - Accounts the external program will interact with
  - Passed as separate instruction accounts (maximum 16)
  - Derived at compile time
  - Include writable/signer flags (@mut, @signer)

- **Data Parameters** - Values encoded in instruction data
  - Serialized into the instruction data buffer (maximum 32 bytes)
  - Currently must be compile-time constants (literals)
  - Supported types: u8, u16, u32, u64, bool, pubkey, string

### Serialization

All examples use **Borsh** serialization (the default):

```
[32 bytes: account1]
[32 bytes: account2]
...
[data_args in order...]
[discriminator byte or bytes appended]
```

For u64 values, integers use little-endian byte order (standard for Solana).

## Limitations (MVP)

- **Data arguments must be literals** - Can't pass variables or expressions
- **No CPI return data** - Can't capture return values from CPI calls
- **Account constraints parsed but not enforced** - @signer, @mut recognized but not validated at VM runtime
- **Raw serializer** - Custom binary formats not yet supported

See `docs/CPI_GUIDE.md` for detailed documentation and workarounds.

## Troubleshooting

### "Unknown interface SPLToken"
Make sure the interface is declared before the function that uses it.

### "Parameter count mismatch"
Count both account AND data parameters. For `mint_to`, there are 4 total: 3 pubkey accounts + 1 u64 data.

### "Data argument must be literal"
This is an MVP limitation. Use constants:
```five
// ❌ Won't work
let amount: u64 = 1000;
SPLToken.mint_to(mint, to, auth, amount);

// ✅ Works
SPLToken.mint_to(mint, to, auth, 1000);
```

## Further Reading

- **Complete CPI Guide:** `docs/CPI_GUIDE.md`
- **Compiler Source:** `five-dsl-compiler/src/interface_serializer.rs`
- **VM Handler:** `five-vm-mito/src/handlers/system/invoke.rs`
- **Protocol Spec:** `five-protocol/OPCODE_SPEC.md` (INVOKE/INVOKE_SIGNED opcodes)
