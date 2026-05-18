use crate::ids::{ConflictGroupId, EntityId, EvidenceId, FactId, ProvenanceId, SourceId, TraceId};
use crate::model::{
    CausisError, ConflictGroup, EvidenceAnchor, EvidenceRecord, Fact, FactValue, ProvenanceEvent,
    ResolutionDecision, Result, Trace, TraceStep,
};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LeaveApprovalReport {
    pub trace: Trace,
    pub request_id: String,
    pub applicant_id: EntityId,
    pub approver_id: EntityId,
    pub evidence: Vec<EvidenceRecord>,
    pub provenance: Vec<ProvenanceEvent>,
    pub facts: Vec<Fact>,
    pub conflicts: Vec<ConflictGroup>,
    pub resolutions: Vec<ResolutionDecision>,
}

impl LeaveApprovalReport {
    pub fn summary(&self) -> String {
        format!("申请 {} 应由 {} 审批", self.request_id, self.approver_id)
    }

    pub fn to_json_pretty(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        push_json_field(&mut out, 1, "request_id", &self.request_id, true);
        push_json_field(
            &mut out,
            1,
            "applicant_id",
            self.applicant_id.as_str(),
            true,
        );
        push_json_field(&mut out, 1, "approver_id", self.approver_id.as_str(), true);
        push_json_field(&mut out, 1, "summary", &self.summary(), true);
        push_trace(&mut out, &self.trace);
        out.push_str(",\n");
        push_resolutions(&mut out, &self.resolutions);
        out.push_str(",\n");
        push_evidence_chain(
            &mut out,
            &self.trace.steps,
            &self.provenance,
            &self.evidence,
        );
        out.push('\n');
        out.push_str("}\n");
        out
    }
}

pub fn run_leave_approval_demo(input_dir: impl AsRef<Path>) -> Result<LeaveApprovalReport> {
    let input_dir = input_dir.as_ref();
    let sources = read_sources(&input_dir.join("sources.csv"))?;
    let mut evidence = Vec::new();
    let mut evidence_by_source = BTreeMap::new();

    for source in sources {
        let record = EvidenceRecord {
            id: EvidenceId::new(format!("evidence-{}", source.source_id.as_str())),
            source_id: source.source_id.clone(),
            source_uri: source.uri.clone(),
            content_hash: stable_hash(&format!("{}:{}", source.source_id, source.uri)),
        };
        evidence_by_source.insert(source.source_id, record.id.clone());
        evidence.push(record);
    }

    let mut facts = read_facts(&input_dir.join("facts.csv"), &evidence_by_source)?;
    let leave_request = read_leave_request(&input_dir.join("leave_requests.csv"))?;
    let conflicts = detect_conflicts(&mut facts);
    let resolutions = resolve_conflicts(&conflicts, &facts)?;

    let applicant_id = leave_request.applicant_id;
    let manager_fact = resolve_fact(&facts, &resolutions, &applicant_id, "reports_to")?;
    let manager_id = entity_value(&manager_fact.value);
    let manager_status_fact = resolve_fact(&facts, &resolutions, &manager_id, "status")?;
    let delegate_fact = resolve_fact(&facts, &resolutions, &manager_id, "delegate")?;
    let policy_fact = resolve_fact(
        &facts,
        &resolutions,
        &EntityId::new("Policy:LeaveApproval"),
        "fallback_rule",
    )?;

    let approver_id = if manager_status_fact.value.as_text() == "business_trip" {
        entity_value(&delegate_fact.value)
    } else {
        manager_id
    };

    let manager_display = display_fact_value(&manager_fact);
    let manager_status_display = display_fact_value(&manager_status_fact);
    let delegate_display = display_fact_value(&delegate_fact);
    let policy_display = display_fact_value(&policy_fact);

    let trace = Trace {
        id: TraceId::new(format!("trace-{}", leave_request.request_id)),
        steps: vec![
            TraceStep {
                order: 1,
                label: format!("{applicant_id} 的直属主管是 {manager_display}"),
                fact_id: Some(manager_fact.id.clone()),
                provenance_id: Some(manager_fact.provenance_id.clone()),
            },
            TraceStep {
                order: 2,
                label: format!("{manager_display} 当前状态为 {manager_status_display}"),
                fact_id: Some(manager_status_fact.id.clone()),
                provenance_id: Some(manager_status_fact.provenance_id.clone()),
            },
            TraceStep {
                order: 3,
                label: format!("制度适用规则：{policy_display}"),
                fact_id: Some(policy_fact.id.clone()),
                provenance_id: Some(policy_fact.provenance_id.clone()),
            },
            TraceStep {
                order: 4,
                label: format!("{manager_display} 将审批委托给 {delegate_display}"),
                fact_id: Some(delegate_fact.id.clone()),
                provenance_id: Some(delegate_fact.provenance_id.clone()),
            },
            TraceStep {
                order: 5,
                label: format!("最终审批人为 {approver_id}"),
                fact_id: None,
                provenance_id: None,
            },
        ],
    };

    let provenance = facts
        .iter()
        .map(|fact| ProvenanceEvent {
            id: fact.provenance_id.clone(),
            evidence_id: evidence_by_source
                .get(&fact.source_id)
                .cloned()
                .unwrap_or_else(|| EvidenceId::new(format!("evidence-{}", fact.source_id))),
            operation: "fixture.fact.normalize".to_string(),
            anchor: EvidenceAnchor {
                source_id: fact.source_id.clone(),
                location: fact.source_anchor.clone(),
            },
        })
        .collect();

    Ok(LeaveApprovalReport {
        trace,
        request_id: leave_request.request_id,
        applicant_id,
        approver_id,
        evidence,
        provenance,
        facts,
        conflicts,
        resolutions,
    })
}

#[derive(Clone, Debug)]
struct SourceRow {
    source_id: SourceId,
    uri: String,
}

#[derive(Clone, Debug)]
struct LeaveRequestRow {
    request_id: String,
    applicant_id: EntityId,
}

fn read_sources(path: &Path) -> Result<Vec<SourceRow>> {
    read_csv(path, 3, |columns, _line| SourceRow {
        source_id: SourceId::new(columns[0].to_string()),
        uri: columns[1].to_string(),
    })
}

fn read_leave_request(path: &Path) -> Result<LeaveRequestRow> {
    let mut rows = read_csv(path, 5, |columns, _line| LeaveRequestRow {
        request_id: columns[0].to_string(),
        applicant_id: EntityId::new(columns[1].to_string()),
    })?;
    rows.pop().ok_or_else(|| CausisError::InvalidFixture {
        path: display_path(path),
        line: 1,
        message: "expected at least one leave request".to_string(),
    })
}

fn read_facts(
    path: &Path,
    evidence_by_source: &BTreeMap<SourceId, EvidenceId>,
) -> Result<Vec<Fact>> {
    read_csv(path, 7, |columns, line| {
        let source_id = SourceId::new(columns[0].to_string());
        let entity_id = EntityId::new(columns[1].to_string());
        let attribute = columns[2].to_string();
        let value_raw = columns[3].to_string();
        let value = parse_fact_value(&attribute, &value_raw, columns[4]);
        let authority = columns[5].parse::<u16>().unwrap_or(0);
        let provenance_id = ProvenanceId::new(format!("prov-{line}"));

        let _ = evidence_by_source.get(&source_id);

        Fact {
            id: FactId::new(format!("fact-{line}")),
            entity_id,
            attribute,
            value_raw,
            value,
            source_anchor: columns[6].to_string(),
            source_id,
            authority,
            provenance_id,
            conflict_group_id: None,
        }
    })
}

fn read_csv<T>(
    path: &Path,
    expected_columns: usize,
    mut build: impl FnMut(Vec<&str>, usize) -> T,
) -> Result<Vec<T>> {
    let contents = fs::read_to_string(path).map_err(|source| CausisError::Io {
        path: display_path(path),
        source,
    })?;

    let mut rows = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let line_number = index + 1;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || line_number == 1 {
            continue;
        }

        let columns = trimmed.split(',').map(str::trim).collect::<Vec<_>>();
        if columns.len() < expected_columns {
            return Err(CausisError::InvalidFixture {
                path: display_path(path),
                line: line_number,
                message: format!(
                    "expected at least {expected_columns} columns, got {}",
                    columns.len()
                ),
            });
        }
        rows.push(build(columns, line_number));
    }
    Ok(rows)
}

fn detect_conflicts(facts: &mut [Fact]) -> Vec<ConflictGroup> {
    let mut grouped: BTreeMap<(EntityId, String), BTreeMap<String, Vec<usize>>> = BTreeMap::new();
    for (index, fact) in facts.iter().enumerate() {
        grouped
            .entry((fact.entity_id.clone(), fact.attribute.clone()))
            .or_default()
            .entry(fact.value.as_text())
            .or_default()
            .push(index);
    }

    let mut conflicts = Vec::new();
    let mut sequence = 1usize;
    for ((entity_id, attribute), values) in grouped {
        if values.len() <= 1 {
            continue;
        }

        let id = ConflictGroupId::new(format!("conflict-{sequence}"));
        sequence += 1;
        let mut fact_ids = Vec::new();
        let mut seen = BTreeSet::new();
        for indices in values.values() {
            for index in indices {
                facts[*index].conflict_group_id = Some(id.clone());
                if seen.insert(facts[*index].id.clone()) {
                    fact_ids.push(facts[*index].id.clone());
                }
            }
        }
        conflicts.push(ConflictGroup {
            id,
            entity_id,
            attribute,
            fact_ids,
        });
    }

    conflicts
}

fn resolve_conflicts(
    conflicts: &[ConflictGroup],
    facts: &[Fact],
) -> Result<Vec<ResolutionDecision>> {
    let mut decisions = Vec::new();
    for conflict in conflicts {
        let mut candidates = conflict
            .fact_ids
            .iter()
            .filter_map(|id| facts.iter().find(|fact| fact.id == *id))
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .authority
                .cmp(&left.authority)
                .then_with(|| left.id.cmp(&right.id))
        });

        let adopted = candidates
            .first()
            .ok_or_else(|| CausisError::EmptyConflictGroup {
                conflict_group_id: conflict.id.clone(),
            })?;
        let rejected_fact_ids = candidates
            .iter()
            .skip(1)
            .map(|fact| fact.id.clone())
            .collect::<Vec<_>>();
        decisions.push(ResolutionDecision {
            conflict_group_id: conflict.id.clone(),
            adopted_fact_id: adopted.id.clone(),
            adopted_value: adopted.value.clone(),
            rejected_fact_ids,
            strategy: "authority_first".to_string(),
            reason: format!(
                "依据权威优先策略，选择来源 {} 的值 {}（权威分 {}）",
                adopted.source_id,
                display_fact_value(adopted),
                adopted.authority
            ),
        });
    }
    Ok(decisions)
}

fn resolve_fact(
    facts: &[Fact],
    resolutions: &[ResolutionDecision],
    entity_id: &EntityId,
    attribute: &str,
) -> Result<Fact> {
    let candidates = facts
        .iter()
        .filter(|fact| fact.entity_id == *entity_id && fact.attribute == attribute)
        .collect::<Vec<_>>();

    if candidates.is_empty() {
        return Err(CausisError::MissingFact {
            entity_id: entity_id.to_string(),
            attribute: attribute.to_string(),
        });
    }

    for candidate in &candidates {
        if let Some(conflict_group_id) = &candidate.conflict_group_id
            && let Some(decision) = resolutions
                .iter()
                .find(|decision| decision.conflict_group_id == *conflict_group_id)
            && decision.adopted_fact_id == candidate.id
        {
            return Ok((*candidate).clone());
        }
    }

    Ok((*candidates[0]).clone())
}

fn entity_value(value: &FactValue) -> EntityId {
    match value {
        FactValue::Entity(value) => value.clone(),
        FactValue::Text(value) => EntityId::new(value.clone()),
        FactValue::Number(value) => EntityId::new(value.to_string()),
    }
}

fn parse_fact_value(attribute: &str, value_raw: &str, value_type: &str) -> FactValue {
    if value_type == "entity" {
        FactValue::Entity(EntityId::new(value_raw.to_string()))
    } else if let Some(normalized) = normalize_text_value(attribute, value_raw) {
        FactValue::Text(normalized.to_string())
    } else if let Ok(value) = value_raw.parse::<i64>() {
        FactValue::Number(value)
    } else {
        FactValue::Text(value_raw.to_string())
    }
}

fn normalize_text_value(attribute: &str, value_raw: &str) -> Option<&'static str> {
    match (attribute, value_raw) {
        ("status", "business_trip" | "出差") => Some("business_trip"),
        ("fallback_rule", "fallback_to_delegate" | "转交代理人") => {
            Some("fallback_to_delegate")
        }
        _ => None,
    }
}

fn display_fact_value(fact: &Fact) -> String {
    if fact.value_raw.is_empty() {
        fact.value.as_text()
    } else {
        fact.value_raw.clone()
    }
}

fn push_trace(out: &mut String, trace: &Trace) {
    push_indent(out, 1);
    out.push_str("\"trace\": {\n");
    push_json_field(out, 2, "id", trace.id.as_str(), true);
    push_indent(out, 2);
    out.push_str("\"steps\": [\n");
    for (index, step) in trace.steps.iter().enumerate() {
        push_indent(out, 3);
        out.push_str("{\n");
        push_number_field(out, 4, "order", step.order, true);
        push_json_field(out, 4, "label", &step.label, step.fact_id.is_some());
        if let Some(fact_id) = &step.fact_id {
            push_json_field(
                out,
                4,
                "fact_id",
                fact_id.as_str(),
                step.provenance_id.is_some(),
            );
        }
        if let Some(provenance_id) = &step.provenance_id {
            push_json_field(out, 4, "provenance_id", provenance_id.as_str(), false);
        }
        push_indent(out, 3);
        out.push('}');
        if index + 1 != trace.steps.len() {
            out.push(',');
        }
        out.push('\n');
    }
    push_indent(out, 2);
    out.push_str("]\n");
    push_indent(out, 1);
    out.push('}');
}

fn push_resolutions(out: &mut String, resolutions: &[ResolutionDecision]) {
    push_indent(out, 1);
    out.push_str("\"resolutions\": [\n");
    for (index, decision) in resolutions.iter().enumerate() {
        push_indent(out, 2);
        out.push_str("{\n");
        push_json_field(
            out,
            3,
            "conflict_group_id",
            decision.conflict_group_id.as_str(),
            true,
        );
        push_json_field(
            out,
            3,
            "adopted_fact_id",
            decision.adopted_fact_id.as_str(),
            true,
        );
        push_json_field(
            out,
            3,
            "adopted_value",
            &decision.adopted_value.as_text(),
            true,
        );
        push_json_field(out, 3, "strategy", &decision.strategy, true);
        push_json_field(out, 3, "reason", &decision.reason, false);
        push_indent(out, 2);
        out.push('}');
        if index + 1 != resolutions.len() {
            out.push(',');
        }
        out.push('\n');
    }
    push_indent(out, 1);
    out.push(']');
}

fn push_evidence_chain(
    out: &mut String,
    steps: &[TraceStep],
    provenance: &[ProvenanceEvent],
    evidence: &[EvidenceRecord],
) {
    push_indent(out, 1);
    out.push_str("\"evidence_chain\": [\n");
    let mut emitted = 0usize;
    for step in steps {
        let Some(provenance_id) = &step.provenance_id else {
            continue;
        };
        let Some(event) = provenance.iter().find(|event| event.id == *provenance_id) else {
            continue;
        };
        let Some(record) = evidence
            .iter()
            .find(|record| record.id == event.evidence_id)
        else {
            continue;
        };

        if emitted > 0 {
            out.push_str(",\n");
        }
        emitted += 1;
        push_indent(out, 2);
        out.push_str("{\n");
        push_number_field(out, 3, "step", step.order, true);
        push_json_field(out, 3, "source_id", record.source_id.as_str(), true);
        push_json_field(out, 3, "source_uri", &record.source_uri, true);
        push_json_field(out, 3, "content_hash", &record.content_hash, true);
        push_json_field(out, 3, "location", &event.anchor.location, false);
        push_indent(out, 2);
        out.push('}');
    }
    out.push('\n');
    push_indent(out, 1);
    out.push(']');
}

fn push_json_field(out: &mut String, indent: usize, name: &str, value: &str, comma: bool) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(name);
    out.push_str("\": \"");
    out.push_str(&escape_json(value));
    out.push('"');
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_number_field(out: &mut String, indent: usize, name: &str, value: usize, comma: bool) {
    push_indent(out, indent);
    out.push('"');
    out.push_str(name);
    out.push_str("\": ");
    out.push_str(&value.to_string());
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn stable_hash(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn display_path(path: &Path) -> String {
    PathBuf::from(path).display().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leave_approval_demo_resolves_to_delegate() {
        let report = run_leave_approval_demo(fixture_path()).expect("demo should run");

        assert_eq!(report.request_id, "LR-2026-001");
        assert_eq!(report.applicant_id.as_str(), "Employee:E001");
        assert_eq!(report.approver_id.as_str(), "Employee:E003");
        assert_eq!(report.conflicts.len(), 1);
        assert_eq!(report.resolutions.len(), 1);
        assert!(report.to_json_pretty().contains("\"evidence_chain\""));
    }

    #[test]
    fn leave_approval_demo_normalizes_chinese_fixture_values() {
        let report = run_leave_approval_demo(fixture_path()).expect("demo should run");
        let manager_status = report
            .facts
            .iter()
            .find(|fact| fact.entity_id.as_str() == "Employee:E002" && fact.attribute == "status")
            .expect("manager status fact should exist");
        let policy_rule = report
            .facts
            .iter()
            .find(|fact| {
                fact.entity_id.as_str() == "Policy:LeaveApproval"
                    && fact.attribute == "fallback_rule"
            })
            .expect("policy rule fact should exist");

        assert_eq!(manager_status.value_raw, "出差");
        assert_eq!(manager_status.value.as_text(), "business_trip");
        assert_eq!(policy_rule.value_raw, "转交代理人");
        assert_eq!(policy_rule.value.as_text(), "fallback_to_delegate");
        assert!(
            report
                .summary()
                .contains("申请 LR-2026-001 应由 Employee:E003 审批")
        );
        assert!(
            report
                .trace
                .steps
                .iter()
                .any(|step| step.label.contains("出差"))
        );
        assert!(
            report
                .trace
                .steps
                .iter()
                .any(|step| step.label.contains("转交代理人"))
        );
    }

    #[test]
    fn leave_approval_demo_covers_pipeline_outputs() {
        let report = run_leave_approval_demo(fixture_path()).expect("demo should run");

        assert_eq!(report.evidence.len(), 4);
        assert_eq!(report.facts.len(), 8);
        assert_eq!(report.provenance.len(), report.facts.len());
        assert_eq!(report.trace.steps.len(), 5);
        assert_eq!(report.conflicts[0].attribute, "reports_to");
        assert_eq!(report.resolutions[0].strategy, "authority_first");
        assert!(report.resolutions[0].reason.contains("权威优先"));
    }

    fn fixture_path() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join("fixtures/leave-approval")
    }
}
