mod support;

use std::collections::BTreeSet;
use std::sync::atomic::Ordering;
use std::time::Duration;

use lapis_error::Error;
use lapis_workflow::Limit;
use lapis_workflow::deep_research;
use lapis_workflow::{AgentBudget, BudgetConfig, ResearchBudget};
use support::research::{
    deep_request, services, services_with_delay, services_with_token_usage, unlimited_budget_config,
};

#[tokio::test]
async fn completes_three_aspects_with_bounded_concurrency() {
    let request = deep_request(3);
    let services = services(&[]);

    let result = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect("deep result");

    assert_eq!(result.completed_aspects.len(), 3);
    assert!(result.failed_aspects.is_empty());
    assert_eq!(result.aspect_reports.len(), 3);
    assert_eq!(result.evidence_index.len(), 3);
    let evidence_ids = result
        .evidence_index
        .iter()
        .map(|evidence| evidence.id.as_str())
        .collect::<BTreeSet<_>>();
    for report in &result.aspect_reports {
        for finding in &report.findings {
            assert!(
                finding
                    .evidence_refs
                    .iter()
                    .all(|id| evidence_ids.contains(id.as_str()))
            );
        }
    }
    assert_eq!(result.open_questions.len(), 3);
    assert_eq!(result.coverage_summary.requested_aspects, 3);
    assert_eq!(result.coverage_summary.completed_aspects, 3);
    assert_eq!(result.coverage_summary.failed_aspects, 0);
    assert_eq!(result.confidence_summary.medium, 3);
    assert_eq!(result.budget_usage.model_calls_used, 6);
    assert_eq!(result.budget_usage.search_calls_used, 3);
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 6);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 3);
    assert_eq!(services.max_in_flight.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn returns_partial_result_after_single_aspect_failure() {
    let request = deep_request(3);
    let services = services(&["aspect-2"]);

    let result = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect("partial result");

    assert_eq!(result.completed_aspects.len(), 2);
    assert_eq!(result.failed_aspects.len(), 1);
    assert_eq!(result.failed_aspects[0].aspect_id, "aspect-2");
}

#[tokio::test]
async fn all_aspects_failed_returns_error() {
    let request = deep_request(2);
    let services = services(&["aspect-1", "aspect-2"]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("all failed");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 2);
    assert_eq!(failure.failed_aspects[0].aspect_id, "aspect-1");
    assert_eq!(failure.failed_aspects[1].aspect_id, "aspect-2");
    assert_eq!(
        failure.failed_aspects[0].error_code,
        "schema_validation_failed"
    );
}

#[tokio::test]
async fn partial_results_disabled_returns_error() {
    let mut request = deep_request(3);
    request.execution_policy.allow_partial_results = false;
    let services = services(&["aspect-2"]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("partial disabled");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].aspect_id, "aspect-2");
}

#[tokio::test]
async fn fail_fast_stops_before_scheduling_remaining_aspects() {
    let mut request = deep_request(2);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.execution_policy.fail_fast = true;
    let services = services(&["aspect-1"]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("fail fast error");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].aspect_id, "aspect-1");
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn all_agent_budget_failures_preserve_failed_aspects() {
    let mut request = deep_request(2);
    request.budget.max_concurrent_agents = Limit::limited(1);
    for task in &mut request.aspect_tasks {
        task.budget.max_search_calls = Limit::limited(0);
    }
    let services = services(&[]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("agent budget failures");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 2);
    assert_eq!(failure.failed_aspects[0].aspect_id, "aspect-1");
    assert_eq!(failure.failed_aspects[1].aspect_id, "aspect-2");
    assert!(
        failure
            .failed_aspects
            .iter()
            .all(|failure| failure.error_code == "budget_exceeded")
    );
    assert!(
        failure
            .failed_aspects
            .iter()
            .all(|failure| failure.message == "agent search call budget exhausted")
    );
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn rejects_plan_exceeding_max_agents() {
    let mut request = deep_request(3);
    request.budget.max_agents = Limit::limited(2);
    let services = services(&[]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("too many aspects");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert!(failure.failed_aspects.is_empty());
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn rejects_plan_when_effective_max_agents_cannot_cover_aspects() {
    let request = deep_request(3);
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_agents: Limit::limited(2),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let failure = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("effective max_agents cannot cover requested aspects");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert!(failure.failed_aspects.is_empty());
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn research_concurrency_above_configured_limits_is_clamped() {
    let request = deep_request(3);
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_agents: Limit::limited(3),
            max_concurrent_agents: Limit::limited(1),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let result = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect("configured concurrency should cap request concurrency");

    assert_eq!(result.completed_aspects.len(), 3);
    assert_eq!(services.max_in_flight.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rejects_agent_budget_above_configured_limits() {
    let request = deep_request(2);
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget::unlimited(),
        per_agent: AgentBudget {
            max_turns: Limit::limited(5),
            ..AgentBudget::unlimited()
        },
    };

    let failure = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("agent budget exceeds configured limits");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert!(failure.failed_aspects.is_empty());
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 0);
}

/// Global search-call budget is enforced pre-dispatch: the second aspect's
/// search call must be rejected before reaching the search provider, so the
/// underlying counter advances exactly once.
#[tokio::test]
async fn global_search_budget_blocks_further_calls() {
    let mut request = deep_request(2);
    request.budget.max_total_search_calls = Limit::limited(1);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.execution_policy.allow_partial_results = false;
    let services = services(&[]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("global search budget");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].error_code, "budget_exceeded");
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 1);
}

/// Global model-call budget is enforced pre-dispatch: once the cap is hit,
/// the next aspect must be rejected before its first model turn dispatches.
#[tokio::test]
async fn global_model_budget_stops_before_extra_model_call() {
    let mut request = deep_request(2);
    request.budget.max_total_model_calls = Limit::limited(2);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.execution_policy.allow_partial_results = false;
    let services = services(&[]);

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("global model budget");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].error_code, "budget_exceeded");
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn request_unlimited_model_budget_inherits_config_cap() {
    let mut request = deep_request(2);
    request.budget.max_total_model_calls = Limit::unlimited();
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.execution_policy.allow_partial_results = false;
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_total_model_calls: Limit::limited(2),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let failure = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("configured model budget should cap unlimited request");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].error_code, "budget_exceeded");
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn config_model_budget_wins_when_request_budget_is_higher() {
    let mut request = deep_request(2);
    request.budget.max_total_model_calls = Limit::limited(10);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.execution_policy.allow_partial_results = false;
    let services = services(&[]);
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_total_model_calls: Limit::limited(2),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let failure = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("configured model budget should be stricter");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].error_code, "budget_exceeded");
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
}

/// `max_tokens` rejects the run as soon as the merged provider-reported
/// `total_tokens` exceeds the cap, even if call counters are still in range.
#[tokio::test]
async fn global_token_budget_stops_when_total_exceeds_max() {
    use lapis_workflow::TokenUsage;
    let mut request = deep_request(1);
    request.budget.max_tokens = Limit::limited(100);
    let services = services_with_token_usage(
        &[],
        Some(TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: Some(200),
        }),
    );

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("global token budget");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert!(failure.failed_aspects.is_empty());
}

/// When the provider omits `total_tokens`, the guard must fall back to
/// `input_tokens + output_tokens` so an under-reporting provider still
/// counts against the cap.
#[tokio::test]
async fn global_token_budget_falls_back_to_input_plus_output() {
    use lapis_workflow::TokenUsage;
    let mut request = deep_request(1);
    request.budget.max_tokens = Limit::limited(100);
    let services = services_with_token_usage(
        &[],
        Some(TokenUsage {
            input_tokens: Some(80),
            output_tokens: Some(40),
            total_tokens: None,
        }),
    );

    let failure = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect_err("global token budget fallback");

    assert!(matches!(failure.error, Error::BudgetExceeded { .. }));
    assert!(failure.failed_aspects.is_empty());
}

/// When one model response reports only `total_tokens` and a later one
/// reports only `input_tokens`/`output_tokens`, the merged total must
/// account for both reports so `max_tokens` cannot be bypassed by mixing
/// provider report formats.
#[test]
fn token_usage_merge_counts_mixed_provider_reporting_formats() {
    use lapis_workflow::TokenUsage;
    let merged = TokenUsage::merge(
        Some(TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: Some(100),
        }),
        Some(TokenUsage {
            input_tokens: Some(30),
            output_tokens: Some(20),
            total_tokens: None,
        }),
    )
    .expect("merged usage");
    assert_eq!(merged.total_or_sum(), Some(150));
}

/// Once cumulative token usage has reached the cap, subsequent provider
/// dispatches must be rejected at the guard before any new model or search
/// call goes out.
#[tokio::test]
async fn global_token_budget_blocks_dispatch_after_cap_is_reached() {
    use lapis_workflow::TokenUsage;
    let mut request = deep_request(2);
    request.budget.max_tokens = Limit::limited(100);
    request.budget.max_concurrent_agents = Limit::limited(1);
    let services = services_with_token_usage(
        &[],
        Some(TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: Some(100),
        }),
    );

    let _ = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await;

    // The first aspect's first model turn reports usage = 100 (== cap), which
    // closes the token budget. Every subsequent dispatch (search and the
    // remaining aspect's model turn) must be rejected at the guard.
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 1);
    assert_eq!(services.search_calls.load(Ordering::SeqCst), 0);
}

/// The operator token cap is merged into the effective budget instead of
/// rejecting an otherwise well-formed request during validation.
#[tokio::test]
async fn request_budget_max_tokens_is_clamped_to_config_cap() {
    use lapis_workflow::TokenUsage;
    let mut request = deep_request(1);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.budget.max_tokens = Limit::limited(1_000);
    let services = services_with_token_usage(
        &[],
        Some(TokenUsage {
            input_tokens: None,
            output_tokens: None,
            total_tokens: Some(100),
        }),
    );
    let limits = BudgetConfig {
        research: ResearchBudget {
            max_tokens: Limit::limited(150),
            ..ResearchBudget::unlimited()
        },
        per_agent: AgentBudget::unlimited(),
    };

    let failure = deep_research(request, &services.model, &services.search, &limits)
        .await
        .expect_err("configured token budget should cap request");

    assert!(matches!(failure.error, Error::PartialResult { .. }));
    assert_eq!(failure.failed_aspects.len(), 1);
    assert_eq!(failure.failed_aspects[0].error_code, "budget_exceeded");
    assert_eq!(services.model_calls.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn post_run_timeout_preserves_completed_aspect_when_partial_allowed() {
    let mut request = deep_request(3);
    request.budget.max_concurrent_agents = Limit::limited(1);
    request.budget.total_timeout_ms = Limit::limited(110);
    request.execution_policy.fail_fast = true;
    request.execution_policy.timeout_ms = Limit::limited(110);
    let services = services_with_delay(&["aspect-2"], Duration::from_millis(30));

    let result = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect("partial result");

    assert_eq!(result.completed_aspects, vec!["aspect-1".to_owned()]);
    assert_eq!(result.aspect_reports.len(), 1);
    assert_eq!(result.evidence_index.len(), 1);
    assert_eq!(result.failed_aspects.len(), 2);
    assert_eq!(result.failed_aspects[0].aspect_id, "aspect-2");
    assert_eq!(result.failed_aspects[1].aspect_id, "aspect-3");
    assert_eq!(result.failed_aspects[1].error_code, "budget_exceeded");
    assert!(!result.failed_aspects[1].aspect_id.starts_with("__"));
    assert_eq!(result.coverage_summary.requested_aspects, 3);
    assert_eq!(result.coverage_summary.completed_aspects, 1);
    assert_eq!(result.coverage_summary.failed_aspects, 2);
}

/// Two concurrent aspects share the same cross-aspect guard: with
/// `max_total_search_calls = 1`, exactly one aspect must succeed at search
/// while the other is rejected pre-dispatch.
#[tokio::test]
async fn concurrent_aspects_share_global_budget_guard() {
    let mut request = deep_request(2);
    request.budget.max_concurrent_agents = Limit::limited(2);
    request.budget.max_total_search_calls = Limit::limited(1);
    let services = services(&[]);

    let result = deep_research(
        request,
        &services.model,
        &services.search,
        &unlimited_budget_config(),
    )
    .await
    .expect("partial result");

    assert_eq!(services.search_calls.load(Ordering::SeqCst), 1);
    assert_eq!(result.completed_aspects.len(), 1);
    assert_eq!(result.failed_aspects.len(), 1);
    assert_eq!(result.failed_aspects[0].error_code, "budget_exceeded");
    assert!(!result.failed_aspects[0].aspect_id.starts_with("__"));
    assert_eq!(result.coverage_summary.requested_aspects, 2);
    assert_eq!(result.coverage_summary.completed_aspects, 1);
    assert_eq!(result.coverage_summary.failed_aspects, 1);
}
