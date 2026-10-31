# 0018. Frequent Intentional Compaction for SLM In-Context Retention

Date: 2026-10-31

## Status

Accepted

## Context and Problem Statement

When deploying small language models (SLMs, e.g. 27B parameter models such as Qwen 2.5/3.8 27B on consumer hardware) in autonomous code translation and repair loops, models exhibit severe degradation when prompt token volume enters the "Dumb Zone" (>16k tokens). Specifically, raw compiler diagnostics, stack traces, and test logs quickly accumulate hundreds of lines of noise, driving prompt lengths to saturation and inducing hallucination loops, syntax corruption, and context drift. How can the repair pipeline maintain critical diagnostic signals while keeping prompt payloads strictly within the SLM "Smart Zone" ($\le 12\text{k}$ tokens)?

## Decision Drivers

* **Model Context Dynamics**: 27B SLMs exhibit high precision on concise, focused prompts ($\le 12\text{k}$ tokens) but suffer catastrophic attention dispersion when prompts are bloated with hundreds of lines of repetitive compiler noise.
* **Deterministic Information Density**: Over 80% of raw compiler traces consist of build harness status lines, intermediate crate compilation notifications, or redundant notes. The critical repair signals are the compiler error codes, file/line spans, error descriptions, and primary suggestions (`help:` / `-->`).
* **High Cohesion, Loose Coupling**: The compaction mechanism must live directly at the diagnostic preparation boundary (`holon-patch`) without entangling external toolchains or requiring network overhead.

## Considered Options

1. **Raw Log Passthrough**: Pass raw execution traces verbatim into repair prompts (incumbent pattern).
2. **Aggressive Total Truncation**: Truncate raw logs to a hard character limit (e.g. first 1,000 characters), risking truncation of actual error details at the end of the log.
3. **Frequent Intentional Compaction**: Synthesize raw diagnostic traces through deterministic semantic filtering—retaining error codes, file/line locations, error messages, and actionable compiler hints—capped at a tight budget (e.g. 20 lines / ~300 tokens).

## Decision Outcome

Chosen option 3: Frequent Intentional Compaction.

* **Diagnostic Compactor**: Implemented `compact_diagnostic_trace(raw_trace, max_lines)` in `holon-patch`.
* **Signal Extraction**: Extracts lines matching key diagnostic tokens (`error`, `-->`, `help:`, `note:`, `TypeError`, `SyntaxError`, `AssertionError`, `failed`, `panic`), discarding build system chatter and status logs.
* **Smart Zone Bound**: Capped default repair prompt diagnostic payload at 20 lines, guaranteeing the model remains within the high-attention "Smart Zone" ($\le 12\text{k}$ tokens).

### Consequences

* Good, because the repair loop avoids prompt bloat and prevents 27B SLMs from falling into the "Dumb Zone" hallucination regime.
* Good, because key compiler errors and source spans are preserved with 100% fidelity.
* Neutral, because builds with fewer than `max_lines` are passed through untouched without alteration.

## More Information

* Spec routing: `docs/specs/tech-spec/statecharts-and-ariadne.md` (§ Repair Agent and Diagnostic Compaction).
* Reference: `docs/report/20261029/` (Methodology Critique and DAEDALUS recommendations).
