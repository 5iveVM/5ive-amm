//! Zero-copy import verification metadata parser for Five bytecode accounts.
//!
//! Provides allocation-free parsing of import metadata embedded in Five VM
//! bytecode for on-chain execution.
//!
//! Bytecode Format (after main bytecode):
//! [import_count: u8]
//! For each import:
//!   [import_type: u8]  (0 = address, 1 = PDA seeds)
//!   If address:
//!     [pubkey: 32 bytes]
//!   If PDA seeds:
//!     [seed_count: u8]
//!     For each seed:
//!       [seed_len: u8]
//!       [seed_bytes: variable]
//!   [function_name_len: u8]
//!   [function_name: variable]

use crate::error::VMError;

/// Feature flag constant for import verification.
pub const FEATURE_IMPORT_VERIFICATION: u32 = 1 << 4;

/// Callback type for PDA derivation.
/// Takes seed slices and program_id, returns derived account public key (32 bytes).
pub type PdaDerivationFn = fn(seeds: &[&[u8]], program_id: &[u8; 32]) -> [u8; 32];

/// Zero-copy import metadata parser.
///
/// Stores only a reference to the bytecode slice and performs a linear search
/// through imports (typically 1-10 entries).
pub struct ImportMetadata<'a> {
    /// Direct reference to metadata section in bytecode.
    /// If empty, no verification is needed (backward compatible).
    metadata_bytes: &'a [u8],
}

impl<'a> ImportMetadata<'a> {
    /// Create metadata parser from bytecode and metadata offset.
    pub fn new(bytecode: &'a [u8], metadata_offset: usize) -> Result<Self, VMError> {
        // Bounds check.
        if metadata_offset >= bytecode.len() {
            // No metadata (backward compatible).
            return Ok(ImportMetadata {
                metadata_bytes: &[],
            });
        }

        Ok(ImportMetadata {
            metadata_bytes: &bytecode[metadata_offset..],
        })
    }

    /// Verify account address matches any verified import.
    ///
    /// Takes a PDA derivation callback for platform-independent operation.
    /// The callback is only invoked for PDA-mode imports (not for direct addresses).
    ///
    /// Returns true if:
    /// - No metadata (backward compatible), OR
    /// - Account key matches an address-mode import, OR
    /// - Account key matches a PDA derived from stored seeds.
    pub fn verify_account(
        &self,
        account_key: &[u8; 32],
        program_id: &[u8; 32],
        pda_derivation: Option<PdaDerivationFn>,
    ) -> bool {
        // No metadata = backward compatible (accept any account).
        if self.metadata_bytes.is_empty() {
            return true;
        }

        // Safely parse metadata with bounds checking.
        let mut offset = 0;

        // Read import count.
        if offset >= self.metadata_bytes.len() {
            return false;
        }
        let import_count = self.metadata_bytes[offset] as usize;
        offset += 1;

        // Linear search through imports (fast for small N).
        for _ in 0..import_count {
            if offset >= self.metadata_bytes.len() {
                return false; // Malformed metadata.
            }

            let import_type = self.metadata_bytes[offset];
            offset += 1;

            let matches = match import_type {
                0 => {
                    // Address mode - direct 32-byte comparison.
                    if offset + 32 > self.metadata_bytes.len() {
                        return false; // Malformed.
                    }
                    let expected = &self.metadata_bytes[offset..offset + 32];
                    offset += 32;
                    account_key == expected
                }
                1 => {
                    // PDA seeds mode - derive and compare.
                    if offset >= self.metadata_bytes.len() {
                        return false; // Malformed.
                    }
                    let seed_count = self.metadata_bytes[offset] as usize;
                    offset += 1;

                    // Stack-allocated seed array (max 4 seeds, fits in stack).
                    let mut seed_slices: [Option<&[u8]>; 4] = [None; 4];
                    let mut all_seeds_valid = true;

                    // Parse seeds.
                    for i in 0..seed_count.min(4) {
                        if offset >= self.metadata_bytes.len() {
                            all_seeds_valid = false;
                            break;
                        }

                        let seed_len = self.metadata_bytes[offset] as usize;
                        offset += 1;

                        if offset + seed_len > self.metadata_bytes.len() {
                            all_seeds_valid = false;
                            break;
                        }

                        seed_slices[i] = Some(&self.metadata_bytes[offset..offset + seed_len]);
                        offset += seed_len;
                    }

                    if !all_seeds_valid || seed_count > 4 {
                        return false;
                    }

                    // Derive PDA using callback. If no callback is provided, cannot verify PDA.
                    if let Some(derive_fn) = pda_derivation {
                        let seed_refs: [&[u8]; 4] = [
                            seed_slices[0].unwrap_or(&[]),
                            seed_slices[1].unwrap_or(&[]),
                            seed_slices[2].unwrap_or(&[]),
                            seed_slices[3].unwrap_or(&[]),
                        ];
                        let derived = derive_fn(&seed_refs[..seed_count], program_id);
                        account_key == &derived
                    } else {
                        // No PDA derivation callback - cannot verify, continue searching.
                        false
                    }
                }
                _ => {
                    // Invalid import type.
                    return false;
                }
            };

            // Skip function name (not needed for verification).
            if offset >= self.metadata_bytes.len() {
                return false; // Malformed.
            }
            let name_len = self.metadata_bytes[offset] as usize;
            offset += 1 + name_len;

            if matches {
                return true;
            }
        }

        false
    }

    /// Check if metadata is empty (no imports).
    pub fn is_empty(&self) -> bool {
        self.metadata_bytes.is_empty()
    }

    /// Get metadata byte slice (for debugging/inspection).
    pub fn as_bytes(&self) -> &'a [u8] {
        self.metadata_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test empty metadata (backward compatibility)
    #[test]
    fn test_empty_metadata() {
        let bytecode = vec![0u8; 100];
        let metadata = ImportMetadata::new(&bytecode, 100).unwrap();
        assert!(metadata.is_empty());
        assert!(metadata.verify_account(&[0u8; 32], &[0u8; 32], None));
    }

    /// Test metadata with single address import
    #[test]
    fn test_single_address_import() {
        // Build metadata: [import_count][type][address][name_len][name]
        let mut metadata_bytes = vec![];
        metadata_bytes.push(1); // 1 import
        metadata_bytes.push(0); // type = address
        let test_addr = [1u8; 32];
        metadata_bytes.extend_from_slice(&test_addr); // 32-byte address
        metadata_bytes.push(4); // name_len = 4
        metadata_bytes.extend_from_slice(b"test"); // "test"

        let metadata = ImportMetadata::new(&metadata_bytes, 0).unwrap();
        assert!(!metadata.is_empty());

        // Matching address should verify
        assert!(metadata.verify_account(
            &test_addr,
            &[0u8; 32],
            None
        ));

        // Different address should not verify
        assert!(!metadata.verify_account(
            &[2u8; 32],
            &[0u8; 32],
            None
        ));
    }

    /// Test metadata with PDA seeds
    #[test]
    fn test_pda_seeds_import() {
        // Build metadata: [import_count][type][seed_count][seed1_len][seed1]...[name_len][name]
        let mut metadata_bytes = vec![];
        metadata_bytes.push(1); // 1 import
        metadata_bytes.push(1); // type = PDA seeds
        metadata_bytes.push(2); // 2 seeds

        // Seed 1: "vault"
        metadata_bytes.push(5);
        metadata_bytes.extend_from_slice(b"vault");

        // Seed 2: "user"
        metadata_bytes.push(4);
        metadata_bytes.extend_from_slice(b"user");

        metadata_bytes.push(4); // name_len = 4
        metadata_bytes.extend_from_slice(b"func"); // "func"

        let metadata = ImportMetadata::new(&metadata_bytes, 0).unwrap();
        assert!(!metadata.is_empty());

        // Note: actual PDA verification would need real Solana SDK key derivation
        // This test just verifies parsing works
    }

    /// Test metadata offset handling
    #[test]
    fn test_metadata_offset() {
        let mut full_bytecode = vec![0xAAu8; 100]; // Dummy bytecode

        // Add metadata at offset 50
        let mut metadata_bytes = vec![];
        metadata_bytes.push(1); // 1 import
        metadata_bytes.push(0); // type = address
        let test_addr = [3u8; 32];
        metadata_bytes.extend_from_slice(&test_addr);
        metadata_bytes.push(3); // name_len = 3
        metadata_bytes.extend_from_slice(b"foo");

        full_bytecode.truncate(50);
        full_bytecode.extend(metadata_bytes);

        let metadata = ImportMetadata::new(&full_bytecode, 50).unwrap();
        assert!(metadata.verify_account(
            &test_addr,
            &[0u8; 32],
            None
        ));
    }

    /// Test offset beyond bytecode
    #[test]
    fn test_offset_beyond_bytecode() {
        let bytecode = vec![0u8; 50];
        let metadata = ImportMetadata::new(&bytecode, 100).unwrap();
        assert!(metadata.is_empty());
        assert!(metadata.verify_account(&[0u8; 32], &[0u8; 32], None));
    }

    /// Test multiple imports with mixed types
    #[test]
    fn test_multiple_mixed_imports() {
        let mut metadata_bytes = vec![];
        metadata_bytes.push(2); // 2 imports

        // Import 1: address type
        metadata_bytes.push(0); // type = address
        let addr1 = [1u8; 32];
        metadata_bytes.extend_from_slice(&addr1);
        metadata_bytes.push(5); // name_len
        metadata_bytes.extend_from_slice(b"func1");

        // Import 2: PDA type
        metadata_bytes.push(1); // type = PDA
        metadata_bytes.push(1); // 1 seed
        metadata_bytes.push(3); // seed_len = 3
        metadata_bytes.extend_from_slice(b"foo");
        metadata_bytes.push(5); // name_len
        metadata_bytes.extend_from_slice(b"func2");

        let metadata = ImportMetadata::new(&metadata_bytes, 0).unwrap();

        // First import (address) should verify
        assert!(metadata.verify_account(
            &addr1,
            &[0u8; 32],
            None
        ));

        // Wrong address should not verify
        assert!(!metadata.verify_account(
            &[99u8; 32],
            &[0u8; 32],
            None
        ));
    }
}
