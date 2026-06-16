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

    let bindgen_build = Command::new("dotnet")
        .args([
            "build",
            "--property",
            "WarningLevel=0",
            &format!("{workspace}/nfm-world/NFMWorld.RustBindGen/NFMWorld.RustBindGen.csproj"),
        ])
        .output();

    if let Err(e) = bindgen_build {
        panic!("Failed to build NFMWorld.RustBindGen: {}", e);
    }

    let bindgen_build = bindgen_build.unwrap();

    if let Err(e) = bindgen_build.clone().exit_ok() {
        let stdout = String::from_utf8_lossy(&bindgen_build.stdout);
        let stderr = String::from_utf8_lossy(&bindgen_build.stderr);
        eprintln!("stdout: {}", stdout);
        eprintln!("stderr: {}", stderr);
        panic!("Failed to build NFMWorld.RustBindGen: {}", e);
    }

    let bindgen = Command::new("dotnet")
        .args([
            "run",
            "--no-build",
            "--project",
            &format!("{workspace}/nfm-world/NFMWorld.RustBindGen/NFMWorld.RustBindGen.csproj"),
        ])
        .output();

    if let Err(e) = bindgen {
        panic!("Failed to run NFMWorld.RustBindGen: {}", e);
    }

    let bindgen = bindgen.unwrap();

    std::fs::write(format!("{workspace}/src/ffi.rs"), bindgen.stdout).unwrap();

    std::fs::rename(
        format!("{path}/NFMWorld.Library.so"),
        format!("{path}/libnfmw.so"),
    )
    .unwrap();

    println!("cargo::rustc-link-lib=nfmw");
}
