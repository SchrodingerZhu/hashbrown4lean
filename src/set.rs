use std::ffi::c_void;

use crate::{ffi::*, get_data_from_external};
use hashbrown::raw::{RawIter, RawTable};

type HashSet = RawTable<*mut lean_object>;

#[derive(Clone)]
enum HashSetIter {
    More {
        // current does not count as a reference
        // it is kept alive via table
        current: *mut lean_object,
        next: Option<RawIter<*mut lean_object>>,
        table: *mut lean_object,
    },
    Finished,
}

impl HashSetIter {
    unsafe fn from_iter(mut iter: RawIter<*mut lean_object>, table: *mut lean_object) -> Self {
        match iter.next() {
            Some(current) => Self::More {
                current: *current.as_ref(),
                next: Some(iter),
                table,
            },
            None => {
                lean_dec_ref(table);
                Self::Finished
            }
        }
    }
    unsafe fn move_next(&mut self) {
        match self {
            Self::More { table, next, .. } => {
                *self = Self::from_iter(next.take().unwrap_unchecked(), *table);
            }
            Self::Finished => {}
        }
    }
}

static mut HASHSET_CLASS: *mut lean_external_class = std::ptr::null_mut();
static mut HASHSET_ITER_CLASS: *mut lean_external_class = std::ptr::null_mut();

#[no_mangle]
unsafe extern "C" fn lean_hashbrown_hashset_create() -> lean_obj_res {
    let set = HashSet::new();
    let set = Box::new(set);
    let data = Box::into_raw(set) as *mut c_void;
    lean_alloc_external(HASHSET_CLASS, data)
}

unsafe extern "C" fn hashset_finalize(set: *mut c_void) {
    let set = set as *mut HashSet;
    let set = Box::from_raw(set);
    for entry in set.iter() {
        lean_dec(*entry.as_ref());
    }
}

unsafe extern "C" fn hashset_foreach(set: *mut c_void, f: lean_obj_arg) {
    let set = set as *mut HashSet;
    let len = (*set).len();
    if len == 0 {
        lean_dec(f);
        return;
    }
    lean_inc_n(f, len - 1);
    for i in (*set).iter() {
        lean_apply_1(f, *i.as_ref());
    }
}

unsafe extern "C" fn hashset_iter_finalize(iter: *mut c_void) {
    let iter = Box::from_raw(iter as *mut HashSetIter);
    match *iter {
        HashSetIter::More { table, .. } => {
            lean_dec_ref(table);
        }
        HashSetIter::Finished => {}
    }
}

unsafe extern "C" fn hashset_iter_foreach(iter: *mut c_void, f: lean_obj_arg) {
    let iter = iter as *mut HashSetIter;
    match &*iter {
        HashSetIter::More { table, .. } => {
            lean_apply_1(f, *table);
        }
        HashSetIter::Finished => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_register_hashset_class() -> lean_obj_res {
    HASHSET_CLASS = lean_register_external_class(Some(hashset_finalize), Some(hashset_foreach));
    lean_io_result_mk_ok(lean_box(0))
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_register_hashset_iter_class() -> lean_obj_res {
    HASHSET_ITER_CLASS =
        lean_register_external_class(Some(hashset_iter_finalize), Some(hashset_iter_foreach));
    lean_io_result_mk_ok(lean_box(0))
}

unsafe fn exlusive_iter(iter: lean_obj_arg) -> lean_obj_res {
    if lean_is_exclusive(iter) {
        iter
    } else {
        let inner: *mut HashSetIter = get_data_from_external(iter);
        let cloned = Box::into_raw(Box::new((*inner).clone()));
        let new_iter = lean_alloc_external(HASHSET_ITER_CLASS, cloned as *mut c_void);
        match &*cloned {
            HashSetIter::More { table, .. } => {
                lean_inc_ref(*table);
            }
            HashSetIter::Finished => {}
        }
        lean_dec_ref(iter);
        new_iter
    }
}

unsafe fn exlusive_hashset(set: lean_obj_arg) -> lean_obj_res {
    if lean_is_exclusive(set) {
        set
    } else {
        let inner: *mut HashSet = get_data_from_external(set);
        let cloned = Box::into_raw(Box::new((*inner).clone()));
        let new_set = lean_alloc_external(HASHSET_CLASS, cloned as *mut c_void);
        for i in (*inner).iter() {
            lean_inc(*i.as_ref());
        }
        lean_dec_ref(set);
        new_set
    }
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_get_iter(obj: lean_obj_arg) -> lean_obj_res {
    let table = get_data_from_external::<HashSet>(obj);
    // keep the table alive with the iter (no need to dec_ref here)
    let iter = Box::into_raw(Box::new(HashSetIter::from_iter((*table).iter(), obj)));
    lean_alloc_external(HASHSET_ITER_CLASS, iter as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_insert(
    obj: lean_obj_arg,
    hash: u64,
    target: lean_obj_arg,
    eq_closure: lean_obj_arg,
    hash_closure: lean_obj_arg,
) -> lean_obj_res {
    let obj = exlusive_hashset(obj);
    let table = get_data_from_external::<HashSet>(obj);
    let eq = |x: &lean_obj_arg| {
        lean_inc(eq_closure);
        let boxed = lean_apply_1(eq_closure, *x);
        lean_unbox(boxed) != 0
    };
    let hasher = |x: &lean_obj_arg| {
        lean_inc(hash_closure);
        let boxed = lean_apply_1(hash_closure, *x);
        let val = lean_unbox_uint64(boxed);
        lean_dec(boxed);
        val
    };
    match (*table).find_or_find_insert_slot(hash, eq, hasher) {
        Ok(bucket) => {
            let prev = *bucket.as_ref();
            lean_dec(prev);
            *bucket.as_mut() = target;
        }
        Err(slot) => {
            (*table).insert_in_slot(hash, slot, target);
        }
    }
    lean_dec(eq_closure);
    lean_dec(hash_closure);
    obj
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_iter_has_element(obj: lean_obj_arg) -> u8 {
    let iter = get_data_from_external::<HashSetIter>(obj);
    let res = match &*iter {
        HashSetIter::More { .. } => 1,
        HashSetIter::Finished => 0,
    };
    lean_dec_ref(obj);
    res
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_iter_get_element(
    obj: lean_obj_arg,
) -> lean_obj_res {
    let iter = get_data_from_external::<HashSetIter>(obj);
    let res = match &*iter {
        HashSetIter::More { current, .. } => {
            lean_inc(*current);
            *current
        }
        HashSetIter::Finished => {
            let msg = "trying to get an element from finished iterator\0".as_ptr() as *const i8;
            lean_internal_panic(msg)
        }
    };
    lean_dec_ref(obj);
    res
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_iter_move_next(obj: lean_obj_arg) -> lean_obj_res {
    let obj = exlusive_iter(obj);
    let iter = get_data_from_external::<HashSetIter>(obj);
    (*iter).move_next();
    obj
}
