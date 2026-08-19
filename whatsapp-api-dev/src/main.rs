use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{bail, Context};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};

const BACKEND_PORT: u16 = 8081;
const FRONTEND_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("whatsapp-api-dev must be inside the workspace")?
        .to_path_buf();
    let web_dir = workspace_root.join("whatsapp-api-web");
    let dx_target_dir = workspace_root.join("target/dx-build");
    let profile = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(PathBuf::from))
        .and_then(|dir| dir.file_name().map(|n| n.to_os_string()))
        .and_then(|n| n.into_string().ok())
        .unwrap_or_else(|| "debug".to_string());
    let server_exe = workspace_root
        .join("target")
        .join(&profile)
        .join("whatsapp-api-server");

    // Build the backend without embedding the frontend.
    let mut build_args = vec!["build", "-p", "whatsapp-api-server"];
    if profile == "release" {
        build_args.push("--release");
    }

    let status = Command::new("cargo")
        .args(&build_args)
        .env("SKIP_WEB_BUILD", "1")
        .current_dir(&workspace_root)
        .status()
        .await
        .context("failed to run `cargo build` for the server")?;
    if !status.success() {
        bail!("server build failed");
    }

    println!(
        "[dev] cargo dev: backend on :{BACKEND_PORT}, frontend on :{FRONTEND_PORT}"
    );

    // Start the backend.
    let backend_port = BACKEND_PORT.to_string();
    let mut server = spawn_child(
        server_exe.as_os_str(),
        &[] as &[&str],
        &[("PORT", backend_port.as_str())],
        &workspace_root,
    )
    .await
    .context("failed to start backend")?;

    // Start the Dioxus dev server.
    let frontend_port = FRONTEND_PORT.to_string();
    let mut dx = spawn_child(
        OsStr::new("dx"),
        &[
            "serve",
            "--port",
            frontend_port.as_str(),
            "--open",
            "false",
        ],
        &[("CARGO_TARGET_DIR", dx_target_dir.as_os_str())],
        &web_dir,
    )
    .await
    .context("failed to start `dx serve`; is dioxus-cli installed?")?;

    // Stream labeled output from both children.
    if let Some(out) = server.stdout.take() {
        tokio::spawn(stream_output(out, "[rust]"));
    }
    if let Some(err) = server.stderr.take() {
        tokio::spawn(stream_output(err, "[rust]"));
    }
    if let Some(out) = dx.stdout.take() {
        tokio::spawn(stream_output(out, "[dx]"));
    }
    if let Some(err) = dx.stderr.take() {
        tokio::spawn(stream_output(err, "[dx]"));
    }

    // Wait for one process to exit or for Ctrl-C.
    tokio::select! {
        res = server.wait() => {
            match res {
                Ok(status) => eprintln!("[dev] backend exited with {status}"),
                Err(e) => eprintln!("[dev] backend wait error: {e}"),
            }
        }
        res = dx.wait() => {
            match res {
                Ok(status) => eprintln!("[dev] dx serve exited with {status}"),
                Err(e) => eprintln!("[dev] dx serve wait error: {e}"),
            }
        }
        _ = tokio::signal::ctrl_c() => {
            eprintln!("[dev] received Ctrl-C, shutting down");
        }
    }

    // Kill both children (errors are expected if a child already exited).
    let _ = server.kill().await;
    let _ = dx.kill().await;

    Ok(())
}

async fn spawn_child(
    program: impl AsRef<std::ffi::OsStr>,
    args: &[impl AsRef<std::ffi::OsStr>],
    envs: &[(impl AsRef<std::ffi::OsStr>, impl AsRef<std::ffi::OsStr>)],
    current_dir: impl AsRef<Path>,
) -> anyhow::Result<Child> {
    Command::new(program)
        .args(args)
        .envs(envs.iter().map(|(k, v)| (k, v)))
        .current_dir(current_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(Into::into)
}

async fn stream_output<R>(reader: R, prefix: &'static str)
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        println!("{prefix} {line}");
    }
}
