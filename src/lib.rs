use ffi::{lean_external_object, lean_object};

mod ffi;
mod map;
mod set;

#[inline]
unsafe fn get_data_from_external<T>(ptr: *mut lean_object) -> *mut T {
    let ptr = ptr as *mut lean_external_object;
    (*ptr).m_data as *mut T
}
