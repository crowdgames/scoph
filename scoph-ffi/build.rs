extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let header_output_dir = env::var("OUT_DIR").unwrap();

    cbindgen::generate(crate_dir)
        .expect("cbindgen.toml should exist")
        .write_to_file(format!("{header_output_dir}/scoph.h"));
}
