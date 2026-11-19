//! Remote control for the desktop daemon: Noise IK WebSocket server that lets
//! paired phones (Rapid Pi Android app) attach to the local engine.
//! Mirrors `packages/coding-agent/src/remote-control/` in Rust.

pub mod noise;
pub mod protocol;
pub mod store;

mod server;

pub use server::RemoteControlServer;
