//! Pure helper characterization tests for limit merge and evidence provenance.
//!
//! These cover crate-public pure helpers without embedding `#[cfg(test)]` in
//! production modules (owner standard: all tests live in moe-research-tests).

use moe_research_workflow::{
    Confidence, Evidence, Limit, ResearchLimits, SourceType, effective_research_limits,
    provenance_mismatch_fields,
};

#[test]
fn permits_next_respects_limited_and_unlimited() {
    assert!(Limit::Unlimited.permits_next(10_000));
    assert!(Limit::Limited(3).permits_next(2));
    assert!(!Limit::Limited(3).permits_next(3));
}

#[test]
fn exceeds_matrix() {
    assert!(!Limit::Limited(1usize).exceeds(Limit::Unlimited));
    assert!(Limit::Unlimited.exceeds(Limit::Limited(1usize)));
    assert!(Limit::Limited(5usize).exceeds(Limit::Limited(3usize)));
    assert!(!Limit::Limited(2usize).exceeds(Limit::Limited(3usize)));
}

#[test]
fn is_exceeded_by_and_u64_variants() {
    assert!(!Limit::Limited(3usize).is_exceeded_by(3));
    assert!(Limit::Limited(3usize).is_exceeded_by(4));
    assert!(Limit::Limited(3usize).is_exceeded_by_u64(4));
    let unlimited_count: Limit<usize> = Limit::Unlimited;
    assert!(!unlimited_count.is_exceeded_by_u64(u64::MAX));
    let unlimited_token: Limit<u64> = Limit::Unlimited;
    assert!(!unlimited_token.is_exceeded_by_u64(u64::MAX));
}

#[test]
fn duration_elapsed_and_token_exhausted() {
    assert!(Limit::Limited(10u64).is_elapsed(10));
    assert!(!Limit::Limited(10u64).is_elapsed(9));
    assert!(Limit::Limited(10u64).is_exhausted_by_u64(10));
    assert!(!Limit::Limited(10u64).is_exceeded_by_u64(10));
    assert!(Limit::Limited(10u64).is_exceeded_by_u64(11));
}

#[test]
fn effective_research_limits_merges_fieldwise() {
    let configured = ResearchLimits {
        max_agents: Limit::Limited(3),
        max_concurrent_agents: Limit::Limited(2),
        max_total_model_calls: Limit::Unlimited,
        max_total_search_calls: Limit::Limited(10),
        total_timeout_ms: Limit::Limited(60_000),
        max_tokens: Limit::Limited(1000),
    };
    let requested = ResearchLimits {
        max_agents: Limit::Limited(5),
        max_concurrent_agents: Limit::Unlimited,
        max_total_model_calls: Limit::Limited(7),
        max_total_search_calls: Limit::Limited(4),
        total_timeout_ms: Limit::Limited(30_000),
        max_tokens: Limit::Unlimited,
    };
    let merged = effective_research_limits(&configured, Some(&requested));
    assert_eq!(merged.max_agents, Limit::Limited(3));
    assert_eq!(merged.max_concurrent_agents, Limit::Limited(2));
    assert_eq!(merged.max_total_model_calls, Limit::Limited(7));
    assert_eq!(merged.max_total_search_calls, Limit::Limited(4));
    assert_eq!(merged.total_timeout_ms, Limit::Limited(30_000));
    assert_eq!(merged.max_tokens, Limit::Limited(1000));
}

#[test]
fn effective_research_limits_none_requested_clones_config() {
    let configured = ResearchLimits {
        max_agents: Limit::Limited(1),
        max_concurrent_agents: Limit::Limited(1),
        max_total_model_calls: Limit::Limited(1),
        max_total_search_calls: Limit::Limited(1),
        total_timeout_ms: Limit::Limited(1),
        max_tokens: Limit::Limited(1),
    };
    let merged = effective_research_limits(&configured, None);
    assert_eq!(merged, configured);
}

fn sample_evidence(snippet: &str) -> Evidence {
    Evidence {
        id: "e1".into(),
        source_title: "Title".into(),
        url: Some("https://example.com".into()),
        provider: "grok".into(),
        query: "q".into(),
        snippet: snippet.into(),
        summary: "sum".into(),
        published_at: None,
        retrieved_at: "2026-01-01T00:00:00Z".into(),
        supports_findings: vec![],
        source_type: SourceType::Documentation,
        confidence: Confidence::Medium,
    }
}

#[test]
fn provenance_mismatch_fields_empty_when_equal() {
    let a = sample_evidence("same");
    let b = sample_evidence("same");
    assert!(provenance_mismatch_fields(&a, &b).is_empty());
}

#[test]
fn provenance_mismatch_fields_lists_all_divergent_names_in_order() {
    let candidate = sample_evidence("orig");
    let mut selected = candidate.clone();
    selected.snippet = "mutated".into();
    selected.summary = "mutated-sum".into();
    selected.url = Some("https://evil.example".into());
    let fields = provenance_mismatch_fields(&selected, &candidate);
    assert_eq!(fields, vec!["url", "snippet", "summary"]);
}
