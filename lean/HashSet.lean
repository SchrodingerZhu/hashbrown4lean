-- Opaque type for HashSet
opaque HashSetPointed : (α : Type) → NonemptyType
def HashSet (α : Type) : Type := (HashSetPointed α).type
instance : Nonempty (HashSet α) := (HashSetPointed α).property

-- Opaque type for HashSetIter
opaque HashSetIterPointed : (α : Type)  → NonemptyType
def HashSetIter (α : Type) : Type := (HashSetIterPointed α).type
instance : Nonempty (HashSetIter α) := (HashSetIterPointed α).property

-- APIs
@[extern "lean_hashbrown_hashset_create"]
opaque HashSet.mk : {α : Type} → HashSet α

@[extern "lean_hashbrown_hashset_insert"]
private opaque HashSet.insertRaw : {α : Type} 
  → HashSet α → UInt64 → α → @&(α → Bool) → HashSet α 

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
opaque HashSetIter.get? : {α : Type} → @& HashSetIter α → Option α

@[extern "lean_hashbrown_hashset_iter_move_next"]
opaque HashSetIter.next : {α : Type} → HashSetIter α → HashSetIter α

def HashSet.insert {α : Type} [Hashable α] [BEq α] (s: HashSet α) (a: α) : HashSet α :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.insertRaw s hash a eq

def HashSet.remove {α : Type} [Hashable α] [BEq α] (s: HashSet α) (a: α) : HashSet α :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.removeRaw s hash eq  

def HashSet.contains {α : Type} [Hashable α] [BEq α] (s: @& HashSet α) (a: α) : Bool :=
  let hash := Hashable.hash a
  let eq := fun (b: α) => a == b
  HashSet.containsRaw s hash eq

private partial def formatTail [Repr α] (acc: Std.Format) (level: Nat) (tail: HashSetIter α) : Std.Format :=
  match tail.get? with
  | some a => 
    let acc := acc ++ ", " ++ Repr.reprPrec a level
    formatTail acc level (tail.next)
  | none => acc

private def formatHashSet [Repr α] (s: HashSet α) (level: Nat) : Std.Format :=
  let iter := HashSet.iter s
  match iter.get? with
  | some hd => "#{" ++ formatTail (Repr.reprPrec hd level) level iter.next ++ "}"
  | none => "#{}"

instance [Repr α] : Repr (HashSet α) where
  reprPrec := formatHashSet

instance [Repr α] : ToString (HashSet α) where
  toString x := Repr.reprPrec x 0 |> Std.Format.pretty
