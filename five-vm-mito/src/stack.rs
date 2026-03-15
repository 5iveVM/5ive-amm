use crate::{
    types::{CallFrame, ExternalCacheState},
    MAX_CALL_DEPTH, MAX_LOCALS, STACK_SIZE, TEMP_BUFFER_SIZE,
};
use core::mem::{align_of, size_of};
use five_protocol::ValueRef;

/// Total storage size tuned to keep room for enlarged locals/call frames while
/// reducing heap-allocation pressure vs a full 32KB block.
#[cfg(target_os = "solana")]
pub const STORAGE_SIZE: usize = 28672;
#[cfg(not(target_os = "solana"))]
pub const STORAGE_SIZE: usize = 32768;

// Calculate offsets and sizes
// We use align_of to ensure proper alignment padding
const VALUE_REF_SIZE: usize = size_of::<ValueRef>();
const CALL_FRAME_SIZE: usize = size_of::<CallFrame>();

// 1. Stack (Operand Stack)
const STACK_OFFSET: usize = 0;
const STACK_BYTES: usize = STACK_SIZE * VALUE_REF_SIZE;

// 2. Call Stack
// Align to CallFrame alignment
const CALL_STACK_OFFSET: usize =
    (STACK_OFFSET + STACK_BYTES + (align_of::<CallFrame>() - 1)) & !(align_of::<CallFrame>() - 1);
const CALL_STACK_BYTES: usize = MAX_CALL_DEPTH * CALL_FRAME_SIZE;

// 3. Locals
// Align to ValueRef alignment
const LOCALS_OFFSET: usize = (CALL_STACK_OFFSET + CALL_STACK_BYTES + (align_of::<ValueRef>() - 1))
    & !(align_of::<ValueRef>() - 1);
const LOCALS_BYTES: usize = MAX_LOCALS * VALUE_REF_SIZE;

// 4. Temp Buffer
// Align to 8 bytes (u64 alignment) for safe casting
const TEMP_BUFFER_OFFSET: usize = (LOCALS_OFFSET + LOCALS_BYTES + 7) & !7;
const TEMP_BUFFER_BYTES: usize = TEMP_BUFFER_SIZE;

// 5. Heap Buffer
// Align to 16 bytes (u128 alignment)
const EXTERNAL_CACHE_OFFSET: usize =
    (TEMP_BUFFER_OFFSET + TEMP_BUFFER_BYTES + (align_of::<ExternalCacheState>() - 1))
        & !(align_of::<ExternalCacheState>() - 1);
const EXTERNAL_CACHE_BYTES: usize = size_of::<ExternalCacheState>();
const HEAP_BUFFER_OFFSET: usize = (EXTERNAL_CACHE_OFFSET + EXTERNAL_CACHE_BYTES + 15) & !15;
// Heap takes the rest of the storage
const HEAP_BUFFER_BYTES: usize = STORAGE_SIZE - HEAP_BUFFER_OFFSET;

/// Aggregate storage for all stack-allocated arrays used by the VM.
///
/// This flattened structure matches the Unsafe VM's addressing model where everything
/// is just an offset into a single memory block. It provides zero-copy access to
/// execution state while guaranteeing strict memory safety via bounds checking.
#[repr(C, align(16))] // Ensure 16-byte alignment for the whole block
pub struct StackStorage {
    /// Flattened memory block
    pub memory: [u8; STORAGE_SIZE],
}

impl StackStorage {
    /// Create a new zero-initialized storage block.
    #[inline]
    pub fn new() -> Self {
        // Zero initialization is sufficient as:
        // - ValueRef::Empty is discriminant 0
        // - CallFrame with all 0s is valid (though context 0 refers to account 0)
        // - Unused slots are ignored
        Self {
            memory: [0; STORAGE_SIZE],
        }
    }

    /// Create a new initialized storage block on the HEAP, optimized to avoid stack copies.
    ///
    /// This uses manual allocation and initialization to ensure the large StackStorage struct
    /// is constructed directly in heap memory, bypassing the BPF stack limit (4KB) and
    /// avoiding expensive memcpy operations (~5k CU savings).
    pub fn new_on_heap() -> alloc::boxed::Box<Self> {
        use alloc::alloc::{alloc_zeroed, Layout};
        use alloc::boxed::Box;

        unsafe {
            let layout = Layout::new::<Self>();
            // alloc_zeroed ensures all bytes are 0
            let ptr = alloc_zeroed(layout) as *mut Self;

            // In Solana BPF, alloc failure usually traps, but we check null just in case
            if ptr.is_null() {
                panic!("Memory allocation failed");
            }

            Box::from_raw(ptr)
        }
    }

    /// Create a new initialized storage block at a specific memory location.
    ///
    /// This allows using a pre-allocated static buffer (static mut) to avoid
    /// BOTH stack limit issues and heap allocation/syscall overhead.
    ///
    /// # Safety
    /// Caller must ensure `ptr` points to a valid memory region of sufficient size
    /// and alignment for `StackStorage`.
    pub unsafe fn new_at_ptr(ptr: *mut u8) -> &'static mut Self {
        use core::ptr;

        let storage = &mut *(ptr as *mut Self);

        // Zero out the memory
        ptr::write_bytes(storage.memory.as_mut_ptr(), 0, STORAGE_SIZE);

        storage
    }

    // --- Accessors ---

    /// Get mutable reference to the operand stack
    #[inline(always)]
    pub fn stack_mut(&mut self) -> &mut [ValueRef] {
        unsafe {
            let ptr = self.memory.as_mut_ptr().add(STACK_OFFSET) as *mut ValueRef;
            core::slice::from_raw_parts_mut(ptr, STACK_SIZE)
        }
    }

    /// Get mutable reference to the call stack
    #[inline(always)]
    pub fn call_stack_mut(&mut self) -> &mut [CallFrame] {
        unsafe {
            let ptr = self.memory.as_mut_ptr().add(CALL_STACK_OFFSET) as *mut CallFrame;
            core::slice::from_raw_parts_mut(ptr, MAX_CALL_DEPTH)
        }
    }

    /// Get mutable reference to local variables
    #[inline(always)]
    pub fn locals_mut(&mut self) -> &mut [core::mem::MaybeUninit<ValueRef>] {
        unsafe {
            // Locals are stored as ValueRefs but typed as MaybeUninit<ValueRef> in FrameManager
            // Since ValueRef is Copy, this cast is safe representation-wise
            let ptr = self.memory.as_mut_ptr().add(LOCALS_OFFSET)
                as *mut core::mem::MaybeUninit<ValueRef>;
            core::slice::from_raw_parts_mut(ptr, MAX_LOCALS)
        }
    }

    /// Get mutable reference to temp buffer
    #[inline(always)]
    pub fn temp_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.memory[TEMP_BUFFER_OFFSET..TEMP_BUFFER_OFFSET + TEMP_BUFFER_BYTES]
    }

    /// Get mutable reference to heap buffer
    #[inline(always)]
    pub fn heap_buffer_mut(&mut self) -> &mut [u8] {
        &mut self.memory[HEAP_BUFFER_OFFSET..HEAP_BUFFER_OFFSET + HEAP_BUFFER_BYTES]
    }

    /// Get mutable reference to external call cache state.
    #[inline(always)]
    pub fn external_cache_state_mut(&mut self) -> &mut ExternalCacheState {
        unsafe {
            let ptr =
                self.memory.as_mut_ptr().add(EXTERNAL_CACHE_OFFSET) as *mut ExternalCacheState;
            &mut *ptr
        }
    }

    /// Split storage into mutable slices for all regions.
    /// This allows simultaneous mutable access to disjoint regions which is required
    /// for initializing the ExecutionContext.
    #[inline(always)]
    pub fn split_mut(
        &mut self,
    ) -> (
        &mut [ValueRef],
        &mut [CallFrame],
        &mut [core::mem::MaybeUninit<ValueRef>],
        &mut [u8],
        &mut ExternalCacheState,
        &mut [u8],
    ) {
        unsafe {
            let ptr = self.memory.as_mut_ptr();
            let stack =
                core::slice::from_raw_parts_mut(ptr.add(STACK_OFFSET) as *mut ValueRef, STACK_SIZE);
            let call_stack = core::slice::from_raw_parts_mut(
                ptr.add(CALL_STACK_OFFSET) as *mut CallFrame,
                MAX_CALL_DEPTH,
            );
            // Locals
            let locals = core::slice::from_raw_parts_mut(
                ptr.add(LOCALS_OFFSET) as *mut core::mem::MaybeUninit<ValueRef>,
                MAX_LOCALS,
            );
            // Temp
            let temp =
                core::slice::from_raw_parts_mut(ptr.add(TEMP_BUFFER_OFFSET), TEMP_BUFFER_BYTES);
            let external_cache_state =
                &mut *(ptr.add(EXTERNAL_CACHE_OFFSET) as *mut ExternalCacheState);
            *external_cache_state = ExternalCacheState::empty();
            // Heap
            let heap =
                core::slice::from_raw_parts_mut(ptr.add(HEAP_BUFFER_OFFSET), HEAP_BUFFER_BYTES);
            (stack, call_stack, locals, temp, external_cache_state, heap)
        }
    }
}
