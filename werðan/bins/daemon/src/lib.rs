//! Daemon library surface: campaign configuration, runner, harness seeding,
//! rule extraction, validation, and smoke verification. The binary in main.rs
//! parses arguments and dispatches into this library.

pub mod bench_batch;
pub mod bench_config;
pub mod bench_harness;
pub mod bench_llm;
pub mod bench_report;
pub mod bench_rules;
pub mod bench_runner;
pub mod bench_smoke;
pub mod bench_validation;
