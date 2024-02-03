#[cfg(target_os = "macos")]
fn main() {
    use std::env;
    use std::path::PathBuf;

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header_contents("libproc_wrapper.h", "#include <libproc.h>")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .clang_args(&[
            "-x",
            "c++",
            "-I",
            "/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/include",
        ])
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libproc_bindings.rs"))
        .expect("Couldn't write bindings!");
}

#[cfg(not(target_os = "macos"))]
fn main() {}
