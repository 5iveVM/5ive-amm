//! Test utilities for FIVE VM testing
//!
//! Provides helper functions for deriving PDAs, creating test accounts, and
//! setting up test fixtures consistently across all tests.
//!
//! Note: These utilities are only available during testing (not in BPF builds).

#![cfg(test)]

use solana_sdk::pubkey::Pubkey;
#[path = "../harness/addresses.rs"]
mod addresses;

/// Derive the VM State PDA using the standard seed ["vm_state"]
///
/// This PDA is used as the canonical VM state account for the FIVE VM program.
/// It should be created and initialized once per program deployment.
///
/// # Arguments
/// * `program_id` - The FIVE VM program ID
///
/// # Returns
/// A tuple of (PDA address, bump seed)
///
/// # Example
/// ```ignore
/// let (vm_state_pda, bump) = derive_vm_state_pda(&program_id);
/// // vm_state_pda is now the deterministic address for this program's VM state
/// ```
pub fn derive_vm_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    addresses::vm_state_pda(program_id)
}

/// Derive a namespace registry PDA for PDA operations
///
/// This matches the pattern used in NamespaceRegistry for PDA namespace tracking.
///
/// # Arguments
/// * `program_id` - The FIVE VM program ID
/// * `prefix` - A single-byte prefix character (e.g., b'$' for default namespace)
/// * `domain` - The domain name as bytes
///
/// # Returns
/// A tuple of (PDA address, bump seed)
pub fn derive_namespace_pda(program_id: &Pubkey, prefix: u8, domain: &[u8]) -> (Pubkey, u8) {
    let prefix_byte = [prefix];
    let seeds: &[&[u8]] = &[b"ns", &prefix_byte, domain];
    Pubkey::find_program_address(seeds, program_id)
}

/// Derive an archive PDA for script versioning
///
/// Used for storing archived versions of scripts during upgrades.
///
/// # Arguments
/// * `program_id` - The FIVE VM program ID
/// * `script_id` - The script ID to archive
/// * `version` - The version number
///
/// # Returns
/// A tuple of (PDA address, bump seed)
pub fn derive_archive_pda(program_id: &Pubkey, script_id: u64, version: u32) -> (Pubkey, u8) {
    let script_id_bytes = script_id.to_le_bytes();
    let version_bytes = version.to_le_bytes();
    let seeds: &[&[u8]] = &[b"archive", &script_id_bytes, &version_bytes];
    Pubkey::find_program_address(seeds, program_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_state_pda_deterministic() {
        // Same program ID should always produce the same PDA
        let program_id = Pubkey::from([1u8; 32]);
        let (pda1, bump1) = derive_vm_state_pda(&program_id);
        let (pda2, bump2) = derive_vm_state_pda(&program_id);

        assert_eq!(pda1, pda2, "VM state PDA should be deterministic");
        assert_eq!(bump1, bump2, "Bump seed should be consistent");
    }

    #[test]
    fn test_vm_state_pda_different_programs() {
        // Different program IDs should produce different PDAs
        let program_id_1 = Pubkey::from([1u8; 32]);
        let program_id_2 = Pubkey::from([2u8; 32]);

        let (pda1, _) = derive_vm_state_pda(&program_id_1);
        let (pda2, _) = derive_vm_state_pda(&program_id_2);

        assert_ne!(pda1, pda2, "Different programs should have different VM state PDAs");
    }

    #[test]
    fn test_namespace_pda_different_prefixes() {
        let program_id = Pubkey::from([1u8; 32]);
        let domain = b"example";

        let (pda_dollar, _) = derive_namespace_pda(&program_id, b'$', domain);
        let (pda_at, _) = derive_namespace_pda(&program_id, b'@', domain);

        assert_ne!(
            pda_dollar, pda_at,
            "Different namespace prefixes should produce different PDAs"
        );
    }

    #[test]
    fn test_namespace_pda_different_domains() {
        let program_id = Pubkey::from([1u8; 32]);

        let (pda1, _) = derive_namespace_pda(&program_id, b'$', b"example1");
        let (pda2, _) = derive_namespace_pda(&program_id, b'$', b"example2");

        assert_ne!(
            pda1, pda2,
            "Different domains should produce different PDAs"
        );
    }

    #[test]
    fn test_archive_pda_deterministic() {
        let program_id = Pubkey::from([1u8; 32]);
        let script_id = 42u64;
        let version = 1u32;

        let (pda1, bump1) = derive_archive_pda(&program_id, script_id, version);
        let (pda2, bump2) = derive_archive_pda(&program_id, script_id, version);

        assert_eq!(pda1, pda2, "Archive PDA should be deterministic");
        assert_eq!(bump1, bump2, "Bump seed should be consistent");
    }

    #[test]
    fn test_archive_pda_different_versions() {
        let program_id = Pubkey::from([1u8; 32]);
        let script_id = 42u64;

        let (pda1, _) = derive_archive_pda(&program_id, script_id, 1);
        let (pda2, _) = derive_archive_pda(&program_id, script_id, 2);

        assert_ne!(
            pda1, pda2,
            "Different versions should produce different PDAs"
        );
    }
}
