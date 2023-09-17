import «HashBrown»

open HashBrown
open HashMap HashSet HashState

def constructMap (x : Nat) (acc : HashMap Nat String) : HashMap Nat String :=
  match x with
  | Nat.zero => acc
  | Nat.succ n => constructMap n <| acc.insert x <| toString x

def constructSet (x : Nat) (acc : SeededHashSet FxMixer Nat := default) : SeededHashSet FxMixer Nat :=
  match x with
  | Nat.zero =>  acc.insert x
  | Nat.succ n => constructSet n <| acc.insert x

partial def hashSet (stdin: IO.FS.Stream) (output: IO.FS.Stream): StateT (SeededHashSet FxMixer String) IO Unit := do 
  let map ← get
  let line ← stdin.getLine
  let str := line.trim
  if str.length == 0
  then output.putStrLn s!"{map}"
  else
    modify (·.insert str)
    hashSet stdin output

def main : IO Unit := do
  let stdin <- IO.getStdin
  let stdout <- IO.getStdout
  hashSet stdin stdout |>.run' default
