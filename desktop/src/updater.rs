//! Updater stub — disabled for lightweight desktop build.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use gpui::Global;
use smol::channel;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Available,
    Updating,
}

#[derive(Clone, Debug)]
pub enum UpdaterEvent {
    StatusChanged(UpdateStatus),
    UpToDate,
    Failed(String),
    #[cfg(target_os = "linux")]
    QuitAndInstall,
}

pub struct UpdaterState(pub(crate) Option<UpdaterStateInner>);

pub(crate) struct UpdaterStateInner {
    status: Mutex<UpdateStatus>,
    automatically_checks_for_updates: AtomicBool,
    event_tx: Mutex<channel::Sender<UpdaterEvent>>,
    event_rx: channel::Receiver<UpdaterEvent>,
}

impl Global for UpdaterState {}

impl UpdaterState {
    pub fn new() -> Self {
        let (tx, rx) = channel::bounded(16);
        Self(Some(UpdaterStateInner {
            status: Mutex::new(UpdateStatus::Idle),
            automatically_checks_for_updates: AtomicBool::new(true),
            event_tx: Mutex::new(tx),
            event_rx: rx,
        }))
    }

    pub fn status(&self) -> UpdateStatus {
        self.0
            .as_ref()
            .map_or(UpdateStatus::Idle, |inner| *inner.status.lock().unwrap())
    }

    pub fn events(&self) -> channel::Receiver<UpdaterEvent> {
        self.0
            .as_ref()
            .map(|inner| inner.event_rx.clone())
            .unwrap_or_else(|| {
                let (_, rx) = channel::bounded(16);
                rx
            })
    }

    pub fn automatically_checks_for_updates(&self) -> bool {
        self.0
            .as_ref()
            .map_or(true, |inner| inner.automatically_checks_for_updates.load(Ordering::Relaxed))
    }

    pub fn set_automatically_checks_for_updates(&self, enabled: bool) {
        if let Some(inner) = self.0.as_ref() {
            inner
                .automatically_checks_for_updates
                .store(enabled, Ordering::Relaxed);
        }
    }

    pub fn install_available_update(&self) -> bool {
        if let Some(inner) = self.0.as_ref() {
            let mut status = inner.status.lock().unwrap();
            if *status == UpdateStatus::Available {
                *status = UpdateStatus::Updating;
                let _ = inner
                    .event_tx
                    .lock()
                    .unwrap()
                    .try_send(UpdaterEvent::StatusChanged(UpdateStatus::Updating));
                return true;
            }
        }
        false
    }
}

impl Default for UpdaterState {
    fn default() -> Self {
        Self::new()
    }
}
