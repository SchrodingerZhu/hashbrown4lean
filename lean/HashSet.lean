import Init.Data.Format.Basic


-- Opque type for HashSet
opaque HashSetPointed : (α : Type) → NonemptyType
def HashSet (α : Type) : Type := (HashSetPointed α).type
instance : Nonempty (HashSet α) := (HashSetPointed α).property

-- Opque type for HashSetIter
opaque HashSetIterPointed : (α : Type)  → NonemptyType
def HashSetIter (α : Type) : Type := (HashSetIterPointed α).type
instance : Nonempty (HashSetIter α) := (HashSetIterPointed α).property

-- Initialization routines
@[extern "lean_hashbrown_register_hashset_class"]
opaque register_hashset_class : IO PUnit

@[extern "lean_hashbrown_register_hashset_iter_class"]
opaque register_hashset_iter_class : IO PUnit

@[init]
def init_module : IO Unit := do 
 register_hashset_class
 register_hashset_iter_class

-- APIs
@[extern "lean_hashbrown_hashset_create"]
opaque HashSet.mk : {α : Type} → HashSet α

@[extern "lean_hashbrown_hashset_insert"]
opaque HashSet.insert_raw : {α : Type} 
  → HashSet α → UInt64 → α → @&(α → Bool) → @&(α → UInt64) → HashSet α 

@[extern "lean_hashbrown_hashset_contains"]
opaque HashSet.contains_raw : {α : Type} 
  → @& HashSet α → UInt64 → @&(α → Bool) → Bool

@[extern "lean_hashbrown_hashset_remove"]
opaque HashSet.remove_raw : {α : Type} 
  → HashSet α → UInt64 → @&(α → Bool) → HashSet α

@[extern "lean_hashbrown_hashset_len"]
opaque HashSet.len : {α : Type} → @& HashSet α → USize    

@[extern "lean_hashbrown_hashset_get_iter"]
opaque HashSet.iter : {α : Type} → HashSet α → HashSetIter α 

@[extern "lean_hashbrown_hashset_iter_has_element"]
opaque HashSetIter.hasElement : {α : Type} → @& HashSetIter α → Bool

@[extern "lean_hashbrown_hashset_iter_get_element"]
opaque HashSetIter.get! : {α : Type} → [Nonempty α] → @& HashSetIter α → α

@[extern "lean_hashbrown_hashset_iter_move_next"]
opaque HashSetIter.next : {α : Type} → HashSetIter α → HashSetIter α

def HashSet.insert {α : Type} [Hashable α] [BEq α] (s: HashSet α) (a: α) : HashSet α :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  let hasher := Hashable.hash
  HashSet.insert_raw s hash a eq hasher

def HashSet.remove {α : Type} [Hashable α] [BEq α] (s: HashSet α) (a: α) : HashSet α :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.remove_raw s hash eq  

def HashSet.contains {α : Type} [Hashable α] [BEq α] (s: @& HashSet α) (a: α) : Bool :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.contains_raw s hash eq

partial def formatTail [Repr α] [Nonempty α] (acc: Std.Format) (level: Nat) (tail: HashSetIter α) : Std.Format :=
  if tail.hasElement then
    let acc := acc ++ "," ++ Repr.reprPrec tail.get! level
    formatTail acc level (tail.next)
  else
    acc

def formatHashSet [Repr α] [Nonempty α] (s: HashSet α) (level: Nat) : Std.Format :=
  let iter := HashSet.iter s
  if iter.hasElement
  then
    "#{" ++ formatTail (Repr.reprPrec iter.get! level) level iter.next ++ "}"
  else "#{}"
  
def HashSetIter.get? {α : Type} [Nonempty α] (iter: @& HashSetIter α) : Option α :=
  if iter.hasElement then some iter.get! else none  

instance [Repr α] [Nonempty α] : Repr (HashSet α) where
  reprPrec := formatHashSet

instance [Repr α] [Nonempty α] : ToString (HashSet α) where
  toString x := Repr.reprPrec x 0 |> Std.Format.pretty