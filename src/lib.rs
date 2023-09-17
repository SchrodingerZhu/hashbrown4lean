#![no_std]
use core::alloc::GlobalAlloc;

use ffi::{lean_align, LEAN_MAX_SMALL_OBJECT_SIZE, LEAN_OBJECT_SIZE_DELTA};

extern crate alloc;

mod ffi;
mod map;
mod set;

#[cfg(not(test))]
#[no_mangle]
extern "C" fn rust_eh_personality() {}

extern "C" {
    fn aligned_alloc(align: usize, size: usize) -> *mut u8;
    fn free(ptr: *mut u8);
}

#[cfg(not(test))]
#[panic_handler]
unsafe fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
    let info = b"liblean_hashbrown rust ffi panics\n\0";
    crate::ffi::lean_internal_panic(info.as_ptr() as _)
}

struct Alloc;

unsafe impl GlobalAlloc for Alloc {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        let alignment = layout.align().max(LEAN_OBJECT_SIZE_DELTA as usize);
        let size = lean_align(layout.size() as u32, alignment as u32) as usize;
        let memory = if alignment == LEAN_OBJECT_SIZE_DELTA as usize
            && size <= LEAN_MAX_SMALL_OBJECT_SIZE as usize
        {
            let slot = ffi::lean_get_slot_idx(size as u32);
            ffi::lean_alloc_small(size as u32, slot) as *mut u8
        } else {
            ffi::lean_inc_heartbeat();
            aligned_alloc(alignment, size)
        };
        if memory.is_null() {
            ffi::lean_internal_panic_out_of_memory()
        } else {
            memory
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: core::alloc::Layout) {
        let alignment = layout.align().max(LEAN_OBJECT_SIZE_DELTA as usize);
        let size = lean_align(layout.size() as u32, alignment as u32) as usize;
        if alignment == LEAN_OBJECT_SIZE_DELTA as usize
            && size <= LEAN_MAX_SMALL_OBJECT_SIZE as usize
        {
            ffi::lean_free_small(ptr as *mut core::ffi::c_void);
        } else {
            free(ptr);
        }
    }
}

#[global_allocator]
static ALLOC: Alloc = Alloc;
