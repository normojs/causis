use crate::ids::{ConflictGroupId, EntityId, EvidenceId, FactId, ProvenanceId, SourceId, TraceId};
use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, CausisError>;

#[derive(Debug)]
pub enum CausisError {
    Io {
        path: String,
        source: std::io::Error,
    },
    InvalidFixture {
        path: String,
        line: usize,
        message: String,
    },
    MissingFact {
        entity_id: String,
        attribute: String,
    },
    EmptyConflictGroup {
        conflict_group_id: ConflictGroupId,
    },
}

impl fmt::Display for CausisError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "failed to read {path}: {source}"),
            Self::InvalidFixture {
                path,
                line,
                message,
            } => write!(f, "invalid fixture {path}:{line}: {message}"),
            Self::MissingFact {
                entity_id,
                attribute,
            } => write!(f, "missing fact {entity_id}.{attribute}"),
            Self::EmptyConflictGroup { conflict_group_id } => {
                write!(f, "conflict group {conflict_group_id} has no facts")
            }
        }
    }
}

impl Error for CausisError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceAnchor {
    pub source_id: SourceId,
    pub location: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvidenceRecord {
    pub id: EvidenceId,
    pub source_id: SourceId,
    pub source_uri: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceEvent {
    pub id: ProvenanceId,
    pub evidence_id: EvidenceId,
    pub operation: String,
    pub anchor: EvidenceAnchor,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FactValue {
    Entity(EntityId),
    Text(String),
    Number(i64),
}

impl FactValue {
    pub fn as_text(&self) -> String {
        match self {
            Self::Entity(value) => value.to_string(),
            Self::Text(value) => value.clone(),
            Self::Number(value) => value.to_string(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fact {
    pub id: FactId,
    pub entity_id: EntityId,
    pub attribute: String,
    pub value_raw: String,
    pub value: FactValue,
    pub source_anchor: String,
    pub source_id: SourceId,
    pub authority: u16,
    pub provenance_id: ProvenanceId,
    pub conflict_group_id: Option<ConflictGroupId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConflictGroup {
    pub id: ConflictGroupId,
    pub entity_id: EntityId,
    pub attribute: String,
    pub fact_ids: Vec<FactId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionDecision {
    pub conflict_group_id: ConflictGroupId,
    pub adopted_fact_id: FactId,
    pub adopted_value: FactValue,
    pub rejected_fact_ids: Vec<FactId>,
    pub strategy: String,
    pub reason: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraceStep {
    pub order: usize,
    pub label: String,
    pub fact_id: Option<FactId>,
    pub provenance_id: Option<ProvenanceId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Trace {
    pub id: TraceId,
    pub steps: Vec<TraceStep>,
}
