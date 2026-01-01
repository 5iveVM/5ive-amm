#![cfg_attr(target_arch = "bpf", no_std)]
#![cfg_attr(target_arch = "bpf", no_main)]

#[cfg(target_arch = "bpf")]
extern crate alloc;

#[cfg(target_arch = "bpf")]
use core::{
    alloc::{GlobalAlloc, Layout},
    panic::PanicInfo,
};

#[cfg(target_arch = "bpf")]
struct NoAlloc;

#[cfg(target_arch = "bpf")]
unsafe impl GlobalAlloc for NoAlloc {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        core::ptr::null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(target_arch = "bpf")]
#[global_allocator]
static A: NoAlloc = NoAlloc;

#[cfg(target_arch = "bpf")]
#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    loop {}
}

#[cfg(target_arch = "bpf")]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[cfg(target_arch = "bpf")]
use five_vm_mito as _;

#[cfg(all(target_arch = "bpf", not(test)))]
#[no_mangle]
pub extern "C" fn main() -> i32 {
    0
}

#[cfg(not(target_arch = "bpf"))]
pub fn main() {
    // Host placeholder so integration tests compile under std targets.
}
