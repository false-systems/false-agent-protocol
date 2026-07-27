use serde::{Deserialize, Serialize};

use crate::{
    context::ExecutionContextEnvelope, work::ProcessOutcome, ActionSummary, BlockerSummary,
    CompletionLevel, EvidenceSummary, RepositoryPoint, RepositoryPointRef, WorkerRunRef,
};

pub const DESCRIBE_METHOD: &str = "worker.describe";
pub const PREPARE_METHOD: &str = "worker.prepare";
pub const FINALIZE_METHOD: &str = "worker.finalize";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkerFeature {
    InteractivePty,
    InitialContext,
    RefreshContext,
    StructuredResult,
    Resume,
    GracefulInterrupt,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContextMode {
    File,
    Environment,
    Argument,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerDescriptor {
    pub name: String,
    pub version: String,
    pub operating_systems: Vec<String>,
    pub features: Vec<WorkerFeature>,
    pub context_modes: Vec<ContextMode>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Interactive,
    NonInteractive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TerminalCapabilities {
    pub is_terminal: bool,
    pub color: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub columns: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrepareParams {
    pub context: ExecutionContextEnvelope,
    pub repository_root: String,
    pub session_directory: String,
    #[serde(default)]
    pub harness_arguments: Vec<String>,
    pub terminal: TerminalCapabilities,
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnvironmentEntry {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StdinMode {
    Inherit,
    Null,
    Piped,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminalMode {
    Inherit,
    Pty,
    None,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetentionPolicy {
    Ephemeral,
    RetainOnFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeArtifact {
    pub path: String,
    pub retention: RetentionPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResultContract {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub structured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LaunchPlan {
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<EnvironmentEntry>,
    pub cwd: String,
    pub stdin_mode: StdinMode,
    pub terminal_mode: TerminalMode,
    pub context_artifacts: Vec<RuntimeArtifact>,
    pub result_contract: ResultContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FinalizeParams {
    pub outcome: ProcessOutcome,
    pub origin: RepositoryPoint,
    pub destination: RepositoryPoint,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_result_path: Option<String>,
    pub session_id: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultGrade {
    Structured,
    Observed,
    ExitOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NormalizedWorkerResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<WorkerRunRef>,
    pub origin: RepositoryPointRef,
    pub destination: RepositoryPointRef,
    #[serde(default)]
    pub actions: Vec<ActionSummary>,
    #[serde(default)]
    pub evidence: Vec<EvidenceSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_completion: Option<CompletionLevel>,
    #[serde(default)]
    pub blockers: Vec<BlockerSummary>,
    pub result_grade: ResultGrade,
}
