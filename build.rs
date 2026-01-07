use std::path::Path;
use std::env;
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

    let out = Command::new("dotnet")
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
            &format!("{workspace}/build")
        ]).output().unwrap();
    
    std::fs::rename(format!("{path}/NFMWorld.Library.so"), format!("{path}/libnfmw.so")).unwrap();
    
    println!("cargo::rustc-link-lib=nfmw");
}