# Error 8122 Investigation Report

## Summary
Error 8122 (CallTargetOutOfBounds) blocks token deployment with bytecode verification enabled. When verification is disabled to allow deployment, E2E tests fail with access violation errors, suggesting the verification was catching a real problem.

## Findings

### Phase 1: Bytecode Analysis
- Compiled token bytecode: 805 bytes
- All 14 CALL instructions verified locally - targets are in bounds
- Max CALL target: offset 753, bytecode length: 805
- No out-of-bounds targets found in local analysis

### Phase 2: Deployment Investigation
- Account allocation: 864 bytes (64-byte header + 800 bytecode)
- Expected allocation: 869 bytes (64-byte header + 805 bytecode)
- Chunk 0 (400 bytes): appends successfully
- Chunk 1 (400 bytes): appends successfully
- Chunk 2 (5 bytes): fails during verification with error 8122

### Phase 3: Verification Disabled Deployment
- Deployment succeeds when verification is disabled
- Script account created and finalized
- E2E tests fail immediately with: "Access violation in program section at address 0x100040060 of size 8"

## Root Cause Analysis

### Why Verification Fails
The verification code checks CALL targets against bytecode.len(). Error 8122 indicates a CALL instruction targets >= bytecode length. However:
- Local analysis shows all targets are in bounds
- Verification code path: `verify_bytecode_content(&script_data[64..869])`
- Account size is only 864, so slice should be out of bounds

Possible causes:
1. Account allocation failure during safe_realloc (not properly reported)
2. Bytecode corruption during chunked upload
3. Verification receiving incomplete or corrupted bytecode
4. Off-by-one error in slice calculation

### Why Execution Fails
Memory access violation during VM execution suggests:
1. Bytecode is malformed (verification was catching real problem)
2. Bytecode structure corrupted during upload
3. VM state initialization failure
4. Account data layout mismatch

## Next Investigation Steps

### 1. Verify Bytecode Integrity
- Extract deployed bytecode from account
- Compare byte-by-byte with source bytecode
- Check for corruption during chunked upload

### 2. Debug safe_realloc Behavior
- Add logging to safe_realloc calls
- Verify account resizing happens correctly
- Check if realloc failures are being swallowed

### 3. Analyze Account Layout
- Verify bytecode starts at offset 64
- Check script account header fields
- Ensure upload_len and bytecode_len are consistent

### 4. Profile Chunked Upload
- Test with different chunk sizes
- Verify all bytes are correctly copied
- Check for alignment or boundary issues

## Current Status
**BLOCKED**: Cannot proceed with token deployment or E2E testing until error 8122 is resolved.

Both with and without verification, there's a fundamental issue:
- With verification: E2E blocked (error 8122)
- Without verification: E2E fails (access violations)

The verification error may be a symptom of a deeper bytecode or upload issue.
