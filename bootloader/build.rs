pub fn main() {
    println!("cargo:rerun-if-changed=ext/layout.ld");
    println!("cargo:rerun-if-changed=ext/init.S");
}
