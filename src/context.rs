use serde::{Deserialize, Serialize};

use crate::{
    digest::sha256_without,
    error::{ErrorCode, ProtocolError},
    ChangedPath, CompletionLevel, Extensions, ProtocolVersion, RepositoryPoint, Timestamp,
    WorkPacketEnvelope, WorkflowStatusSummary,
};

pub type ChangedSurface = ChangedPath;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryCapabilities {
    #[serde(default)]
    pub reachable_completion: Vec<CompletionLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionContextEnvelope {
    pub protocol: ProtocolVersion,
    pub generation: u64,
    pub generated_at: Timestamp,
    pub repository: RepositoryPoint,
    #[serde(default)]
    pub changed_surfaces: Vec<ChangedSurface>,
    pub repository_capabilities: RepositoryCapabilities,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<WorkPacketEnvelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflow_status: Option<WorkflowStatusSummary>,
    pub context_digest: String,
    #[serde(default)]
    pub extensions: Extensions,
}

impl ExecutionContextEnvelope {
    pub fn seal(&mut self) -> Result<(), ProtocolError> {
        self.context_digest = sha256_without(self, "context_digest")?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        self.protocol.validate().map_err(|error| {
            ProtocolError::new(
                ErrorCode::ProtocolIncompatible,
                error.to_string(),
                false,
                "false-agent-protocol",
            )
        })?;
        if let Some(work) = &self.work {
            work.validate().map_err(|error| {
                ProtocolError::new(ErrorCode::InvalidRequest, error.to_string(), false, "teko")
            })?;
        }
        let expected = sha256_without(self, "context_digest")?;
        if expected != self.context_digest {
            return Err(ProtocolError::new(
                ErrorCode::ContextDigestMismatch,
                "execution context digest does not match its content",
                false,
                "toimija",
            ));
        }
        Ok(())
    }
}
