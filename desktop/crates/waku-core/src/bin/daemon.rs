//! Waku daemon binary — session process manager for the desktop.
//! Spawned by the desktop to manage provider runtimes and persist session state.
//! Also runs the remote-control server (/rc) for paired phones via Noise IK WebSocket,
//! displaying a QR code pairing link on startup.

use std::env;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{Context as _, bail};
use serde_json::json;

use waku_core::{
    daemon::WakuBackend,
    persistence::StateStore,
    remote_control::RemoteControlServer,
    serve, ServerOptions,
    settings::DaemonSettingsStore,
};
use waku_protocol::{DAEMON_TOKEN_ENV, DaemonReady, PROTOCOL_VERSION};

fn main() {
    if let Err(error) = run() {
        eprintln!("waku-daemon failed: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let mut bind = "127.0.0.1:0".to_string();
    let mut rc_port: u16 = 48123;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => bind = args.next().context("--bind requires a value")?,
            "--parent-pid" => {
                let _ = args.next();
            }
            "--rc-port" => {
                rc_port = args
                    .next()
                    .context("--rc-port requires a value")?
                    .parse::<u16>()
                    .context("--rc-port must be a port number")?;
            }
            "--allow-origin" => {
                let _ = args.next();
            }
            "--allow-non-loopback" => {}
            _ => bail!("unknown argument: {arg}"),
        }
    }

    let token = env::var(DAEMON_TOKEN_ENV).context("WAKU_DAEMON_TOKEN not set")?;
    let _app_executable = env::var(waku_protocol::APP_EXECUTABLE_ENV)
        .context("WAKU_APP_EXECUTABLE not set")?;

    let agent_dir = dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".rapidpi");

    // --- Remote-control server (phone pairing) ---
    let mut rc_server = RemoteControlServer::new(&agent_dir)?;
    let rc_endpoints = vec![format!("ws://127.0.0.1:{rc_port}/rc")];
    let pairing_uri = rc_server.create_pairing(rc_endpoints.clone())?;
    rc_server.listen(rc_port)?;

    // Print QR code for pairing (terminal-friendly)
    render_qr_code(&pairing_uri);
    eprintln!(
        "remote-control: pairing code: {pairing_uri}\n\
         remote-control: listening on ws://127.0.0.1:{rc_port}/rc\n\
         remote-control: pair a phone with the QR code above"
    );

    // --- Daemon settings + state stores ---
    let settings_path = agent_dir.join("daemon-settings.json");
    let settings_store = DaemonSettingsStore::open(settings_path)
        .context("could not open daemon settings store")?;

    let state_path = agent_dir.join("daemon-app.db");
    let state_store = StateStore::daemon(state_path);

    let backend = WakuBackend::new(settings_store, state_store)
        .context("could not create Waku backend")?;

    let listener = TcpListener::bind(&bind)
        .with_context(|| format!("could not bind to {bind}"))?;
    let actual_addr = listener.local_addr().context("could not get local address")?;

    let ready = DaemonReady {
        address: actual_addr.to_string(),
        protocol_version: PROTOCOL_VERSION,
        pid: std::process::id(),
    };
    let ready_json = serde_json::to_string(&ready).context("could not serialize DaemonReady")?;
    println!("{ready_json}");

    let options = ServerOptions {
        allowed_origins: Default::default(),
        allow_shutdown: true,
    };

    // Run the WebSocket daemon server (blocks forever)
    serve(
        listener,
        token,
        Arc::new(backend),
        Arc::new(AtomicBool::new(false)),
        options,
    )
    .context("waku-daemon server stopped")
}

/// Render a QR code to the terminal using half-block characters.
fn render_qr_code(uri: &str) {
    match qrcode::QrCode::new(uri.as_bytes()) {
        Ok(qr) => {
            eprintln!("\nremote-control: scan this QR code to pair a phone:");
            let modules = qr.to_vec();
            let width = qr.width();
            for y in (0..width).step_by(2) {
                for x in 0..width {
                    let top = modules[y * width + x];
                    let bottom = y + 1 < width && modules[(y + 1) * width + x];
                    match (top, bottom) {
                        (true, true) => eprint!("██"),
                        (true, false) => eprint!("▀█"),
                        (false, true) => eprint!("█▀"),
                        (false, false) => eprint!("  "),
                    }
                }
                eprintln!();
            }
            eprintln!();
        }
        Err(e) => {
            eprintln!("remote-control: could not render QR code: {e}");
            eprintln!("remote-control: pairing link: {uri}");
        }
    }
}
