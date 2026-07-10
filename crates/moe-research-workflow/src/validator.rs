use std::collections::{HashMap, HashSet};

use moe_research_error::{Error, Result};

use crate::log_safe::{json_error_message_for_log, safe_evidence_id_for_log};
use crate::policy::{EvidencePolicy, OutputPolicy};
use crate::report::{
    AspectReport, AspectResearchResult, Evidence, ValidationIssue, ValidationStatus,
};
use crate::research::AspectRequest;

pub struct OutputValidator<'a> {
    aspect: &'a AspectRequest,
    evidence_policy: &'a EvidencePolicy,
    output_policy: &'a OutputPolicy,
}

impl<'a> OutputValidator<'a> {
    pub fn new(
        aspect: &'a AspectRequest,
        evidence_policy: &'a EvidencePolicy,
        output_policy: &'a OutputPolicy,
    ) -> Self {
        Self {
            aspect,
            evidence_policy,
            output_policy,
        }
    }

    pub fn validate_content(
        &self,
        content: &str,
        candidate_evidence: &[Evidence],
    ) -> Result<(AspectResearchResult, ValidationStatus)> {
        let result = match serde_json::from_str::<AspectResearchResult>(content) {
            Ok(result) => result,
            Err(error) => {
                tracing::warn!(
                    event = "output_validation_failed",
                    status = "failed",
                    aspect_id = %self.aspect.id,
                    validation_phase = "json_parse",
                    content_bytes = content.len(),
                    json_error_class = ?error.classify(),
                    json_error_line = error.line(),
                    json_error_column = error.column(),
                    json_error_message = %json_error_message_for_log(&error),
                    error_code = "schema_validation_failed",
                    retryable = false,
                    "final output JSON parsing failed"
                );
                return Err(Error::SchemaValidationFailed {
                    message: "final output must be valid AspectResearchResult JSON".to_owned(),
                });
            }
        };

        let mut issues = self.validate_report(&result.aspect_report, &result.evidence);
        issues.extend(validate_selected_evidence(
            &result.aspect_report,
            &result.evidence,
            candidate_evidence,
        ));

        if issues.is_empty() {
            return Ok((result, ValidationStatus { ok: true, issues }));
        }

        let selected_evidence_ids = evidence_ids_for_log(&result.evidence);
        let candidate_evidence_ids = evidence_ids_for_log(candidate_evidence);
        let validation_issues = issues
            .iter()
            .map(|issue| (&issue.code, issue.path.as_deref(), &issue.message))
            .collect::<Vec<_>>();
        let first_issue = &issues[0];
        tracing::warn!(
            event = "output_validation_failed",
            status = "failed",
            aspect_id = %self.aspect.id,
            issue_count = issues.len(),
            selected_evidence_count = result.evidence.len(),
            candidate_evidence_count = candidate_evidence.len(),
            selected_evidence_ids = ?selected_evidence_ids,
            candidate_evidence_ids = ?candidate_evidence_ids,
            first_issue_code = %first_issue.code,
            first_issue_path = first_issue.path.as_deref(),
            first_issue_message = %first_issue.message,
            error_code = "schema_validation_failed",
            retryable = false,
            "aspect output validation failed"
        );
        tracing::debug!(
            event = "output_validation_issues",
            status = "failed",
            aspect_id = %self.aspect.id,
            validation_issues = ?validation_issues,
            "aspect output validation issue details"
        );

        // Surface the first issue's code, the path it occurred at, and the
        // human-readable message. Including the path means a
        // `mutated_evidence_provenance` failure now reports exactly which
        // evidence index and which field diverged, so an operator can
        // immediately compare the model output to the search tool output
        // without grepping through the entire wire trace.
        let first = &issues[0];
        let path_suffix = first
            .path
            .as_deref()
            .map(|path| format!(" at {path}"))
            .unwrap_or_default();
        Err(Error::SchemaValidationFailed {
            message: format!(
                "final output failed validation: {code}{path_suffix} ({message})",
                code = first.code,
                message = first.message,
            ),
        })
    }

    #[allow(clippy::too_many_lines)]
    fn validate_report(
        &self,
        report: &AspectReport,
        selected_evidence: &[Evidence],
    ) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if report.aspect_id != self.aspect.id {
            issues.push(issue(
                "aspect_id_mismatch",
                "report aspect_id does not match requested aspect",
                "aspect_report.aspect_id",
            ));
        }

        if report.aspect_name != self.aspect.name {
            issues.push(issue(
                "aspect_name_mismatch",
                "report aspect_name does not match requested aspect",
                "aspect_report.aspect_name",
            ));
        }

        if report.question.trim().is_empty() {
            issues.push(issue(
                "empty_question",
                "report question must not be empty",
                "aspect_report.question",
            ));
        }

        if let Some(max_findings) = self.output_policy.max_findings_per_aspect
            && report.findings.len() > max_findings
        {
            issues.push(issue(
                "too_many_findings",
                "report contains more findings than allowed",
                "aspect_report.findings",
            ));
        }

        let evidence_ids = selected_evidence
            .iter()
            .map(|evidence| evidence.id.as_str())
            .collect::<HashSet<_>>();
        let mut finding_ids = HashSet::new();

        for (index, finding) in report.findings.iter().enumerate() {
            if finding.id.trim().is_empty() {
                issues.push(issue(
                    "empty_finding_id",
                    "finding id must not be empty",
                    format!("aspect_report.findings[{index}].id"),
                ));
            }
            if !finding_ids.insert(finding.id.as_str()) {
                issues.push(issue(
                    "duplicate_finding_id",
                    "finding id must be unique",
                    format!("aspect_report.findings[{index}].id"),
                ));
            }

            if self.evidence_policy.require_evidence_for_findings
                && finding.evidence_refs.len() < self.evidence_policy.min_evidence_per_finding
            {
                issues.push(issue(
                    "missing_required_evidence",
                    "finding does not have enough evidence references",
                    format!("aspect_report.findings[{index}].evidence_refs"),
                ));
            }

            let mut refs = HashSet::new();
            for evidence_ref in &finding.evidence_refs {
                if !refs.insert(evidence_ref.as_str()) {
                    issues.push(issue(
                        "duplicate_evidence_ref",
                        "finding evidence references must be unique",
                        format!("aspect_report.findings[{index}].evidence_refs"),
                    ));
                }
                if !evidence_ids.contains(evidence_ref.as_str()) {
                    issues.push(issue(
                        "unknown_evidence_ref",
                        "finding references evidence that is not present in selected evidence",
                        format!("aspect_report.findings[{index}].evidence_refs"),
                    ));
                }
            }
        }

        for (index, evidence) in selected_evidence.iter().enumerate() {
            if evidence.id.trim().is_empty() {
                issues.push(issue(
                    "empty_evidence_id",
                    "evidence id must not be empty",
                    format!("evidence[{index}].id"),
                ));
            }

            if evidence.source_title.trim().is_empty() {
                issues.push(issue(
                    "empty_evidence_source_title",
                    "evidence source_title must not be empty",
                    format!("evidence[{index}].source_title"),
                ));
            }

            if evidence.provider.trim().is_empty() {
                issues.push(issue(
                    "empty_evidence_provider",
                    "evidence provider must not be empty",
                    format!("evidence[{index}].provider"),
                ));
            }

            if evidence.snippet.trim().is_empty() && evidence.summary.trim().is_empty() {
                issues.push(issue(
                    "empty_evidence_content",
                    "evidence must include snippet or summary",
                    format!("evidence[{index}]"),
                ));
            }
        }

        issues
    }
}

fn validate_selected_evidence(
    report: &AspectReport,
    selected: &[Evidence],
    candidates: &[Evidence],
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let candidates_by_id = candidates
        .iter()
        .map(|evidence| (evidence.id.as_str(), evidence))
        .collect::<HashMap<_, _>>();
    let mut selected_ids = HashSet::new();
    let finding_ids = report
        .findings
        .iter()
        .map(|finding| finding.id.as_str())
        .collect::<HashSet<_>>();
    let mut cited_by_evidence: HashMap<&str, Vec<&str>> = HashMap::new();

    for finding in &report.findings {
        for evidence_ref in &finding.evidence_refs {
            cited_by_evidence
                .entry(evidence_ref.as_str())
                .or_default()
                .push(finding.id.as_str());
        }
    }

    for (index, evidence) in selected.iter().enumerate() {
        if !selected_ids.insert(evidence.id.as_str()) {
            issues.push(issue(
                "duplicate_evidence_id",
                "selected evidence id must be unique",
                format!("evidence[{index}].id"),
            ));
        }

        let Some(candidate) = candidates_by_id.get(evidence.id.as_str()) else {
            issues.push(issue(
                "unknown_selected_evidence",
                "selected evidence was not present in search tool output",
                format!("evidence[{index}].id"),
            ));
            continue;
        };

        let mismatches = provenance_mismatch_fields(evidence, candidate);
        if !mismatches.is_empty() {
            // Name every field that diverged so log readers see the full
            // diff in one event instead of having to retry to find the
            // next mismatch.
            let fields = mismatches.join(", ");
            issues.push(issue(
                "mutated_evidence_provenance",
                &format!(
                    "selected evidence provenance must match search tool output; mismatched fields: {fields}"
                ),
                format!("evidence[{index}].{}", mismatches[0]),
            ));
        }

        for finding_id in &evidence.supports_findings {
            if !finding_ids.contains(finding_id.as_str()) {
                issues.push(issue(
                    "unknown_supported_finding",
                    "evidence supports_findings references an unknown finding",
                    format!("evidence[{index}].supports_findings"),
                ));
            }
        }

        let cited_by = cited_by_evidence
            .get(evidence.id.as_str())
            .cloned()
            .unwrap_or_default();
        if cited_by.is_empty() {
            issues.push(issue(
                "uncited_selected_evidence",
                "selected evidence must be cited by at least one finding",
                format!("evidence[{index}].id"),
            ));
        }

        let supports = evidence
            .supports_findings
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        let cited = cited_by.into_iter().collect::<HashSet<_>>();
        if supports != cited {
            issues.push(issue(
                "supports_findings_mismatch",
                "evidence supports_findings must match finding evidence_refs",
                format!("evidence[{index}].supports_findings"),
            ));
        }
    }

    issues
}

/// Compares a selected evidence object against its search-tool candidate
/// and returns the names of every provenance field that diverges, in
/// declaration order.
///
/// Returning a list (instead of a bool) lets the validator surface every
/// divergence in a single error message — so the operator sees, e.g.,
/// `mismatched fields: summary, snippet` rather than having to fix one
/// field, re-run, and rediscover the next one. The order matches the
/// schema declaration so a stable diff appears in logs.
fn provenance_mismatch_fields(selected: &Evidence, candidate: &Evidence) -> Vec<&'static str> {
    let mut fields = Vec::new();
    if selected.source_title != candidate.source_title {
        fields.push("source_title");
    }
    if selected.url != candidate.url {
        fields.push("url");
    }
    if selected.provider != candidate.provider {
        fields.push("provider");
    }
    if selected.query != candidate.query {
        fields.push("query");
    }
    if selected.snippet != candidate.snippet {
        fields.push("snippet");
    }
    if selected.summary != candidate.summary {
        fields.push("summary");
    }
    if selected.published_at != candidate.published_at {
        fields.push("published_at");
    }
    if selected.retrieved_at != candidate.retrieved_at {
        fields.push("retrieved_at");
    }
    fields
}

fn evidence_ids_for_log(evidence: &[Evidence]) -> Vec<String> {
    evidence
        .iter()
        .map(|evidence| safe_evidence_id_for_log(&evidence.id))
        .collect()
}

fn issue(code: &str, message: &str, path: impl Into<String>) -> ValidationIssue {
    ValidationIssue {
        code: code.to_owned(),
        message: message.to_owned(),
        path: Some(path.into()),
    }
}
