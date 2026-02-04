use crate::{
    types::{CallFrame, LocalVariables},
    MAX_CALL_DEPTH, MAX_LOCALS, STACK_SIZE, TEMP_BUFFER_SIZE,
};
use five_protocol::ValueRef;

/// Aggregate storage for all stack-allocated arrays used by the VM.
///
/// This keeps all large arrays in a single struct that can live on the
/// stack and be borrowed by [`ExecutionContext`]. This avoids heap usage
/// while providing zero-copy access to execution state.
pub struct StackStorage<'a> {
    /// Operand stack
    pub stack: [ValueRef; STACK_SIZE],
    /// Function call frames
    pub call_stack: [CallFrame<'a>; MAX_CALL_DEPTH],
    /// Local variables
    pub locals: LocalVariables,
    /// Temporary byte buffer
    pub temp_buffer: [u8; TEMP_BUFFER_SIZE],
    /// Static heap buffer (avoids initial alloc)
    pub heap_buffer: [u8; 1024],
}

impl<'a> StackStorage<'a> {
    /// Create a new initialized storage block for a given script.
    #[inline]
    pub fn new(bytecode: &'a [u8]) -> Self {
        Self {
            stack: [ValueRef::Empty; STACK_SIZE],
            call_stack: [CallFrame::new(0, 0, 0, bytecode); MAX_CALL_DEPTH],
            locals: [core::mem::MaybeUninit::uninit(); MAX_LOCALS],
            temp_buffer: [0; TEMP_BUFFER_SIZE],
            heap_buffer: [0; 1024],
        }
    }

    /// Create a new initialized storage block on the HEAP, optimized to avoid stack copies.
    ///
    /// This uses manual allocation and initialization to ensure the large StackStorage struct
    /// is constructed directly in heap memory, bypassing the BPF stack limit (4KB) and
    /// avoiding expensive memcpy operations (~5k CU savings).
    pub fn new_on_heap(bytecode: &'a [u8]) -> alloc::boxed::Box<Self> {
        use alloc::alloc::{alloc, Layout};
        use alloc::boxed::Box;
        use core::ptr;

        unsafe {
            let layout = Layout::new::<Self>();
            let ptr = alloc(layout) as *mut Self;
            
            // In Solana BPF, alloc failure usually traps, but we check null just in case
            if ptr.is_null() {
                // Return null pointer disguised as Box? No, just panic/trap.
                panic!("Memory allocation failed");
            }
            
            let storage = &mut *ptr;
            
            // Initialize fields one by one to avoid stack struct creation
            
            // 1. Stack
            for i in 0..STACK_SIZE {
                storage.stack[i] = ValueRef::Empty;
            }
            
            // 2. Call Stack
            for i in 0..MAX_CALL_DEPTH {
                storage.call_stack[i] = CallFrame::new(0, 0, 0, bytecode);
            }
            
            // 3. Locals - Skipped for Zero-Cost Initialization
            // We use MaybeUninit, so we don't need to initialize them.
            // The FrameManager tracks valid locals via local_count.

            // 4. Temp Buffer
            // Zero out temp buffer efficiently
            ptr::write_bytes(storage.temp_buffer.as_mut_ptr(), 0, TEMP_BUFFER_SIZE);
            
            // 6. Heap Buffer
            ptr::write_bytes(storage.heap_buffer.as_mut_ptr(), 0, 1024);

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
    pub unsafe fn new_at_ptr(ptr: *mut u8, bytecode: &'a [u8]) -> &'a mut Self {
        use core::ptr;
        
        // Assert pointer alignment at runtime if needed, but we trust caller for now.
        // StackStorage typically needs 8 or 16 byte alignment.
        // VM_HEAP is u128 aligned (16 bytes), so it should be fine.
        
        let storage = &mut *(ptr as *mut Self);
        
        // Initialize fields one by one (In-Place)
        
        // 1. Stack
        for i in 0..STACK_SIZE {
            storage.stack[i] = ValueRef::Empty;
        }
        
        // 2. Call Stack
        for i in 0..MAX_CALL_DEPTH {
            storage.call_stack[i] = CallFrame::new(0, 0, 0, bytecode);
        }
        
        // 3. Locals (Skipped - MaybeUninit)

        // 4. Temp Buffer
        ptr::write_bytes(storage.temp_buffer.as_mut_ptr(), 0, TEMP_BUFFER_SIZE);
        
        // 6. Heap Buffer
        ptr::write_bytes(storage.heap_buffer.as_mut_ptr(), 0, 1024);
        
        storage
    }
}
