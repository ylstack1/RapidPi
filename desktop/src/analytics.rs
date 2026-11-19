//! Minimal analytics stub — no network, no TLS, no external dependencies.
//! Events are accepted but never sent. Analytics is intentionally disabled
//! to avoid pulling in rust-umami/reqwest/rustls/aws-lc-sys which requires
//! C compilation not available in this environment.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use uuid::Uuid;

/// A cheap handle to the analytics worker (no-op stub).
#[derive(Clone)]
pub struct Analytics {
    enabled: Arc<AtomicBool>,
}

impl Analytics {
    /// Starts analytics (always disabled in this stub).
    pub fn new(_language: &'static str, _distinct_id: Uuid, _sharing_enabled: bool) -> Self {
        Self {
            enabled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Enqueues an event (no-op in this stub).
    pub fn track(&self, _event: Event) {}

    /// Applies the user's persisted sharing preference (no-op in this stub).
    pub fn set_enabled(&self, _enabled: bool) {}
}

/// The product vocabulary for analytics events.
pub enum Event {
    AppLaunched {
        task_count: usize,
        project_count: usize,
    },
    ProjectAdded,
    RightPanelOpened,
    TurnSubmitted {
        provider: &'static str,
        model: String,
        turn_number: usize,
        workspace: &'static str,
        projectless: bool,
        attachment_count: usize,
        has_input: bool,
    },
    TurnFinished {
        provider: &'static str,
        turn_number: usize,
        outcome: TurnOutcome,
        duration_seconds: u64,
    },
    PermissionResponded {
        provider: &'static str,
        kind: &'static str,
        decision: &'static str,
    },
    ConversationRolledBack {
        provider: &'static str,
        turns: usize,
    },
    ResponseForked {
        provider: &'static str,
        turn_number: usize,
    },
}

#[derive(Clone, Copy)]
pub enum TurnOutcome {
    Completed,
    Failed,
    Cancelled,
    ProcessExited,
    StartFailed,
    PreparationFailed,
}

impl TurnOutcome {
    fn id(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::ProcessExited => "process_exited",
            Self::StartFailed => "start_failed",
            Self::PreparationFailed => "preparation_failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytics_is_disabled_by_default() {
        let analytics = Analytics::new("en", Uuid::new_v4(), true);
        assert!(!analytics.enabled.load(Ordering::Relaxed));
    }

    #[test]
    fn tracking_is_a_noop() {
        let analytics = Analytics::new("en", Uuid::new_v4(), true);
        analytics.track(Event::ProjectAdded);
        analytics.track(Event::AppLaunched {
            task_count: 1,
            project_count: 1,
        });
        // No panic, no network — just a no-op.
    }
}
