#![feature(exit_status_error)]

use std::env;
use std::path::Path;
use std::process::Command;

// Example custom build script.
fn main() {
    let workspace = env::var("CARGO_MANIFEST_DIR").unwrap();
    let _ = std::fs::create_dir("./build");
    let path = Path::new(&workspace).join("build");
    let path = path.display();

    println!("cargo::rerun-if-changed=nfm-world/NFMWorld.Library");
    println!("cargo::rustc-link-search=native={path}");
    println!("cargo:rustc-env=LD_LIBRARY_PATH={path}");

    let publish = Command::new("dotnet")
        .args([
            "publish",
            &format!("{workspace}/nfm-world/NFMWorld.Library/NFMWorld.Library.csproj"),
            "-c",
            "Release",
            "-r",
            "linux-x64",
            "--self-contained",
            "-p:PublishAot=true",
            "-o",
            &format!("{workspace}/build"),
        ])
        .output();

    if let Err(e) = publish {
        panic!("Failed to publish NFMWorld.Library: {}", e);
    }

    let publish = publish.unwrap();

    if let Err(e) = publish.clone().exit_ok() {
        let stdout = String::from_utf8_lossy(&publish.stdout);
        let stderr = String::from_utf8_lossy(&publish.stderr);
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", stderr);
        panic!("Failed to publish NFMWorld.Library: {}", e);
    }

    let bindgen = Command::new("dotnet")
        .args([
            "publish",
            &format!("{workspace}/nfm-world/NFMWorld.RustBindGen/NFMWorld.RustBindGen.csproj"),
            "-c",
            "Release",
            "-r",
            "linux-x64",
            "--self-contained",
            "-p:PublishAot=true",
            "-o",
            &format!("{workspace}/build"),
        ])
        .output();

    if let Err(e) = bindgen {
        panic!("Failed to publish NFMWorld.RustBindGen: {}", e);
    }

    let bindgen = bindgen.unwrap();

    if let Err(e) = bindgen.clone().exit_ok() {
        let stdout = String::from_utf8_lossy(&bindgen.stdout);
        let stderr = String::from_utf8_lossy(&bindgen.stderr);
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", stderr);
        panic!("Failed to publish NFMWorld.RustBindGen: {}", e);
    }

    let bindgen_out = Command::new(format!("{workspace}/build/NFMWorld.RustBindGen"))
        .current_dir(format!("{workspace}/nfm-world"))
        .output()
        .unwrap()
        .exit_ok()
        .unwrap();

    std::fs::write(format!("{workspace}/src/ffi.rs"), bindgen_out.stdout).unwrap();

    std::fs::rename(
        format!("{path}/NFMWorld.Library.so"),
        format!("{path}/libnfmw.so"),
    )
    .unwrap();

    println!("cargo::rustc-link-lib=nfmw");
}
