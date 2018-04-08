extern crate build_helper;
extern crate gcc;
extern crate pkg_config;

use build_helper::Profile;
use build_helper::cargo::manifest;
use gcc::Build;
use std::fs;

fn main() {
    // Copy the dtextc.dat file into the build directory.
    let mut data_file = manifest::dir();
    data_file.push("c_src/dtextc.dat");
    let mut dest = manifest::dir();
    match build_helper::profile() {
        Profile::Debug => dest.push("target/debug/dtextc.dat"),
        Profile::Release => dest.push("target/release/dtextc.dat"),
    }
    fs::copy(data_file, dest.clone()).unwrap();

    // Compile and link the original C executable as a library.
    let paths = fs::read_dir("./c_src/").unwrap();

    let c_files = paths.map(|entry| entry.unwrap().path()).filter(|ref p| {
        if let Some(ext) = p.extension() {
            ext == "c"
        } else {
            false
        }
    });

    let mut dest_string = "\"".to_string();
    dest_string.push_str(dest.to_str().unwrap());
    dest_string.push_str("\"");
    Build::new()
        .files(c_files)
        .include("c_src")
        .define("ALLOW_GDT", None)        // Enables the built-in debugger
        .define("MORE_TERMINFO", None)    // Sets the terminal-interaction lib to ncurses
        .define("TEXTFILE", Some(dest_string.as_str()))
        .define("AS_RUST_LIB", None)
        //.flag("-Werror=implicit-function-declaration")
        .flag("-Wno-parentheses")
        .flag("-Wno-unused-parameter")
        .flag("-Wno-unused-but-set-variable")
        .flag("-Wno-missing-braces")
        .compile("c_zork");

    // Link the ncurses library.
    //println!("cargo:rustc-include-lib=dylib=ncurses");
    pkg_config::probe_library("ncurses").unwrap();
}
