use crate::error::{CompactResult, Result, VMError, VMErrorCode};
use five_protocol::ValueRef;

#[cfg(target_os = "solana")]
use alloc::vec::Vec;
#[cfg(not(target_os = "solana"))]
use std::vec::Vec;

/// Manages memory for the VM, including temporary stack buffer and dynamic heap.
pub struct MemoryManager<'a> {
    pub temp_buffer: &'a mut [u8],
    pub temp_pos: usize,
    pub heap_storage: Vec<u8>,
}

impl<'a> MemoryManager<'a> {
    #[inline(always)]
    pub fn new(temp_buffer: &'a mut [u8]) -> Self {
        Self {
            temp_buffer,
            temp_pos: 0,
            heap_storage: Vec::with_capacity(512),
        }
    }

    // --- Temp buffer operations ---

    #[inline(always)]
    pub fn alloc_temp(&mut self, size: u8) -> CompactResult<u8> {
        if self.temp_pos + size as usize > self.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        let offset = self.temp_pos;
        self.temp_pos += size as usize;
        Ok(offset as u8)
    }

    #[inline(always)]
    pub fn get_temp_data(&self, offset: u8, size: u8) -> CompactResult<&[u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        if end > self.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        Ok(&self.temp_buffer[start..end])
    }

    #[inline(always)]
    pub fn get_temp_data_mut(&mut self, offset: u8, size: u8) -> CompactResult<&mut [u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        if end > self.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        Ok(&mut self.temp_buffer[start..end])
    }

    #[inline(always)]
    pub fn temp_buffer(&self) -> &[u8] {
        &self.temp_buffer[..]
    }

    #[inline(always)]
    pub fn temp_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.temp_buffer[..]
    }

    /// Allocate a temp buffer slot for Option/Result storage
    #[inline(always)]
    pub fn allocate_temp_slot(&mut self) -> CompactResult<u8> {
        let slot_size = 17u8;
        if self.temp_pos + slot_size as usize > self.temp_buffer.len() {
            return Err(VMErrorCode::MemoryError);
        }
        let offset = self.temp_pos as u8;
        self.temp_pos += slot_size as usize;
        Ok(offset)
    }

    #[inline]
    pub fn temp_buffer_fixed_mut(&mut self) -> Result<&mut [u8; crate::TEMP_BUFFER_SIZE]> {
        // This requires casting or ensuring size.
        // StackStorage guarantees TEMP_BUFFER_SIZE.
        // But here we have a slice.
        // Since we cannot easily return array reference from slice without unsafe or TryInto,
        // and keeping API compatibility, we might need to handle this carefully.
        // However, the original code used `self.storage.temp_buffer` which WAS an array.
        // Here we have a slice.
        // For now, let's skip this one or implement it if possible.
        // The trait `TryInto` works for `&mut [u8]` to `&mut [u8; N]`.

        let ptr = self.temp_buffer.as_mut_ptr();
        unsafe {
             Ok(&mut *(ptr as *mut [u8; crate::TEMP_BUFFER_SIZE]))
        }
    }

    #[inline]
    pub fn write_value_to_temp(&mut self, value: &ValueRef) -> Result<u16> {
        let size = value.serialized_size();

        if self.temp_pos + size > crate::TEMP_BUFFER_SIZE {
            return Err(VMError::MemoryError);
        }

        let offset = self.temp_pos;
        value
            .serialize_into(&mut self.temp_buffer[offset..offset + size])
            .map_err(|_| VMError::ProtocolError)?;
        self.temp_pos += size;
        Ok(offset as u16)
    }

    #[inline]
    pub fn read_value_from_temp(&self, offset: u16) -> Result<ValueRef> {
        if offset as usize >= self.temp_buffer.len() {
            return Err(VMError::MemoryError);
        }

        ValueRef::deserialize_from(&self.temp_buffer[offset as usize..])
            .map_err(|_| VMError::ProtocolError)
    }

    #[inline]
    pub fn temp_offset(&self) -> usize {
        self.temp_pos
    }

    #[inline]
    pub fn set_temp_offset(&mut self, offset: usize) {
        self.temp_pos = offset;
    }

    #[inline]
    pub fn reset_temp_buffer(&mut self) {
        self.temp_pos = 0;
    }

    // --- Heap operations ---

    #[inline]
    pub fn heap_alloc(&mut self, size: usize) -> CompactResult<u32> {
        let offset = self.heap_storage.len();
        self.heap_storage.try_reserve(size).map_err(|_| VMErrorCode::OutOfMemory)?;

        for _ in 0..size {
            self.heap_storage.push(0);
        }

        Ok(offset as u32)
    }

    #[inline]
    pub fn get_heap_data_mut(&mut self, offset: u32, size: u32) -> CompactResult<&mut [u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        if end > self.heap_storage.len() {
             return Err(VMErrorCode::MemoryError);
        }
        Ok(&mut self.heap_storage[start..end])
    }

    #[inline]
    pub fn get_heap_data(&self, offset: u32, size: u32) -> CompactResult<&[u8]> {
        let start = offset as usize;
        let end = start + size as usize;
        if end > self.heap_storage.len() {
             return Err(VMErrorCode::MemoryError);
        }
        Ok(&self.heap_storage[start..end])
    }
}
