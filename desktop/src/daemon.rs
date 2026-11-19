//! Connect to the local waku-daemon, which manages the Rapid Pi engine.
//! The desktop spawns waku-daemon as a child process and speaks to it over
//! a loopback WebSocket. Android is standalone and is never a desktop backend.

use std::env;
use std::path::PathBuf;

use anyhow::{bail, Context as _, Result};
use waku_client::DaemonSupervisor;

pub fn connect() -> Result<DaemonSupervisor> {
    let daemon_exe = locate_daemon_executable()
        .context("could not locate waku-daemon executable")?;
    DaemonSupervisor::spawn(&daemon_exe, false)
        .with_context(|| format!("failed to spawn waku-daemon at {}", daemon_exe.display()))
}

/// Locate the waku-daemon executable alongside the desktop binary.
fn locate_daemon_executable() -> Result<PathBuf> {
    let current_exe = env::current_exe().context("could not locate current executable")?;
    let exe_dir = current_exe
        .parent()
        .context("executable has no parent directory")?;
    let daemon_name = if cfg!(target_os = "windows") {
        "waku-daemon.exe"
    } else {
        "waku-daemon"
    };
    let daemon_path = exe_dir.join(daemon_name);
    if daemon_path.exists() {
        return Ok(daemon_path);
    }
    // Fallback: look next to the Cargo target dir during development.
    let workspace_root = exe_dir
        .ancestors()
        .nth(4)
        .context("could not determine workspace root")?;
    let dev_path = workspace_root.join("target").join("debug").join(daemon_name);
    if dev_path.exists() {
        return Ok(dev_path);
    }
    bail!("waku-daemon not found at {daemon_path:?} or {dev_path:?}")
}

pub fn local_hostname() -> Option<String> {
    #[cfg(unix)]
    {
        let mut buffer = [0_u8; 256];
        let result = unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) };
        if result == 0 {
            let length = buffer
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(buffer.len());
            return String::from_utf8_lossy(&buffer[..length]).to_string().into();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = ();
    }
    Some("localhost".to_string())
}
