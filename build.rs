use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {

    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());

    let mem_file = include_bytes!("stm32f303_memory.x");

    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(mem_file)
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=stm32f303_memory.x");
}
