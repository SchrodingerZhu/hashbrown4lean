import Lake
open Lake DSL

package «hashbrown» {
  -- add package configuration options here
}

lean_lib «HashBrown» {
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
  let libFile := pkg.buildDir / "lib" / name
  let cargoFile ← inputFile <| pkg.dir / "Cargo.toml"
  let librsFile ← inputFile <| pkg.dir / "src" / "lib.rs"
  let setFile ← inputFile <| pkg.dir / "src" / "set.rs"
  let mapFile ← inputFile <| pkg.dir / "src" / "map.rs"
  let ffiFile ← inputFile <| pkg.dir / "src" / "ffi.rs"
  buildFileAfterDepArray libFile #[cargoFile, librsFile, setFile, mapFile, ffiFile] (fun _ => proc {
    cmd := "cargo",
    args := #["build", "--release", "-Zunstable-options", "--target-dir", (pkg.buildDir / "rust").toString, "--out-dir", (pkg.buildDir / "lib").toString]
  } true) (pure BuildTrace.nil)
  