use crate::ffi::*;
use core::ops::{Deref, DerefMut};
use hashbrown::raw::{RawIter, RawTable};

type HashedObject = (u64, LeanObject);

#[derive(Clone)]
#[repr(transparent)]
pub struct HashSet(RawTable<HashedObject>);

impl Deref for HashSet {
    type Target = RawTable<HashedObject>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HashSet {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ExternalClass for HashSet {
    unsafe fn foreach<F: Fn(LeanObject)>(&self, f: F) {
        for i in self.iter() {
            f(i.as_ref().1.clone());
        }
    }
}

impl ExternalClass for HashSetIter {
    unsafe fn foreach<F: Fn(LeanObject)>(&self, f: F) {
        match self {
            HashSetIter::More {
                current,
                table,
                next: _,
            } => {
                f((*current).clone());
                table.foreach(f);
            }
            HashSetIter::Finished => {}
        }
    }
}

#[derive(Clone)]
pub enum HashSetIter {
    More {
        current: LeanObject,
        next: RawIter<HashedObject>,
        table: Object<HashSet>,
    },
    Finished,
}

impl HashSetIter {
    fn new(mut iter: RawIter<HashedObject>, table: Object<HashSet>) -> Self {
        match iter.next() {
            Some(current) => Self::More {
                current: unsafe { current.as_ref().1.clone() },
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
                    *current = unsafe { next.as_ref().1.clone() };
                }
                None => {
                    *self = Self::Finished;
                }
            },
            Self::Finished => {}
        }
    }
}

#[no_mangle]
extern "C" fn lean_hashbrown_hashset_create() -> Object<HashSet> {
    HashSet(RawTable::new()).into()
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_get_iter(obj: Object<HashSet>) -> Object<HashSetIter> {
    HashSetIter::new(unsafe { obj.iter() }, obj).into()
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_remove(
    mut obj: Object<HashSet>,
    hash: u64,
    eq_closure: BorrowedLeanObject,
) -> Object<HashSet> {
    let set = obj.make_mut();
    let eq = |x: &HashedObject| unsafe {
        x.0 == hash && {
            let closure = eq_closure.to_owned().into_raw();
            let boxed = { lean_apply_1(closure, x.1.clone().into_raw()) };
            lean_unbox(boxed) != 0
        }
    };
    set.remove_entry(hash, eq);
    obj
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_contains(
    obj: BorrowedObject<HashSet>,
    hash: u64,
    eq_closure: BorrowedLeanObject,
) -> u8 {
    let eq = |x: &HashedObject| {
        x.0 == hash
            && unsafe {
                let closure = eq_closure.to_owned().into_raw();
                let boxed = { lean_apply_1(closure, x.1.clone().into_raw()) };
                lean_unbox(boxed) != 0
            }
    };
    obj.find(hash, eq).is_some() as u8
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_len(obj: BorrowedObject<HashSet>) -> usize {
    obj.len()
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_insert(
    mut obj: Object<HashSet>,
    hash: u64,
    target: LeanObject,
    eq_closure: BorrowedLeanObject,
) -> Object<HashSet> {
    let eq = |x: &HashedObject| {
        x.0 == hash
            && unsafe {
                let closure = eq_closure.to_owned().into_raw();
                let boxed = lean_apply_1(closure, x.1.clone().into_raw());
                lean_unbox(boxed) != 0
            }
    };
    let hasher = |x: &HashedObject| x.0;
    let set = obj.make_mut();
    match set.find_or_find_insert_slot(hash, eq, hasher) {
        Ok(occupied) => unsafe {
            *occupied.as_mut() = (hash, target);
        },
        Err(empty) => unsafe {
            set.insert_in_slot(hash, empty, (hash, target));
        },
    }
    obj
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_iter_has_element(obj: BorrowedObject<HashSetIter>) -> u8 {
    match &*obj {
        HashSetIter::More { .. } => 1,
        HashSetIter::Finished => 0,
    }
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_iter_get_element(
    obj: BorrowedObject<HashSetIter>,
) -> LeanObject {
    option_to_lean(match &*obj {
        HashSetIter::More { current, .. } => Some(current.clone()),
        HashSetIter::Finished => None,
    })
}

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_iter_move_next(
    mut obj: Object<HashSetIter>,
) -> Object<HashSetIter> {
    obj.make_mut().move_next();
    obj
}
