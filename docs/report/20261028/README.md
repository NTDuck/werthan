# 20261028 — Comprehensive Study of ReCodeAgent: Paper, Artifacts, Execution Traces, Drifts, and SLM Failure Modes

**Executive Summary.** This report provides an exhaustive, evidence-grounded study of **ReCodeAgent** (*"ReCodeAgent: A Multi-agent Workflow for Language-Agnostic Translation and Validation of Large-Scale Repositories"*, Ibrahimzada et al., accepted at ASE 2026, arXiv:2604.07341). Our investigation combines four primary data sources:
1. The **published paper & formal methodology** (the 4-agent feed-forward pipeline: Analyzer, Planner, Translator, Validator).
2. The **official artifact repository** (`benchmarks/assets/ReCodeAgent/` / `https://github.com/Intelligent-CAT-Lab/ReCodeAgent`), auditing its Python/Go source code, MCP language servers, FastMCP AST extractors, and Jinja2 prompt templates (`configs/prompt_templates.yaml`).
3. The **official Zenodo execution traces** (`benchmarks/assets/ReCodeAgent-zenodo/results-unpacked/results/`, DOI: 10.5281/zenodo.19214481), spanning 1.45 GB of raw trajectory logs (`orchestrator.log`), Claude Code CLI session records (`.jsonl`), token consumption logs, cost ledgers, and ablation data across 118 benchmark projects.
4. Our **in-tree replication & SLM deployment empirical campaigns**, contrasting ReCodeAgent's frontier-model execution with local execution under a 27B-class Small Language Model (`swift-1.5-qwen3.8-27b`) on a local NVIDIA RTX 3090.

---

## Key Findings at a Glance

1. **Massive Implementation Drift from the Published 4-Agent Diagram:**
   While the paper presents a clean, linear 4-agent architecture, the code actually orchestrates an unacknowledged hierarchy of up to **10 specialized subagents** spawned via Claude Code CLI (`source-translator-executor`, `test-translator-executor`, `structure-validator`, `name-validator`, `stub-todo-validator`, `rust-safety-validator`, `test-validator`, `test-executor`, `coverage-analyzer`, `test-generator-executor`).
2. **The "CRUST" Benchmark Shortcut (85% of Dataset is Scaffolding-Bound):**
   In 100 of the 118 evaluation projects (the CRUST C-to-Rust suite), ReCodeAgent does *not* translate repository structure, types, or tests from scratch. Human experts pre-wrote the complete Rust module tree, type declarations, signatures, and test binaries (`src/bin/*.rs`). The prompts explicitly instruct the LLM: *"DO NOT re-extract functions... Tests are already translated and validated by a human validator... only change the unimplemented!() blocks"*. ReCodeAgent operates on CRUST as a function-body filler, not an autonomous repository modernizer.
3. **Weak Oracles & Equivalence Hallucination:**
   Validation relies heavily on generating parallel test files (`ClassNameTest_generated`) in both source and target, declaring "functional equivalence" if both execute with exit code 0. No differential fuzzing, property testing, or state invariant verification is performed. Tests are frequently assert-free or trivial sanity checks.
4. **Frontier Model Forgiveness vs. SLM Catastrophe:**
   ReCodeAgent's experiments were powered by **Claude 3.5 Sonnet** (`claude-sonnet-4-5-20250929`) and **Claude Haiku 4.5** via AWS Bedrock, consuming up to 1.6+ million cached input tokens and hundreds of tool calls per run. Frontier models' massive 200k context windows and forgiving reasoning allowed them to tolerate 800-line Jinja prompt instructions and multi-step bash scripts. When transferred to a 27B SLM with bounded effective context, ReCodeAgent's methodology completely collapses: prompts saturate the context window, agents enter infinite tool-calling loops, and the lack of state persistence or backward topological recovery leads to futile-cap campaign death.

---

## Report Structure & Reading Order

| # | Chapter | File | Core Focus |
|---|---|---|---|
| 1 | **Paper & Theoretical Methodology** | [`01-paper-methodology.md`](01-paper-methodology.md) | Formal architecture, 4-stage pipeline, topological dependency planning, claim of language-agnosticism, and reported baseline metrics. |
| 2 | **Artifact Repository Audit** | [`02-artifact-repo-audit.md`](02-artifact-repo-audit.md) | In-depth code walkthrough of `src/agents/recodeagent/`, MCP language servers (Go), FastMCP Tree-sitter analyzers, and Jinja2 templates. |
| 3 | **Zenodo Trace Forensics** | [`03-zenodo-trace-forensics.md`](03-zenodo-trace-forensics.md) | Empirical analysis of the 1.45 GB Zenodo results: trajectory logs, session `.jsonl` transcripts, model usage, token costs ($USD), and graphectory metrics. |
| 4 | **Implementation Drifts & Benchmark Realities** | [`04-implementation-drifts.md`](04-implementation-drifts.md) | Five critical discrepancies between paper claims and repository reality: CRUST human stubs, shadow subagents, rigid name preservation, script-in-the-loop validation, and synthetic oracle equivalence. |
| 5 | **Methodological Failures & The SLM Frontier** | [`05-methodology-failures-and-slm-reality.md`](05-methodology-failures-and-slm-reality.md) | Why ReCodeAgent fails on Small Language Models (SLMs): context window saturation ("the dumb zone"), lack of intentional compaction, feed-forward fragility, and architectural prerequisites for robust SLM modernization. |

---

## Data & Artifact Locations in Repo

- **Paper Artifacts:** `benchmarks/assets/ReCodeAgent/` (Git submodule/directory)
- **Zenodo Unpacked Traces:** `benchmarks/assets/ReCodeAgent-zenodo/results-unpacked/results/` (Kept untracked / ignored via `.gitignore`)
  - Trajectories: `results/trajectories/`
  - Ablation Study: `results/ablation_study/ablation-study-effectiveness.csv`
  - Graphectory Analysis: `results/ablation_study/graphectory_analysis/`
- **Benchmark Configs:** `benchmarks/assets/configs/ReCodeAgent/` (118 YAML configs across `crust`, `oxidizer`, `alphatrans`, `skel`)
- **In-Tree Baseline Runs:** `benchmarks/results/daemon/` and `docs/report/20251025/`
