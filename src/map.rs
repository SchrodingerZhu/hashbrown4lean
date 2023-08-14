use crate::{ffi::*, get_data_from_external};
use alloc::boxed::Box;
use core::ffi::c_void;
use hashbrown::raw::{RawIter, RawTable};

#[derive(Copy, Clone)]
struct KVPair {
    key: *mut lean_object,
    value: *mut lean_object,
}
type HashMap = RawTable<KVPair>;

#[derive(Clone)]
enum HashMapIter {
    More {
        // current does not count as a reference
        // it is kept alive via table
        current: KVPair,
        next: Option<RawIter<KVPair>>,
        table: *mut lean_object,
    },
    Finished,
}

impl HashMapIter {
    unsafe fn from_iter(mut iter: RawIter<KVPair>, table: *mut lean_object) -> Self {
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

static mut HASHMAP_CLASS: *mut lean_external_class = core::ptr::null_mut();
static mut HASHMAP_ITER_CLASS: *mut lean_external_class = core::ptr::null_mut();

#[no_mangle]
unsafe extern "C" fn lean_hashbrown_hashmap_create() -> lean_obj_res {
    let set = HashMap::new();
    let set = Box::new(set);
    let data = Box::into_raw(set) as *mut c_void;
    lean_alloc_external(HASHMAP_CLASS, data)
}

unsafe extern "C" fn hashmap_finalize(set: *mut c_void) {
    let set = set as *mut HashMap;
    let set = Box::from_raw(set);
    for entry in set.iter() {
        lean_dec(entry.as_ref().key);
        lean_dec(entry.as_ref().value);
    }
}

unsafe extern "C" fn hashmap_foreach(set: *mut c_void, f: lean_obj_arg) {
    let set = set as *mut HashMap;
    let len = (*set).len();
    if len == 0 {
        return;
    }
    lean_inc_n(f, 2 * len);
    for i in (*set).iter() {
        lean_inc(i.as_ref().key);
        lean_inc(i.as_ref().value);
        lean_apply_1(f, i.as_ref().key);
        lean_apply_1(f, i.as_ref().value);
    }
}

unsafe extern "C" fn hashmap_iter_finalize(iter: *mut c_void) {
    let iter = Box::from_raw(iter as *mut HashMapIter);
    match *iter {
        HashMapIter::More { table, .. } => {
            lean_dec_ref(table);
        }
        HashMapIter::Finished => {}
    }
}

unsafe extern "C" fn hashmap_iter_foreach(iter: *mut c_void, f: lean_obj_arg) {
    let iter = iter as *mut HashMapIter;
    match &*iter {
        HashMapIter::More { table, .. } => {
            lean_inc_ref(*table);
            lean_inc(f);
            lean_apply_1(f, *table);
        }
        HashMapIter::Finished => {}
    }
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_register_hashmap_class() -> lean_obj_res {
    HASHMAP_CLASS = lean_register_external_class(Some(hashmap_finalize), Some(hashmap_foreach));
    lean_io_result_mk_ok(lean_box(0))
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_register_hashmap_iter_class() -> lean_obj_res {
    HASHMAP_ITER_CLASS =
        lean_register_external_class(Some(hashmap_iter_finalize), Some(hashmap_iter_foreach));
    lean_io_result_mk_ok(lean_box(0))
}

unsafe fn exlusive_iter(iter: lean_obj_arg) -> lean_obj_res {
    if lean_is_exclusive(iter) {
        iter
    } else {
        let inner: *mut HashMapIter = get_data_from_external(iter);
        let cloned = Box::into_raw(Box::new((*inner).clone()));
        let new_iter = lean_alloc_external(HASHMAP_ITER_CLASS, cloned as *mut c_void);
        match &*cloned {
            HashMapIter::More { table, .. } => {
                lean_inc_ref(*table);
            }
            HashMapIter::Finished => {}
        }
        lean_dec_ref(iter);
        new_iter
    }
}

unsafe fn exlusive_hashmap(set: lean_obj_arg) -> lean_obj_res {
    if lean_is_exclusive(set) {
        set
    } else {
        let inner: *mut HashMap = get_data_from_external(set);
        let cloned = Box::into_raw(Box::new((*inner).clone()));
        let new_set = lean_alloc_external(HASHMAP_CLASS, cloned as *mut c_void);
        for i in (*inner).iter() {
            lean_inc(i.as_ref().key);
            lean_inc(i.as_ref().value);
        }
        lean_dec_ref(set);
        new_set
    }
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_get_iter(obj: lean_obj_arg) -> lean_obj_res {
    let table = get_data_from_external::<HashMap>(obj);
    // keep the table alive with the iter (no need to dec_ref here)
    let iter = Box::into_raw(Box::new(HashMapIter::from_iter((*table).iter(), obj)));
    lean_alloc_external(HASHMAP_ITER_CLASS, iter as *mut c_void)
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_remove(
    obj: lean_obj_arg,
    hash: u64,
    eq_closure: lean_obj_arg,
) -> lean_obj_res {
    let obj = exlusive_hashmap(obj);
    let table = get_data_from_external::<HashMap>(obj);
    let eq = |x: &KVPair| {
        lean_inc(x.key);
        lean_inc(eq_closure);
        let boxed = lean_apply_1(eq_closure, x.key);
        lean_unbox(boxed) != 0
    };
    if let Some(x) = (*table).remove_entry(hash, eq) {
        lean_dec(x.key);
        lean_dec(x.value);
    }
    obj
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_contains(
    obj: lean_obj_arg,
    hash: u64,
    eq_closure: lean_obj_arg,
) -> u8 {
    let table = get_data_from_external::<HashMap>(obj);
    let eq = |x: &KVPair| {
        lean_inc(x.key);
        lean_inc(eq_closure);
        let boxed = lean_apply_1(eq_closure, x.key);
        lean_unbox(boxed) != 0
    };

    (*table).find(hash, eq).is_some() as u8
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_get_value(
    obj: lean_obj_arg,
    hash: u64,
    eq_closure: lean_obj_arg,
) -> lean_obj_res {
    let table = get_data_from_external::<HashMap>(obj);
    let eq = |x: &KVPair| {
        lean_inc(x.key);
        lean_inc(eq_closure);
        let boxed = lean_apply_1(eq_closure, x.key);
        lean_unbox(boxed) != 0
    };

    option_to_lean(match (*table).find(hash, eq) {
        Some(kv) => {
            lean_inc(kv.as_ref().value);
            Some(kv.as_ref().value)
        }
        None => None,
    })
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_len(obj: lean_obj_arg) -> usize {
    let table = get_data_from_external::<HashMap>(obj);

    (*table).len()
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_insert(
    obj: lean_obj_arg,
    hash: u64,
    key: lean_obj_arg,
    value: lean_obj_arg,
    eq_closure: lean_obj_arg,
    hash_closure: lean_obj_arg,
) -> lean_obj_res {
    let obj = exlusive_hashmap(obj);
    let table = get_data_from_external::<HashMap>(obj);
    let eq = |x: &KVPair| {
        lean_inc(x.key);
        lean_inc(eq_closure);
        let boxed = lean_apply_1(eq_closure, x.key);
        lean_unbox(boxed) != 0
    };
    let hasher = |x: &KVPair| {
        lean_inc(x.key);
        lean_inc(hash_closure);
        let boxed = lean_apply_1(hash_closure, x.key);
        let val = lean_unbox_uint64(boxed);
        lean_dec(boxed);
        val
    };
    match (*table).find_or_find_insert_slot(hash, eq, hasher) {
        Ok(bucket) => {
            let prev = *bucket.as_ref();
            lean_dec(prev.key);
            lean_dec(prev.value);
            *bucket.as_mut() = KVPair { key, value };
        }
        Err(slot) => {
            (*table).insert_in_slot(hash, slot, KVPair { key, value });
        }
    }
    obj
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_has_kv(obj: lean_obj_arg) -> u8 {
    let iter = get_data_from_external::<HashMapIter>(obj);

    match &*iter {
        HashMapIter::More { .. } => 1,
        HashMapIter::Finished => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_get_key(obj: lean_obj_arg) -> lean_obj_res {
    let iter = get_data_from_external::<HashMapIter>(obj);

    option_to_lean(match &*iter {
        HashMapIter::More { current, .. } => {
            lean_inc(current.key);
            Some(current.key)
        }
        HashMapIter::Finished => None,
    })
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_get_value(obj: lean_obj_arg) -> lean_obj_res {
    let iter = get_data_from_external::<HashMapIter>(obj);

    option_to_lean(match &*iter {
        HashMapIter::More { current, .. } => {
            lean_inc(current.value);
            Some(current.value)
        }
        HashMapIter::Finished => None,
    })
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_move_next(obj: lean_obj_arg) -> lean_obj_res {
    let obj = exlusive_iter(obj);
    let iter = get_data_from_external::<HashMapIter>(obj);
    (*iter).move_next();
    obj
}
