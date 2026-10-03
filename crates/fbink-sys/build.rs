use std::{env, path::PathBuf, process::Command};

const PINNED_FBINK_REVISION: &str = "92e127008145b2a22fba7c59815d810d716310dd";

fn main() {
    println!("cargo:rerun-if-env-changed=KINDLE_CC");
    println!("cargo:rerun-if-env-changed=KINDLE_AR");
    println!("cargo:rerun-if-env-changed=KINDLE_RANLIB");
    println!("cargo:rerun-if-changed=c/fbink_bridge.c");
    println!("cargo:rerun-if-changed=c/input_bridge.c");
    println!("cargo:rerun-if-changed=../../vendor/FBInk");
    println!("cargo:rustc-env=KCP_FBINK_REVISION={PINNED_FBINK_REVISION}");

    let target = env::var("TARGET").expect("TARGET is set by Cargo");
    if target != "armv7-unknown-linux-gnueabihf" {
        return;
    }

    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let fbink = manifest.join("../../vendor/FBInk");
    let revision_file = fbink.join("fbink.h");
    if !revision_file.is_file() {
        panic!(
            "pinned FBInk submodule is missing at {}; initialize submodules before cross-building",
            fbink.display()
        );
    }

    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let fbink_out = out.join("fbink");
    // Make does not track compiler changes; rebuild C objects after Cargo reruns.
    if fbink_out.exists() {
        std::fs::remove_dir_all(&fbink_out).expect("remove previous FBInk objects");
    }
    let cc = env::var("KINDLE_CC").unwrap_or_else(|_| "arm-linux-gnueabihf-gcc".to_owned());
    let ar = env::var("KINDLE_AR").unwrap_or_else(|_| "arm-linux-gnueabihf-ar".to_owned());
    let ranlib =
        env::var("KINDLE_RANLIB").unwrap_or_else(|_| "arm-linux-gnueabihf-ranlib".to_owned());

    run(
        Command::new("make")
            .arg("-C")
            .arg(&fbink)
            .arg(format!("OUT_DIR={}", fbink_out.display()))
            .arg("KINDLE=1")
            .arg("MINIMAL=1")
            .arg("IMAGE=1")
            .arg(format!("CC={cc}"))
            .arg(format!("AR={ar}"))
            .arg(format!("RANLIB={ranlib}"))
            .arg("staticlib"),
        "build pinned FBInk",
    );

    let bridge_obj = out.join("fbink_bridge.o");
    run(
        Command::new(&cc)
            .arg("-O2")
            .arg("-std=gnu11")
            .arg("-D_GNU_SOURCE")
            .arg("-D_REENTRANT=1")
            .arg("-DFBINK_FOR_KINDLE")
            .arg("-DFBINK_MINIMAL")
            .arg("-DFBINK_WITH_DRAW")
            .arg("-DFBINK_WITH_IMAGE")
            .arg(format!("-DFBINK_VERSION=\"{}\"", PINNED_FBINK_REVISION))
            .arg("-I")
            .arg(&fbink)
            .arg("-c")
            .arg(manifest.join("c/fbink_bridge.c"))
            .arg("-o")
            .arg(&bridge_obj),
        "compile FBInk bridge",
    );

    let input_obj = out.join("input_bridge.o");
    run(
        Command::new(&cc)
            .arg("-O2")
            .arg("-std=gnu11")
            .arg("-c")
            .arg(manifest.join("c/input_bridge.c"))
            .arg("-o")
            .arg(&input_obj),
        "compile input ioctl bridge",
    );

    let bridge_lib = out.join("libkcp_fbink_bridge.a");
    run(
        Command::new(&ar)
            .arg("crs")
            .arg(&bridge_lib)
            .arg(&bridge_obj)
            .arg(&input_obj),
        "archive FBInk bridge",
    );

    println!("cargo:rustc-link-search=native={}", out.display());
    println!("cargo:rustc-link-search=native={}", fbink_out.display());
    println!("cargo:rustc-link-lib=static=kcp_fbink_bridge");
    println!("cargo:rustc-link-lib=static=fbink");
}

fn run(command: &mut Command, description: &str) {
    let status = command
        .status()
        .unwrap_or_else(|error| panic!("failed to {description}: {error}"));
    assert!(status.success(), "failed to {description}: {status}");
}
