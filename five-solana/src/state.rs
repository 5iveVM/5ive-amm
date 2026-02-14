//! State management for Five VM on Solana.

use bytemuck::{Pod, Zeroable};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

/// VM state account that tracks initialization and deployed scripts.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FIVEVMState {
    pub authority: Pubkey,
    pub script_count: u64,
    pub deploy_fee_lamports: u32,  // Flat deploy fee in lamports
    pub execute_fee_lamports: u32, // Flat execute fee in lamports
    pub is_initialized: u8,   // Using u8 instead of bool for bytemuck compatibility
    pub version: u8,
    pub fee_vault_shard_count: u8,
    pub vm_state_bump: u8,    // Canonical vm_state PDA bump for O(1) validation
    pub _padding: [u8; 4],    // Align to 8 bytes
}

impl FIVEVMState {
    pub const VERSION: u8 = 1;
    pub const DEFAULT_FEE_VAULT_SHARD_COUNT: u8 = 10;
    pub const LEN: usize = 32 + 8 + 4 + 4 + 1 + 1 + 1 + 1 + 4; // 56 bytes

    pub fn new() -> Self {
        Self {
            authority: Pubkey::default(),
            script_count: 0,
            deploy_fee_lamports: 0,
            execute_fee_lamports: 0,
            is_initialized: 0,
            version: Self::VERSION,
            fee_vault_shard_count: Self::DEFAULT_FEE_VAULT_SHARD_COUNT,
            vm_state_bump: 0,
            _padding: [0; 4],
        }
    }

    pub fn initialize(&mut self, authority: Pubkey, vm_state_bump: u8) {
        self.authority = authority;
        self.is_initialized = 1;
        self.version = Self::VERSION;
        self.script_count = 0;
        self.deploy_fee_lamports = 10_000;
        // Targeted baseline: ~$50k/month at 3 TPS when SOL is ~$75.
        self.execute_fee_lamports = 85_734;
        self.fee_vault_shard_count = Self::DEFAULT_FEE_VAULT_SHARD_COUNT;
        self.vm_state_bump = vm_state_bump;
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized != 0
    }

    pub fn create_script_id(&mut self) -> u64 {
        let id = self.script_count;
        self.script_count += 1;
        id
    }

    #[inline(always)]
    pub fn fee_vault_shard_count(&self) -> u8 {
        if self.fee_vault_shard_count == 0 {
            Self::DEFAULT_FEE_VAULT_SHARD_COUNT
        } else {
            self.fee_vault_shard_count
        }
    }

    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8001));
        }
        let state = bytemuck::from_bytes::<Self>(&data[..Self::LEN]);
        if state.version != Self::VERSION {
            return Err(ProgramError::Custom(8012));
        }
        Ok(state)
    }

    #[allow(dead_code)]
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8001));
        }
        let state = bytemuck::from_bytes_mut::<Self>(&mut data[..Self::LEN]);
        if state.version == 0 {
            // Deterministically stamp version during controlled migration paths.
            state.version = Self::VERSION;
        } else if state.version != Self::VERSION {
            return Err(ProgramError::Custom(8012));
        }
        Ok(state)
    }
}

impl Default for FIVEVMState {
    fn default() -> Self {
        Self::new()
    }
}

/// Script account header stored at the beginning of each deployed script.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ScriptAccountHeader {
    pub magic: [u8; 4],        // 4 bytes: b"5IVE"
    pub version: u8,           // 1 byte: header version (4)
    pub permissions: u8,       // 1 byte: permission bitmask (NEW)
    pub _reserved0: [u8; 2],   // 2 bytes: reserved
    pub owner: Pubkey,         // 32 bytes: deployer/authority
    pub script_id: u64,        // 8 bytes: script id
    pub bytecode_len: u32,     // 4 bytes: bytecode size
    pub metadata_len: u32,     // 4 bytes: metadata length
    pub func_count: u16,       // 2 bytes: total functions count
    pub _reserved1: [u8; 6],   // 6 bytes: reserved for future use
    // Total: 4+1+1+2+32+8+4+4+2+6 = 64 bytes (exactly)
}


// Legacy alias retained for integration tests while the runtime migrates
#[allow(dead_code)]
pub type FIVEScriptHeader = ScriptAccountHeader;

impl ScriptAccountHeader {
    pub const LEN: usize = 64;
    pub const MAGIC: [u8; 4] = [b'5', b'I', b'V', b'E'];

    pub fn create_from_bytecode(
        bytecode: &[u8],
        owner: Pubkey,
        script_id: u64,
        permissions: u8,
    ) -> Self {
        // Extract total_function_count from bytecode if available (byte 9)
        // Format: magic(4) + features(4) + public(1) + total(1)
        let total_function_count = if bytecode.len() >= 10 { bytecode[9] } else { 0 };

        Self {
            magic: Self::MAGIC,
            version: 4,
            permissions,
            _reserved0: [0; 2],
            owner,
            script_id,
            bytecode_len: bytecode.len() as u32,
            metadata_len: 0,
            func_count: total_function_count as u16,
            _reserved1: [0; 6],
        }
    }

    pub fn new(bytecode_len: usize, owner: Pubkey, script_id: u64) -> Self {
        Self {
            magic: Self::MAGIC,
            version: 4,
            permissions: 0,
            _reserved0: [0; 2],
            owner,
            script_id,
            bytecode_len: bytecode_len as u32,
            metadata_len: 0,
            func_count: 0,
            _reserved1: [0; 6],
        }
    }

    pub fn is_valid(data: &[u8]) -> bool {
        if data.len() < Self::LEN {
            return false;
        }
        // Magic bytes are at offset 0
        &data[0..4] == Self::MAGIC
    }

    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8003));
        }
        let header = bytemuck::from_bytes::<Self>(&data[..Self::LEN]);
        if header.magic != Self::MAGIC {
            return Err(ProgramError::Custom(8002));
        }
        Ok(header)
    }

    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8003));
        }
        let header = bytemuck::from_bytes_mut::<Self>(&mut data[..Self::LEN]);
        if header.magic != Self::MAGIC {
            return Err(ProgramError::Custom(8002));
        }
        Ok(header)
    }

    pub fn bytecode_len(&self) -> usize {
        self.bytecode_len as usize
    }

    pub fn metadata_len(&self) -> usize {
        self.metadata_len as usize
    }

    // Chunked upload tracking (reserved header bytes):
    // - _reserved1[0..4]: current uploaded byte count (little-endian u32)
    // - _reserved1[4]: upload_complete flag (1 = complete)
    // - _reserved1[5]: upload_mode flag (1 = chunked upload in progress)
    pub fn upload_len(&self) -> u32 {
        u32::from_le_bytes([
            self._reserved1[0],
            self._reserved1[1],
            self._reserved1[2],
            self._reserved1[3],
        ])
    }

    pub fn set_upload_len(&mut self, value: u32) {
        self._reserved1[..4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn upload_complete(&self) -> bool {
        self._reserved1[4] == 1
    }

    pub fn set_upload_complete(&mut self, complete: bool) {
        self._reserved1[4] = if complete { 1 } else { 0 };
    }

    pub fn upload_mode(&self) -> bool {
        self._reserved1[5] == 1
    }

    pub fn set_upload_mode(&mut self, enabled: bool) {
        self._reserved1[5] = if enabled { 1 } else { 0 };
    }

    #[allow(dead_code)]
    pub fn set_metadata_len(&mut self, value: usize) {
        self.metadata_len = value as u32;
    }

    #[allow(dead_code)]
    pub fn set_func_count(&mut self, value: u8) {
        self.func_count = value as u16;
    }

    pub fn copy_into_account(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        if dst.len() < Self::LEN {
            return Err(ProgramError::Custom(8004));
        }
        dst[..Self::LEN].copy_from_slice(bytemuck::bytes_of(self));
        Ok(())
    }

    #[allow(dead_code)]
    pub fn bytecode_slice<'a>(&self, data: &'a [u8]) -> Result<&'a [u8], ProgramError> {
        let start = Self::LEN + self.metadata_len();
        let end = start + self.bytecode_len();
        if end > data.len() {
            return Err(ProgramError::Custom(8005));
        }
        Ok(&data[start..end])
    }
}

/// Namespace registry PDA account that tracks namespace ownership and metadata
///
/// PDA Seeds: ["ns", prefix (1 byte), domain (variable)]
/// Example: ["ns", "$", "five"]
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct NamespaceRegistry {
    pub magic: [u8; 4],        // 4 bytes: b"NSRG"
    pub script_count: u32,     // 4 bytes: total bytecode accounts under namespace
    pub owner: Pubkey,         // 32 bytes: namespace owner
    pub registered_at: i64,    // 8 bytes: registration timestamp
    pub expires_at: i64,       // 8 bytes: expiration timestamp
    // Total: 4+4+32+8+8 = 56 bytes
}

impl NamespaceRegistry {
    pub const LEN: usize = 56;
    pub const MAGIC: [u8; 4] = [b'N', b'S', b'R', b'G'];

    /// Create a new namespace registry
    #[allow(dead_code)]
    pub fn new(owner: Pubkey, registered_at: i64, expires_at: i64) -> Self {
        Self {
            magic: Self::MAGIC,
            owner,
            registered_at,
            expires_at,
            script_count: 0,
        }
    }

    /// Check if namespace is valid
    #[allow(dead_code)]
    pub fn is_valid(data: &[u8]) -> bool {
        if data.len() < Self::LEN {
            return false;
        }
        &data[0..4] == Self::MAGIC
    }

    /// Check if namespace registration has expired
    #[allow(dead_code)]
    pub fn is_expired(&self, current_timestamp: i64) -> bool {
        current_timestamp >= self.expires_at
    }

    /// Deserialize namespace registry from account data
    #[allow(dead_code)]
    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8006));
        }
        let registry = bytemuck::from_bytes::<Self>(&data[..Self::LEN]);
        if registry.magic != Self::MAGIC {
            return Err(ProgramError::Custom(8007));
        }
        Ok(registry)
    }

    /// Deserialize mutable namespace registry from account data
    #[allow(dead_code)]
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8006));
        }
        let registry = bytemuck::from_bytes_mut::<Self>(&mut data[..Self::LEN]);
        if registry.magic != Self::MAGIC {
            return Err(ProgramError::Custom(8007));
        }
        Ok(registry)
    }

    #[allow(dead_code)]
    pub fn copy_into_account(&self, dst: &mut [u8]) -> Result<(), ProgramError> {
        if dst.len() < Self::LEN {
            return Err(ProgramError::Custom(8008));
        }
        dst[..Self::LEN].copy_from_slice(bytemuck::bytes_of(self));
        Ok(())
    }

    /// Increment script count in namespace
    #[allow(dead_code)]
    pub fn increment_script_count(&mut self) {
        self.script_count = self.script_count.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        eprintln!("ScriptAccountHeader size: {}", std::mem::size_of::<ScriptAccountHeader>());
        eprintln!("LEN constant: {}", ScriptAccountHeader::LEN);

        let owner = Pubkey::from([1u8; 32]);
        let mut header = ScriptAccountHeader::new(37, owner, 7);
        header.set_metadata_len(5);
        header.set_func_count(3);

        // Allocate: header (LEN) + metadata (5) + bytecode (37)
        let mut data = vec![0u8; ScriptAccountHeader::LEN + 5 + 37];
        header.copy_into_account(&mut data).unwrap();
        data[ScriptAccountHeader::LEN + 5..ScriptAccountHeader::LEN + 5 + 37]
            .copy_from_slice(&[0xAA; 37]);

        let loaded = ScriptAccountHeader::from_account_data(&data).unwrap();
        assert_eq!(loaded.bytecode_len(), 37);
        assert_eq!(loaded.metadata_len(), 5);
        assert_eq!(loaded.func_count, 3);
        let bytecode = loaded.bytecode_slice(&data).unwrap();
        assert_eq!(bytecode.len(), 37);
        assert!(bytecode.iter().all(|b| *b == 0xAA));
    }

    #[test]
    fn namespace_registry_roundtrip() {
        let owner = Pubkey::from([2u8; 32]);
        let registry = NamespaceRegistry::new(owner, 1000, 2000);

        let mut data = vec![0u8; NamespaceRegistry::LEN];
        registry.copy_into_account(&mut data).unwrap();

        let loaded = NamespaceRegistry::from_account_data(&data).unwrap();
        assert_eq!(loaded.owner, owner);
        assert_eq!(loaded.registered_at, 1000);
        assert_eq!(loaded.expires_at, 2000);
        assert_eq!(loaded.script_count, 0);
        assert!(!loaded.is_expired(1500));
        assert!(loaded.is_expired(2000));
    }

    #[test]
    fn test_header_version_4_permissions() {
        // Test that v4 header properly stores and retrieves permissions
        let owner = Pubkey::from([1u8; 32]);
        let header = ScriptAccountHeader::new(10, owner, 42);

        // Default permissions should be 0
        assert_eq!(header.permissions, 0);
        assert_eq!(header.version, 4);

        // Create header with permissions
        let bytecode = vec![0x35, 0x49, 0x56, 0x45]; // 5IVE magic
        let header_with_perms = ScriptAccountHeader::create_from_bytecode(
            &bytecode,
            owner,
            42,
            0x04, // PERMISSION_PDA_SPECIAL_CHARS
        );

        assert_eq!(header_with_perms.permissions, 0x04);

        let mut data = vec![0u8; ScriptAccountHeader::LEN];
        header_with_perms.copy_into_account(&mut data).unwrap();

        let loaded = ScriptAccountHeader::from_account_data(&data).unwrap();
        assert_eq!(loaded.permissions, 0x04);
        assert_eq!(loaded.version, 4);
    }

    #[test]
    fn test_namespace_registry_script_count() {
        let owner = Pubkey::from([3u8; 32]);
        let mut registry = NamespaceRegistry::new(owner, 1000, 2000);

        assert_eq!(registry.script_count, 0);

        registry.increment_script_count();
        assert_eq!(registry.script_count, 1);

        registry.increment_script_count();
        assert_eq!(registry.script_count, 2);
    }

    #[test]
    fn test_namespace_registry_expiration() {
        let owner = Pubkey::from([4u8; 32]);
        let registry = NamespaceRegistry::new(owner, 1000, 3000);

        // Before expiration
        assert!(!registry.is_expired(2999));

        // At expiration time
        assert!(registry.is_expired(3000));

        // After expiration
        assert!(registry.is_expired(3001));
    }

    #[test]
    fn test_header_permissions_preserved_in_roundtrip() {
        let owner = Pubkey::from([5u8; 32]);
        let bytecode = vec![0x35, 0x49, 0x56, 0x45, 0x00, 0x01]; // 5IVE + minor data

        // Test all 3 permission bits individually
        for perm in &[0x01u8, 0x02, 0x04] {
            let header = ScriptAccountHeader::create_from_bytecode(
                &bytecode,
                owner,
                100,
                *perm,
            );

            let mut data = vec![0u8; ScriptAccountHeader::LEN + bytecode.len()];
            header.copy_into_account(&mut data).unwrap();

            let loaded = ScriptAccountHeader::from_account_data(&data).unwrap();
            assert_eq!(loaded.permissions, *perm, "Permission {} not preserved", perm);
        }
    }

    #[test]
    fn test_header_version_4_creation() {
        let owner = Pubkey::from([6u8; 32]);
        let header = ScriptAccountHeader::new(100, owner, 1);

        // Version should be 4
        assert_eq!(header.version, 4);
        // Magic should still be correct
        assert_eq!(header.magic, ScriptAccountHeader::MAGIC);
    }

    #[test]
    fn test_five_vm_state_initialize() {
        let mut vm_state = FIVEVMState::new();
        assert!(!vm_state.is_initialized());
        assert_eq!(vm_state.script_count, 0);

        let authority = Pubkey::from([7u8; 32]);
        vm_state.initialize(authority, 42);

        assert!(vm_state.is_initialized());
        assert_eq!(vm_state.authority, authority);
        assert_eq!(vm_state.version, FIVEVMState::VERSION);
        assert_eq!(vm_state.deploy_fee_lamports, 10_000);
        assert_eq!(vm_state.execute_fee_lamports, 85_734);
        assert_eq!(vm_state.vm_state_bump, 42);
        assert_eq!(
            vm_state.fee_vault_shard_count(),
            FIVEVMState::DEFAULT_FEE_VAULT_SHARD_COUNT
        );
    }

    #[test]
    fn test_five_vm_state_create_script_id() {
        let mut vm_state = FIVEVMState::new();
        vm_state.initialize(Pubkey::default(), 0);

        assert_eq!(vm_state.script_count, 0);

        let id1 = vm_state.create_script_id();
        assert_eq!(id1, 0);
        assert_eq!(vm_state.script_count, 1);

        let id2 = vm_state.create_script_id();
        assert_eq!(id2, 1);
        assert_eq!(vm_state.script_count, 2);
    }

    #[test]
    fn test_five_vm_state_from_account_data_errors() {
        let data_too_short = vec![0u8; FIVEVMState::LEN - 1];

        // Test immutable
        let result = FIVEVMState::from_account_data(&data_too_short);
        assert!(matches!(result, Err(ProgramError::Custom(8001))));

        // Test mutable
        let mut data_too_short_mut = vec![0u8; FIVEVMState::LEN - 1];
        let result_mut = FIVEVMState::from_account_data_mut(&mut data_too_short_mut);
        assert!(matches!(result_mut, Err(ProgramError::Custom(8001))));
    }

    #[test]
    fn test_script_account_header_failures() {
        let short_data = vec![0u8; ScriptAccountHeader::LEN - 1];
        assert_eq!(ScriptAccountHeader::from_account_data(&short_data).err(), Some(ProgramError::Custom(8003)));

        let mut short_data_mut = vec![0u8; ScriptAccountHeader::LEN - 1];
        assert_eq!(ScriptAccountHeader::from_account_data_mut(&mut short_data_mut).err(), Some(ProgramError::Custom(8003)));

        let invalid_magic_data = vec![0u8; ScriptAccountHeader::LEN];
        // Default init is all zeros, so magic is invalid
        assert_eq!(ScriptAccountHeader::from_account_data(&invalid_magic_data).err(), Some(ProgramError::Custom(8002)));

        let header = ScriptAccountHeader::new(10, Pubkey::default(), 1);
        let mut short_dst = vec![0u8; ScriptAccountHeader::LEN - 1];
        assert_eq!(header.copy_into_account(&mut short_dst), Err(ProgramError::Custom(8004)));

        // Bytecode slice error
        let mut data = vec![0u8; ScriptAccountHeader::LEN];
        header.copy_into_account(&mut data).unwrap();
        // data has length LEN (64), but bytecode_len is 10.
        // bytecode_slice expects data.len() >= LEN + metadata + bytecode
        assert_eq!(header.bytecode_slice(&data).err(), Some(ProgramError::Custom(8005)));

        // is_valid checks
        assert!(!ScriptAccountHeader::is_valid(&short_data));
        assert!(!ScriptAccountHeader::is_valid(&invalid_magic_data));
    }

    #[test]
    fn test_namespace_registry_failures() {
         let short_data = vec![0u8; NamespaceRegistry::LEN - 1];
         assert_eq!(NamespaceRegistry::from_account_data(&short_data).err(), Some(ProgramError::Custom(8006)));

         let mut short_data_mut = vec![0u8; NamespaceRegistry::LEN - 1];
         assert_eq!(NamespaceRegistry::from_account_data_mut(&mut short_data_mut).err(), Some(ProgramError::Custom(8006)));

         let invalid_magic_data = vec![0u8; NamespaceRegistry::LEN];
         assert_eq!(NamespaceRegistry::from_account_data(&invalid_magic_data).err(), Some(ProgramError::Custom(8007)));

         let registry = NamespaceRegistry::new(Pubkey::default(), 0, 0);
         let mut short_dst = vec![0u8; NamespaceRegistry::LEN - 1];
         assert_eq!(registry.copy_into_account(&mut short_dst), Err(ProgramError::Custom(8008)));

         // is_valid checks
         assert!(!NamespaceRegistry::is_valid(&short_data));
         assert!(!NamespaceRegistry::is_valid(&invalid_magic_data));
    }
}
