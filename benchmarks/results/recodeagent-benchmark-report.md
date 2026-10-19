# ReCodeAgent Benchmark Report

Run date: 2026-10-18 · Runner: `werðan/daemon --benchmark-all` · Dataset: `benchmarks/assets/ReCodeAgent/data/tool_projects`
Inference: `qwen3.8-27b` via ashalliants-df2 (RTX 3090, real HTTP completion, 118/118 real)

## Overall

| Metric | Value |
|---|---|
| Total projects benchmarked | 12 |
| Overall compilation success rate | 50.00% |
| Overall mean test pass rate | 25.00% |
| Mean wall time per project | 261588.75 ms |
| Total wall time | 3139.07 s |

## Per-Category Breakdown

| Category | Projects | Compilation rate | Mean test pass rate | Mean wall time (ms) | Passed |
|---|---|---|---|---|---|
| crust | 3 | 100.00% | 0.00% | 318067.00 | 0 |
| oxidizer | 3 | 0.00% | 0.00% | 237922.33 | 0 |
| alphatrans | 3 | 0.00% | 0.00% | 244979.00 | 0 |
| skel | 3 | 100.00% | 100.00% | 245386.67 | 3 |

## Per-Project Results

| Project | Category | Wall time (ms) | Compilation | Test pass rate | Receipts | Inference | Status |
|---|---|---|---|---|---|---|---|
| commons-cli | alphatrans | 62578 | ❌ | 0.0% | 4/6 | real | FAILED |
| 2dpartint | crust | 326807 | ✅ | 0.0% | 4/6 | simulated | FAILED |
| checkdigit | oxidizer | 403486 | ❌ | 0.0% | 4/6 | real | FAILED |
| bst | skel | 195347 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| commons-csv | alphatrans | 573649 | ❌ | 0.0% | 4/6 | simulated | FAILED |
| 42-kocaeli-printf | crust | 352329 | ✅ | 0.0% | 4/6 | real | FAILED |
| go-edlib | oxidizer | 233003 | ❌ | 0.0% | 4/6 | real | FAILED |
| colorsys | skel | 210350 | ✅ | 100.0% | 6/6 | simulated | PASSED |
| commons-fileupload | alphatrans | 98710 | ❌ | 0.0% | 4/6 | real | FAILED |
| aes128-simd | crust | 275065 | ✅ | 0.0% | 4/6 | real | FAILED |
| gohistogram | oxidizer | 77278 | ❌ | 0.0% | 4/6 | real | FAILED |
| heapq | skel | 330463 | ✅ | 100.0% | 6/6 | simulated | PASSED |
