use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};

use crate::{
    ActionRef, CompletionWitnessRef, EvidenceRef, GateReceiptRef, GateRef, ObligationRef,
    ProtocolVersion, RepositoryPointRef, RepositoryRef, SemanticError, SpecRef, ToolCallRef,
    WorkRef, WorkerRunRef,
};

pub type Extensions = BTreeMap<String, Value>;
const MAX_ITEMS: usize = 4096;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct Timestamp(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct ConsistencyToken(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct TreeOid(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(transparent)]
pub struct BranchName(pub String);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum CompletionLevel {
    Changed,
    Validated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpecSummary {
    pub id: SpecRef,
    pub title: String,
    pub objective: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub goals: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub non_goals: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkSummary {
    pub id: WorkRef,
    pub spec: SpecRef,
    pub objective: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScopeSummary {
    pub write: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ObligationKind {
    Manual,
    Gate,
    Scope,
    RepositoryChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObligationSummary {
    pub id: ObligationRef,
    pub description: String,
    pub kind: ObligationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateRequirementSummary {
    pub id: GateRef,
    pub obligation: ObligationRef,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkPacketEnvelope {
    pub protocol: ProtocolVersion,
    pub spec: SpecSummary,
    pub work: WorkSummary,
    pub scope: ScopeSummary,
    pub obligations: Vec<ObligationSummary>,
    pub gates: Vec<GateRequirementSummary>,
    pub required_completion: CompletionLevel,
    #[serde(default)]
    pub extensions: Extensions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryPoint {
    pub id: RepositoryPointRef,
    pub repository: RepositoryRef,
    pub consistency_token: ConsistencyToken,
    pub tree_oid: TreeOid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<BranchName>,
    pub dirty: bool,
    pub captured_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RepositoryPointEnvelope {
    pub protocol: ProtocolVersion,
    pub repository: RepositoryPoint,
    #[serde(default)]
    pub extensions: Extensions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChangedPath {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObligationState {
    pub obligation: ObligationRef,
    pub supported: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockerSummary {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkflowStatusSummary {
    pub obligations: Vec<ObligationState>,
    pub blockers: Vec<BlockerSummary>,
    pub closable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapabilitySummary {
    pub reachable_completion: Vec<CompletionLevel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionContextEnvelope {
    pub protocol: ProtocolVersion,
    pub generated_at: Timestamp,
    pub generation: u64,
    pub work: WorkPacketEnvelope,
    pub repository: RepositoryPoint,
    pub changed_paths: Vec<ChangedPath>,
    pub workflow_status: WorkflowStatusSummary,
    pub capabilities: CapabilitySummary,
    pub context_digest: String,
    #[serde(default)]
    pub extensions: Extensions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunBinding {
    pub run: WorkerRunRef,
    pub work: WorkRef,
    pub spec: SpecRef,
    pub origin: RepositoryPointRef,
    pub context_digest: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    WriteFile,
    ApplyPatch,
    RevertBatch,
    ExecuteBuild,
    RunGate,
    Commit,
    Push,
    ProposeCompletion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActionSummary {
    pub id: ActionRef,
    pub kind: ActionKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call: Option<ToolCallRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transforms_from: Option<RepositoryPointRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transforms_to: Option<RepositoryPointRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvidenceSummary {
    pub id: EvidenceRef,
    pub digest: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supports: Vec<ObligationRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkerRunResultEnvelope {
    pub protocol: ProtocolVersion,
    pub run: WorkerRunRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding: Option<RunBinding>,
    pub origin: RepositoryPointRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination: Option<RepositoryPointRef>,
    pub actions: Vec<ActionSummary>,
    pub evidence: Vec<EvidenceSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claimed_completion: Option<CompletionLevel>,
    pub unresolved_blockers: Vec<BlockerSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_ref: Option<String>,
    pub result_digest: String,
    #[serde(default)]
    pub extensions: Extensions,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GateReceiptBinding {
    pub receipt: GateReceiptRef,
    pub gate: GateRef,
    pub work: WorkRef,
    pub repository_point: RepositoryPointRef,
    pub evidence: EvidenceRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompletionWitnessEnvelope {
    pub protocol: ProtocolVersion,
    pub witness: CompletionWitnessRef,
    pub work: WorkRef,
    pub spec: SpecRef,
    pub origin: RepositoryPointRef,
    pub destination: RepositoryPointRef,
    pub satisfied_obligations: Vec<ObligationRef>,
    pub gate_receipts: Vec<GateReceiptRef>,
    pub worker_runs: Vec<WorkerRunRef>,
    pub evidence: Vec<EvidenceRef>,
    pub completion: CompletionLevel,
    pub witness_digest: String,
    #[serde(default)]
    pub extensions: Extensions,
}

impl WorkPacketEnvelope {
    pub fn validate(&self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        if self.work.spec != self.spec.id {
            return Err(SemanticError::InvalidPayload(format!(
                "work `{}` references spec `{}`, packet contains `{}`",
                self.work.id, self.work.spec, self.spec.id
            )));
        }
        if self.scope.write.is_empty() {
            return Err(SemanticError::InvalidPayload(
                "work packet has no write scope".to_string(),
            ));
        }
        if self.obligations.len() > MAX_ITEMS
            || self.gates.len() > MAX_ITEMS
            || self.scope.write.len() > MAX_ITEMS
        {
            return Err(SemanticError::InvalidPayload(format!(
                "work packet collections exceed the {MAX_ITEMS}-item limit"
            )));
        }
        for gate in &self.gates {
            if !self
                .obligations
                .iter()
                .any(|obligation| obligation.id == gate.obligation)
            {
                return Err(SemanticError::InvalidPayload(format!(
                    "gate `{}` references missing obligation `{}`",
                    gate.id, gate.obligation
                )));
            }
        }
        Ok(())
    }
}

impl RepositoryPointEnvelope {
    pub fn validate(&self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        if self.repository.consistency_token.0.trim().is_empty()
            || self.repository.tree_oid.0.trim().is_empty()
        {
            return Err(SemanticError::InvalidPayload(
                "repository point requires a consistency token and tree OID".to_string(),
            ));
        }
        Ok(())
    }
}

impl ExecutionContextEnvelope {
    pub fn seal(&mut self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        self.work.validate()?;
        self.context_digest = digest_without(self, "context_digest")?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        self.work.validate()?;
        if self.protocol != self.work.protocol {
            return Err(SemanticError::InvalidPayload(format!(
                "execution context protocol {}.{} does not match work packet {}.{}",
                self.protocol.major,
                self.protocol.minor,
                self.work.protocol.major,
                self.work.protocol.minor
            )));
        }
        if self.changed_paths.len() > MAX_ITEMS
            || self.workflow_status.obligations.len() > MAX_ITEMS
            || self.workflow_status.blockers.len() > MAX_ITEMS
            || self.capabilities.reachable_completion.len() > MAX_ITEMS
        {
            return Err(SemanticError::InvalidPayload(format!(
                "execution context collections exceed the {MAX_ITEMS}-item limit"
            )));
        }
        validate_repository_point(&self.repository)?;
        verify_digest(self, "context_digest", &self.context_digest)
    }
}

impl WorkerRunResultEnvelope {
    pub fn seal(&mut self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        if self.actions.len() > MAX_ITEMS
            || self.evidence.len() > MAX_ITEMS
            || self.unresolved_blockers.len() > MAX_ITEMS
        {
            return Err(SemanticError::InvalidPayload(format!(
                "worker result collections exceed the {MAX_ITEMS}-item limit"
            )));
        }
        self.result_digest = digest_without(self, "result_digest")?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        if self.actions.len() > MAX_ITEMS
            || self.evidence.len() > MAX_ITEMS
            || self.unresolved_blockers.len() > MAX_ITEMS
        {
            return Err(SemanticError::InvalidPayload(format!(
                "worker result collections exceed the {MAX_ITEMS}-item limit"
            )));
        }
        if let Some(binding) = &self.binding {
            if binding.run != self.run || binding.origin != self.origin {
                return Err(SemanticError::InvalidPayload(
                    "run binding does not match result run/origin".to_string(),
                ));
            }
        }
        verify_digest(self, "result_digest", &self.result_digest)
    }
}

impl CompletionWitnessEnvelope {
    pub fn seal(&mut self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        self.witness_digest = digest_without(self, "witness_digest")?;
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SemanticError> {
        self.protocol.validate()?;
        if self.satisfied_obligations.len() > MAX_ITEMS
            || self.gate_receipts.len() > MAX_ITEMS
            || self.worker_runs.len() > MAX_ITEMS
            || self.evidence.len() > MAX_ITEMS
        {
            return Err(SemanticError::InvalidPayload(format!(
                "completion witness collections exceed the {MAX_ITEMS}-item limit"
            )));
        }
        verify_digest(self, "witness_digest", &self.witness_digest)
    }
}

fn validate_repository_point(point: &RepositoryPoint) -> Result<(), SemanticError> {
    if point.consistency_token.0.trim().is_empty() || point.tree_oid.0.trim().is_empty() {
        Err(SemanticError::InvalidPayload(
            "repository point requires a consistency token and tree OID".to_string(),
        ))
    } else {
        Ok(())
    }
}

fn verify_digest<T: Serialize>(
    value: &T,
    field: &str,
    observed: &str,
) -> Result<(), SemanticError> {
    let expected = digest_without(value, field)?;
    if expected == observed {
        Ok(())
    } else {
        Err(SemanticError::DigestMismatch {
            expected,
            observed: observed.to_string(),
        })
    }
}

fn digest_without<T: Serialize>(value: &T, field: &str) -> Result<String, SemanticError> {
    let mut value = serde_json::to_value(value)
        .map_err(|error| SemanticError::InvalidPayload(error.to_string()))?;
    let map = value
        .as_object_mut()
        .ok_or_else(|| SemanticError::InvalidPayload("envelope is not an object".to_string()))?;
    map.insert(field.to_string(), Value::String(String::new()));
    let bytes = serde_json::to_vec(&canonical_value(value))
        .map_err(|error| SemanticError::InvalidPayload(error.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn canonical_value(value: Value) -> Value {
    match value {
        Value::Object(object) => {
            let mut entries = object.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonical_value(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonical_value).collect()),
        scalar => scalar,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_fixtures_parse_and_validate() {
        let work: WorkPacketEnvelope = serde_json::from_str(crate::WORK_PACKET_FIXTURE).unwrap();
        work.validate().unwrap();

        let context: ExecutionContextEnvelope =
            serde_json::from_str(crate::EXECUTION_CONTEXT_FIXTURE).unwrap();
        context.validate().unwrap();

        let result: WorkerRunResultEnvelope =
            serde_json::from_str(crate::WORKER_RESULT_FIXTURE).unwrap();
        result.validate().unwrap();
    }

    #[test]
    fn worker_result_validation_enforces_collection_limits() {
        let mut result: WorkerRunResultEnvelope =
            serde_json::from_str(crate::WORKER_RESULT_FIXTURE).unwrap();
        result.actions = vec![result.actions[0].clone(); MAX_ITEMS + 1];
        result.result_digest = digest_without(&result, "result_digest").unwrap();

        assert!(matches!(
            result.validate(),
            Err(SemanticError::InvalidPayload(message))
                if message.contains("worker result collections")
        ));
    }

    #[test]
    fn digest_detects_mutation() {
        let mut context: ExecutionContextEnvelope =
            serde_json::from_str(crate::EXECUTION_CONTEXT_FIXTURE).unwrap();
        context.generation += 1;
        assert!(matches!(
            context.validate(),
            Err(SemanticError::DigestMismatch { .. })
        ));
    }

    #[test]
    fn additive_extensions_and_new_minor_are_accepted() {
        let mut work: WorkPacketEnvelope =
            serde_json::from_str(crate::WORK_PACKET_FIXTURE).unwrap();
        work.protocol.minor = 7;
        work.extensions
            .insert("future".to_string(), Value::Bool(true));
        work.validate().unwrap();
    }
}
