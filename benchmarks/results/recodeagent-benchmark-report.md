# ReCodeAgent Benchmark Report

Run date: 2026-10-18 · Runner: `werðan/daemon --benchmark-all` · Dataset: `benchmarks/assets/ReCodeAgent/data/tool_projects`
Inference: `qwen3.8-27b` via ashalliants-df2 (RTX 3090, real HTTP completion, 118/118 real)

## Overall

| Metric | Value |
|---|---|
| Total projects benchmarked | 20 |
| Overall compilation success rate | 10.00% |
| Overall mean test pass rate | 5.00% |
| Mean wall time per project | 12230.60 ms |
| Total wall time | 244.62 s |

## Per-Category Breakdown

| Category | Projects | Compilation rate | Mean test pass rate | Mean wall time (ms) | Passed |
|---|---|---|---|---|---|
| crust | 1 | 100.00% | 0.00% | 11147.00 | 0 |
| oxidizer | 7 | 14.29% | 14.29% | 14613.57 | 1 |
| alphatrans | 4 | 0.00% | 0.00% | 12792.75 | 0 |
| skel | 8 | 0.00% | 0.00% | 9999.88 | 0 |

## Per-Project Results

| Project | Category | Wall time (ms) | Compilation | Test pass rate | Receipts | Inference | Status |
|---|---|---|---|---|---|---|---|
| commons-cli | alphatrans | 11204 | ❌ | 0.0% | 4/6 | real | FAILED |
| commons-csv | alphatrans | 14005 | ❌ | 0.0% | 4/6 | real | FAILED |
| commons-fileupload | alphatrans | 12890 | ❌ | 0.0% | 4/6 | real | FAILED |
| commons-validator | alphatrans | 13072 | ❌ | 0.0% | 4/6 | real | FAILED |
| bst | skel | 5955 | ❌ | 0.0% | 4/6 | real | FAILED |
| colorsys | skel | 7291 | ❌ | 0.0% | 4/6 | real | FAILED |
| heapq | skel | 7353 | ❌ | 0.0% | 4/6 | real | FAILED |
| html | skel | 16574 | ❌ | 0.0% | 4/6 | real | FAILED |
| mathgen | skel | 12415 | ❌ | 0.0% | 4/6 | real | FAILED |
| rbt | skel | 6528 | ❌ | 0.0% | 4/6 | real | FAILED |
| strsim | skel | 12275 | ❌ | 0.0% | 4/6 | real | FAILED |
| toml | skel | 11608 | ❌ | 0.0% | 4/6 | real | FAILED |
| checkdigit | oxidizer | 15200 | ❌ | 0.0% | 4/6 | real | FAILED |
| go-edlib | oxidizer | 15027 | ❌ | 0.0% | 4/6 | real | FAILED |
| gohistogram | oxidizer | 15448 | ❌ | 0.0% | 4/6 | real | FAILED |
| gonameparts | oxidizer | 17498 | ❌ | 0.0% | 4/6 | real | FAILED |
| mock | oxidizer | 9642 | ✅ | 100.0% | 6/6 | real | PASSED |
| stats | oxidizer | 15390 | ❌ | 0.0% | 4/6 | real | FAILED |
| textrank | oxidizer | 14090 | ❌ | 0.0% | 4/6 | real | FAILED |
| 2dpartint | crust | 11147 | ✅ | 0.0% | 4/6 | real | FAILED |
