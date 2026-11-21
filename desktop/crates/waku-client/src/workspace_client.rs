use uuid::Uuid;
use waku_protocol::provider_session::{ProviderSessionFork, ProviderSessionForkRequest};
use waku_protocol::{Command, ResponsePayload, WorkspaceOperation, WorkspaceResult};

use crate::client::EngineClient;

/// Typed convenience wrapper around daemon-owned filesystem and Git RPCs.
#[derive(Clone)]
pub struct WorkspaceClient {
    client: EngineClient,
}

impl WorkspaceClient {
    pub fn new(client: EngineClient) -> Self {
        Self { client }
    }

    pub fn request(&self, operation: WorkspaceOperation) -> anyhow::Result<WorkspaceResult> {
        match self
            .client
            .request(Uuid::nil(), Uuid::nil(), Command::Workspace { operation })?
        {
            ResponsePayload::Workspace { result } => Ok(result),
            _ => anyhow::bail!("Rapid Pi daemon returned an invalid workspace response"),
        }
    }

    pub fn fork_provider_session(
        &self,
        request: ProviderSessionForkRequest,
    ) -> anyhow::Result<ProviderSessionFork> {
        match self.client.request(
            Uuid::nil(),
            Uuid::nil(),
            Command::ForkProviderSession { request },
        )? {
            ResponsePayload::ProviderSessionForked { result } => Ok(result),
            _ => anyhow::bail!("Rapid Pi daemon returned an invalid provider-fork response"),
        }
    }
}
