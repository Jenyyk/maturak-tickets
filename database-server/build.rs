fn main() {
    println!("cargo:rustc-link-search=native=/home/jan/Desktop/rust/maturak-tickets/");
    println!("cargo:rustc-link-lib=dylib=brdc_bacon");
}
