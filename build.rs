fn main() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let lib_dir = format!("{manifest}/lib");

    // Tell the linker where to find libvosk.so
    println!("cargo:rustc-link-search=native={lib_dir}");

    // Embed an absolute RPATH so the binary finds libvosk.so at runtime
    // without requiring LD_LIBRARY_PATH to be set
    println!("cargo:rustc-link-arg=-Wl,-rpath,{lib_dir}");
}
