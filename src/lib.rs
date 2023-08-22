#![no_std]

extern crate alloc;

mod ffi;
//mod map;
mod set;

#[cfg(not(test))]
#[no_mangle]
extern "C" fn rust_eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
unsafe fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    lean_sys::panic::panic_handler(info)
}

#[cfg(not(test))]
#[global_allocator]
static ALLOC: lean_sys::alloc::LeanAlloc = lean_sys::alloc::LeanAlloc;
