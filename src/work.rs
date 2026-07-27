use serde::{Deserialize, Serialize};

use crate::{
    CompletionLevel, EvidenceRef, RepositoryPoint, WorkPacketEnvelope, WorkRef,
    WorkerRunResultEnvelope, WorkflowStatusSummary,
};

pub const DESCRIBE_METHOD: &str = "work.describe";
pub const CURRENT_METHOD: &str = "work.current";
pub const OBSERVE_METHOD: &str = "work.observe";
pub const EXPLAIN_METHOD: &str = "work.explain";
pub const CLOSE_METHOD: &str = "work.close";

pub type ActiveWorkPacket = WorkPacketEnvelope;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkDescriptor {
    pub provider_version: String,
    pub protocol_version: String,
    pub repository_binding: bool,
    pub completion_levels: Vec<CompletionLevel>,
    pub evidence_import: bool,
    pub explain: bool,
    pub close: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrentWorkParams {
    pub repository_point: RepositoryPoint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previously_known_work: Option<WorkRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CurrentWorkResult {
    None,
    Active { work: Box<ActiveWorkPacket> },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObserveParams {
    pub work: WorkRef,
    pub worker_result: WorkerRunResultEnvelope,
    pub repository_point: RepositoryPoint,
    #[serde(default)]
    pub related_evidence: Vec<EvidenceRef>,
    pub outcome: ProcessOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProcessOutcome {
    Exited { code: i32 },
    Signaled { signal: i32 },
    Interrupted { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainParams {
    pub repository_point: RepositoryPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExplainResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work: Option<WorkRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<WorkflowStatusSummary>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloseParams {
    pub repository_point: RepositoryPoint,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloseResult {
    pub closed: bool,
    pub explanation: String,
}
