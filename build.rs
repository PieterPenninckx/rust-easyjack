use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("gcc").args(&["src/test/jack_wrapper.c", "-c", "-fPIC", "-std=c11", "-o"])
                       .arg(&format!("{}/jack_wrapper.o", out_dir))
                       .status().unwrap();

    // we also create a static library to pull in the remaining functions, but only running in test
    Command::new("ar").args(&["crus", "libjack_wrapper.a", "jack_wrapper.o"])
                      .current_dir(&Path::new(&out_dir))
                      .status().unwrap();

    println!("cargo:rerun-if-changed=src/test/jack_wrapper.c");
    println!("cargo:rustc-link-search=native={}", out_dir);
    // but don't link it in!
    // we should only link it when we need it
}
