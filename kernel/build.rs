pub fn main() {
    println!("cargo:rustc-link-search=native=ext");
    println!("cargo:rustc-link-lib=static=sd");
    println!("cargo:rerun-if-changed=ext/layout.ld");
    println!("cargo:rerun-if-changed=ext/start.S");
}
