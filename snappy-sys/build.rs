extern crate pkg_config;
extern crate gcc;

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

macro_rules! t {
    ($e:expr) => (match $e {
        Ok(n) => n,
        Err(e) => panic!("\n{} failed with {}\n", stringify!($e), e),
    })
}

fn main() {
    let want_static = env::var("SNAPPY_SYS_STATIC").unwrap_or(String::new()) == "1";
    let from_source = env::var("SNAPPY_SYS_STATIC_FROM_SOURCE").unwrap_or(String::new()) == "1";
    if !from_source && configure_snappy(want_static) {
        return;
    }
    build_snappy();
}

fn configure_snappy(want_static: bool) -> bool {
    // ~ try pkg_config first
    if pkg_config::probe_library("snappy").is_ok() {
        return true;
    }
    // ~ then try search in statically predefined directories
    let libsnappy_file = if want_static { "libsnappy.a" } else { "libsnappy.so" };
    if let Some(path) = first_path_with_file(libsnappy_file) {
        if want_static {
            println!("cargo:rustc-link-search={}", path);
            println!("cargo:rustc-link-lib=static=snappy");
            configure_stdcpp();
        } else {
            println!("cargo:rustc-link-search=native={}", path);
            println!("cargo:rustc-link-lib=dylib=snappy");
        }
        return true;
    }
    return false;
}

fn build_snappy() {
    let src = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("snappy");
    let build = PathBuf::from(&env::var("OUT_DIR").unwrap()).join("build");

    let cc = gcc::Config::new().get_compiler();
    let mut cflags = OsString::new();
    for arg in cc.args() {
        cflags.push(arg);
        cflags.push(" ");
    }
    t!(fs::create_dir_all(&build));
    cp_r(&src, &build);
    run(Command::new("./autogen.sh").current_dir(&build));
    run(Command::new("./configure")
        .env("CC", cc.path())
        .env("CFLAGS", cflags)
        .current_dir(&build)
        .arg("--disable-shared")
        .arg("--enable-static")
        .arg("--with-pic"));
    run(Command::new("make").current_dir(&build));
    println!("cargo:rustc-link-lib=static=snappy");
    println!("cargo:rustc-link-search={}/.libs", build.to_string_lossy());
    println!("cargo:root={}", build.to_string_lossy());
    configure_stdcpp();
}

fn configure_stdcpp() {
    // From: https://github.com/alexcrichton/gcc-rs/blob/master/src/lib.rs
    let target = env::var("TARGET").unwrap();
    let cpp = if target.contains("darwin") { "c++" } else { "stdc++" };
    println!("cargo:rustc-link-lib={}", cpp);
}

fn cp_r(dir: &Path, dst: &Path) {
    for entry in t!(fs::read_dir(dir)) {
        let entry = entry.expect("entry");
        let path = entry.path();
        let dst = dst.join(path.file_name().unwrap());
        if t!(fs::metadata(&path)).is_file() {
            t!(fs::copy(path, dst));
        } else {
            t!(fs::create_dir_all(&dst));
            cp_r(&path, &dst);
        }
    }
}

fn run(cmd: &mut Command) {
    println!("running: {:?}", cmd);
    let status = match cmd.status() {
        Ok(s) => s,
        Err(e) => panic!("failed to run: {}", e),
    };
    if !status.success() {
        panic!("failed to run successfully: {}", status);
    }
}

fn first_path_with_file(file: &str) -> Option<String> {
    // we want to look in LD_LIBRARY_PATH and then some default folders
    if let Some(ld_path) = env::var_os("LD_LIBRARY_PATH") {
        for p in env::split_paths(&ld_path) {
            if is_file_in(file, &p) {
                return p.to_str().map(|s| String::from(s))
            }
        }
    }
    for p in vec!["/usr/lib","/usr/local/lib"] {
        if is_file_in(file, &Path::new(p)) {
            return Some(String::from(p))
        }
    }
    return None
}

fn is_file_in(file: &str, folder: &Path) -> bool {
    let full = folder.join(file);
    match fs::metadata(full) {
        Ok(ref found) if found.is_file() => true,
        _ => false
    }
}
