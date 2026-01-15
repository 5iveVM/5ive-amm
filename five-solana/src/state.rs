//! State management for FIVE VM on Solana

use bytemuck::{Pod, Zeroable};
use pinocchio::program_error::ProgramError;
use pinocchio::pubkey::Pubkey;

/// VM State account that tracks VM initialization and deployed scripts
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct FIVEVMState {
    pub authority: Pubkey,
    pub script_count: u64,
    pub deploy_fee_bps: u32,  // BPS of rent for deployment
    pub execute_fee_bps: u32, // BPS of standard tx fee for execution
    pub is_initialized: u8,   // Using u8 instead of bool for bytemuck compatibility
    pub _padding: [u8; 7],    // Align to 8 bytes
}

impl FIVEVMState {
    pub const LEN: usize = 32 + 8 + 4 + 4 + 1 + 7; // 56 bytes

    /// Create a new uninitialized VM state
    pub fn new() -> Self {
        Self {
            authority: Pubkey::default(),
            script_count: 0,
            deploy_fee_bps: 0,
            execute_fee_bps: 0,
            is_initialized: 0,
            _padding: [0; 7],
        }
    }

    /// Initialize the VM state with an authority
    pub fn initialize(&mut self, authority: Pubkey) {
        self.authority = authority;
        self.is_initialized = 1;
        self.script_count = 0;
        self.deploy_fee_bps = 10000;
        self.execute_fee_bps = 10000;
    }

    /// Check if VM is initialized
    pub fn is_initialized(&self) -> bool {
        self.is_initialized != 0
    }

    /// Increment script count and return new ID
    pub fn create_script_id(&mut self) -> u64 {
        let id = self.script_count;
        self.script_count += 1;
        id
    }

    /// Deserialize VM state from account data
    pub fn from_account_data(data: &[u8]) -> Result<&Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8001));
        }

        Ok(bytemuck::from_bytes(&data[..Self::LEN]))
    }

    /// Deserialize mutable VM state from account data
    #[allow(dead_code)]
    pub fn from_account_data_mut(data: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if data.len() < Self::LEN {
            return Err(ProgramError::Custom(8001));
        }
        Ok(bytemuck::from_bytes_mut(&mut data[..Self::LEN]))
    }
}

impl Default for FIVEVMState {
    fn default() -> Self {
        Self::new()
    }
}

/// Script account header stored at the beginning of each deployed script
///
/// **Deploy-Time Verification Strategy:**
/// All bytecode verification happens during deployment:
/// - Valid opcodes (all instructions complete)
/// - Valid CALL targets (< total_function_count)
/// - Function counts validated (public_count <= total_count <= MAX_FUNCTIONS)
/// - Metadata format validated (if present)
///
/// Execute-time trusts this deploy-time verification and only validates:
/// - Stack bounds (Five VM internal)
/// - Memory bounds (Five VM internal)
/// - Account constraints (runtime-dependent)
///
/// **Permissions (v4):**
/// - Bit 0: PERMISSION_PRE_BYTECODE - Can run as pre-execution hook
/// - Bit 1: PERMISSION_POST_BYTECODE - Can run as post-execution hook
/// - Bit 2: PERMISSION_PDA_SPECIAL_CHARS - Can use !, @, #, $, % in PDA seeds
/// - Bits 3-7: Reserved for future use
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

    /// Create header with function count and permissions
    ///
    /// This is called during deployment after bytecode has been verified.
    /// Optimization fields (public_function_count, features, etc) are extracted
    /// from bytecode during execution, trusting deploy-time verification.
    ///
    /// It automatically extracts metadata (like total_function_count) from the bytecode.
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

    /// Legacy constructor for backward compatibility
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
}
