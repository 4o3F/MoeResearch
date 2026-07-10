use std::collections::BTreeMap;

use crate::agent_loop::{AgentRuntimeFailure, AgentRuntimeOutput};
use crate::error_log_safe::error_message_for_log;
use crate::report::{
    AspectFailure, AspectReport, AspectResearchResult, Confidence, ConfidenceSummary, Evidence,
    OpenQuestion, ResearchBudgetUsage,
};
use crate::research::EffectiveResearchPlan;
use moe_research_error::Error;

pub(super) struct DeepResearchRun {
    pub(super) completed: Vec<String>,
    pub(super) failures: Vec<AspectFailure>,
    pub(super) aspect_reports: Vec<AspectReport>,
    pub(super) evidence_by_id: BTreeMap<String, Evidence>,
    pub(super) open_questions: Vec<OpenQuestion>,
    pub(super) budget_usage: ResearchBudgetUsage,
}

impl DeepResearchRun {
    pub(super) fn new() -> Self {
        Self {
            completed: Vec::new(),
            failures: Vec::new(),
            aspect_reports: Vec::new(),
            evidence_by_id: BTreeMap::new(),
            open_questions: Vec::new(),
            budget_usage: ResearchBudgetUsage::zero(),
        }
    }
}

pub(super) fn record_aspect_result(
    run: &mut DeepResearchRun,
    aspect_id: &str,
    result: std::result::Result<AgentRuntimeOutput, AgentRuntimeFailure>,
    allow_partial_results: bool,
) {
    match result {
        Ok(result) => record_aspect_success(run, result),
        Err(mut failure) => {
            let aspect_error = aspect_failure(aspect_id, &failure.error);
            let partial_evidence_count = failure
                .partial_output
                .as_ref()
                .map_or(0, |output| output.result.evidence.len());
            let preserve_partial_evidence = allow_partial_results && partial_evidence_count > 0;
            tracing::warn!(
                event = "aspect_failed",
                status = "failed",
                aspect_id,
                error_code = failure.error.code().as_str(),
                error_message = %error_message_for_log(&failure.error),
                retryable = failure.error.retryable(),
                partial_evidence_count,
                preserve_partial_evidence,
                "aspect failed"
            );
            if allow_partial_results && let Some(mut output) = failure.partial_output.take() {
                namespace_aspect_evidence(&mut output.result);
                for evidence in &output.result.evidence {
                    run.evidence_by_id
                        .entry(evidence.id.clone())
                        .or_insert_with(|| evidence.clone());
                }
            }
            run.failures.push(aspect_error);
        }
    }
}

pub(super) fn record_aspect_success(run: &mut DeepResearchRun, mut output: AgentRuntimeOutput) {
    namespace_aspect_evidence(&mut output.result);
    run.completed
        .push(output.result.aspect_report.aspect_id.clone());
    run.open_questions
        .extend(output.result.aspect_report.open_questions.clone());
    for evidence in &output.result.evidence {
        run.evidence_by_id
            .entry(evidence.id.clone())
            .or_insert_with(|| evidence.clone());
    }
    run.aspect_reports.push(output.result.aspect_report);
}

pub(super) fn namespace_aspect_evidence(result: &mut AspectResearchResult) {
    let aspect_id = result.aspect_report.aspect_id.clone();
    let mut remapped_ids = BTreeMap::new();

    for evidence in &mut result.evidence {
        let original_id = evidence.id.clone();
        let namespaced_id = format!("{aspect_id}:{original_id}");
        evidence.id.clone_from(&namespaced_id);
        remapped_ids.insert(original_id, namespaced_id);
    }

    for finding in &mut result.aspect_report.findings {
        for evidence_ref in &mut finding.evidence_refs {
            if let Some(namespaced_id) = remapped_ids.get(evidence_ref) {
                *evidence_ref = namespaced_id.clone();
            }
        }
    }
}

pub(super) fn order_failures_by_request(
    request: &EffectiveResearchPlan,
    failures: Vec<AspectFailure>,
) -> Vec<AspectFailure> {
    let mut by_aspect_id = failures
        .into_iter()
        .map(|failure| (failure.aspect_id.clone(), failure))
        .collect::<BTreeMap<_, _>>();

    request
        .task
        .aspects
        .iter()
        .filter_map(|aspect| by_aspect_id.remove(&aspect.id))
        .collect()
}

/// Builds the per-aspect failure record embedded inside a partial or failed
/// `DeepResearchResult`.
///
/// `error_code` is the `snake_case` transport-neutral `ErrorCode` identifier
/// rather than `Debug` output, so external clients can dispatch on a stable
/// string. `message` is the same redacted summary used in the public envelope.
pub(super) fn aspect_failure(aspect_id: &str, error: &Error) -> AspectFailure {
    AspectFailure {
        aspect_id: aspect_id.to_owned(),
        error_code: error.code().as_str().to_owned(),
        message: error.public_message(),
        retryable: error.retryable(),
    }
}

pub(super) fn confidence_summary(aspect_reports: &[AspectReport]) -> ConfidenceSummary {
    let mut summary = ConfidenceSummary::zero();
    for report in aspect_reports {
        match report.confidence {
            Confidence::High => summary.high += 1,
            Confidence::Medium => summary.medium += 1,
            Confidence::Low => summary.low += 1,
        }
    }
    summary
}
