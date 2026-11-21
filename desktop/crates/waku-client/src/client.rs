//! RPC client for the rapid-pi engine.
//!
//! Wraps [`crate::process::EngineClient`] and provides typed request/notify/subscribe
//! methods over the stdio JSON-RPC transport.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use anyhow::{Context as _, anyhow, bail};
use crossbeam_channel::{Receiver, Sender, bounded, unbounded};
use parking_lot::Mutex;
use uuid::Uuid;

use crate::process::EngineClient as ProcessEngineClient;
use waku_protocol::{
    ClientMessage, Command, ReplayCursor, Request, ResponseOutcome,
    ResponsePayload, RpcError, SequencedEvent, ServerMessage,
};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const MAX_BUFFERED_EVENTS_PER_RUNTIME: usize = 4096;

struct ClientInner {
    process: Mutex<Option<ProcessEngineClient>>,
    pending: Mutex<HashMap<Uuid, crossbeam_channel::Sender<Result<ResponsePayload, RpcError>>>>,
    sessions: Mutex<HashMap<(Uuid, Uuid), Sender<SequencedEvent>>>,
    pending_events: Mutex<HashMap<(Uuid, Uuid), VecDeque<SequencedEvent>>>,
    task_state_subscribers: Mutex<Vec<Sender<u64>>>,
    last_sequences: Mutex<HashMap<(Uuid, Uuid), LastSequence>>,
    disconnected: AtomicBool,
}

#[derive(Clone, Copy)]
struct LastSequence {
    epoch: Uuid,
    sequence: u64,
}

#[derive(Clone)]
pub struct EngineClient {
    inner: Arc<ClientInner>,
}

impl EngineClient {
    /// Connect to a running engine process.
    pub fn connect(process: ProcessEngineClient) -> anyhow::Result<Self> {
        let inner = Arc::new(ClientInner {
            process: Mutex::new(Some(process)),
            pending: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            pending_events: Mutex::new(HashMap::new()),
            task_state_subscribers: Mutex::new(Vec::new()),
            last_sequences: Mutex::new(HashMap::new()),
            disconnected: AtomicBool::new(false),
        });

        // Spawn a reader thread to process incoming server messages.
        let thread_inner = inner.clone();
        std::thread::Builder::new()
            .name("rapidpi-engine-reader".into())
            .spawn(move || read_loop(thread_inner))
            .context("could not start Rapid Pi engine reader thread")?;

        Ok(Self { inner })
    }

    /// Create a disconnected client. All requests will fail.
    pub fn disconnected() -> Self {
        let inner = Arc::new(ClientInner {
            process: Mutex::new(None),
            pending: Mutex::new(HashMap::new()),
            sessions: Mutex::new(HashMap::new()),
            pending_events: Mutex::new(HashMap::new()),
            task_state_subscribers: Mutex::new(Vec::new()),
            last_sequences: Mutex::new(HashMap::new()),
            disconnected: AtomicBool::new(true),
        });
        Self { inner }
    }

    pub fn subscribe(&self, session_id: Uuid, runtime_id: Uuid) -> Receiver<SequencedEvent> {
        let (events, receiver) = unbounded();
        let key = (session_id, runtime_id);
        let mut sessions = self.inner.sessions.lock();
        sessions.insert(key, events.clone());
        if let Some(buffered) = self.inner.pending_events.lock().remove(&key) {
            for event in buffered {
                let _ = events.send(event);
            }
        }
        receiver
    }

    pub fn same_connection(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    pub fn is_disconnected(&self) -> bool {
        self.inner.disconnected.load(Ordering::Acquire)
    }

    pub fn unsubscribe(&self, session_id: Uuid, runtime_id: Uuid) {
        self.inner.sessions.lock().remove(&(session_id, runtime_id));
    }

    pub fn subscribe_task_state(&self) -> Receiver<u64> {
        let (events, receiver) = unbounded();
        self.inner.task_state_subscribers.lock().push(events);
        receiver
    }

    pub fn request(
        &self,
        session_id: Uuid,
        runtime_id: Uuid,
        command: Command,
    ) -> anyhow::Result<ResponsePayload> {
        if self.inner.disconnected.load(Ordering::Acquire) {
            bail!("Rapid Pi engine is disconnected");
        }
        let request_id = Uuid::new_v4();
        let (response, response_rx) = bounded(1);
        self.inner.pending.lock().insert(request_id, response);
        let message = ClientMessage::Request(Request {
            request_id,
            session_id,
            runtime_id,
            command,
        });
        let mut process = self.inner.process.lock();
        let process = process.as_mut().ok_or_else(|| {
            anyhow::anyhow!("Rapid Pi engine process is not available")
        })?;
        process.send(message.clone()).map_err(|e| {
            self.inner.pending.lock().remove(&request_id);
            anyhow!(e)
        })?;
        match response_rx.recv_timeout(REQUEST_TIMEOUT) {
            Ok(Ok(payload)) => Ok(payload),
            Ok(Err(error)) => Err(anyhow!(error.message)),
            Err(error) => {
                self.inner.pending.lock().remove(&request_id);
                Err(anyhow!("timed out waiting for Rapid Pi engine: {error}"))
            }
        }
    }

    pub fn notify(
        &self,
        session_id: Uuid,
        runtime_id: Uuid,
        command: Command,
    ) -> anyhow::Result<()> {
        if self.inner.disconnected.load(Ordering::Acquire) {
            bail!("Rapid Pi engine is disconnected");
        }
        let mut process = self.inner.process.lock();
        let process = process.as_mut().ok_or_else(|| {
            anyhow::anyhow!("Rapid Pi engine process is not available")
        })?;
        process.send(ClientMessage::Request(Request {
            request_id: Uuid::nil(),
            session_id,
            runtime_id,
            command,
        }))?;
        Ok(())
    }

    pub fn last_sequences(&self) -> Vec<ReplayCursor> {
        self.inner
            .last_sequences
            .lock()
            .iter()
            .map(|(&(session_id, runtime_id), cursor)| ReplayCursor {
                session_id,
                runtime_id,
                epoch: cursor.epoch,
                sequence: cursor.sequence,
            })
            .collect()
    }

    pub fn shutdown(&self) {
        if let Some(process) = self.inner.process.lock().as_mut() {
            let _ = process.send(ClientMessage::Shutdown);
        }
        self.inner.disconnected.store(true, Ordering::Release);
    }
}

fn read_loop(inner: Arc<ClientInner>) {
    loop {
        if inner.disconnected.load(Ordering::Acquire) {
            break;
        }
        let mut process = inner.process.lock();
        let Some(process) = process.as_mut() else {
            break;
        };
        match process.recv() {
            Ok(Some(server_msg)) => match server_msg {
                ServerMessage::Response {
                    request_id,
                    outcome,
                } => {
                    if let Some(sender) = inner.pending.lock().remove(&request_id) {
                        let result = match outcome {
                            ResponseOutcome::Ok { payload } => Ok(payload),
                            ResponseOutcome::Error { error } => Err(RpcError { message: error.message }),
                        };
                        let _ = sender.send(result);
                    }
                }
                ServerMessage::Event(event) => {
                    let should_deliver = {
                        let mut sequences = inner.last_sequences.lock();
                        let previous = sequences
                            .entry((event.session_id, event.runtime_id))
                            .or_insert(LastSequence {
                                epoch: event.epoch,
                                sequence: 0,
                            });
                        if previous.epoch == event.epoch && event.sequence <= previous.sequence {
                            false
                        } else {
                            previous.epoch = event.epoch;
                            previous.sequence = event.sequence;
                            true
                        }
                    };
                    if should_deliver {
                        let key = (event.session_id, event.runtime_id);
                        let sessions = inner.sessions.lock();
                        if let Some(events) = sessions.get(&key) {
                            let _ = events.send(event);
                        } else {
                            let mut pending = inner.pending_events.lock();
                            let buffered = pending.entry(key).or_default();
                            buffered.push_back(event);
                            while buffered.len() > MAX_BUFFERED_EVENTS_PER_RUNTIME {
                                buffered.pop_front();
                            }
                        }
                    }
                }
                ServerMessage::TaskStateChanged { revision } => {
                    inner
                        .task_state_subscribers
                        .lock()
                        .retain(|subscriber| subscriber.send(revision).is_ok());
                }
                ServerMessage::ShuttingDown => {
                    inner.disconnected.store(true, Ordering::Release);
                    break;
                }
                ServerMessage::Hello { .. } | ServerMessage::Rejected { .. } => {}
            },
            Ok(None) => break, // EOF
            Err(_) => break,
        }
    }

    inner.disconnected.store(true, Ordering::Release);
    let pending = std::mem::take(&mut *inner.pending.lock());
    for (_, sender) in pending {
        let _ = sender.send(Err(RpcError {
            message: "Rapid Pi engine disconnected".into(),
        }));
    }
    drop(std::mem::take(&mut *inner.sessions.lock()));
    inner.task_state_subscribers.lock().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder() {
        // Tests would require a running engine
    }
}