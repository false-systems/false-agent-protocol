use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    ActionRef, CompletionWitnessRef, EvidenceRef, GateReceiptRef, GateRef, ObligationRef,
    RepositoryPointRef, RepositoryRef, SpecRef, ToolCallRef, WorkRef, WorkerRunRef,
};

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SemanticError {
    #[error("invalid typed reference: expected prefix `{expected_prefix}`, observed `{observed}`")]
    InvalidId {
        expected_prefix: &'static str,
        observed: String,
    },
    #[error(
        "unsupported protocol major version {observed}; this build supports major {supported}"
    )]
    UnsupportedMajor { supported: u16, observed: u16 },
    #[error("invalid semantic fact: {subject} {relation:?} {object}")]
    InvalidFact {
        subject: &'static str,
        relation: Relation,
        object: &'static str,
    },
    #[error("conflicting fact authority: product {product:?} cannot author {relation:?}")]
    ConflictingAuthority {
        product: Product,
        relation: Relation,
    },
    #[error("digest mismatch: expected `{expected}`, observed `{observed}`")]
    DigestMismatch { expected: String, observed: String },
    #[error("invalid protocol payload: {0}")]
    InvalidPayload(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProtocolVersion {
    pub major: u16,
    pub minor: u16,
}

impl ProtocolVersion {
    pub const V1: Self = Self { major: 1, minor: 0 };

    pub fn validate(self) -> Result<(), SemanticError> {
        if self.major != Self::V1.major {
            return Err(SemanticError::UnsupportedMajor {
                supported: Self::V1.major,
                observed: self.major,
            });
        }
        Ok(())
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::V1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum EntityRef {
    Spec(SpecRef),
    Work(WorkRef),
    Obligation(ObligationRef),
    Gate(GateRef),
    GateReceipt(GateReceiptRef),
    Repository(RepositoryRef),
    RepositoryPoint(RepositoryPointRef),
    WorkerRun(WorkerRunRef),
    Action(ActionRef),
    ToolCall(ToolCallRef),
    Evidence(EvidenceRef),
    CompletionWitness(CompletionWitnessRef),
}

impl EntityRef {
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Spec(_) => "spec",
            Self::Work(_) => "work",
            Self::Obligation(_) => "obligation",
            Self::Gate(_) => "gate",
            Self::GateReceipt(_) => "gate_receipt",
            Self::Repository(_) => "repository",
            Self::RepositoryPoint(_) => "repository_point",
            Self::WorkerRun(_) => "worker_run",
            Self::Action(_) => "action",
            Self::ToolCall(_) => "tool_call",
            Self::Evidence(_) => "evidence",
            Self::CompletionWitness(_) => "completion_witness",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Relation {
    Implements,
    BoundTo,
    Requires,
    Executes,
    ContainsAction,
    CausedBy,
    TransformsFrom,
    TransformsTo,
    Verifies,
    Supports,
    Invalidates,
    Closes,
    SealedBy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum Product {
    Teko,
    Toimija,
    Kisko,
    Sauma,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct FactSource {
    pub product: Product,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_entity: Option<EntityRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_sequence: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SemanticFact {
    pub subject: EntityRef,
    pub relation: Relation,
    pub object: EntityRef,
    pub source: FactSource,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceRef>,
}

pub fn validate_fact(fact: &SemanticFact) -> Result<(), SemanticError> {
    use EntityRef::*;
    use Relation::*;

    let valid = matches!(
        (&fact.subject, fact.relation, &fact.object),
        (Work(_), Implements, Spec(_))
            | (Work(_), BoundTo, RepositoryPoint(_))
            | (WorkerRun(_), BoundTo, RepositoryPoint(_))
            | (GateReceipt(_), BoundTo, RepositoryPoint(_))
            | (Work(_), Requires, Obligation(_))
            | (Work(_), Requires, Gate(_))
            | (WorkerRun(_), Executes, Work(_))
            | (WorkerRun(_), ContainsAction, Action(_))
            | (Action(_), CausedBy, ToolCall(_))
            | (Action(_), TransformsFrom, RepositoryPoint(_))
            | (WorkerRun(_), TransformsFrom, RepositoryPoint(_))
            | (Action(_), TransformsTo, RepositoryPoint(_))
            | (WorkerRun(_), TransformsTo, RepositoryPoint(_))
            | (GateReceipt(_), Verifies, RepositoryPoint(_))
            | (GateReceipt(_), Supports, Obligation(_))
            | (Evidence(_), Supports, Obligation(_))
            | (Evidence(_), Supports, CompletionWitness(_))
            | (RepositoryPoint(_), Invalidates, GateReceipt(_))
            | (CompletionWitness(_), Closes, Work(_))
            | (Evidence(_), SealedBy, Evidence(_))
            | (CompletionWitness(_), SealedBy, Evidence(_))
    );
    if valid {
        Ok(())
    } else {
        Err(SemanticError::InvalidFact {
            subject: fact.subject.kind(),
            relation: fact.relation,
            object: fact.object.kind(),
        })
    }
}

fn validate_fact_source(fact: &SemanticFact) -> Result<(), SemanticError> {
    use Product::{Kisko, Sauma, Teko, Toimija};
    use Relation::*;
    let allowed = match fact.source.product {
        Teko => matches!(
            fact.relation,
            Implements | BoundTo | Requires | Verifies | Supports | Closes | SealedBy
        ),
        Toimija => matches!(
            fact.relation,
            BoundTo | TransformsFrom | TransformsTo | Invalidates
        ),
        Kisko => matches!(
            fact.relation,
            Executes
                | BoundTo
                | ContainsAction
                | CausedBy
                | TransformsFrom
                | TransformsTo
                | Supports
                | SealedBy
        ),
        Sauma => fact.relation == SealedBy,
    };
    if allowed {
        Ok(())
    } else {
        Err(SemanticError::ConflictingAuthority {
            product: fact.source.product,
            relation: fact.relation,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticProjection {
    pub protocol: ProtocolVersion,
    pub entities: Vec<EntityRef>,
    pub facts: Vec<SemanticFact>,
}

impl SemanticProjection {
    pub fn derive(facts: impl IntoIterator<Item = SemanticFact>) -> Result<Self, SemanticError> {
        let mut entities = BTreeSet::new();
        let mut unique = BTreeSet::new();
        for fact in facts {
            validate_fact(&fact)?;
            validate_fact_source(&fact)?;
            entities.insert(fact.subject.clone());
            entities.insert(fact.object.clone());
            entities.extend(fact.evidence.iter().cloned().map(EntityRef::Evidence));
            unique.insert(fact);
        }
        Ok(Self {
            protocol: ProtocolVersion::V1,
            entities: entities.into_iter().collect(),
            facts: unique.into_iter().collect(),
        })
    }

    pub fn render(&self) -> String {
        self.facts
            .iter()
            .map(|fact| {
                format!(
                    "{} {:?} {} [{}]",
                    display_ref(&fact.subject),
                    fact.relation,
                    display_ref(&fact.object),
                    format!("{:?}", fact.source.product).to_ascii_lowercase()
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn display_ref(entity: &EntityRef) -> &str {
    match entity {
        EntityRef::Spec(value) => value.as_str(),
        EntityRef::Work(value) => value.as_str(),
        EntityRef::Obligation(value) => value.as_str(),
        EntityRef::Gate(value) => value.as_str(),
        EntityRef::GateReceipt(value) => value.as_str(),
        EntityRef::Repository(value) => value.as_str(),
        EntityRef::RepositoryPoint(value) => value.as_str(),
        EntityRef::WorkerRun(value) => value.as_str(),
        EntityRef::Action(value) => value.as_str(),
        EntityRef::ToolCall(value) => value.as_str(),
        EntityRef::Evidence(value) => value.as_str(),
        EntityRef::CompletionWitness(value) => value.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> FactSource {
        FactSource {
            product: Product::Teko,
            source_entity: None,
            event_sequence: None,
        }
    }

    fn fact(subject: EntityRef, relation: Relation, object: EntityRef) -> SemanticFact {
        SemanticFact {
            subject,
            relation,
            object,
            source: source(),
            evidence: Vec::new(),
        }
    }

    #[test]
    fn required_allowed_shapes_validate() {
        let work = EntityRef::Work("work_01".parse().unwrap());
        let spec = EntityRef::Spec("spec_01".parse().unwrap());
        let run = EntityRef::WorkerRun("run_01".parse().unwrap());
        let point = EntityRef::RepositoryPoint("rpoint_01".parse().unwrap());
        assert!(validate_fact(&fact(work.clone(), Relation::Implements, spec)).is_ok());
        assert!(validate_fact(&fact(run, Relation::Executes, work.clone())).is_ok());
        assert!(validate_fact(&fact(work, Relation::BoundTo, point)).is_ok());
    }

    #[test]
    fn forbidden_shapes_fail() {
        let run = EntityRef::WorkerRun("run_01".parse().unwrap());
        let spec = EntityRef::Spec("spec_01".parse().unwrap());
        assert!(validate_fact(&fact(run, Relation::Implements, spec)).is_err());

        let point = EntityRef::RepositoryPoint("rpoint_01".parse().unwrap());
        let work = EntityRef::Work("work_01".parse().unwrap());
        assert!(validate_fact(&fact(point, Relation::Closes, work)).is_err());
    }

    #[test]
    fn projection_is_sorted_and_deduplicated() {
        let item = fact(
            EntityRef::Work("work_01".parse().unwrap()),
            Relation::Implements,
            EntityRef::Spec("spec_01".parse().unwrap()),
        );
        let projection = SemanticProjection::derive([item.clone(), item]).unwrap();
        assert_eq!(projection.facts.len(), 1);
        assert_eq!(projection.entities.len(), 2);
    }

    #[test]
    fn projection_rejects_conflicting_product_authority() {
        let mut item = fact(
            EntityRef::WorkerRun("run_01".parse().unwrap()),
            Relation::Executes,
            EntityRef::Work("work_01".parse().unwrap()),
        );
        item.source.product = Product::Teko;
        assert!(matches!(
            SemanticProjection::derive([item]),
            Err(SemanticError::ConflictingAuthority { .. })
        ));
    }

    #[test]
    fn unsupported_major_fails_but_newer_minor_is_additive() {
        assert!(ProtocolVersion { major: 1, minor: 9 }.validate().is_ok());
        assert!(ProtocolVersion { major: 2, minor: 0 }.validate().is_err());
    }
}
