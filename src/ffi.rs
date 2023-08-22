use lean_base::TObj;
pub use lean_sys::*;

#[inline]
pub unsafe fn option_to_lean<T>(x: Option<TObj<T>>) -> lean_obj_res {
    match x {
        Some(x) => {
            let ctor = lean_alloc_ctor(1, 1, 0);
            {
                let ctor = ctor as *mut lean_ctor_object;
                (*ctor).m_objs.as_mut_slice(1)[0] = x.into_raw();
            }
            ctor
        }
        None => lean_box(0),
    }
}
