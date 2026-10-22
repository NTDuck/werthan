---
status: "accepted"
date: 2026-10-22
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# OpenAI-Compatible Inference Transport via Rig Stock Provider

## Context and Problem Statement

The werðan workspace carries hard-coded Qwen-era client paths, a bespoke inference client, and runtime dotenvy loading inside the daemon. This couples LLM transport to a specific vendor and scatters endpoint configuration across the codebase. The daemon must talk to any OpenAI-compatible endpoint (ninfer, vLLM, llama.cpp server) through one predictable configuration chain.

How does all LLM transport route through a single, vendor-neutral provider with a deterministic config chain?

## Decision Drivers

* Vendor neutrality: any OpenAI-compatible endpoint is a drop-in target; no Qwen-specific code paths remain in the transport layer.
* Deterministic configuration: endpoint and model resolve through one chain — `migration.yml` (`inference.base_url`, `inference.model`), env fallback (`BASE_URL`, `MODEL_ID`), compiled default.
* Minimal surface: reuse Rig's stock provider machinery instead of maintaining a bespoke client.
* Secret hygiene: `INFERENCE_API_KEY` resolves from the environment, never from a committed file.

## Considered Options

* Retain the bespoke Qwen client with hard-coded endpoints and runtime dotenvy loading.
* Wrap a custom transport trait over Rig to allow pluggable wire protocols.
* Use Rig's stock OpenAI-compatible provider (`rig::providers::openai::Client::from_url`) exclusively, with the daemon's `bench_llm::OpenAiCompatModel` driving it from `inference.base_url` / `inference.model` and `INFERENCE_API_KEY`.

## Decision Outcome

Chosen option: "Rig's stock OpenAI-compatible provider (`rig::providers::openai::Client::from_url`) as the sole LLM transport", because it satisfies vendor neutrality, collapses the config chain into one predictable resolution order, and deletes the bespoke client surface.

* The daemon builds `bench_llm::OpenAiCompatModel` from the campaign's `inference.base_url` / `inference.model` with `INFERENCE_API_KEY` as the env key.
* Config resolution: `migration.yml` → env fallback (`BASE_URL`, `MODEL_ID`) → compiled default.
* Hard-coded Qwen-era client paths and runtime dotenvy loading are gone from the transport layer.

### Consequences

* Good, because any OpenAI-compatible endpoint serves werðan with zero transport-layer changes.
* Good, because `bench_llm` is verifiable without a Qwen dependency ("qwen-free verified").
* Bad, because non-OpenAI wire protocols (native Anthropic, Bedrock) need a new ADR before use.
* Neutral, because dotenvy remains available to other tooling; only the daemon transport path drops it.

### Confirmation

`cargo test --workspace` exercises `bench_llm::OpenAiCompatModel` construction from each config-chain source. A campaign run against the local ninfer endpoint with only `migration.yml` set confirms the resolution order. Grep gates: no `dotenvy` call sites in `werðan/bins/daemon/src/`, no hard-coded model ids in the transport layer.

## More Information

Evidence: commits 2219d45, 1d4da72, 4c8d267. Session evidence: "De-Qwen daemon refactor is complete: bench_llm now uses rig's stock OpenAI provider... qwen-free verified".

This decision rests on the methodology pinning in [ADR-0005](0005-methodology-source-of-truth.md); the inference transport serves the pipeline that methodology defines. [ADR-0007](0007-compile-time-env-ingestion.md) supplies the compile-time env ingestion that backs the fallback tier of the config chain.
