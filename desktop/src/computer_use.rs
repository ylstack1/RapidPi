//! Computer use types for the desktop app.
//! Re-exports the protocol types and adds desktop-specific preview stubs.

use std::default::Default;
use std::fmt::Debug;

/// Re-export the real protocol types so the desktop app can use them.
pub use waku_protocol::computer_use::{
    ComputerAppGrant, ComputerPermissions, ComputerTarget, ComputerToolRequest, ComputerUsePhase,
    ComputerUseState,
};

/// A pending computer-use approval.
#[derive(Clone, Debug)]
pub struct PendingComputerApproval {
    pub grant_key: String,
    pub persistable: bool,
    pub bundle_id: String,
    pub app_name: String,
    pub target: Option<ComputerTarget>,
    pub request: ComputerToolRequest,
    pub sensitive: bool,
}

/// An image captured or used during computer use sessions.
#[derive(Clone, Debug, Default)]
pub struct PreviewImage {
    pub source_id: u64,
}

impl PreviewImage {
    pub fn new(_source_id: u64, _image: (), _cx: &mut gpui::App) -> Self {
        Self::default()
    }
}

/// A container for preview frames during computer use.
#[derive(Default)]
pub struct PreviewFrames<T>(std::marker::PhantomData<T>);

impl<T> PreviewFrames<T> {
    pub fn begin(&self) -> u64 {
        0
    }

    pub fn complete(&mut self, _generation: u64, _image: Option<T>) -> bool {
        false
    }

    pub fn current(&self) -> Option<PreviewFrameRef> {
        None
    }
}

#[derive(Clone, Debug)]
pub struct PreviewFrameRef {
    pub source_id: u64,
}

/// Returns `false` — computer use is not available in this desktop build.
pub fn is_available() -> bool {
    false
}

/// Returns a helper display name string.
pub fn helper_display_name() -> String {
    "Helper".to_string()
}

/// Decode a computer-use preview image URL into a source id and image.
pub fn decode_preview_image_url<R>(
    _image_url: &str,
    _renderer: R,
    _current_source: Option<u64>,
) -> anyhow::Result<Option<(u64, ())>> {
    Ok(None)
}
