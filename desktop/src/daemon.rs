//! Launches the Rapid Pi remote control service via the CLI.
//!
//! The desktop no longer spawns a separate daemon or manages the engine via
//! stdio. Instead it relies on the CLI's built-in `rapid-pi service` command
//! (Noise IK WebSocket server at `/rc`, QR code pairing, LAN binding). The
//! desktop starts the service and the phone pairs directly with it.

use waku_client::DaemonSupervisor;

pub fn connect() -> anyhow::Result<DaemonSupervisor> {
    // No-op spawn: the desktop no longer manages the engine via stdio.
    // The rapid-pi service (`rapid-pi service install`) is installed
    // separately by the user for remote control / phone pairing.
    DaemonSupervisor::spawn(std::path::Path::new("rapid-pi"), false)
}

pub fn local_hostname() -> Option<String> {
    let output = std::process::Command::new("hostname").output().ok()?;
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string().into();
    }
    Some("localhost".to_string())
}