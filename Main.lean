import «HashSet»

def main : IO Unit :=
  let empty : HashSet Nat := HashSet.mk
  let insert1 := empty.insert 1
  let insert2 := insert1.insert 2
  let insert3 := insert2.insert 3
  let xx := insert3.insert 1
  IO.println s!"Hello, {xx}!"
