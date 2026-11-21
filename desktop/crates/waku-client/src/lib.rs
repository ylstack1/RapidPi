//! Rust transport and lifecycle support for clients of the rapid-pi engine.
//!
//! This crate intentionally depends only on [`waku_protocol`], so GUI and CLI
//! clients cannot accidentally reach engine-owned filesystem, Git, database,
//! or provider implementations.

mod client;
pub mod command_env;
pub mod composer_complete;
pub mod computer_use;
pub mod driver;
pub mod persistence;
mod process;
mod workspace_client;

pub use process::{
    DaemonClient, DaemonExposureSettings, DaemonSupervisor, EngineClient, EngineSupervisor,
    parse_allowed_origins,
};

pub use waku_protocol::*;
pub use workspace_client::WorkspaceClient;

pub mod git_branch {
    pub use waku_protocol::git::{BranchEntry, BranchSnapshot};
}

pub mod git_commit {
    pub use waku_protocol::git::AgentInvocation;
    pub use waku_protocol::git::CommitSnapshot as Snapshot;
}

pub mod worktree {
    pub use waku_protocol::git::CreatedWorktree;
}