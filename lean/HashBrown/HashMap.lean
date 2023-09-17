import HashBrown.HashState
namespace HashBrown
namespace HashMap
-- Opaque type for HashMap
opaque HashMapPointed : (κ ν : Type) → NonemptyType
def HashMap (κ ν : Type) : Type := (HashMapPointed κ ν).type
instance : Nonempty (HashMap κ ν) := (HashMapPointed κ ν).property

-- Opaque type for HashMapIter
opaque HashMapIterPointed : (κ ν : Type)  → NonemptyType
def HashMapIter (κ ν : Type) : Type := (HashMapIterPointed κ ν).type
instance : Nonempty (HashMapIter κ ν) := (HashMapIterPointed κ ν).property

@[extern "lean_hashbrown_hashmap_create"]
opaque HashMap.mk : {κ ν : Type} → HashMap κ ν

@[extern "lean_hashbrown_hashmap_insert"]
private opaque HashMap.insertRaw : {κ ν : Type} 
  → HashMap κ ν → UInt64 → κ → ν → @&(κ → Bool) → HashMap κ ν 

@[extern "lean_hashbrown_hashmap_contains"]
private opaque HashMap.containsRaw : {κ ν : Type} 
  → @& HashMap κ ν → UInt64 → @&(κ → Bool) → Bool

@[extern "lean_hashbrown_hashmap_get_value"]
private opaque HashMap.getValueRaw? : {κ ν : Type}
  → @& HashMap κ ν → UInt64 → @&(κ → Bool) → Option ν

@[extern "lean_hashbrown_hashmap_remove"]
private opaque HashMap.removeRaw : {κ ν : Type} 
  → HashMap κ ν → UInt64 → @&(κ → Bool) → HashMap κ ν

@[extern "lean_hashbrown_hashmap_len"]
opaque HashMap.len : {κ ν: Type} → @& HashMap κ ν → USize    

@[extern "lean_hashbrown_hashmap_get_iter"]
opaque HashMap.iter : {κ ν : Type} → HashMap κ ν → HashMapIter κ ν 

@[extern "lean_hashbrown_hashmap_iter_has_kv"]
opaque HashMapIter.hasKV : {κ ν : Type} → @& HashMapIter κ ν → Bool

@[extern "lean_hashbrown_hashmap_iter_get_key"]
opaque HashMapIter.getKey? : {κ ν : Type} → @& HashMapIter κ ν → Option κ

@[extern "lean_hashbrown_hashmap_iter_get_value"]
opaque HashMapIter.getValue? : {κ ν : Type} → @& HashMapIter κ ν → Option ν

@[extern "lean_hashbrown_hashmap_iter_move_next"]
opaque HashMapIter.next :  {κ ν : Type} → HashMapIter κ ν → HashMapIter κ ν

def HashMap.insert {κ ν : Type} [Hashable κ] [BEq κ] (s: HashMap κ ν) (k: κ) (v : ν) : HashMap κ ν :=
  let hash := Hashable.hash k
  let eq := fun (k': κ) => k == k'
  HashMap.insertRaw s hash k v eq

def HashMap.remove {κ ν : Type} [Hashable κ] [BEq κ] (s: HashMap κ ν) (k: κ) : HashMap κ ν :=
  let hash := Hashable.hash k
  let eq := fun (k': κ) => k == k'
  HashMap.removeRaw s hash eq  

def HashMap.contains {κ ν : Type} [Hashable κ] [BEq κ] (s: @& HashMap κ ν) (k: κ) : Bool :=
  let hash := Hashable.hash k
  let eq := fun (k': κ) => k == k'
  HashMap.containsRaw s hash eq  

def HashMap.getValue? {κ ν : Type} [Hashable κ] [BEq κ] (s: @& HashMap κ ν) (k: κ) : Option ν :=
  let hash := Hashable.hash k
  let eq := fun (k': κ) => k == k'
  HashMap.getValueRaw? s hash eq

private partial def formatTail [Repr κ] [Repr ν] (acc: Std.Format) (level: Nat) (tail: HashMapIter κ ν) : Std.Format :=
  match tail.getKey?, tail.getValue? with
  | some k, some v => 
    let acc := acc ++ ", " ++ Repr.reprPrec k level ++ " ⇒ " ++ Repr.reprPrec v level
    formatTail acc level (tail.next)
  | _, _ => acc

private def formatHashMap [Repr κ] [Repr ν] (s: HashMap κ ν) (level: Nat) : Std.Format :=
  let iter := HashMap.iter s
  match iter.getKey?, iter.getValue? with
  | some k, some v => 
    "#{" ++ formatTail ((Repr.reprPrec k level) ++ " ⇒ " ++ (Repr.reprPrec v level)) level iter.next ++ "}"
  | _, _ => "#{}"

instance [Repr κ] [Repr ν] : Repr (HashMap κ ν) where
  reprPrec := formatHashMap

instance [Repr κ] [Repr ν] : ToString (HashMap κ ν) where
  toString x := Repr.reprPrec x 0 |> Std.Format.pretty

instance : EmptyCollection (HashMap κ ν) where
  emptyCollection := HashMap.mk

instance : Inhabited (HashMap κ ν) where
  default := HashMap.mk

structure SeededHashMap (η : Type) (κ : Type) (ν : Type) where
  private mk::
  private map : HashMap κ ν
  private seed : η

def SeededHashMap.insert [HashState.HashState η] [HashState.Hasher η κ] [BEq κ] 
  (s: SeededHashMap η κ ν) (k: κ) (v : ν) : SeededHashMap η κ ν :=
    let updated := HashState.Hasher.update s.seed k
    let hash := HashState.HashState.finish updated
    let eq := fun k' => k == k'
    {
      map := HashMap.insertRaw s.map hash k v eq,
      seed := s.seed
    }

def SeededHashMap.contains [HashState.HashState η] [HashState.Hasher η κ] [BEq κ] 
  (s: SeededHashMap η κ ν) (k: κ) : Bool :=
    let updated := HashState.Hasher.update s.seed k
    let hash := HashState.HashState.finish updated
    let eq := fun k' => k == k'
    HashMap.containsRaw s.map hash eq

def SeededHashMap.remove [HashState.HashState η] [HashState.Hasher η κ] [BEq κ] 
  (s: SeededHashMap η κ ν) (k: κ) : SeededHashMap η κ ν :=
    let updated := HashState.Hasher.update s.seed k
    let hash := HashState.HashState.finish updated
    let eq := fun k' => k == k'
    {
      map := HashMap.removeRaw s.map hash eq,
      seed := s.seed
    }

def SeededHashMap.getValue? [HashState.HashState η] [HashState.Hasher η κ] [BEq κ] 
  (s: SeededHashMap η κ ν) (k: κ) : Option ν :=
    let updated := HashState.Hasher.update s.seed k
    let hash := HashState.HashState.finish updated
    let eq := fun k' => k == k'
    HashMap.getValueRaw? s.map hash eq

def SeededHashMap.len (s: SeededHashMap η κ ν) : USize :=
  HashMap.len s.map

def SeededHashMap.iter (s: SeededHashMap η κ ν) : HashMapIter κ ν :=
  HashMap.iter s.map

instance [Inhabited η] : Inhabited (SeededHashMap η κ ν) where
  default := { map := HashMap.mk, seed := default }

instance [Inhabited η] : EmptyCollection (SeededHashMap η κ ν) where
  emptyCollection := default

instance [Repr κ] [Repr ν] : Repr (SeededHashMap η κ ν) where
  reprPrec := reprPrec ∘ SeededHashMap.map

instance [Repr κ] [Repr ν] : ToString (SeededHashMap η κ ν) where
  toString x := Repr.reprPrec x 0 |> Std.Format.pretty

