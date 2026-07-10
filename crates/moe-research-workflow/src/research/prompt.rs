use serde::Serialize;

use super::super::policy::ToolName;
use super::plan::EffectiveAspectPlan;
use super::request::ResearchContext;

/// Upper bound on the inline aspect-agent prompt size, in bytes.
///
/// 64 KiB is comfortably above realistic prompt assets (current MoeResearch prompts
/// are 1-10 KiB) but bounds the per-request payload so a buggy Layer 1
/// implementation cannot accidentally explode token usage by inlining a huge
/// Markdown file.
pub(crate) const ASPECT_PROMPT_MAX_BYTES: usize = 64 * 1024;

/// Internal Layer 2 user-prompt projection.
///
/// This is intentionally narrower than `EffectiveAspectPlan`: runtime controls,
/// provider routing, resource limits, and inline system-prompt content stay out
/// of the model user message.
#[derive(Debug, Serialize)]
pub(crate) struct AspectPromptInput<'a> {
    aspect: AspectPromptAspect<'a>,
    context: &'a ResearchContext,
    available_tools: &'a [ToolName],
    evidence_requirements: AspectPromptEvidenceRequirements,
    output_requirements: AspectPromptOutputRequirements<'a>,
}

#[derive(Debug, Serialize)]
struct AspectPromptAspect<'a> {
    id: &'a str,
    name: &'a str,
    role: &'a str,
    question: &'a str,
    scope: &'a [String],
    boundaries: &'a [String],
    success_criteria: &'a [String],
}

#[derive(Debug, Serialize)]
struct AspectPromptEvidenceRequirements {
    require_evidence_for_findings: bool,
    min_evidence_per_finding: usize,
}

#[derive(Debug, Serialize)]
struct AspectPromptOutputRequirements<'a> {
    language: &'a str,
    max_findings_per_aspect: Option<usize>,
}

impl<'a> From<&'a EffectiveAspectPlan> for AspectPromptInput<'a> {
    fn from(plan: &'a EffectiveAspectPlan) -> Self {
        Self {
            aspect: AspectPromptAspect {
                id: &plan.task.id,
                name: &plan.task.name,
                role: &plan.task.role,
                question: &plan.task.question,
                scope: &plan.task.scope,
                boundaries: &plan.task.boundaries,
                success_criteria: &plan.task.success_criteria,
            },
            context: &plan.context,
            available_tools: &plan.task.tools,
            evidence_requirements: AspectPromptEvidenceRequirements {
                require_evidence_for_findings: plan.policy.evidence.require_evidence_for_findings,
                min_evidence_per_finding: plan.policy.evidence.min_evidence_per_finding,
            },
            output_requirements: AspectPromptOutputRequirements {
                language: &plan.policy.output.language,
                max_findings_per_aspect: plan.policy.output.max_findings_per_aspect,
            },
        }
    }
}
