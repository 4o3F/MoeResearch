/// Byte budget added to each side of a citation when extracting the
/// snippet shown in `SearchResult.snippet`. Calibrated so a typical
/// citation marker (`[[N]](url)`) lands inside a readable sentence.
const SNIPPET_PAD_BYTES: usize = 80;

/// Hard upper bound on `SearchResult.snippet`. Anything longer is
/// truncated to a char boundary with a trailing ellipsis so log
/// consumers always see a bounded line.
const SNIPPET_MAX_BYTES: usize = 240;

/// Larger byte budget for the per-source `SearchResult.summary`. The
/// summary is the model's main reading material so it warrants more
/// context than the snippet, but it still needs a ceiling — without
/// it every evidence row in a single search would carry the same
/// 1 KiB+ Markdown blob and waste a large fraction of Layer 2's
/// prompt budget.
const SUMMARY_PAD_BYTES: usize = 240;
const SUMMARY_MAX_BYTES: usize = 600;

/// Controls how aggressively `excerpt_around` walks away from the
/// citation when picking word boundaries.
///
/// `Tight` stops at the first whitespace just outside the citation
/// range, so the resulting snippet hugs the original indices — useful
/// when we just want to round mid-word cuts back to a clean boundary.
///
/// `Wide` extends to the *farthest* whitespace within `pad_bytes`, so
/// the resulting excerpt includes as much surrounding context as the
/// budget allows — used for summaries so two evidence rows from the
/// same `output_text` describe their own passage instead of sharing
/// one identical Markdown blob.
#[derive(Clone, Copy)]
enum ExpandStrategy {
    Tight,
    Wide,
}

/// Builds the per-source `snippet`: a readable sentence rounded outward
/// from the citation indices to the nearest word boundaries. Without
/// indices we fall back to a clamped excerpt of the whole text rather
/// than failing — Grok occasionally omits indices on some annotation
/// variants.
pub(super) fn citation_snippet(text: &str, start: Option<usize>, end: Option<usize>) -> String {
    excerpt_around(
        text,
        start,
        end,
        SNIPPET_PAD_BYTES,
        SNIPPET_MAX_BYTES,
        ExpandStrategy::Tight,
    )
}

/// Builds the per-source `summary`: a longer excerpt around the citation
/// using the wide expansion strategy, so two evidence rows from the
/// same `output_text` describe their own passage rather than sharing
/// one identical Markdown blob.
pub(super) fn citation_local_summary(
    text: &str,
    start: Option<usize>,
    end: Option<usize>,
) -> String {
    excerpt_around(
        text,
        start,
        end,
        SUMMARY_PAD_BYTES,
        SUMMARY_MAX_BYTES,
        ExpandStrategy::Wide,
    )
}

/// Extracts a UTF-8 safe excerpt around `[start, end)` padded by
/// `pad_bytes` on each side, snapped to whitespace per `strategy`,
/// then clamped to `max_bytes` with a trailing ellipsis when truncated.
///
/// Returns a clamped excerpt of the whole input when the indices are
/// missing, malformed, or do not land on UTF-8 char boundaries. The
/// returned string is always trimmed.
fn excerpt_around(
    text: &str,
    start: Option<usize>,
    end: Option<usize>,
    pad_bytes: usize,
    max_bytes: usize,
    strategy: ExpandStrategy,
) -> String {
    let trimmed_fallback = || clamp_to_max(text.trim(), max_bytes);
    let (Some(start), Some(end)) = (start, end) else {
        return trimmed_fallback();
    };
    if start >= end
        || end > text.len()
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return trimmed_fallback();
    }

    let left = expand_left(text, start, pad_bytes, strategy);
    let right = expand_right(text, end, pad_bytes, strategy);
    clamp_to_max(text[left..right].trim(), max_bytes)
}

/// Walks left from `anchor` by at most `budget` bytes and returns the
/// byte offset where the excerpt should start so it begins at a clean
/// word boundary.
///
/// In `Tight` mode the search starts at `anchor` and walks backward,
/// returning at the first whitespace encountered — the smallest legal
/// expansion. In `Wide` mode the search starts at `anchor - budget`
/// and walks forward, returning at the first whitespace it finds —
/// the largest legal expansion within budget. Wide also short-circuits
/// to byte 0 when the budget reaches the start of text, because the
/// edge is a cleaner boundary than any interior whitespace. Both modes
/// fall back to the UTF-8 boundary at or below `anchor - budget` when
/// no whitespace exists in the search window.
fn expand_left(text: &str, anchor: usize, budget: usize, strategy: ExpandStrategy) -> usize {
    let lower = anchor.saturating_sub(budget);
    // Wide prefers maximum extension; start-of-text outranks any
    // interior whitespace within budget.
    if matches!(strategy, ExpandStrategy::Wide) && lower == 0 {
        return 0;
    }
    let bytes = text.as_bytes();
    let range: Box<dyn Iterator<Item = usize>> = match strategy {
        ExpandStrategy::Tight => Box::new((lower..anchor).rev()),
        ExpandStrategy::Wide => Box::new(lower..anchor),
    };
    for i in range {
        if bytes[i].is_ascii_whitespace() {
            return i + 1;
        }
    }
    // Edge of text counts as a word boundary.
    if lower == 0 {
        return 0;
    }
    let mut p = lower;
    while p > 0 && !text.is_char_boundary(p) {
        p -= 1;
    }
    p
}

/// Mirror of [`expand_left`] walking right; same strategy semantics.
fn expand_right(text: &str, anchor: usize, budget: usize, strategy: ExpandStrategy) -> usize {
    let upper = anchor.saturating_add(budget).min(text.len());
    // Wide prefers maximum extension; end-of-text outranks any
    // interior whitespace within budget.
    if matches!(strategy, ExpandStrategy::Wide) && upper == text.len() {
        return upper;
    }
    let bytes = text.as_bytes();
    let range: Box<dyn Iterator<Item = usize>> = match strategy {
        ExpandStrategy::Tight => Box::new(anchor..upper),
        ExpandStrategy::Wide => Box::new((anchor..upper).rev()),
    };
    for i in range {
        if bytes[i].is_ascii_whitespace() {
            return i;
        }
    }
    if upper == text.len() {
        return upper;
    }
    let mut p = upper;
    while p < text.len() && !text.is_char_boundary(p) {
        p += 1;
    }
    p
}

/// Clamps a string to at most `max_bytes` bytes at a UTF-8 boundary,
/// appending an ellipsis when truncation occurs.
fn clamp_to_max(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_owned();
    }
    let mut cut = max_bytes;
    while cut > 0 && !text.is_char_boundary(cut) {
        cut -= 1;
    }
    format!("{}…", text[..cut].trim_end())
}
