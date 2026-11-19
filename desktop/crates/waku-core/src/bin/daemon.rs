//! Waku daemon binary — session process manager for the desktop.
//! Spawned by the desktop to manage provider runtimes and persist session state.

use std::env;
use std::net::TcpListener;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{Context as _, bail};

use waku_core::{
    daemon::WakuBackend,
    persistence::StateStore,
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
    let mut allowed_origins = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => bind = args.next().context("--bind requires a value")?,
            "--parent-pid" => {
                let _ = args.next();
            }
            "--allow-origin" => {
                allowed_origins.push(args.next().context("--allow-origin requires a value")?);
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
        allowed_origins: allowed_origins.into_iter().collect(),
        allow_shutdown: true,
    };

    serve(
        listener,
        token,
        Arc::new(backend),
        Arc::new(AtomicBool::new(false)),
        options,
    )
    .context("waku-daemon server stopped")
}
