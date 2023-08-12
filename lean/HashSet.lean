-- Opaque type for HashSet
opaque HashSetPointed : (α : Type) → NonemptyType
def HashSet (α : Type) : Type := (HashSetPointed α).type
instance : Nonempty (HashSet α) := (HashSetPointed α).property

-- Opaque type for HashSetIter
opaque HashSetIterPointed : (α : Type)  → NonemptyType
def HashSetIter (α : Type) : Type := (HashSetIterPointed α).type
instance : Nonempty (HashSetIter α) := (HashSetIterPointed α).property

-- Initialization routines
@[extern "lean_hashbrown_register_hashset_class"]
private opaque registerHashSetClass : IO PUnit

@[extern "lean_hashbrown_register_hashset_iter_class"]
private opaque registerHashSetIterClass : IO PUnit

@[init]
private def initModule : IO Unit := do 
 registerHashSetClass
 registerHashSetIterClass

-- APIs
@[extern "lean_hashbrown_hashset_create"]
opaque HashSet.mk : {α : Type} → HashSet α

@[extern "lean_hashbrown_hashset_insert"]
private opaque HashSet.insertRaw : {α : Type} 
  → HashSet α → UInt64 → α → @&(α → Bool) → @&(α → UInt64) → HashSet α 

@[extern "lean_hashbrown_hashset_contains"]
private opaque HashSet.containsRaw : {α : Type} 
  → @& HashSet α → UInt64 → @&(α → Bool) → Bool

@[extern "lean_hashbrown_hashset_remove"]
private opaque HashSet.removeRaw : {α : Type} 
  → HashSet α → UInt64 → @&(α → Bool) → HashSet α

@[extern "lean_hashbrown_hashset_len"]
opaque HashSet.len : {α : Type} → @& HashSet α → USize    

@[extern "lean_hashbrown_hashset_get_iter"]
opaque HashSet.iter : {α : Type} → HashSet α → HashSetIter α 

@[extern "lean_hashbrown_hashset_iter_has_element"]
opaque HashSetIter.hasElement : {α : Type} → @& HashSetIter α → Bool

@[extern "lean_hashbrown_hashset_iter_get_element"]
private opaque HashSetIter.get! : {α : Type} → [Nonempty α] → @& HashSetIter α → α

@[extern "lean_hashbrown_hashset_iter_move_next"]
opaque HashSetIter.next : {α : Type} → HashSetIter α → HashSetIter α

def HashSet.insert {α : Type} [Hashable α] [BEq α] (s: HashSet α) (a: α) : HashSet α :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  let hasher := Hashable.hash
  HashSet.insertRaw s hash a eq hasher

def HashSet.remove {α : Type} [Hashable α] [BEq α] (s: HashSet α) (a: α) : HashSet α :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.removeRaw s hash eq  

def HashSet.contains {α : Type} [Hashable α] [BEq α] (s: @& HashSet α) (a: α) : Bool :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.containsRaw s hash eq

private partial def formatTail [Repr α] [Nonempty α] (acc: Std.Format) (level: Nat) (tail: HashSetIter α) : Std.Format :=
  if tail.hasElement then
    let acc := acc ++ "," ++ Repr.reprPrec tail.get! level
    formatTail acc level (tail.next)
  else
    acc

private def formatHashSet [Repr α] [Nonempty α] (s: HashSet α) (level: Nat) : Std.Format :=
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