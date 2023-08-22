use core::{
    ffi::c_void,
    ops::{Deref, DerefMut},
};

use hashbrown::raw::RawTable;
use lean_base::{
    closure::Closure1,
    external::{AsExternalObj, External, ForeachObj},
    Obj, TObj, TObjRef,
};

#[derive(Clone)]
pub struct HashedObject {
    hash: u64,
    object: TObj<c_void>,
}

#[derive(Clone, Default)]
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

unsafe impl ForeachObj for HashSet {
    fn foreach_obj<F: Fn(Obj)>(&self, f: &F) {
        unsafe {
            for i in self.0.iter() {
                let obj = i.as_ref().clone();
                f(obj.object.into_obj());
            }
        }
    }
}

impl AsExternalObj for HashSet {}

pub type LeanHashSet = TObj<External<HashSet>>;
pub type LeanHashSetRef<'a> = TObjRef<'a, External<HashSet>>;

#[no_mangle]
pub extern "C" fn lean_hashbrown_hashset_create() -> LeanHashSet {
    LeanHashSet::from(HashSet::default())
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_insert(
    mut set: LeanHashSet,
    hash: u64,
    object: TObj<c_void>,
    eq: TObjRef<Closure1<bool, c_void>>,
) -> LeanHashSet {
    let mutator = set.make_mut();
    match mutator.find_or_find_insert_slot(
        hash,
        |x| x.hash == hash && eq.to_owned().invoke(x.object.clone()).unpack(),
        |x| x.hash,
    ) {
        Ok(bucket) => {
            bucket.as_mut().object = object;
        }
        Err(slot) => {
            mutator.insert_in_slot(hash, slot, HashedObject { hash, object });
        }
    }
    set
}

#[no_mangle]
pub unsafe extern "C" fn lean_hashbrown_hashset_contains(
    set: LeanHashSetRef,
    hash: u64,
    eq: TObjRef<Closure1<bool, c_void>>,
) -> bool {
    set.find(hash, |x| {
        x.hash == hash && eq.to_owned().invoke(x.object.clone()).unpack()
    })
    .is_some()
}
