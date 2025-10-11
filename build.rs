fn main() {
    println!("cargo:rustc-link-arg=-Wl,--subsystem,windows");
}