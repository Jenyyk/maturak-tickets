use std::{env, path::PathBuf, process::Command};

fn main() {
    println!("cargo:warning=running build.rs");
    println!("cargo:rerun-if-env-changed=FORCE_REBUILD");
    println!("cargo:rerun-if-changes=build.rs");

    let root_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let brdc_bacon_dir = out_dir.join("brdc_bacon");

    println!("cargo:warning=work dir: {}", brdc_bacon_dir.display());
    println!("cargo:rerun-if-changes={}", brdc_bacon_dir.display());

    // clone or pull
    if brdc_bacon_dir.exists() {
        let status = Command::new("git")
            .current_dir(&brdc_bacon_dir)
            .args(["pull"])
            .status()
            .expect("git pull failed");
        assert!(status.success(), "git pull failed");
    } else {
        let status = Command::new("git")
            .args([
                "clone",
                "https://github.com/ZenithMeetsNadir/brdc-bacon/",
                brdc_bacon_dir.to_str().unwrap(),
            ])
            .status()
            .expect("git clone failed");
        assert!(status.success(), "git clone failed");
    }

    let cargo_target = env::var("TARGET").unwrap();
    let zig_target = cargo_target.replace("unknown-", "");
    println!("cargo:warning=zig target is: {}", zig_target);

    let mut args: Vec<String> = Vec::new();
    args.push("build".into());
    args.push("--release=safe".into());
    args.push("--prefix".into());
    args.push(brdc_bacon_dir.to_str().unwrap().to_string());
    args.push(format!("-Dtarget={}", zig_target));
    args.push("-Dcpu=baseline".into());

    // build with zig
    let status = Command::new("zig")
        .current_dir(&brdc_bacon_dir)
        .args(args.iter().map(|s| s.as_str()).collect::<Vec<&str>>())
        .status()
        .expect("Failed to execute zig build");
    assert!(status.success(), "zig build failed");

    // link that shi
    let lib_output_path = brdc_bacon_dir.join("lib");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_output_path.display()
    );
    println!("cargo:rustc-link-lib=dylib=brdc_bacon");

    // make copy for ease of use
    let lib_copy_path = root_dir.join(format!("libbrdc_bacon_{}.so", zig_target));
    let _out_lib = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&lib_copy_path)
        .expect("Failed to copy library");

    std::fs::copy(lib_output_path.join("libbrdc_bacon.so"), lib_copy_path)
        .expect("Failed to copy library");
}
