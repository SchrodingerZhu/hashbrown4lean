import «HashSet»
-- import «HashMap»

-- def constructMap (x : Nat) (acc : HashMap Nat String) : HashMap Nat String :=
--   let acc := acc.insert x (toString x)
--   match x with
--   | Nat.zero => acc
--   | Nat.succ n => constructMap n acc

def constructSet (x : Nat) (acc : HashSet Nat) : HashSet Nat :=
  match x with
  | Nat.zero =>  acc.insert x
  | Nat.succ n => constructSet n <| acc.insert x

def option_test (x : Nat) : Option String :=
  match x with
  | Nat.zero => none
  | Nat.succ n => some (toString n)
def main : IO Unit := do
  let set : HashSet Nat := HashSet.mk
  let set := set.insert 1 |>.insert 2
  -- let set := set.insert 3
  -- let set := set.insert 1
  -- let set := set.remove 3
  IO.println s!"Hello, {set.len}!"
  -- IO.println s!"Hello, {uu.len}!"
