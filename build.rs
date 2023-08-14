use std::path::PathBuf;

fn main() {
    let output = std::process::Command::new("lean")
        .arg("--print-prefix")
        .output()
        .expect("failed to execute lean")
        .stdout;
    let parsed = std::str::from_utf8(&output).unwrap().trim();
    let include = PathBuf::new()
        .join(parsed)
        .join("include");
    let path = PathBuf::new()
        .join(parsed)
        .join("include")
        .join("lean")
        .join("lean.h")
        .display()
        .to_string();

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(path)
        .clang_arg(format!("-I{}", include.display()))
        .allowlist_type("lean_.*")
        .allowlist_var("LEAN_.*")
        .allowlist_var("LEAN_.*")
        .allowlist_var("Lean.*")
        .allowlist_function("lean_.*")
        .allowlist_recursively(true)
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
