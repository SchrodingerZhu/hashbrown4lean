#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(clippy::useless_transmute)]

use core::{ffi::c_void, marker::PhantomData, ops::Deref};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[inline]
pub fn lean_is_scalar(obj: *mut lean_object) -> bool {
    (obj as usize) & 1 == 1
}

#[inline]
pub unsafe fn lean_is_mt(obj: *mut lean_object) -> bool {
    (*obj).m_rc < 0
}

#[inline]
pub unsafe fn lean_is_st(obj: *mut lean_object) -> bool {
    (*obj).m_rc > 0
}

#[inline]
pub unsafe fn lean_has_rc(obj: *mut lean_object) -> bool {
    (*obj).m_rc != 0
}

#[inline]
pub unsafe fn lean_dec_ref(obj: *mut lean_object) {
    if (*obj).m_rc > 1 {
        (*obj).m_rc -= 1;
    } else if lean_has_rc(obj) {
        lean_dec_ref_cold(obj);
    }
}

#[inline]
pub unsafe fn lean_inc_ref(obj: *mut lean_object) {
    if lean_is_st(obj) {
        (*obj).m_rc += 1;
    } else if lean_has_rc(obj) {
        lean_inc_ref_cold(obj);
    }
}

#[inline]
pub unsafe fn lean_inc_ref_n(obj: *mut lean_object, n: usize) {
    if lean_is_st(obj) {
        (*obj).m_rc += n as i32;
    } else if lean_has_rc(obj) {
        lean_inc_ref_n_cold(obj, n as u32);
    }
}

#[inline]
pub unsafe fn lean_inc(obj: *mut lean_object) {
    if !lean_is_scalar(obj) {
        lean_inc_ref(obj);
    }
}

#[inline]
pub unsafe fn lean_inc_n(obj: *mut lean_object, n: usize) {
    if !lean_is_scalar(obj) {
        lean_inc_ref_n(obj, n);
    }
}

#[inline]
pub unsafe fn lean_dec(obj: *mut lean_object) {
    if !lean_is_scalar(obj) {
        lean_dec_ref(obj);
    }
}

#[inline]
pub unsafe fn lean_is_exclusive(obj: *mut lean_object) -> bool {
    lean_is_st(obj) && (*obj).m_rc == 1
}

#[inline]
pub unsafe fn lean_box(obj: usize) -> lean_obj_res {
    ((obj << 1) | 1) as lean_obj_res
}

#[inline]
pub unsafe fn lean_unbox(obj: lean_obj_res) -> usize {
    obj as usize >> 1
}

#[inline]
pub unsafe fn lean_unbox_uint64(obj: lean_obj_res) -> u64 {
    let obj = obj as *mut lean_ctor_object;
    *((*obj).m_objs.as_ptr() as *const u64)
}

#[inline]
pub unsafe fn lean_align(size: u32, alignment: u32) -> u32 {
    size / alignment * alignment + alignment * (if size % alignment == 0 { 0 } else { 1 })
}

#[inline]
pub unsafe fn lean_get_slot_idx(sz: u32) -> u32 {
    debug_assert!(sz > 0);
    debug_assert_eq!(lean_align(sz, LEAN_OBJECT_SIZE_DELTA), sz);
    sz / LEAN_OBJECT_SIZE_DELTA - 1
}

#[inline]
pub unsafe fn lean_alloc_ctor_memory(sz: u32) -> *mut lean_object {
    let sz1 = lean_align(sz, LEAN_OBJECT_SIZE_DELTA);
    let idx = lean_get_slot_idx(sz1);
    debug_assert!(sz1 <= LEAN_MAX_SMALL_OBJECT_SIZE);
    let r = lean_alloc_small(sz1, idx) as *mut lean_object;
    if sz1 > sz {
        let last = r as usize + sz1 as usize - core::mem::size_of::<usize>();
        *(last as *mut usize) = 0;
    }
    r
}

#[inline]
pub unsafe fn lean_set_st_header(obj: *mut lean_object, tag: u32, other: u32) {
    (*obj).m_rc = 1;
    (*obj).set_m_cs_sz(0);
    (*obj).set_m_tag(tag);
    (*obj).set_m_other(other);
}

#[inline]
pub unsafe fn lean_alloc_ctor(tag: u32, num_objs: u32, scalar_sz: u32) -> *mut lean_object {
    debug_assert!(tag <= LeanMaxCtorTag);
    debug_assert!(num_objs < LEAN_MAX_CTOR_FIELDS);
    debug_assert!(scalar_sz < LEAN_MAX_CTOR_SCALARS_SIZE);
    let obj = lean_alloc_ctor_memory(
        core::mem::size_of::<lean_ctor_object>() as u32
            + num_objs * core::mem::size_of::<*mut lean_object>() as u32
            + scalar_sz,
    );
    lean_set_st_header(obj, tag, num_objs);
    obj
}

#[inline]
pub unsafe fn lean_io_result_mk_ok(obj: lean_obj_arg) -> lean_obj_res {
    let r = lean_alloc_ctor(0, 2, 0);
    {
        let ctor = r as *mut lean_ctor_object;
        let objs = (*ctor).m_objs.as_mut_slice(2);
        objs[0] = obj;
        objs[1] = lean_box(0);
    }
    r
}

#[inline]
pub unsafe fn lean_alloc_small_object(mut sz: u32) -> *mut lean_object {
    sz = lean_align(sz, LEAN_OBJECT_SIZE_DELTA);
    let slot_idx = lean_get_slot_idx(sz);
    debug_assert!(sz <= LEAN_MAX_SMALL_OBJECT_SIZE);
    lean_alloc_small(sz, slot_idx) as *mut lean_object
}

#[inline]
pub unsafe fn lean_alloc_external(
    class: *mut lean_external_class,
    data: *mut c_void,
) -> *mut lean_object {
    let obj = lean_alloc_small_object(core::mem::size_of::<lean_external_object>() as u32);
    lean_set_st_header(obj, LeanExternal, 0);
    {
        let ext = obj as *mut lean_external_object;
        (*ext).m_class = class;
        (*ext).m_data = data;
    }
    obj
}

#[inline]
pub fn option_to_lean(x: Option<LeanObject>) -> LeanObject {
    unsafe {
        match x {
            Some(x) => {
                let ctor = lean_alloc_ctor(1, 1, 0);
                {
                    let ctor = ctor as *mut lean_ctor_object;
                    (*ctor).m_objs.as_mut_slice(1)[0] = x.into_raw();
                }
                LeanObject(ctor)
            }
            None => LeanObject(lean_box(0)),
        }
    }
}

#[repr(transparent)]
pub struct LeanObject(*mut lean_object);

impl Drop for LeanObject {
    fn drop(&mut self) {
        unsafe {
            lean_dec(self.0);
        }
    }
}

impl Clone for LeanObject {
    fn clone(&self) -> Self {
        unsafe {
            lean_inc(self.0);
        }
        Self(self.0)
    }
}

impl LeanObject {
    pub fn into_raw(self) -> *mut lean_object {
        let ptr = self.0;
        core::mem::forget(self);
        ptr
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Object<T>(LeanObject, PhantomData<T>);

impl<T> Deref for Object<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = get_data_from_external::<T>(self.0 .0);
            &*ptr
        }
    }
}

#[inline]
unsafe fn get_data_from_external<T>(ptr: *mut lean_object) -> *mut T {
    let ptr = ptr as *mut lean_external_object;
    (*ptr).m_data as *mut T
}

unsafe extern "C" fn extern_foreach<T: ExternalClass>(
    this: *mut c_void,
    b_closure: b_lean_obj_arg,
) {
    let this = this as *mut T;
    (*this).foreach(|b_obj| {
        lean_inc(b_closure);
        lean_apply_1(b_closure, b_obj.into_raw());
    });
}

unsafe extern "C" fn extern_drop<T: ExternalClass>(this: *mut c_void) {
    let _ = alloc::boxed::Box::from_raw(this as *mut T);
}

pub trait ExternalClass: Sized + 'static {
    unsafe fn foreach<F: Fn(LeanObject)>(&self, f: F);
    const CLASS: lean_external_class = lean_external_class {
        m_foreach: Some(extern_foreach::<Self>),
        m_finalize: Some(extern_drop::<Self>),
    };
}

impl<T: ExternalClass> From<T> for Object<T> {
    fn from(t: T) -> Self {
        unsafe {
            let boxed = alloc::boxed::Box::new(t);
            let ptr = alloc::boxed::Box::into_raw(boxed);
            let obj = lean_alloc_external(&T::CLASS as *const _ as *mut _, ptr as *mut _);
            Object(LeanObject(obj), PhantomData)
        }
    }
}

impl<T: Clone + ExternalClass> Object<T> {
    pub fn make_mut(&mut self) -> &mut T {
        unsafe {
            if !lean_is_exclusive(self.0 .0) {
                *self = (**self).clone().into();
            }
            &mut *get_data_from_external::<T>(self.0 .0)
        }
    }
}

#[repr(transparent)]
pub struct BorrowedLeanObject<'a>(*mut lean_object, PhantomData<&'a lean_object>);

impl<'a> BorrowedLeanObject<'a> {
    pub fn to_owned(&self) -> LeanObject {
        unsafe {
            lean_inc(self.0);
        }
        LeanObject(self.0)
    }
}

#[repr(transparent)]
pub struct BorrowedObject<'a, T>(BorrowedLeanObject<'a>, PhantomData<&'a T>);

impl<'a, T> BorrowedObject<'a, T> {
    #[inline]
    pub fn to_owned(&self) -> Object<T> {
        Object(self.0.to_owned(), PhantomData)
    }
}

impl<T> Deref for BorrowedObject<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr = get_data_from_external::<T>(self.0 .0);
            &*ptr
        }
    }
}
