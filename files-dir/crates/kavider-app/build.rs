//! Compiles the Slint UI at build time and forces the Material style on every platform,
//! so the desktop binary and the Android APK render the same widgets and palette.

fn main() {
    let config = slint_build::CompilerConfiguration::new().with_style("material");

    println!("cargo:rerun-if-changed=ui/main.slint");
    println!("cargo:rerun-if-changed=ui/assets/zoom-test.png");

    if let Err(error) = slint_build::compile_with_config("ui/main.slint", config) {
        // No `expect` outside tests (brief section 9), even in a build script.
        eprintln!("kavider: failed to compile ui/main.slint: {error}");
        std::process::exit(1);
    }
}
