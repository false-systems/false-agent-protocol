use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    ProtocolIncompatible,
    CapabilityMissing,
    PluginNotFound,
    PluginTimeout,
    PluginCrashed,
    MalformedFrame,
    OversizedFrame,
    InvalidRequest,
    InvalidResponse,
    RepositoryMismatch,
    WorkMismatch,
    ContextDigestMismatch,
    LaunchPlanRejected,
    ResultInvalid,
    OperationUnsupported,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize, Error, PartialEq)]
#[error("{code:?}: {message}")]
pub struct ProtocolError {
    pub code: ErrorCode,
    pub message: String,
    pub recoverable: bool,
    pub authority: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ProtocolError {
    pub fn new(
        code: ErrorCode,
        message: impl Into<String>,
        recoverable: bool,
        authority: impl Into<String>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            recoverable,
            authority: authority.into(),
            details: None,
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::InternalError,
            message,
            false,
            "false-agent-protocol",
        )
    }
}
