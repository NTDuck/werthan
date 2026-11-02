//! Smoke verification entrypoint: honest inference endpoint probe shared by
//! the binary and contract tests (VAL-DAEMON-05).

/// Report produced by one smoke verification probe.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::serde::Serialize)]
pub struct SmokeReport {
    /// Inference base URL that the probe resolved.
    pub base_url: ::std::string::String,
    /// Model identifiers that the endpoint serves.
    pub model_ids: ::std::vec::Vec<::std::string::String>,
    /// First model identifier that the endpoint served.
    pub model_id: ::core::option::Option<::std::string::String>,
}
/// Binary CLI smoke path: probe the compiled default endpoint and exit nonzero on failure.
pub async fn run_smoke_cli() {
    let bench_config = super::bench_config::CampaignConfig::default();
    let environment_default = super::bench_config::default_base_url();
    let probe_res = ::tokio::task::spawn_blocking(move || {
        self::run_smoke_verification_once(&bench_config, &environment_default)
    })
    .await;
    let probe_final = match probe_res {
        ::core::result::Result::Ok(inner) => inner,
        ::core::result::Result::Err(join_error) => {
            ::std::println!("[smoke] Probe task failed: {join_error}");
            ::std::process::exit(1);
        },
    };
    match probe_final {
        ::core::result::Result::Ok(report) => {
            let shown_model = report
                .model_id
                .or_else(super::bench_config::default_model_id)
                .unwrap_or_else(|| "none".to_string());
            ::std::println!(
                "[smoke] Inference endpoint reachable. Base URL: {}. Model id: {shown_model}",
                report.base_url
            );
        },
        ::core::result::Result::Err(failure) => {
            ::std::println!("[smoke] {failure}");
            ::std::process::exit(1);
        },
    }
}
/// Run one smoke verification against the campaign-configured endpoint.
pub fn run_smoke_verification_once(
    bench_config: &super::bench_config::CampaignConfig,
    environment_default: &::core::primitive::str,
) -> ::core::result::Result<self::SmokeReport, ::std::string::String> {
    let base_url_res = self::resolve_inference_url(&bench_config.inference.base_url, environment_default);
    let base_url = base_url_res?;
    let report_res = self::smoke_endpoint_probe(&base_url);
    let report = match report_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(probe_error) => {
            return ::core::result::Result::Err(probe_error);
        },
    };
    ::core::result::Result::Ok(report)
}

/// Resolve the inference base URL honestly.
pub fn resolve_inference_url(
    configured: &::core::primitive::str,
    environment_default: &::core::primitive::str,
) -> ::core::result::Result<::std::string::String, ::std::string::String> {
    let configured_nonempty = !configured.is_empty();
    if configured_nonempty {
        return ::core::result::Result::Ok(configured.to_string());
    }
    let environment_nonempty = !environment_default.is_empty();
    if environment_nonempty {
        return ::core::result::Result::Ok(environment_default.to_string());
    }
    ::core::result::Result::Err("no inference backend URL configured".to_string())
}

/// Probe the inference endpoint behind the given base URL.
pub fn smoke_endpoint_probe(
    base_url: &::core::primitive::str,
) -> ::core::result::Result<self::SmokeReport, ::std::string::String> {
    let models_url = ::std::format!("{base_url}/models");
    let blocking_client = ::reqwest::blocking::Client::new();
    let send_res = blocking_client.get(&models_url).send();
    let response = match send_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(request_error) => {
            return ::core::result::Result::Err(::std::format!(
                "endpoint unreachable: {request_error}"
            ));
        },
    };
    let status_ok = response.status().is_success();
    if !status_ok {
        let detail = ::std::format!("endpoint returned status {}", response.status());
        return ::core::result::Result::Err(detail);
    }
    let parse_res = response.json::<self::ModelsResponse>();
    let models = match parse_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(parse_error) => {
            return ::core::result::Result::Err(::std::format!(
                "model list parse failed: {parse_error}"
            ));
        },
    };
    let model_ids = models
        .data
        .iter()
        .map(|entry| entry.id.clone())
        .collect::<::std::vec::Vec<::std::string::String>>();
    let served_any = !model_ids.is_empty();
    if !served_any {
        return ::core::result::Result::Err("endpoint served an empty model list".to_string());
    }
    let served_first = model_ids.first().cloned();
    let report = self::SmokeReport {
        base_url: { base_url.to_string() },
        model_ids,
        model_id: { served_first },
    };
    ::core::result::Result::Ok(report)
}

/// Response shape of the OpenAI-compatible /v1/models listing.
#[derive(::serde::Deserialize)]
struct ModelsResponse {
    /// Model descriptor entries served by the endpoint.
    data: ::std::vec::Vec<self::ModelEntry>,
}

/// One model descriptor from the /v1/models listing.
#[derive(::serde::Deserialize)]
struct ModelEntry {
    /// Model identifier string.
    id: ::std::string::String,
}
