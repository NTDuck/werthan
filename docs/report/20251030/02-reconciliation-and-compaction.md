# 2 · Codebase Reconciliation & Frequent Intentional Compaction

## 2.1 The Small Language Model (SLM) Context Dilemma

Modernization loops employing small language models (27B parameter tier) exhibit a stark dichotomy between two operational regimes:
- **The "Smart Zone" ($\le 12\text{k}$ tokens)**: The model demonstrates crisp semantic reasoning, precise syntactic generation, and tight adherence to Rust compiler borrow-check hints.
- **The "Dumb Zone" ($> 16\text{k}$ tokens)**: Under context bloat—typically driven by uncurated compiler error logs, multiple stack traces, and verbose build status chatter—the model's effective attention degrades rapidly. In this regime, models emit redundant stubs, drop type annotations, and enter non-convergent hallucination loops.

## 2.2 Methodological Implementation: ADR-0018

To enforce that werðan's repair agents permanently operate within the Smart Zone, we introduced **Frequent Intentional Compaction** into the surgical code repair agent (`holon-patch`), ratified via **ADR-0018** (`docs/decisions/0018-frequent-intentional-compaction.md`):

```rust
/// Compact a diagnostic trace into a focused fault card within the SLM "Smart Zone" (<= 12k tokens).
/// Distills compiler errors, file/line spans, error codes, and suggestions, stripping noisy build boilerplate.
pub fn compact_diagnostic_trace(
    raw_trace: &::core::primitive::str,
    max_lines: ::core::primitive::usize,
) -> ::std::string::String {
    let lines = raw_trace.lines().collect::<::std::vec::Vec<&str>>();
    if lines.len() <= max_lines {
        return raw_trace.to_string();
    }
    let mut relevant_lines = ::std::vec::Vec::new();
    for line in &lines {
        let trimmed = line.trim();
        let is_key_diagnostic = trimmed.starts_with("error")
            || trimmed.starts_with("-->")
            || trimmed.starts_with("help:")
            || trimmed.starts_with("note:")
            || trimmed.contains("TypeError")
            || trimmed.contains("SyntaxError")
            || trimmed.contains("AssertionError")
            || trimmed.contains("failed")
            || trimmed.contains("panic");
        if is_key_diagnostic {
            relevant_lines.push(*line);
            if relevant_lines.len() >= max_lines {
                break;
            }
        }
    }
    if relevant_lines.is_empty() {
        lines
            .iter()
            .take(max_lines)
            .copied()
            .collect::<::std::vec::Vec<&str>>()
            .join("\n")
    } else {
        relevant_lines.join("\n")
    }
}
```

In `render_repair_prompt`, raw diagnostics are deterministically compressed to at most 20 actionable lines before prompt delivery, distilling compiler error codes, source span arrows (`-->`), and compiler suggestions (`help:`).

## 2.3 Lockstep Specification Synchronization

In strict adherence to the project's zero-drift governance:
1. **Decision Ledger**: Append-only architectural decision recorded at `docs/decisions/0018-frequent-intentional-compaction.md`.
2. **Specification Index**: `docs/specs/INDEX.md` updated to map `ADR-0018`.
3. **Technical Spec**: `docs/specs/tech-spec/statecharts-and-ariadne.md` §9 updated to document the compaction bound.
4. **Stepdown & Hygiene Gates**: Function ordering strictly adheres to the caller-before-callee rule (`stepdown`), fully qualified paths (`pathqual`), and zero single-character variable bindings.

## 2.4 Council of Subagents Verification Gate

Prior to campaign launch, a specialized reviewer agent (`CouncilVerifier`) executed an adversarial verification pass across the modified crates and documentation:
- **Quality & SLAP**: Single level of abstraction confirmed; pure function with zero external side effects.
- **Spec Cross-References**: Verified bidirectional navigation between `INDEX.md`, `statecharts-and-ariadne.md`, and `0018-frequent-intentional-compaction.md`.
- **Validation Suite**: 9/9 unit and validation contract tests in `holon-patch` passed without warnings.
- **Verdict**: Zero fatal drift confirmed.
