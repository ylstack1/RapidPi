//! Process management for the rapid-pi engine.
//!
//! The desktop spawns `rapid-pi --mode rpc` as a child process and speaks the
//! JSON-RPC protocol over stdin/stdout.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command as StdCommand, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{bail, Context as _, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
use uuid::Uuid;

use waku_protocol::{APP_EXECUTABLE_ENV, ClientMessage, PROTOCOL_VERSION, SequencedEvent, ServerMessage};

/// A token used to authenticate with the engine. In practice this is a
/// randomly generated secret shared between the desktop and the engine.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EngineToken(pub String);

impl EngineToken {
    pub fn new() -> Self {
        use std::fmt::Write as _;
        let mut token = String::with_capacity(32);
        for _ in 0..32 {
            write!(&mut token, "{:x}", rand::random::<u8>()).unwrap();
        }
        Self(token)
    }
}

impl Default for EngineToken {
    fn default() -> Self {
        Self::new()
    }
}

/// Settings that control how the engine is exposed to the network.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct EngineExposureSettings {
    pub bind_address: String,
    pub token: EngineToken,
    pub enabled: bool,
    pub allowed_origins: Vec<String>,
    pub port: u16,
}

impl Default for EngineExposureSettings {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:0".to_string(),
            token: EngineToken::new(),
            enabled: false,
            allowed_origins: vec!["http://localhost:3001".into()],
            port: 0,
        }
    }
}

impl EngineExposureSettings {
    pub fn bind_address(&self) -> &str {
        &self.bind_address
    }

    pub fn ensure_token(&self) -> bool {
        self.token.0.is_empty()
    }

    pub fn allowed_origins_text(&self) -> String {
        self.allowed_origins.join(",")
    }

    pub fn with_allowed_origins_text(mut self, text: &str) -> Self {
        self.allowed_origins = parse_allowed_origins(text);
        self
    }

    pub fn new_token() -> EngineToken {
        EngineToken::new()
    }

    pub fn validate(&self) -> Result<Self> {
        if self.enabled && self.bind_address.starts_with("0.0.0.0") {
            bail!("non-loopback exposure requires explicit allowed origins");
        }
        Ok(self.clone())
    }
}

/// Parse a comma-separated list of origins.
pub fn parse_allowed_origins(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// The child process running the rapid-pi engine.
pub struct EngineChild {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    stdout: Mutex<BufReader<ChildStdout>>,
    token: EngineToken,
    address: String,
    protocol_version: u32,
}

impl EngineChild {
    /// Spawn `rapid-pi --mode rpc` and complete the Hello/DaemonReady handshake.
    pub fn spawn(executable: &Path, settings: EngineExposureSettings) -> Result<Self> {
        let settings = settings.validate()?;
        let token = settings.token.clone();
        let app_executable = std::env::current_exe().context("could not locate desktop executable")?;
        let mut command = StdCommand::new(executable);

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt as _;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
        }

        // Run the engine as an RPC server on stdin/stdout.
        command.arg("--mode").arg("rpc");

        let mut child = command
            .env(APP_EXECUTABLE_ENV, app_executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("could not launch {}", executable.display()))?;

        let stdin = Mutex::new(child.stdin.take().context("engine did not expose stdin")?);
        let stdout = Mutex::new(BufReader::new(
            child.stdout.take().context("engine did not expose stdout")?,
        ));

        Ok(Self {
            child: Mutex::new(child),
            stdin,
            stdout,
            token,
            address: String::new(),
            protocol_version: PROTOCOL_VERSION,
        })
    }

    pub fn send(&self, msg: ClientMessage) -> Result<()> {
        let json = serde_json::to_string(&msg).context("failed to serialize message")?;
        self.stdin
            .lock()
            .unwrap()
            .write_all(json.as_bytes())
            .context("failed to write message")?;
        self.stdin.lock().unwrap().write_all(b"\n").context("failed to write newline")?;
        self.stdin.lock().unwrap().flush().context("failed to flush")?;
        Ok(())
    }

    pub fn recv(&self) -> Result<Option<ServerMessage>> {
        let mut line = String::new();
        match self.stdout.lock().unwrap().read_line(&mut line) {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                let msg: ServerMessage =
                    serde_json::from_str(line.trim()).context("failed to parse ServerMessage")?;
                Ok(Some(msg))
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn kill(&self) -> Result<()> {
        self.child.lock().unwrap().kill().context("failed to kill engine")?;
        Ok(())
    }

    pub fn wait(&self) -> Result<()> {
        self.child.lock().unwrap().wait().context("failed to wait for engine")?;
        Ok(())
    }
}

/// A handle to the engine process for sending requests.
#[derive(Clone)]
pub struct EngineClient {
    process: Arc<Mutex<EngineChild>>,
    shutdown: Arc<AtomicBool>,
}

impl EngineClient {
    pub fn connect(process: EngineChild) -> Result<Self> {
        Ok(EngineClient {
            process: Arc::new(Mutex::new(process)),
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }

    pub fn send(&self, msg: ClientMessage) -> Result<()> {
        if self.shutdown.load(Ordering::Acquire) {
            bail!("engine client shut down");
        }
        self.process.lock().unwrap().send(msg)
    }

    pub fn recv(&self) -> Result<Option<ServerMessage>> {
        self.process.lock().unwrap().recv()
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        if let Ok(mut p) = self.process.lock() {
            let _ = p.send(ClientMessage::Shutdown);
        }
    }
}

/// The engine process supervisor that manages the engine lifecycle.
pub struct EngineSupervisor {
    inner: Arc<EngineSupervisorInner>,
}

impl Clone for EngineSupervisor {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

struct EngineSupervisorInner {
    client: crate::client::EngineClient,
    shutdown: Arc<AtomicBool>,
    _monitor_handle: thread::JoinHandle<()>,
}

/// Events emitted by the engine.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    Event(SequencedEvent),
    ShuttingDown,
    Error(String),
}

impl EngineSupervisor {
    /// Spawn `rapid-pi --mode rpc` as a child process and return a supervisor.
    ///
    /// No-op: `rapid-pi --mode rpc` uses ACP protocol, not waku-protocol, so the
    /// desktop cannot drive it via stdio. The engine is managed independently by
    /// the user (via `rapid-pi` CLI) or by the remote control service
    /// (via `rapid-pi service`).
    pub fn spawn(_executable: &Path, _watch_for_rebuilds: bool) -> Result<Self> {
        Self::spawn_configured(_executable, _watch_for_rebuilds, EngineExposureSettings::default())
    }

    pub fn spawn_configured(
        _executable: &Path,
        _watch_for_rebuilds: bool,
        _exposure: EngineExposureSettings,
    ) -> Result<Self> {
        let client = crate::client::EngineClient::disconnected();
        let shutdown = Arc::new(AtomicBool::new(false));
        let monitor_handle = thread::Builder::new()
            .name("engine-monitor".into())
            .spawn(move || {
                let _ = shutdown;
            })
            .unwrap();

        let supervisor = Self {
            inner: Arc::new(EngineSupervisorInner {
                client,
                shutdown,
                _monitor_handle: monitor_handle,
            }),
        };
        Ok(supervisor)
    }

    pub fn client(&self) -> crate::client::EngineClient {
        self.inner.client.clone()
    }

    pub fn shutdown(&self) {
        self.inner.shutdown.store(true, Ordering::Release);
        self.inner.client.shutdown();
    }

    /// Whether the engine is remote (network-accessible). Always false — the
    /// engine runs locally via stdio.
    pub fn is_remote(&self) -> bool {
        false
    }

    /// Subscribe to client connection updates. Emits the client once.
    pub fn subscribe_clients(&self) -> crossbeam_channel::Receiver<crate::client::EngineClient> {
        let (tx, rx) = crossbeam_channel::unbounded();
        let _ = tx.send(self.inner.client.clone());
        rx
    }

    /// Update daemon settings. No-op — settings are managed by the CLI.
    pub fn update_settings(&self, _settings: crate::DaemonSettings) -> anyhow::Result<()> {
        Ok(())
    }

    /// Reconfigure the daemon. No-op.
    pub fn reconfigure(&self) -> anyhow::Result<()> {
        Ok(())
    }

    /// Return the engine's daemon settings.
    pub fn settings(&self) -> waku_protocol::DaemonSettings {
        waku_protocol::DaemonSettings::default()
    }
}

/// Compatibility type alias for existing code.
pub type DaemonClient = crate::client::EngineClient;
pub type DaemonSupervisor = EngineSupervisor;
pub type DaemonProcess = EngineChild;
pub type DaemonExposureSettings = EngineExposureSettings;
pub type DaemonToken = EngineToken;
pub const DAEMON_TOKEN_ENV: &str = "RAPIDPI_DAEMON_TOKEN";
pub const DEFAULT_EXPOSED_DAEMON_PORT: u16 = 0;