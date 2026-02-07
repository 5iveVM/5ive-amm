use std::collections::HashMap;

use five_vm_mito::error::VMError;

/// Constant pool builder with deduplication.
#[derive(Default)]
pub struct ConstantPoolBuilder {
    pool: Vec<[u8; 8]>,
    map_u64: HashMap<u64, u16>,
    map_u128: HashMap<u128, u16>,
    map_pubkey: HashMap<[u8; 32], u16>,
    map_string: HashMap<Vec<u8>, u16>,
    string_blob: Vec<u8>,
}

impl ConstantPoolBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pool_slots(&self) -> u16 {
        self.pool.len() as u16
    }

    pub fn string_blob(&self) -> &[u8] {
        &self.string_blob
    }

    pub fn pool_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.pool.len() * 8);
        for slot in &self.pool {
            out.extend_from_slice(slot);
        }
        out
    }

    fn ensure_capacity(&self, slots_needed: usize) -> Result<(), VMError> {
        let total = self.pool.len() + slots_needed;
        if total > u16::MAX as usize {
            return Err(VMError::InvalidScript);
        }
        Ok(())
    }

    pub fn add_u64(&mut self, value: u64) -> Result<u16, VMError> {
        if let Some(&idx) = self.map_u64.get(&value) {
            return Ok(idx);
        }
        self.ensure_capacity(1)?;
        let idx = self.pool.len() as u16;
        self.pool.push(value.to_le_bytes());
        self.map_u64.insert(value, idx);
        Ok(idx)
    }

    pub fn add_u128(&mut self, value: u128) -> Result<u16, VMError> {
        if let Some(&idx) = self.map_u128.get(&value) {
            return Ok(idx);
        }
        self.ensure_capacity(2)?;
        let idx = self.pool.len() as u16;
        let bytes = value.to_le_bytes();
        self.pool.push(bytes[0..8].try_into().unwrap());
        self.pool.push(bytes[8..16].try_into().unwrap());
        self.map_u128.insert(value, idx);
        Ok(idx)
    }

    pub fn add_pubkey(&mut self, value: &[u8; 32]) -> Result<u16, VMError> {
        if let Some(&idx) = self.map_pubkey.get(value) {
            return Ok(idx);
        }
        self.ensure_capacity(4)?;
        let idx = self.pool.len() as u16;
        self.pool.push(value[0..8].try_into().unwrap());
        self.pool.push(value[8..16].try_into().unwrap());
        self.pool.push(value[16..24].try_into().unwrap());
        self.pool.push(value[24..32].try_into().unwrap());
        self.map_pubkey.insert(*value, idx);
        Ok(idx)
    }

    pub fn add_string(&mut self, value: &[u8]) -> Result<u16, VMError> {
        if let Some(&idx) = self.map_string.get(value) {
            return Ok(idx);
        }
        self.ensure_capacity(1)?;
        let idx = self.pool.len() as u16;

        let offset = self.string_blob.len() as u32;
        let len = value.len() as u32;
        self.string_blob.extend_from_slice(value);

        let mut slot = [0u8; 8];
        slot[0..4].copy_from_slice(&offset.to_le_bytes());
        slot[4..8].copy_from_slice(&len.to_le_bytes());
        self.pool.push(slot);
        self.map_string.insert(value.to_vec(), idx);
        Ok(idx)
    }
}
