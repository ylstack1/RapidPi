//! Computer Use helper process integration.

use std::path::PathBuf;

// ComputerUseConfig is defined in waku-protocol but re-exported here for compatibility
// The original ComputerUseConfig from this crate is kept for types not in waku-protocol

/// Configuration for computer use server connections.
#[derive(Clone)]
pub struct ComputerUseConfig {
    pub server_path: PathBuf,
    pub repl_path: PathBuf,
    pub skill_path: PathBuf,
    pub process_directory: PathBuf,
}

/// Re-export computer use types from waku-protocol
pub use waku_protocol::computer_use::{
    ComputerAppGrant, ComputerPermissions, ComputerTarget, ComputerToolRequest,
    ComputerUsePhase, ComputerUseState,
};

/// Resolve whether computer use is enabled based on options.
///
/// Clamps the requested enablement to what this build ships (development only).
pub fn resolve_enabled(requested: bool) -> bool {
    requested && cfg!(debug_assertions)
}

/// Get the current working directory.
pub fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_default()
}

/// Path to the pi extension skill directory for computer use.
pub fn pi_extension_path() -> PathBuf {
    // This is a placeholder; the actual path is determined at runtime
    // based on the Rapid Pi application installation.
    PathBuf::from("/Applications/Waku.app/Contents/Resources/skills/waku-computer-use")
}

/// Path to the JS REPL server for computer use.
pub fn js_repl_server_path() -> PathBuf {
    // This is a placeholder; the actual path is determined at runtime.
    PathBuf::from("/Applications/Waku.app/Contents/Resources/waku_js_repl")
}

/// Root path for computer use skills.
pub fn skill_root_path() -> PathBuf {
    // This is a placeholder; the actual path is determined at runtime.
    PathBuf::from("/Applications/Waku.app/Contents/Resources/skills")
}

/// Decode a preview update from data.
pub fn decode_preview_update(data: &[u8]) -> anyhow::Result<ComputerUseState> {
    // TODO: Implement actual preview update decoding
    Ok(ComputerUseState {
        target: None,
        phase: ComputerUsePhase::AwaitingApproval,
        visible: false,
        image_url: None,
    })
}

/// Probe computer use permissions.
pub fn probe_permissions(_prompt: bool) -> anyhow::Result<ComputerPermissions> {
    Ok(ComputerPermissions {
        screen_recording: false,
        accessibility: false,
    })
}