# Five Templates - Testing Guide

## VM State PDA Derivation

The VM State account **must be a PDA** (Program Derived Address), not a keypair. Use the following pattern in your tests:

```javascript
import { PublicKey } from '@solana/web3.js';

// Derive VM State PDA
const [vmStatePda, vmStateBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("vm_state")],
    programId
);
```

## Account Creation Pattern

Due to Solana's runtime validation, accounts must be **pre-created** before being passed to program instructions:

### 1. Create the Account First
```javascript
const createTx = new Transaction();
createTx.add(SystemProgram.createAccount({
    fromPubkey: payer.publicKey,
    newAccountPubkey: accountKeypair.publicKey,
    lamports: rentExemptAmount,
    space: accountSize,
    programId: fiveProgramId
}));
await sendAndConfirmTransaction(connection, createTx, [payer, accountKeypair]);
```

### 2. Then Call the Initialization Instruction
```javascript
const initTx = new Transaction();
initTx.add(new TransactionInstruction({
    keys: [
        { pubkey: accountKeypair.publicKey, isSigner: false, isWritable: true },
        // ... other accounts
    ],
    programId: fiveProgramId,
    data: instructionData
}));
await sendAndConfirmTransaction(connection, initTx, [payer]);
```

## Required @signer Constraints

In your `.v` files, accounts that will be created via `SystemProgram.createAccount` need the `@signer` constraint:

```v
pub init_mint(
    mint_account: Mint @mut @init @signer,  // ← @signer required
    authority: account @signer,
    // ... other params
) -> pubkey {
    // ...
}
```

This ensures the SDK generates the correct account metadata for the transaction.

## Common Issues

### Error: "Provided owner is not allowed"
- **Cause**: Trying to access an account before it's created
- **Solution**: Split into two transactions (create, then initialize)

### Error: "Account not found"  
- **Cause**: Using a keypair instead of deriving the VM State PDA
- **Solution**: Use `PublicKey.findProgramAddressSync` with seeds `["vm_state"]`

### Error: "Missing signer"
- **Cause**: Account marked `@signer` in DSL but not passed as signer in transaction
- **Solution**: Include the keypair in the `signers` array when sending the transaction
