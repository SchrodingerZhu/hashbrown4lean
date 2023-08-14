#![no_std]
use core::alloc::GlobalAlloc;

use ffi::{lean_external_object, lean_internal_panic_out_of_memory, lean_object};
extern crate alloc;

mod ffi;
mod map;
mod set;

#[inline]
unsafe fn get_data_from_external<T>(ptr: *mut lean_object) -> *mut T {
    let ptr = ptr as *mut lean_external_object;
    (*ptr).m_data as *mut T
}

#[cfg(not(test))]
#[no_mangle]
extern "C" fn rust_eh_personality() {}

extern "C" {
    fn malloc(size: usize) -> *mut u8;
    fn aligned_alloc(align: usize, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

#[cfg(not(test))]
#[panic_handler]
unsafe fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    let info = "liblean_hashbrown rust ffi panics\n";
    lean_internal_panic(info.as_ptr() as _);
}

struct LibcAlloc;

unsafe impl GlobalAlloc for LibcAlloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let memory = if layout.align() <= core::mem::align_of::<usize>() {
            malloc(layout.size())
        } else {
            let alignment = layout.align();
            let size = layout.size();
            let aligned: usize = size + (size.wrapping_neg() & alignment.wrapping_sub(1));
            aligned_alloc(layout.align(), aligned)
        };
        if memory.is_null() {
            lean_internal_panic_out_of_memory()
        } else {
            memory
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: core::alloc::Layout) {
        free(ptr)
    }
}

#[global_allocator]
static ALLOC: LibcAlloc = LibcAlloc;
