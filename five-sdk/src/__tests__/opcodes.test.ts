
import { describe, expect, it } from '@jest/globals';
import { VLEEncoder } from '../lib/vle-encoder.js';

// We can't easily test WASM interaction without the WASM module built,
// but we can test the VLEEncoder logic which prepares data for execution.
// The VLEEncoder uses the WASM module for VLE encoding if available, or falls back.
// Here we verify it exports types that match our expectations.

describe('Opcode Consistency Checks', () => {
  // Hardcoded list of key opcodes from the Spec to verify against potential regressions
  // or mismatches if we were checking against a real WASM build.
  // Since we are mocking WASM in this environment, we are verifying the *intent*
  // and ensuring any hardcoded values in SDK (if any) are correct.

  // Note: SDK doesn't hardcode opcodes directly anymore (it relies on WASM),
  // except for some special handling in VLEEncoder.

  it('VLEEncoder defines correct TYPE_IDS matching Protocol', () => {
    // These IDs must match five-protocol/src/types.rs or equivalent
    expect(VLEEncoder.getTypeId('u8')).toBe(1);
    expect(VLEEncoder.getTypeId('u16')).toBe(2);
    expect(VLEEncoder.getTypeId('u32')).toBe(3);
    expect(VLEEncoder.getTypeId('u64')).toBe(4);
    expect(VLEEncoder.getTypeId('i64')).toBe(8);
    expect(VLEEncoder.getTypeId('bool')).toBe(9);
    expect(VLEEncoder.getTypeId('pubkey')).toBe(10);
    expect(VLEEncoder.getTypeId('string')).toBe(11);
    expect(VLEEncoder.getTypeId('account')).toBe(12);
    expect(VLEEncoder.getTypeId('array')).toBe(13);
  });

  // Since we can't load the real WASM module to call `get_constants`,
  // we can't verify the WASM export here. That is covered by the Rust tests
  // we added in `five-wasm`.

  // Ideally, we would have an integration test that builds the WASM and checks this,
  // but that's heavy for this environment.
});
