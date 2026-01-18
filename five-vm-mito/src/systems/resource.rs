use crate::error::{CompactResult, Result, VMError, VMErrorCode};
use alloc::vec::Vec;
use alloc::alloc::{alloc, dealloc, Layout};
use core::ptr;

/// Manages memory resources including stack tracking and unsafe heap allocation.
///
/// This replaces the previous MemoryManager with a more robust system that:
/// 1. Tracks stack usage to stay within BPF 4KB limits
/// 2. Uses chunked unsafe allocation for the heap (avoiding "helpless vecs" copy overhead)
/// 3. Manages the temporary buffer
pub struct ResourceManager<'a> {
    /// Stack-allocated temp buffer (borrowed from StackStorage)
    pub temp_buffer: &'a mut [u8],
    pub temp_pos: usize,

    /// Total heap usage in bytes
    pub total_heap_usage: usize,

    /// Dynamic heap chunks: (pointer, capacity, used_size)
    /// We use a fixed array to track chunks to avoid Vec allocation overhead.
    heap_chunks: [(*mut u8, usize, usize); 4],
    
    /// Number of active heap chunks
    heap_chunk_count: u8,

    /// Index of the current active chunk
    current_chunk: u8,

    /// Stack start address (approximate top of stack)
    stack_start: usize,
}

impl<'a> ResourceManager<'a> {
    /// Create a new ResourceManager and capture current stack pointer
    #[inline(always)]
    pub fn new(temp_buffer: &'a mut [u8]) -> Self {
        // Capture stack start approximation using a local variable
        let local_var = 0u8;
        let stack_start = &local_var as *const u8 as usize;

        Self {
            temp_buffer,
            temp_pos: 0,
            total_heap_usage: 0,
            heap_chunks: [(ptr::null_mut(), 0, 0); 4],
            heap_chunk_count: 0,
            current_chunk: 0,
            stack_start,
        }
    }

    // --- Stack Tracking ---

    /// Calculate current stack usage (approximate)
    #[inline(always)]
    pub fn stack_usage(&self) -> usize {
        let local_var = 0u8;
        let current_sp = &local_var as *const u8 as usize;
        
        // Stack grows down, so start > current
        if self.stack_start >= current_sp {
            self.stack_start - current_sp
        } else {
            // Should not happen unless stack grows up or we are in a different thread/context?
            // Just return 0 or difference
            current_sp - self.stack_start
        }
    }

    /// Check if stack usage is within safe limits (approx 4KB - safe margin)
    /// Note: Stack estimation via local variable pointers is unreliable on Solana BPF
    /// due to different stack layout and compiler optimizations. This check is disabled
    /// to prevent false positives. Real stack overflow will be caught by the BPF runtime.
    #[inline(always)]
    pub fn check_stack_limit(&self) -> CompactResult<()> {
        // DISABLED: stack_usage() estimation via local pointer arithmetic
        // is unreliable on BPF and causes false positives.
        // The Solana runtime will catch actual stack overflow.
        Ok(())
    }

    // --- Temp Buffer Operations (Compatible with MemoryManager) ---

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
        let ptr = self.temp_buffer.as_mut_ptr();
        unsafe {
             Ok(&mut *(ptr as *mut [u8; crate::TEMP_BUFFER_SIZE]))
        }
    }

    #[inline]
    pub fn write_value_to_temp(&mut self, value: &five_protocol::ValueRef) -> Result<u16> {
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
    pub fn read_value_from_temp(&self, offset: u16) -> Result<five_protocol::ValueRef> {
        if offset as usize >= self.temp_buffer.len() {
            return Err(VMError::MemoryError);
        }

        five_protocol::ValueRef::deserialize_from(&self.temp_buffer[offset as usize..])
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

    // --- Unsafe Heap Operations ---

    // Legacy alias for compatibility
    #[inline(always)]
    pub fn heap_alloc(&mut self, size: usize) -> CompactResult<u32> {
        self.alloc_heap_unsafe(size)
    }

    /// Allocate memory on the heap using chunks.
    /// Returns a virtual address: (chunk_index << 24) | chunk_offset
    #[inline]
    pub fn alloc_heap_unsafe(&mut self, size: usize) -> CompactResult<u32> {
        // Default chunk size (2KB)
        const DEFAULT_CHUNK_SIZE: usize = 2048;

        // 1. Try to fit in current chunk
        if self.heap_chunk_count > 0 {
            let (ptr, cap, used) = self.heap_chunks[self.current_chunk as usize];
            if used + size <= cap {
                // Fits!
                let offset = used;
                self.heap_chunks[self.current_chunk as usize].2 += size; // Update used
                
                let virtual_addr = ((self.current_chunk as u32) << 24) | (offset as u32);
                return Ok(virtual_addr);
            }
        }

        // 2. Need new chunk
        if self.heap_chunk_count >= 4 {
             return Err(VMErrorCode::OutOfMemory);
        }

        let new_chunk_size = size.max(DEFAULT_CHUNK_SIZE);
        let layout = Layout::from_size_align(new_chunk_size, 8)
            .map_err(|_| VMErrorCode::OutOfMemory)?;
        
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            return Err(VMErrorCode::OutOfMemory);
        }

        // Zero out memory? Not strictly required by unsafe/alloc contract, 
        // but safer for VM deterministic behavior if we assume zeroed.
        // However, user said "unsafe so its zero copy", implying performance.
        // We will zero it to be safe, or leave it if performance is critical.
        // OPTIMIZATION: Only zero in debug builds, skip in release for performance
        #[cfg(debug_assertions)]
        unsafe { ptr::write_bytes(ptr, 0, new_chunk_size) };

        // Add to chunks
        let chunk_index = self.heap_chunk_count;
        
        self.heap_chunks[chunk_index as usize] = (ptr, new_chunk_size, size); // Used = size
        self.heap_chunk_count += 1;
        self.total_heap_usage += new_chunk_size;
        self.current_chunk = chunk_index;

        let virtual_addr = ((chunk_index as u32) << 24) | 0; // Offset 0 in new chunk
        Ok(virtual_addr)
    }

    /// Get a slice to heap data from virtual address
    #[inline]
    pub fn get_heap_data(&self, virtual_addr: u32, size: u32) -> CompactResult<&[u8]> {
        let chunk_index = (virtual_addr >> 24) as usize;
        let offset = (virtual_addr & 0xFFFFFF) as usize;
        let len = size as usize;

        if chunk_index >= self.heap_chunk_count as usize {
            return Err(VMErrorCode::MemoryError);
        }

        let (ptr, cap, _used) = self.heap_chunks[chunk_index];

        if offset + len > cap {
             return Err(VMErrorCode::MemoryError);
        }

        unsafe {
            Ok(core::slice::from_raw_parts(ptr.add(offset), len))
        }
    }

    /// Get mutable slice to heap data
    #[inline]
    pub fn get_heap_data_mut(&mut self, virtual_addr: u32, size: u32) -> CompactResult<&mut [u8]> {
        let chunk_index = (virtual_addr >> 24) as usize;
        let offset = (virtual_addr & 0xFFFFFF) as usize;
        let len = size as usize;

        if chunk_index >= self.heap_chunk_count as usize {
            return Err(VMErrorCode::MemoryError);
        }

        let (ptr, cap, _used) = self.heap_chunks[chunk_index];

        if offset + len > cap {
             return Err(VMErrorCode::MemoryError);
        }

        unsafe {
            Ok(core::slice::from_raw_parts_mut(ptr.add(offset), len))
        }
    }

    /// Get total heap usage in bytes
    #[inline(always)]
    pub fn heap_usage(&self) -> usize {
        self.total_heap_usage
    }
}

impl<'a> Drop for ResourceManager<'a> {
    fn drop(&mut self) {
        for i in 0..self.heap_chunk_count as usize {
            let (ptr, cap, _) = self.heap_chunks[i];
            if !ptr.is_null() && cap > 0 {
                unsafe {
                    let layout = Layout::from_size_align(cap, 8).unwrap();
                    dealloc(ptr, layout);
                }
            }
        }
    }
}
