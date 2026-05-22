use std::collections::HashSet;

use crate::{
    error::{Error, Result},
    schema::{
        common::{AspectSpec, EvidencePolicy, OutputPolicy},
        report::{AspectReport, ValidationIssue, ValidationStatus},
    },
};

pub struct OutputValidator<'a> {
    aspect: &'a AspectSpec,
    evidence_policy: &'a EvidencePolicy,
    output_policy: &'a OutputPolicy,
}

impl<'a> OutputValidator<'a> {
    pub fn new(
        aspect: &'a AspectSpec,
        evidence_policy: &'a EvidencePolicy,
        output_policy: &'a OutputPolicy,
    ) -> Self {
        Self {
            aspect,
            evidence_policy,
            output_policy,
        }
    }

    pub fn validate_content(&self, content: &str) -> Result<(AspectReport, ValidationStatus)> {
        let report = serde_json::from_str::<AspectReport>(content).map_err(|_| {
            Error::SchemaValidationFailed {
                message: "final output must be valid AspectReport JSON".to_owned(),
            }
        })?;

        let issues = self.validate_report(&report);
        if issues.is_empty() {
            return Ok((report, ValidationStatus { ok: true, issues }));
        }

        Err(Error::SchemaValidationFailed {
            message: format!("final output failed validation: {}", issues[0].code),
        })
    }

    fn validate_report(&self, report: &AspectReport) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        if report.aspect_id != self.aspect.aspect_id {
            issues.push(issue(
                "aspect_id_mismatch",
                "report aspect_id does not match requested aspect",
                "aspect_id",
            ));
        }

        if report.aspect_name != self.aspect.name {
            issues.push(issue(
                "aspect_name_mismatch",
                "report aspect_name does not match requested aspect",
                "aspect_name",
            ));
        }

        if report.question.trim().is_empty() {
            issues.push(issue(
                "empty_question",
                "report question must not be empty",
                "question",
            ));
        }

        if let Some(max_findings) = self.output_policy.max_findings_per_aspect
            && report.findings.len() > max_findings
        {
            issues.push(issue(
                "too_many_findings",
                "report contains more findings than allowed",
                "findings",
            ));
        }

        let evidence_ids = report
            .evidence
            .iter()
            .map(|evidence| evidence.id.as_str())
            .collect::<HashSet<_>>();

        for (index, finding) in report.findings.iter().enumerate() {
            if self.evidence_policy.require_evidence_for_findings
                && finding.evidence_refs.len() < self.evidence_policy.min_evidence_per_finding
            {
                issues.push(issue(
                    "missing_required_evidence",
                    "finding does not have enough evidence references",
                    format!("findings[{index}].evidence_refs"),
                ));
            }

            for evidence_ref in &finding.evidence_refs {
                if !evidence_ids.contains(evidence_ref.as_str()) {
                    issues.push(issue(
                        "unknown_evidence_ref",
                        "finding references evidence that is not present in the report",
                        format!("findings[{index}].evidence_refs"),
                    ));
                }
            }
        }

        for (index, evidence) in report.evidence.iter().enumerate() {
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

pub fn validate_output(
    content: &str,
    aspect: &AspectSpec,
    evidence_policy: &EvidencePolicy,
    output_policy: &OutputPolicy,
) -> Result<(AspectReport, ValidationStatus)> {
    OutputValidator::new(aspect, evidence_policy, output_policy).validate_content(content)
}

fn issue(code: &str, message: &str, path: impl Into<String>) -> ValidationIssue {
    ValidationIssue {
        code: code.to_owned(),
        message: message.to_owned(),
        path: Some(path.into()),
    }
}
