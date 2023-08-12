import Lake
open Lake DSL

package «swisstable» {
  -- add package configuration options here
}

lean_lib «HashSet» {
  precompileModules := true
  srcDir := "lean"
  -- add library configuration options here
}

@[default_target]
lean_exe «swisstable» {
  root := `Main
}

extern_lib liblean_hashbrown pkg := do
  let name := nameToStaticLib "lean_hashbrown"
  let libFile := pkg.dir / "target" / "release" / name
  let cargoFile ← inputFile <| pkg.dir / "Cargo.toml"
  let librsFile ← inputFile <| pkg.dir / "src" / "lib.rs"
  let setFile ← inputFile <| pkg.dir / "src" / "set.rs"
  let ffiFile ← inputFile <| pkg.dir / "src" / "ffi.rs"
  buildFileAfterDepArray libFile #[cargoFile, librsFile, setFile, ffiFile] (fun _ => proc {
    cmd := "cargo",
    args := #["build", "--release"],
  } true) (pure BuildTrace.nil) 