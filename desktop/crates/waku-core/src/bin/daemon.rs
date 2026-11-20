//! Rapid Pi daemon binary — session process manager for the desktop.
//!
//! Spawned by the desktop to manage provider runtimes and persist session
//! state. Remote control is Rapid Pi's own feature (the `/remote-control`
//! command in the CLI): this binary only starts `rapid-pi service run`, which
//! holds the Noise IK server, and asks that service for a pairing code with
//! `rapid-pi service pair --link`. Nothing of the pairing protocol is
//! reimplemented here.

use std::env;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context as _, bail};

use waku_core::{
    ServerOptions, command_env::find_executable, daemon::WakuBackend, persistence::StateStore,
    serve, settings::DaemonSettingsStore,
};
use waku_protocol::{DAEMON_TOKEN_ENV, DaemonReady, PROTOCOL_VERSION};

/// How long the service host gets to open its control socket before the
/// pairing request is made.
const SERVICE_START_DELAY: Duration = Duration::from_millis(1500);

fn main() {
    if let Err(error) = run() {
        eprintln!("rapid-pi daemon failed: {error:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let mut args = env::args().skip(1);
    let mut bind = "0.0.0.0:0".to_string();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bind" => bind = args.next().context("--bind requires a value")?,
            "--parent-pid" => {
                let _ = args.next();
            }
            // The desktop already chose the bind address; the flag only says
            // the user opted into non-loopback exposure.
            "--allow-non-loopback" => {}
            _ => bail!("unknown argument: {arg}"),
        }
    }

    let token = env::var(DAEMON_TOKEN_ENV).context("RAPIDPI_DAEMON_TOKEN not set")?;
    let _app_executable =
        env::var(waku_protocol::APP_EXECUTABLE_ENV).context("RAPIDPI_APP_EXECUTABLE not set")?;

    let agent_dir = dirs::home_dir()
        .unwrap_or_else(std::env::temp_dir)
        .join(".rapidpi");

    // Remote control runs as Rapid Pi's own service. It is best effort: a
    // missing engine binary only means the desktop has no phone pairing.
    let _remote_control = RemoteControl::start();

    let settings_path = agent_dir.join("daemon-settings.json");
    let settings_store =
        DaemonSettingsStore::open(settings_path).context("could not open daemon settings store")?;

    let state_path = agent_dir.join("daemon-app.db");
    let state_store = StateStore::daemon(state_path);

    let backend =
        WakuBackend::new(settings_store, state_store).context("could not create backend")?;

    let listener =
        TcpListener::bind(&bind).with_context(|| format!("could not bind to {bind}"))?;
    let actual_addr = listener
        .local_addr()
        .context("could not get local address")?;

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

    serve(
        listener,
        token,
        Arc::new(backend),
        Arc::new(AtomicBool::new(false)),
        options,
    )
    .context("rapid-pi daemon server stopped")
}

/// The `rapid-pi service run` process this daemon keeps alive.
///
/// The service may already be running (a second copy exits 0 by itself), in
/// which case dropping this guard is a no-op because the child already exited.
struct RemoteControl {
    host: Child,
}

impl RemoteControl {
    /// Starts the service host and, once it serves, prints a pairing code.
    fn start() -> Option<Self> {
        let engine = find_executable("rapid-pi")?;
        let host = Command::new(&engine)
            .arg("service")
            .arg("run")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .ok()?;
        let pairing_engine = engine.clone();
        // The pairing request blocks until a phone scans the code or the code
        // expires, so it runs on its own thread. Its output is forwarded to
        // stderr: the daemon's stdout is the readiness pipe the desktop closes
        // after the first line, and writing more into it would break the
        // child with EPIPE.
        let _ = std::thread::Builder::new()
            .name("rapidpi-pairing".into())
            .spawn(move || {
                std::thread::sleep(SERVICE_START_DELAY);
                let Ok(mut pairing) = Command::new(&pairing_engine)
                    .arg("service")
                    .arg("pair")
                    .arg("--link")
                    .stdin(Stdio::null())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()
                else {
                    return;
                };
                let Some(stdout) = pairing.stdout.take() else {
                    return;
                };
                for line in std::io::BufRead::lines(std::io::BufReader::new(stdout))
                    .map_while(Result::ok)
                {
                    eprintln!("{line}");
                }
                let _ = pairing.wait();
            });
        Some(Self { host })
    }
}

impl Drop for RemoteControl {
    fn drop(&mut self) {
        let _ = self.host.kill();
        let _ = self.host.wait();
    }
}
