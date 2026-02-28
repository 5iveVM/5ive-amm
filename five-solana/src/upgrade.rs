// Upgrade mechanism implementation for FIVE VM

use crate::error::FIVEError;
use bytemuck::{Pod, Zeroable};
use pinocchio::{account_info::AccountInfo, program_error::ProgramError, pubkey::Pubkey};
use solana_nostd_sha256::hashv;

/// Enhanced script header with upgrade support
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FIVEScriptHeaderV2 {
    // Existing fields
    pub owner: Pubkey,
    pub script_id: u64,
    pub bytecode_len: u32,

    // Upgrade fields
    pub version: u32,
    pub upgrade_authority: Pubkey,
    pub previous_version_pda: Pubkey,
    pub deployment_slot: u64,
    pub is_immutable: u8,
    pub _padding: [u8; 7], // Align to 8 bytes
}

impl FIVEScriptHeaderV2 {
    pub const LEN: usize = 32 + 8 + 4 + 4 + 32 + 32 + 8 + 1 + 7; // 128 bytes

    pub fn validate(&self, account_data_len: usize) -> Result<(), ProgramError> {
        // Check bytecode length is reasonable
        if self.bytecode_len as usize > five_vm_mito::MAX_SCRIPT_SIZE {
            return Err(ProgramError::Custom(9101));
        }

        // Check account is large enough
        let required_len = Self::LEN + self.bytecode_len as usize;
        if account_data_len < required_len {
            return Err(ProgramError::Custom(9102));
        }

        // Check version is reasonable
        if self.version > 1000 {
            return Err(ProgramError::Custom(9103));
        }

        Ok(())
    }

    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(9104));
        }

        let header: &Self = bytemuck::from_bytes(&data[..Self::LEN]);
        header.validate(data.len())?;
        Ok(header)
    }

    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(9105));
        }

        let data_len = data.len();
        let header: &mut Self = bytemuck::from_bytes_mut(&mut data[..Self::LEN]);
        header.validate(data_len)?;
        Ok(header)
    }

    pub fn get_bytecode<'a>(&self, data: &'a [u8]) -> Result<&'a [u8], ProgramError> {
        let bytecode_start = Self::LEN;
        let bytecode_len = self.bytecode_len as usize;

        if data.len() < bytecode_start + bytecode_len {
            return Err(ProgramError::Custom(9106));
        }

        Ok(&data[bytecode_start..bytecode_start + bytecode_len])
    }
}

/// Version history tracking
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ScriptVersionHistory {
    pub script_id: u64,
    pub current_version: u32,
    pub version_count: u32,
    pub versions: [VersionRecord; 10],
}

impl ScriptVersionHistory {
    pub const LEN: usize = 8 + 4 + 4 + (VersionRecord::LEN * 10);

    pub fn find_version(&self, version: u32) -> Option<&VersionRecord> {
        self.versions
            .iter()
            .take(self.version_count as usize)
            .find(|v| v.version == version && v.is_active != 0)
    }

    pub fn add_version(&mut self, record: VersionRecord) -> Result<(), ProgramError> {
        if self.version_count >= 10 {
            return Err(FIVEError::VersionHistoryFull.into());
        }

        self.versions[self.version_count as usize] = record;
        self.version_count += 1;
        self.current_version = record.version;
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct VersionRecord {
    pub deployment_slot: u64,
    pub version: u32,
    pub is_active: u8,
    pub _padding: [u8; 3],
    pub deployer: Pubkey,
    pub bytecode_hash: [u8; 32],
}

impl VersionRecord {
    pub const LEN: usize = 4 + 8 + 32 + 32 + 1 + 3; // 80 bytes
}

/// Validate upgrade authority
pub fn validate_upgrade_authority(
    authority: &Pubkey,
    proof_account: &AccountInfo,
) -> Result<bool, ProgramError> {
    // Simple single signer check for now
    // Can be extended for multisig, governance, etc.
    if proof_account.key() != authority {
        return Ok(false);
    }

    if !proof_account.is_signer() {
        return Ok(false);
    }

    Ok(true)
}

/// Calculate bytecode hash for verification using Solana-optimized SHA256
pub fn calculate_bytecode_hash(bytecode: &[u8]) -> [u8; 32] {
    hashv(&[bytecode])
}

/// Create a new version record
pub fn create_version_record(
    version: u32,
    bytecode: &[u8],
    deployer: &Pubkey,
    slot: u64,
) -> VersionRecord {
    VersionRecord {
        version,
        deployment_slot: slot,
        deployer: *deployer,
        bytecode_hash: calculate_bytecode_hash(bytecode),
        is_active: 1,
        _padding: [0; 3],
    }
}

/// Archive current version before upgrade
pub fn archive_current_version(
    program_id: &Pubkey,
    script_id: u64,
    version: u32,
    bytecode: &[u8],
) -> Result<Pubkey, ProgramError> {
    // Deterministic placeholder derivation until PDA helpers are available in Pinocchio.
    // Includes program_id + script/version + bytecode hash to avoid collisions.
    let bytecode_hash = calculate_bytecode_hash(bytecode);
    let derived = hashv(&[
        b"archive",
        program_id.as_ref(),
        &script_id.to_le_bytes(),
        &version.to_le_bytes(),
        &bytecode_hash,
    ]);
    Ok(Pubkey::from(derived))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_header_v2_validation() {
        let mut data = vec![0u8; FIVEScriptHeaderV2::LEN + 100];

        // Create valid header
        let header = FIVEScriptHeaderV2 {
            owner: Pubkey::default(),
            script_id: 1,
            bytecode_len: 100,
            version: 1,
            upgrade_authority: Pubkey::default(),
            previous_version_pda: Pubkey::default(),
            deployment_slot: 12345,
            is_immutable: 0,
            _padding: [0; 7],
        };

        // Write header to data
        data[..FIVEScriptHeaderV2::LEN].copy_from_slice(bytemuck::bytes_of(&header));

        // Test deserialization
        let parsed = FIVEScriptHeaderV2::from_account_data(&data).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.bytecode_len, 100);
        assert_eq!(parsed.is_immutable, 0);
    }

    #[test]
    fn test_version_history() {
        let mut history = ScriptVersionHistory {
            script_id: 1,
            current_version: 0,
            version_count: 0,
            versions: [VersionRecord {
                deployment_slot: 0,
                version: 0,
                is_active: 0,
                _padding: [0; 3],
                deployer: Pubkey::default(),
                bytecode_hash: [0; 32],
            }; 10],
        };

        // Add first version
        let record1 = create_version_record(1, b"bytecode_v1", &Pubkey::default(), 1000);
        history.add_version(record1).unwrap();

        assert_eq!(history.version_count, 1);
        assert_eq!(history.current_version, 1);

        // Find version
        let found = history.find_version(1).unwrap();
        assert_eq!(found.version, 1);
        assert_eq!(found.deployment_slot, 1000);
    }

    #[test]
    fn test_bytecode_hash() {
        let bytecode1 = b"test bytecode";
        let bytecode2 = b"different bytecode";

        let hash1 = calculate_bytecode_hash(bytecode1);
        let hash2 = calculate_bytecode_hash(bytecode2);

        // Different bytecode should have different hashes
        assert_ne!(hash1, hash2);

        // Same bytecode should have same hash
        let hash1_again = calculate_bytecode_hash(bytecode1);
        assert_eq!(hash1, hash1_again);
    }

    #[test]
    fn test_archive_pda_derivation() {
        let program_id = Pubkey::default();
        let script_id = 42u64;
        let version = 3u32;

        let archive_pda =
            archive_current_version(&program_id, script_id, version, b"test").unwrap();

        // Verify PDA is deterministic
        let archive_pda2 =
            archive_current_version(&program_id, script_id, version, b"test").unwrap();

        assert_eq!(archive_pda, archive_pda2);
    }

    #[test]
    fn test_version_history_full() {
        let mut history = ScriptVersionHistory {
            script_id: 1,
            current_version: 0,
            version_count: 0,
            versions: [VersionRecord {
                deployment_slot: 0,
                version: 0,
                is_active: 0,
                _padding: [0; 3],
                deployer: Pubkey::default(),
                bytecode_hash: [0; 32],
            }; 10],
        };

        // Fill history with 10 versions
        for i in 0..10 {
            let record =
                create_version_record(i + 1, b"bytecode", &Pubkey::default(), 1000 + i as u64);
            assert!(history.add_version(record).is_ok());
        }

        assert_eq!(history.version_count, 10);

        // Try adding 11th version
        let record11 = create_version_record(11, b"bytecode_v11", &Pubkey::default(), 2000);
        let result = history.add_version(record11);

        // Should return VersionHistoryFull which is 6001
        assert!(matches!(result, Err(ProgramError::Custom(6001))));
    }

    #[test]
    fn test_validate_upgrade_authority() {
        let authority = [1u8; 32];
        let other_key = [2u8; 32];
        let mut lamports = 0;
        let mut data = [];
        let owner = Pubkey::default();

        // 1. Correct authority and signer
        let proof_account = AccountInfo::new(
            &authority,
            true, // is_signer
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        let result = validate_upgrade_authority(&authority, &proof_account).unwrap();
        assert!(result, "Should be valid when key matches and is signer");

        // 2. Correct authority but NOT signer
        let proof_account_unsigned = AccountInfo::new(
            &authority,
            false, // is_signer
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        let result = validate_upgrade_authority(&authority, &proof_account_unsigned).unwrap();
        assert!(!result, "Should be invalid when not signer");

        // 3. Incorrect authority (signer or not)
        let proof_account_wrong = AccountInfo::new(
            &other_key,
            true, // is_signer
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        let result = validate_upgrade_authority(&authority, &proof_account_wrong).unwrap();
        assert!(!result, "Should be invalid when keys mismatch");
    }

    #[test]
    fn test_script_header_v2_validation_failures() {
        let mut data = vec![0u8; FIVEScriptHeaderV2::LEN + 100];
        let mut header = FIVEScriptHeaderV2 {
            owner: Pubkey::default(),
            script_id: 1,
            bytecode_len: 100,
            version: 1,
            upgrade_authority: Pubkey::default(),
            previous_version_pda: Pubkey::default(),
            deployment_slot: 12345,
            is_immutable: 0,
            _padding: [0; 7],
        };

        // 1. Bytecode too large
        header.bytecode_len = five_vm_mito::MAX_SCRIPT_SIZE as u32 + 1;
        // Actually we can use bytemuck::bytes_of but we need to mutate the buffer
        data[..FIVEScriptHeaderV2::LEN].copy_from_slice(bytemuck::bytes_of(&header));

        // validate is called on the struct itself, passing account_data_len
        assert_eq!(header.validate(data.len()), Err(ProgramError::Custom(9101)));
        header.bytecode_len = 100; // Reset

        // 2. Account too small
        assert_eq!(
            header.validate(FIVEScriptHeaderV2::LEN + 50),
            Err(ProgramError::Custom(9102))
        );

        // 3. Version too high
        header.version = 1001;
        assert_eq!(header.validate(data.len()), Err(ProgramError::Custom(9103)));
        header.version = 1; // Reset

        // 4. from_account_data short data
        let short_data = vec![0u8; FIVEScriptHeaderV2::LEN - 1];
        assert_eq!(
            FIVEScriptHeaderV2::from_account_data(&short_data).err(),
            Some(ProgramError::Custom(9104))
        );

        // 5. from_account_data_mut short data
        let mut short_data_mut = vec![0u8; FIVEScriptHeaderV2::LEN - 1];
        assert_eq!(
            FIVEScriptHeaderV2::from_account_data_mut(&mut short_data_mut).err(),
            Some(ProgramError::Custom(9105))
        );

        // 6. get_bytecode short data
        // Restore valid header in data
        data[..FIVEScriptHeaderV2::LEN].copy_from_slice(bytemuck::bytes_of(&header));
        // Truncate data to cut off bytecode
        let truncated_data = &data[..FIVEScriptHeaderV2::LEN + 50]; // Bytecode len is 100

        // get_bytecode is called on &self.
        let valid_header = FIVEScriptHeaderV2::from_account_data(&data).unwrap();
        assert_eq!(
            valid_header.get_bytecode(truncated_data).err(),
            Some(ProgramError::Custom(9106))
        );
    }

    #[test]
    fn test_version_history_find_missing() {
        let history = ScriptVersionHistory {
            script_id: 1,
            current_version: 0,
            version_count: 0,
            versions: [VersionRecord {
                deployment_slot: 0,
                version: 0,
                is_active: 0,
                _padding: [0; 3],
                deployer: Pubkey::default(),
                bytecode_hash: [0; 32],
            }; 10],
        };

        assert!(history.find_version(1).is_none());
    }

    #[test]
    fn test_padding_zeroed() {
        let record = create_version_record(1, b"test", &Pubkey::default(), 123);
        assert_eq!(record._padding, [0; 3]);
    }
}
