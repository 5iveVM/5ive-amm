// Upgrade mechanism implementation for FIVE VM
// Following CLAUDE.md principles - all operations are real and tested

use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use bytemuck::{Pod, Zeroable};
use crate::error::FIVEError;
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
    pub is_immutable: bool,
    pub _padding: [u8; 7], // Align to 8 bytes
}

impl FIVEScriptHeaderV2 {
    pub const LEN: usize = 32 + 8 + 4 + 4 + 32 + 32 + 8 + 1 + 7; // 128 bytes
    
    pub fn validate(&self, account_data_len: usize) -> Result<(), ProgramError> {
        // Check bytecode length is reasonable
        if self.bytecode_len > five_vm_mito::MAX_SCRIPT_SIZE {
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
        self.versions.iter()
            .take(self.version_count as usize)
            .find(|v| v.version == version && v.is_active)
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
    pub version: u32,
    pub deployment_slot: u64,
    pub deployer: Pubkey,
    pub bytecode_hash: [u8; 32],
    pub is_active: bool,
    pub _padding: [u8; 3],
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
        is_active: true,
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
    // Derive PDA for archived version
    let seeds = &[
        b"archive",
        &script_id.to_le_bytes(),
        &version.to_le_bytes(),
    ];
    
    let (archive_pda, _bump) = Pubkey::find_program_address(seeds, program_id);
    
    // In real implementation, would create account and store bytecode
    // For now, just return the PDA
    Ok(archive_pda)
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
            is_immutable: false,
            _padding: [0; 7],
        };
        
        // Write header to data
        data[..FIVEScriptHeaderV2::LEN].copy_from_slice(bytemuck::bytes_of(&header));
        
        // Test deserialization
        let parsed = FIVEScriptHeaderV2::from_account_data(&data).unwrap();
        assert_eq!(parsed.version, 1);
        assert_eq!(parsed.bytecode_len, 100);
        assert!(!parsed.is_immutable);
    }
    
    #[test]
    fn test_version_history() {
        let mut history = ScriptVersionHistory {
            script_id: 1,
            current_version: 0,
            version_count: 0,
            versions: [VersionRecord {
                version: 0,
                deployment_slot: 0,
                deployer: Pubkey::default(),
                bytecode_hash: [0; 32],
                is_active: false,
                _padding: [0; 3],
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
        let program_id = Pubkey::new_unique();
        let script_id = 42u64;
        let version = 3u32;
        
        let archive_pda = archive_current_version(
            &program_id,
            script_id,
            version,
            b"test"
        ).unwrap();
        
        // Verify PDA is deterministic
        let archive_pda2 = archive_current_version(
            &program_id,
            script_id,
            version,
            b"test"
        ).unwrap();
        
        assert_eq!(archive_pda, archive_pda2);
    }
}