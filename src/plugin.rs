use std::{fmt, num::NonZeroU64, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::{
    error::{ErrorCode, ProtocolError},
    ProtocolVersion, RepositoryRef,
};

pub const JSONRPC_VERSION: &str = "2.0";
pub const WORK_V1: &str = "work.v1";
pub const WORKER_V1: &str = "worker.v1";
pub const INITIALIZE_METHOD: &str = "plugin.initialize";
pub const SHUTDOWN_METHOD: &str = "plugin.shutdown";
pub const MAX_STRING_SIZE: usize = 256 * 1024;
pub const MAX_ENVIRONMENT_ENTRIES: usize = 128;
pub const MAX_ARGUMENTS: usize = 256;
pub const MAX_ARGUMENT_LENGTH: usize = 64 * 1024;
pub const MAX_OBLIGATIONS: usize = 4096;
pub const MAX_EVIDENCE_REFERENCES: usize = 4096;

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ProtocolError> {
                let value = value.into();
                if value.is_empty()
                    || value.len() > 128
                    || !value.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-')
                    })
                {
                    return Err(ProtocolError::new(
                        ErrorCode::InvalidRequest,
                        concat!("invalid ", stringify!($name)),
                        false,
                        "false-agent-protocol",
                    ));
                }
                Ok(Self(value))
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = ProtocolError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }
    };
}

string_id!(PluginId);
string_id!(CapabilityId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RequestId(NonZeroU64);

impl RequestId {
    pub fn new(value: u64) -> Result<Self, ProtocolError> {
        NonZeroU64::new(value).map(Self).ok_or_else(|| {
            ProtocolError::new(
                ErrorCode::InvalidRequest,
                "request ID must be nonzero",
                false,
                "false-agent-protocol",
            )
        })
    }

    pub fn get(self) -> u64 {
        self.0.get()
    }
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(self.get())
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u64::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PluginKind {
    Work,
    Worker,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Lifecycle {
    Spawned,
    Initialized,
    Ready,
    ShuttingDown,
    Exited,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

impl RpcRequest {
    pub fn new(id: RequestId, method: impl Into<String>, params: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.into(),
            id,
            method: method.into(),
            params,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub id: RequestId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
}

impl RpcResponse {
    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.jsonrpc != JSONRPC_VERSION || (self.result.is_some() == self.error.is_some()) {
            return Err(ProtocolError::new(
                ErrorCode::InvalidResponse,
                "response must contain exactly one of result or error",
                false,
                "plugin",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HostInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolRange {
    pub minimum: ProtocolVersion,
    pub maximum: ProtocolVersion,
}

impl ProtocolRange {
    pub fn v1() -> Self {
        Self {
            minimum: ProtocolVersion::V1,
            maximum: ProtocolVersion::V1,
        }
    }

    pub fn select(self, offered: Self) -> Result<ProtocolVersion, ProtocolError> {
        if self.minimum.major != offered.minimum.major
            || self.maximum.major != offered.maximum.major
        {
            return Err(ProtocolError::new(
                ErrorCode::ProtocolIncompatible,
                "no compatible protocol major version",
                false,
                "false-agent-protocol",
            ));
        }
        Ok(ProtocolVersion {
            major: self.maximum.major,
            minor: self.maximum.minor.min(offered.maximum.minor),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryIdentity {
    pub id: RepositoryRef,
    pub root_hint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeParams {
    pub host: HostInfo,
    pub protocol: ProtocolRange,
    pub requested_capabilities: Vec<CapabilityId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryIdentity>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginInfo {
    pub id: PluginId,
    pub name: String,
    pub version: String,
    pub kind: PluginKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilityDescriptor {
    pub id: CapabilityId,
    pub methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginLimits {
    pub maximum_frame_size: usize,
    pub maximum_string_size: usize,
    pub maximum_obligations: usize,
    pub maximum_evidence_references: usize,
    pub maximum_environment_entries: usize,
    pub maximum_argument_count: usize,
    pub maximum_argument_length: usize,
    pub request_timeout_ms: u64,
    pub initialization_timeout_ms: u64,
    pub shutdown_timeout_ms: u64,
}

impl Default for PluginLimits {
    fn default() -> Self {
        Self {
            maximum_frame_size: crate::framing::MAX_FRAME_SIZE,
            maximum_string_size: MAX_STRING_SIZE,
            maximum_obligations: MAX_OBLIGATIONS,
            maximum_evidence_references: MAX_EVIDENCE_REFERENCES,
            maximum_environment_entries: MAX_ENVIRONMENT_ENTRIES,
            maximum_argument_count: MAX_ARGUMENTS,
            maximum_argument_length: MAX_ARGUMENT_LENGTH,
            request_timeout_ms: 5_000,
            initialization_timeout_ms: 5_000,
            shutdown_timeout_ms: 1_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InitializeResult {
    pub plugin: PluginInfo,
    pub selected_protocol: ProtocolVersion,
    pub capabilities: Vec<CapabilityDescriptor>,
    pub limits: PluginLimits,
}

impl InitializeResult {
    pub fn validate(&self, requested: &[CapabilityId]) -> Result<(), ProtocolError> {
        self.selected_protocol.validate().map_err(|error| {
            ProtocolError::new(
                ErrorCode::ProtocolIncompatible,
                error.to_string(),
                false,
                "plugin",
            )
        })?;
        let mut ids = std::collections::BTreeSet::new();
        for capability in &self.capabilities {
            if !ids.insert(capability.id.clone()) {
                return Err(ProtocolError::new(
                    ErrorCode::InvalidResponse,
                    "plugin returned duplicate capability IDs",
                    false,
                    "plugin",
                ));
            }
        }
        if requested.iter().any(|id| !ids.contains(id)) {
            return Err(ProtocolError::new(
                ErrorCode::CapabilityMissing,
                "plugin omitted a requested capability",
                true,
                "plugin",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_ids_and_negotiation_fail_closed() {
        assert!(RequestId::new(0).is_err());
        assert!(ProtocolRange::v1()
            .select(ProtocolRange {
                minimum: ProtocolVersion { major: 2, minor: 0 },
                maximum: ProtocolVersion { major: 2, minor: 0 },
            })
            .is_err());
    }
}
