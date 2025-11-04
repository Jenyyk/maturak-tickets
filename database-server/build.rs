use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:warning=running build.rs");
    println!("cargo:rerun-if-env-changed=FORCE_REBUILD");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let brdc_bacon_dir = out_dir.join("brdc_bacon");

    println!("Work dir: {}", brdc_bacon_dir.display());

    println!(
        "cargo:rerun-if-changes={}",
        brdc_bacon_dir.to_str().unwrap()
    );

    // clone or refresh git repo
    if brdc_bacon_dir.exists() {
        let status = Command::new("git")
            .current_dir(&brdc_bacon_dir)
            .args(["pull"])
            .status()
            .expect("Failed to execute git pull");
        assert!(status.success(), "Failed to pull repo");
    } else {
        let status = Command::new("git")
            .args([
                "clone",
                "https://github.com/ZenithMeetsNadir/brdc-bacon/",
                brdc_bacon_dir.to_str().unwrap(),
            ])
            .status()
            .expect("Failed to execute git clone");
        assert!(status.success(), "Failed to clone repo");
    }

    // build zig library
    let status = Command::new("zig")
        .current_dir(&brdc_bacon_dir)
        .args([
            "build",
            "--release=safe",
            "--prefix",
            brdc_bacon_dir.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute zig build");
    assert!(status.success(), "Failed to build zig library");

    // link that shi
    let lib_output_path = brdc_bacon_dir.join("lib");

    println!(
        "cargo:rustc-link-search=native={}",
        lib_output_path.to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=dylib=brdc_bacon");
}
