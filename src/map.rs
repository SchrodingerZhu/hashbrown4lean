use crate::ffi::*;
use core::ops::{Deref, DerefMut};
use hashbrown::raw::{RawIter, RawTable};

#[derive(Clone)]
pub struct HashedPair {
    hash: u64,
    key: LeanObject,
    value: LeanObject,
}

#[derive(Clone)]
pub struct HashMap(RawTable<HashedPair>);

impl Deref for HashMap {
    type Target = RawTable<HashedPair>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HashMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub enum HashMapIter {
    More {
        // current does not count as a reference
        // it is kept alive via table
        current: (LeanObject, LeanObject),
        next: RawIter<HashedPair>,
        table: Object<HashMap>,
    },
    Finished,
}

impl HashMapIter {
    fn new(mut iter: RawIter<HashedPair>, table: Object<HashMap>) -> Self {
        match iter.next() {
            Some(current) => Self::More {
                current: unsafe { (current.as_ref().key.clone(), current.as_ref().value.clone()) },
                next: iter,
                table,
            },
            None => Self::Finished,
        }
    }
    fn move_next(&mut self) {
        match self {
            Self::More {
                next: iter,
                current,
                table: _,
            } => match iter.next() {
                Some(next) => {
                    *current = unsafe { (next.as_ref().key.clone(), next.as_ref().value.clone()) };
                }
                None => {
                    *self = Self::Finished;
                }
            },
            Self::Finished => {}
        }
    }
}

impl ExternalClass for HashMap {
    unsafe fn foreach<F: Fn(LeanObject)>(&self, f: F) {
        for i in self.iter() {
            f(i.as_ref().key.clone());
            f(i.as_ref().value.clone());
        }
    }
}

impl ExternalClass for HashMapIter {
    unsafe fn foreach<F: Fn(LeanObject)>(&self, f: F) {
        match self {
            HashMapIter::More {
                current,
                table,
                next: _,
            } => {
                f(current.0.clone());
                f(current.1.clone());
                table.foreach(f);
            }
            HashMapIter::Finished => {}
        }
    }
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashmap_create() -> Object<HashMap> {
    HashMap(RawTable::new()).into()
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashmap_get_iter(obj: Object<HashMap>) -> Object<HashMapIter> {
    HashMapIter::new(unsafe { obj.iter() }, obj).into()
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashmap_remove(
    mut obj: Object<HashMap>,
    hash: u64,
    eq_closure: BorrowedLeanObject,
) -> Object<HashMap> {
    let map = obj.make_mut();
    let eq = |x: &HashedPair| unsafe {
        x.hash == hash && {
            let closure = eq_closure.to_owned().into_raw();
            let boxed = { lean_apply_1(closure, x.key.clone().into_raw()) };
            lean_unbox(boxed) != 0
        }
    };
    map.remove_entry(hash, eq);
    obj
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_contains(
    obj: BorrowedObject<HashMap>,
    hash: u64,
    eq_closure: BorrowedLeanObject,
) -> u8 {
    let eq = |x: &HashedPair| {
        x.hash == hash
            && unsafe {
                let closure = eq_closure.to_owned().into_raw();
                let boxed = { lean_apply_1(closure, x.key.clone().into_raw()) };
                lean_unbox(boxed) != 0
            }
    };
    obj.find(hash, eq).is_some() as u8
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashmap_get_value(
    obj: BorrowedObject<HashMap>,
    hash: u64,
    eq_closure: BorrowedLeanObject,
) -> LeanObject {
    let eq = |x: &HashedPair| {
        x.hash == hash
            && unsafe {
                let closure = eq_closure.to_owned().into_raw();
                let boxed = { lean_apply_1(closure, x.key.clone().into_raw()) };
                lean_unbox(boxed) != 0
            }
    };
    option_to_lean(
        obj.find(hash, eq)
            .map(|x| unsafe { x.as_ref() }.value.clone()),
    )
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashmap_len(obj: BorrowedObject<HashMap>) -> usize {
    obj.len()
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_insert(
    mut obj: Object<HashMap>,
    hash: u64,
    key: LeanObject,
    value: LeanObject,
    eq_closure: BorrowedLeanObject,
) -> Object<HashMap> {
    let eq = |x: &HashedPair| {
        x.hash == hash
            && unsafe {
                let closure = eq_closure.to_owned().into_raw();
                let boxed = lean_apply_1(closure, x.key.clone().into_raw());
                lean_unbox(boxed) != 0
            }
    };
    let hasher = |x: &HashedPair| x.hash;
    let map = obj.make_mut();
    match map.find_or_find_insert_slot(hash, eq, hasher) {
        Ok(occupied) => unsafe {
            *occupied.as_mut() = HashedPair { hash, key, value };
        },
        Err(empty) => unsafe {
            map.insert_in_slot(hash, empty, HashedPair { hash, key, value });
        },
    }
    obj
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_has_kv(
    iter: BorrowedObject<HashMapIter>,
) -> u8 {
    match &*iter {
        HashMapIter::More { .. } => 1,
        HashMapIter::Finished => 0,
    }
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_get_key(
    obj: BorrowedObject<HashMapIter>,
) -> LeanObject {
    option_to_lean(match &*obj {
        HashMapIter::More { current, .. } => Some(current.0.clone()),
        HashMapIter::Finished => None,
    })
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_get_value(
    obj: BorrowedObject<HashMapIter>,
) -> LeanObject {
    option_to_lean(match &*obj {
        HashMapIter::More { current, .. } => Some(current.1.clone()),
        HashMapIter::Finished => None,
    })
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashmap_iter_move_next(
    mut obj: Object<HashMapIter>,
) -> Object<HashMapIter> {
    obj.make_mut().move_next();
    obj
}
