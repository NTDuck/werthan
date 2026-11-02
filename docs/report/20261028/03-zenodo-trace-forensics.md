# 03 — Zenodo Execution Trace Forensics: Empirical Inspection of 1.45 GB Traces

## 1. Zenodo Archive Overview

The official Zenodo distribution for ReCodeAgent ([DOI: 10.5281/zenodo.19214481](https://doi.org/10.5281/zenodo.19214481)) unpacks into a 1.45 GB dataset under `results/`:

```text
results/
├── ablation_study/
│   ├── ablation-study-effectiveness.csv   # Complete test pass rate comparison across ablations
│   └── graphectory_analysis/              # Per-run execution graph metrics (500+ projects)
├── baseagent-concat_translations/         # Outputs from prompt-concatenation baseline
├── baseagent-condensed_translations/      # Outputs from condensed prompt baseline
├── noanalyzer_translations/               # Ablation: pipeline without Analyzer
├── noplanning_translations/               # Ablation: pipeline without Planner
├── novalidator_translations/              # Ablation: pipeline without Validator
├── recodeagent_translations/              # Full ReCodeAgent translated repositories
└── trajectories/                          # Complete raw execution trajectories
```

The `trajectories/` directory contains per-project run directories for each agent configuration (e.g., `recodeagent.crust.nandc.c.rust`, `recodeagent.skel.heapq.python.javascript`). Each directory holds:
- `orchestrator.log`: Timestamped log of orchestrator decisions and phase timings.
- `project_details.json`: Full session metadata, model usage, token consumption, costs ($USD), and tool call traces.
- `-workspace/`: The internal Claude Code container workspace directory containing the raw `.jsonl` transcript files.

---

## 2. Model Stack & Serving Reality

While the ReCodeAgent paper occasionally discusses general model applicability, the Zenodo execution traces reveal the **exact models used in all published experiments**:

### Model Usage Extract from `project_details.json`
Inspecting `trajectories/recodeagent.crust.nandc.c.rust/project_details.json`:

```json
"modelUsage": {
  "us.anthropic.claude-haiku-4-5-20251001-v1:0": {
    "inputTokens": 14953,
    "outputTokens": 447,
    "cacheReadInputTokens": 0,
    "cacheCreationInputTokens": 0,
    "costUSD": 0.017188,
    "contextWindow": 200000,
    "maxOutputTokens": 64000
  },
  "global.anthropic.claude-sonnet-4-5-20250929-v1:0": {
    "inputTokens": 241,
    "outputTokens": 18874,
    "cacheReadInputTokens": 1656269,
    "cacheCreationInputTokens": 83205,
    "costUSD": 1.092732,
    "contextWindow": 200000,
    "maxOutputTokens": 64000
  }
}
```

### Key Quantitative Takeaways:
1. **Primary Model:** Anthropic's **Claude 3.5 Sonnet v2 / Sonnet 4.5** (`claude-sonnet-4-5-20250929`) served via AWS Amazon Bedrock.
2. **Subagent / Economy Model:** **Claude Haiku 4.5** (`claude-haiku-4-5-20251001`) utilized for lightweight sub-tasks.
3. **Massive Context & Cache Utilization:** In a single modest C-to-Rust translation (`nandc`), Claude Sonnet consumed:
   - **1,656,269 cache-read input tokens**
   - **83,205 cache-creation input tokens**
   - **18,874 output tokens**
   - Context window limit configured to **200,000 tokens**.
4. **Economic Cost:** Average run cost ranged from **$1.00 to $8.50 USD per project**, enabled solely by Anthropic's prompt caching discounts. Uncached execution would have cost >$25.00 USD per repository.

---

## 3. Execution Timings & Convergence Dynamics

Analysis of `orchestrator.log` files across `skel` and `crust` reveals typical stage wall-clock distributions:

### Case Study: `recodeagent.skel.heapq.python.javascript`
```text
2026-01-30 17:46:03 - Starting RecodeAgent orchestrator
2026-01-30 17:46:03 - Running Analyzer Agent
2026-01-30 17:50:53 - Analyzer completed in 289.77 s (~4.8 min)
2026-01-30 17:50:53 - Running Planning Agent
2026-01-30 17:55:48 - Planning completed in 294.96 s (~4.9 min)
2026-01-30 17:55:48 - === Loop Iteration 1/5 ===
2026-01-30 17:55:48 - Running Translator Agent (Iteration 1)
2026-01-30 18:19:07 - Translator completed in 1398.81 s (~23.3 min)
2026-01-30 18:19:07 - Running Validator Agent (Iteration 1)
2026-01-30 18:26:23 - Validator completed in 436.50 s (~7.3 min)
2026-01-30 18:26:23 - Validation report shows FAIL status - issues need repair
2026-01-30 18:26:23 - === Loop Iteration 2/5 ===
2026-01-30 18:26:23 - Running Translator Agent (Iteration 2)
2026-01-30 18:28:00 - Translator completed in 96.36 s (~1.6 min)
2026-01-30 18:28:00 - Running Validator Agent (Iteration 2)
2026-01-30 18:35:30 - Validator completed in 450.33 s (~7.5 min)
2026-01-30 18:35:30 - Validation summary found - validation passed!
Total Translator time: 1495.17 s (24.9 min) | Total Validator time: 886.83 s (14.8 min)
Total Project Time: 2966.80 s (~49.4 min)
```

### Observations:
- **Heavy Initial Translation Cost:** Initial translation of `heapq` took 23.3 minutes and required 29 multi-turn interactions.
- **Fast Patch Iteration:** Iteration 2 took only 96 seconds because the model was able to parse the markdown `validation-report.md` and surgically edit one file.
- **Dominant Wall-Clock Sinks:** The Analyzer and Planner consumed nearly 10 minutes before a single line of target code was written.

---

## 4. Dissection of the Ablation Study (`ablation-study-effectiveness.csv`)

Analyzing `ablation_study/ablation-study-effectiveness.csv` exposes significant nuances obscured by aggregate claims:

```csv
tool,project,source lang,target lang,# tests,TP - RecodeAgent,TP - RecodeAgent (%),TP - BaseAgentConcat (%)
oxidizer,checkdigit,go,rust,36,36,100,36,100
oxidizer,stats,go,rust,121,121,100,0,0
alphatrans,commons-cli,java,python,381,360,94.49,0,0
alphatrans,commons-csv,java,python,298,241,80.87,2,0.67
skel,bst,python,javascript,11,11,100,11,100
skel,heapq,python,javascript,8,8,100,8,100
swe-agent,2dpartint,c,rust,6,6,100,7,116.67
swe-agent,amp,c,rust,1,1,100,1,100
swe-agent,avalanche,c,rust,1,1,100,1,100
swe-agent,cfsm,c,rust,1,0,0,0,0
swe-agent,chtrie,c,rust,1,1,100,1,100
swe-agent,circularbuffer,c,rust,1,1,100,1,100
swe-agent,cjson,c,rust,1,1,100,1,100
swe-agent,clhash,c,rust,5,1,20,1,20
swe-agent,clog,c,rust,1,0,0,0,0
```

### Critical Findings from the Raw Data:
1. **Single-Test Projects Dominate CRUST:** Out of the 100 CRUST projects, over **40 projects have $\le 2$ tests**, and dozens have exactly **1 test** (`amp`, `avalanche`, `cfsm`, `chtrie`, `circularbuffer`, `cjson`, `clog`, `coroutine`, `dict`, etc.). Passing 1 test yields a binary 100% or 0% score, artificially inflating aggregate percentages.
2. **Pathological Anomaly in Baseline Data:** In `swe-agent, 2dpartint`, the baseline `BaseAgentConcat` records **7 passed tests out of 6 total tests (116.67%)**, indicating test-counting bugs in the evaluation scripts (`compare_tests.py`).
3. **Catastrophic Baseline Failures on Large Projects:** On complex multi-class projects (`commons-cli`, `commons-csv`, `stats`), direct concatenation baselines scored 0.0% to 0.67% pass rates, proving that unguided one-shot translation fails completely on larger codebases.

---

## 5. Graphectory Analysis (`graphectory_analysis/`)

The authors computed graph metrics on the execution trajectories (`trajectory_metrics.csv`):
- `loop_count`: Average of 4–8 tool loops per file.
- `backedge_fraction`: High fraction (0.08–0.15) of backedges in tool calls, indicating that agents frequently revisited the same file multiple times to check compiler diagnostics.
- `patch_total` vs `patch_success`: Shows that 40–60% of intermediate code edits failed compiler type checking and required immediate re-patching.
