use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../whatsapp-api-web/src");
    println!("cargo:rerun-if-changed=../whatsapp-api-web/Cargo.toml");
    println!("cargo:rerun-if-env-changed=SKIP_WEB_BUILD");

    if std::env::var("SKIP_WEB_BUILD").is_ok_and(|v| v == "1" || v == "true") {
        println!("cargo:warning=SKIP_WEB_BUILD=1: skipping frontend build");
        return;
    }

    if Command::new("dx").arg("--version").output().is_err() {
        println!(
            "cargo:warning=dioxus-cli (dx) not found; building without the web frontend.\n\
             \tcargo install dioxus-cli --version \"~0.6\""
        );
        return;
    }

    let profile = std::env::var("PROFILE").expect("PROFILE not set by cargo");
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("server crate not in workspace")
        .to_path_buf();
    let dx_target = workspace_root.join("target/dx-build");

    // A nested `cargo` invocation inside a build script would deadlock on the
    // workspace target-dir lock, so give dx (and its cargo) its own target dir.
    let mut dx_args = vec!["build", "--platform", "web", "--package", "whatsapp-api-web"];
    if profile == "release" {
        dx_args.push("-r");
    }
    let output = Command::new("dx")
        .args(&dx_args)
        .current_dir(workspace_root.join("whatsapp-api-web"))
        .env("CARGO_TARGET_DIR", &dx_target)
        .output()
        .unwrap_or_else(|e| {
            panic!("failed to run `dx build`: {e}");
        });

    if !output.status.success() {
        panic!(
            "`dx build` failed with status {:?}:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let public_dir = dx_target
        .join("dx/whatsapp-api-web")
        .join(&profile)
        .join("web/public");
    if !public_dir.exists() {
        panic!(
            "`dx build` succeeded but frontend output not found at {}",
            public_dir.display()
        );
    }

    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR not set")).join("web");
    copy_dir(&public_dir, &out);
    println!("cargo:warning=embedded web frontend ({profile}) from {}", public_dir.display());
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("failed to create web output dir");
    for entry in std::fs::read_dir(from).expect("failed to read web build output") {
        let entry = entry.expect("failed to read web build output entry");
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_dir(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).expect("failed to copy web asset");
        }
    }
}
