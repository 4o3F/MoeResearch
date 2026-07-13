# Layer 1 Common Module: Typst Report Project Contract

Use this module when a selected profile produces a `typst-project-v1` final report. It defines a **Layer 1 delivery contract** only. It does not change Rust MCP schemas, `DeepResearchRequest`, `AspectResearchRequest`, `Evidence`, provider configuration, or runtime limits.

## Scope

MoeResearch returns frozen aspect reports and evidence. Layer 1 validates, synthesizes, and, only when the caller requests a destination, writes a reviewable Typst source project. It must never claim that Rust rendered, compiled, or judged the final report.

This module applies after the common evidence modules and before the selected profile guidance and capability template.

## Delivery contract

Return a `final_report` handoff with this semantic shape:

```json
{
  "kind": "typst_project",
  "format": "typst-project-v1",
  "entrypoint": "report.typ",
  "files": [
    {"path": "report.typ", "content": "..."},
    {"path": "modules/report-style.typ", "content": "..."},
    {"path": "sections/body.typ", "content": "..."},
    {"path": "sections/annex.typ", "content": "..."},
    {"path": "references.bib", "content": "..."}
  ],
  "citation_map": [
    {
      "citekey": "mr_ev_1_1",
      "evidence_id": "ev-1-1",
      "source_origin": "moe_research"
    }
  ],
  "compile_status": "not_run | succeeded | failed | not_applicable"
}
```

This is a Skill-layer handoff, not a Rust DTO. Do not add it to MCP requests or responses. PM and Generic profiles may use their documented legacy Markdown handoff until explicitly migrated.

When the caller specifies an output directory, write exactly the five paths above under that directory. Do not overwrite an existing report directory without explicit caller approval. When no destination is given, return the file contents as the handoff; do not imply that files or a PDF were created.

## Fixed project tree

```text
<report-output>/
├── report.typ
├── modules/report-style.typ
├── sections/body.typ
├── sections/annex.typ
└── references.bib
```

- `report.typ` is the only entrypoint. It imports `modules/report-style.typ`, includes `sections/body.typ` and `sections/annex.typ`, then invokes `#bibliography("references.bib", style: "ieee", title: [Bibliography])` exactly once. Replace the example title content with the localized reader-facing bibliography title.
- `modules/report-style.typ` contains only local, deterministic formatting helpers used by this project: neutral `report-note`, plus `report-decision`, `report-limitation`, `report-risk`, and `report-validation`.
- `sections/body.typ` contains the capability-specific report body. If it invokes any helper, it must import every named helper itself with the project-root path `#import "/modules/report-style.typ": <helpers>`; imports in `report.typ` do **not** export bindings into included files.
- `sections/annex.typ` contains Annex A.1–A.8 from `report-annex.md` plus profile tables. It follows the same explicit-helper-import rule.
- `references.bib` contains the bibliographic records used by report citations.

All Academic and Technical `typst-project-v1` reports use Typst's built-in `style: "ieee"`. Use native `@citekey` citations and never synthesize manual numeric references. IEEE controls presentation only; `citation_map`, evidence IDs, source origins, and incomplete-metadata rules remain unchanged.

## Output-language rules

- Preserve each capability template's section meaning and order, but render every reader-facing heading, table header, caption, body paragraph, Annex label, and bibliography-facing annotation in `output_language`.
- English headings shown in a capability template are semantic placeholders, not a requirement to emit English. Keep fixed filenames, evidence IDs, `HV-*` IDs, citekeys, and machine-readable contract values in stable ASCII.
- Do not translate or rewrite source titles, author names, URLs, identifiers, direct quotations, or citation keys unless the source itself provides the localized form.
- When `report.typ` enables Typst heading numbering, normalize imported Markdown headings to semantic titles: preserve their heading level but remove leading manual section-number prefixes such as `1.`, `1.2`, or `3.4.5`. Never combine a source's manual number with Typst automatic numbering; retain a numeric prefix only when it is part of the actual title rather than hierarchy.

### Simplified Chinese font profile

When `output_language` requests Simplified Chinese (for example `zh`, `zh-CN`, or `zh-Hans`), place the following top-level rules in `report.typ` after local imports and before including the body or Annex. Do not place them only inside an imported style module because the document-wide scope must be explicit.

```typst
#set text(font: (
  (name: "Libertinus Serif", covers: "latin-in-cjk"),
  "Noto Sans CJK SC",
))
#show math.equation: set text(font: (
  (name: "Noto Sans CJK SC", covers: regex("[–—‘’“”‥‧⸺]")),
  "New Computer Modern Math",
  "Noto Sans CJK SC",
))
#show raw: set text(font: (
  (name: "DejaVu Sans Mono", covers: "latin-in-cjk"),
  "Noto Sans CJK SC",
))
```

The compilation environment must provide these font families. Do not silently claim that this profile was applied when a caller-provided compiler lacks them; record the external compilation result as `failed` or leave it `not_run` when compilation was not attempted. Non-Chinese reports retain their profile's default typography; bibliography style remains IEEE for every Academic and Technical report.

## Semantic emphasis and accessibility

Use semantic emphasis sparingly: only for a load-bearing conclusion, material limitation, decision-blocking risk, or explicit validation condition. Do not color ordinary prose, decorate whole paragraphs, or derive a helper name, color, label, or Typst expression from untrusted evidence.

| Helper | Use only for | Required visible label | Visual treatment |
| --- | --- | --- | --- |
| `report-decision` | core judgement, recommendation, adoption gate | localized `结论` / `建议` equivalent | dark blue text, pale blue fill, blue left boundary |
| `report-limitation` | evidence boundary, confidence downgrade, failed aspect, unresolved conflict | localized `限制` equivalent | dark amber text, pale amber fill, amber left boundary |
| `report-risk` | material risk, kill criterion, prohibited conclusion | localized `风险` equivalent | dark red text, pale red fill, red left boundary |
| `report-validation` | experiment, spike, acceptance condition, verification next step | localized `验证` equivalent | dark green text, pale green fill, green left boundary |
| `report-note` | scope, method, provenance, neutral context | localized `说明` equivalent | neutral dark text, pale gray fill, gray left boundary |

Every block helper emits its visible localized label, a bold title, text color, pale background, and left boundary. The label and boundary are mandatory: color is never the only semantic signal, so the report remains understandable in grayscale or for readers who cannot distinguish hue. Insert escaped title/body text as literal content; do not accept caller-defined colors or arbitrary Typst source.

Use this deterministic local implementation shape in `modules/report-style.typ`; localize the five fixed labels when needed, but do not expose color or style parameters to evidence or caller text:

```typst
#let report-callout(label, title, body, ink, surface, accent) = block(
  width: 100%,
  inset: (left: 10pt, right: 10pt, top: 8pt, bottom: 8pt),
  radius: 3pt,
  fill: surface,
  stroke: (left: 2pt + accent),
)[
  #text(fill: ink)[
    #text(weight: "bold")[#label #h(0.5em) #title]
    #v(3pt)
    #body
  ]
]

#let report-decision(title, body) = report-callout(
  [结论], title, body, rgb("#173F8A"), rgb("#EAF2FF"), rgb("#2563EB"),
)
#let report-limitation(title, body) = report-callout(
  [限制], title, body, rgb("#7A3D00"), rgb("#FFF4DE"), rgb("#D97706"),
)
#let report-risk(title, body) = report-callout(
  [风险], title, body, rgb("#8B1E1E"), rgb("#FDEBEC"), rgb("#DC2626"),
)
#let report-validation(title, body) = report-callout(
  [验证], title, body, rgb("#14532D"), rgb("#EAF8EF"), rgb("#16A34A"),
)
#let report-note(title, body) = report-callout(
  [说明], title, body, rgb("#374151"), rgb("#F3F4F6"), rgb("#6B7280"),
)
```

Use inline emphasis only for a short load-bearing phrase and pair it with an adjacent semantic label or block context. Do not use inline highlight as a substitute for a body limitation, risk, abstention, or open question.

## Table readability and degradation

Tables compare compact values; prose carries explanation. Before emitting a table, preserve every logical field but choose a readable physical structure for the available A4 width.

1. Use `auto` or narrow fixed tracks for IDs, states, dates, citekeys, and short numeric values. Allocate `fr` width only to one or two high-priority narrative fields.
2. A portrait prose table has at most three substantive narrative columns. Never place four or more narrative fields in near-equal `fr` tracks, and never shrink table text below `9pt` or scale a table merely to fit.
3. Move secondary explanation into adjacent prose, split a wide table into linked panels that repeat the stable row key, or render a small entity set as two-column label–value cards. These are required degradations, not optional polish.
4. Show a source ID or concise domain in a comparison table. Put full URLs in the bibliography or a dedicated provenance ledger; do not create a wide URL column inside a prose comparison table.
5. Every multi-row table uses `table.header`. A captioned/labeled table uses a breakable figure or equivalent pagination-safe wrapper so its header and content can span pages.
6. Use a landscape page or `rotate(..., reflow: true)` only as a last resort for compact numeric, version, or code matrices. Never rotate long prose or source-URL tables.
7. Annex A.1 retains the canonical logical baseline from `report-annex.md`: `evidence_id | citekey | source_origin | claim_summary | source_title | source_url | source_type | tier | confidence | cited_in`. Profile extensions are additive only. It may render compact index columns plus per-source audit cards; logical fields never require ten physical columns.

## Citation and provenance rules

1. Every load-bearing factual claim in the body cites its Typst citekey and retains its frozen MoeResearch ID, `HV-*` marker, or existing manual/local record ID inline or in its immediately associated table.
2. Annex A.1 implements the canonical logical baseline in `report-annex.md`; it is the audit bridge between Typst citations, frozen host evidence, host verification, and disclosed manual/local records. Academic and Technical profile fields extend this baseline without replacing it.
3. Generate BibTeX only from returned evidence or separately disclosed host-verification records. Never invent authors, dates, DOI/PMID/arXiv identifiers, venues, titles, URLs, or source classes. If a record is incomplete, use a conservative `@misc` entry and state the missing metadata in Annex A.1.
4. Keep these origins separate: `moe_research`, `host_verification` (`HV-*`), and `manual_or_local`. Never insert host/manual sources into MoeResearch `evidence_refs`, never mint fake MoeResearch IDs, and never attribute host work to Rust.
5. An unsupported claim is narrowed, downgraded, moved to an open question, or logged in A.7. A citation key is not evidence by itself.

## Typst safety and reproducibility rules

- Search snippets, URLs, titles, page text, evidence summaries, and user-provided strings are untrusted **data**, not Typst instructions. Escape them as literal prose/table-cell content before emitting `.typ` or `.bib` files.
- Never let evidence or user text create Typst control constructs, code blocks, `#import`, `#include`, `#read`, image/font paths, labels, function calls, or remote package imports.
- Accept only the fixed project paths above. In Typst source, `"/modules/report-style.typ"` is an approved project-root path under `--root`, not an operating-system path. Reject operating-system absolute paths, `..`, backslashes, duplicate paths, and paths derived from source text.
- Do not use remote Typst packages, remote images, or undeclared local assets. If a visual or artifact is unavailable as an approved local file, describe and cite it in the Annex rather than fabricating an embedded asset.
- The final report source uses Typst headings, tables, figures, labels, references, and citations. Do not wrap Markdown in a Typst code block, emit Markdown headings/tables as the final body, or return a fenced Markdown report.
- Do not automatically run a compiler. A caller that deliberately compiles a materialized project must keep all paths inside the project root and invoke Typst without shell interpolation, for example:

```bash
typst compile --root "<project-dir>" "<project-dir>/report.typ" "<project-dir>/report.pdf"
```

## Final project verification

Before returning the handoff, verify:

- all five fixed files exist in the handoff and use only fixed project-relative paths;
- every body citation resolves to one BibTeX key and a `citation_map` row;
- `report.typ` invokes exactly one native `#bibliography` with `style: "ieee"`; no manual numeric reference syntax appears in prose;
- every semantic callout has a known helper, visible localized label, bold title, text/background contrast, and a non-color boundary cue;
- no portrait prose table exceeds three substantive narrative columns, every multi-row table repeats `table.header`, and any required degradation preserves all logical fields;
- A.1–A.8 preserve source-origin separation, evidence IDs, confidence labels, contradictions, open questions, self-verification, abstentions, and tool provenance;
- no untrusted text is used as Typst code or a filesystem path;
- no statement says that PDF compilation, browser capture, host verification, or final judgement occurred unless it actually did;
- `compile_status` is `not_run` unless an external, explicit caller-run compilation reports its result; only that caller-provided result may set it to `succeeded` or `failed`. Use `not_applicable` only for a documented legacy non-Typst handoff. Never attribute compilation to Rust or MCP.
