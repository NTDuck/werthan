---
status: "accepted"
date: 2026-10-23
decision-makers: ["NTDuck", "omp"]
consulted: ["werðan architecture team"]
informed: ["werðan development team"]
---

# Daemon-Owned OpenAI-Compatible Transport with Hard Deadlines

## Context and Problem Statement

[ADR-0006](0006-de-qwen-inference-transport.md) routed all LLM transport through Rig's stock OpenAI provider (`rig::providers::openai::Client::from_url`). Benchmark campaigns against the local ninfer endpoint then wedged: the server completed generation (journalctl showed continuous decode throughput, and independent probes with the exact request shapes completed in 63–292 s), while the daemon's socket showed the request ACKed but zero response bytes arriving. The configured wall-clock timeout never fired because Rig 0.9.1 builds its own `reqwest::Client` internally with no timeout knob and no client-injection point; the adapter's `tokio::time::timeout` wrap around the provider future never progressed to expiry while the body read parked.

How does all LLM transport stay OpenAI-compatible and vendor-neutral while carrying a wall-clock deadline that cannot be bypassed by the HTTP stack?

## Decision Drivers

* Hard deadline: a stalled response body read must surface as an error after the configured `inference.timeout_secs`, not park forever.
* Vendor neutrality: any OpenAI-compatible endpoint remains a drop-in target.
* Minimal dependency surface: no fork of Rig and no new transport trait layer.

## Considered Options

* Keep the stock Rig provider and wrap futures in `tokio::time::timeout` (ADR-0006 state).
* Fork or patch Rig 0.9.1 to accept a custom `reqwest::Client`.
* Have `bench_llm::OpenAiCompatModel` own the HTTP exchange directly over a daemon-built `reqwest::Client` with client-level deadlines, reusing Rig's message types for request serialization and implementing the `CompletionModel` trait for prompt-layer compatibility.

## Decision Outcome

Chosen option: "daemon-owned reqwest transport inside `bench_llm::OpenAiCompatModel`", because it enforces the deadline where it cannot be bypassed (hyper client-level `timeout` + `connect_timeout`) with no fork, and keeps the Rig `CompletionModel`/`Agent` prompt plumbing intact.

* The client builder applies `.timeout(inference_timeout_secs)` and `.connect_timeout(inference_timeout_secs)`; every exchange reports `inference timeout after N seconds` on expiry via `reqwest::Error::is_timeout`.
* Request bodies serialize from Rig's `CompletionRequest` (preamble → system, documents → user turns, history and prompt through Rig's OpenAI wire conversion); the payload carries explicit `temperature` and `max_tokens` resolved from the defaults.
* Responses parse the OpenAI chat-completions shape with the extra `reasoning_content` channel that OpenAI-compatible reasoning backends fill; text extraction falls back through `content` → `reasoning_content` → `refusal`, then the strict `usable_answer` gate (fenced code or post-`</think` conclusion) applies as before.
* Non-2xx responses surface the status plus a bounded body snippet; parse failures surface verbatim.
* `raw_response` is the parsed JSON value, consumed by honesty auditing.

### Consequences

* Good, because stalled body reads now return `inference timeout after N seconds` instead of parking a tokio worker forever.
* Good, because reasoning-only responses (empty `content`, populated `reasoning_content`) still reach the strict answer gate instead of being misread as "no text channel".
* Bad, because Rig provider upgrades no longer update the wire path automatically; wire-shape changes require adapter updates.
* Neutral, because the config chain (`migration.yml` → env → compiled default) and `INFERENCE_API_KEY` handling from ADR-0006 are unchanged.

### Confirmation

`cargo test -p daemon` covers the answer-gate and config-contract paths. Live verification: a 40960-token non-streaming request through the new transport completed in 281 s (`finish=stop`), and parallel campaigns sustain server-side decode batching (`running 3`) with per-campaign watchdogs over socket counters.

## More Information

This decision supersedes [ADR-0006](0006-de-qwen-inference-transport.md) for the transport mechanism only; its vendor-neutrality and config-chain drivers carry forward unchanged. Evidence for the defect: campaign stalls with `bytes_sent` static and `bytes_received` absent for 24+ minutes at a 600 s budget, while the server decoded continuously.
