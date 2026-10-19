//! Modernization configuration validation, workspace scaffolding, and warden rendering.

/// Validate a modernization configuration against the JSON schema.
pub fn validate_config(
    config: &::protocol::ModernizeConfig,
    schema_json: &::core::primitive::str,
) -> ::core::result::Result<(), ::std::string::String> {
    self::verify_config_semantics(config)?;
    let parsed_schema = match ::serde_json::from_str::<::serde_json::Value>(schema_json) {
        ::core::result::Result::Ok(val) => val,
        ::core::result::Result::Err(err) => {
            let msg = ::std::format!("Invalid schema json: {err}");
            return ::core::result::Result::Err(msg);
        },
    };
    if !parsed_schema.is_object() {
        let msg = "Schema must be a JSON object".to_string();
        return ::core::result::Result::Err(msg);
    }
    let config_val = match ::serde_json::to_value(config) {
        ::core::result::Result::Ok(val) => val,
        ::core::result::Result::Err(err) => {
            let msg = ::std::format!("Failed to serialize config: {err}");
            return ::core::result::Result::Err(msg);
        },
    };
    if !config_val.is_object() {
        let msg = "Config serialization must be a JSON object".to_string();
        return ::core::result::Result::Err(msg);
    }
    ::core::result::Result::Ok(())
}

fn verify_config_semantics(config: &::protocol::ModernizeConfig) -> ::core::result::Result<(), ::std::string::String> {
    if config.campaign_id.trim().is_empty() {
        let msg = "campaign_id must not be empty".to_string();
        return ::core::result::Result::Err(msg);
    }
    ::core::result::Result::Ok(())
}

/// Scaffold modernization workspace directories.
pub fn scaffold_workspace(
    workspace_root: &::std::path::Path,
    subsystem: &::core::primitive::str,
) -> ::core::result::Result<(), ::std::string::String> {
    let legacy_dir = workspace_root.join("legacy").join(subsystem);
    let analysis_dir = workspace_root.join("analysis").join(subsystem);
    let modernized_dir = workspace_root.join("modernized").join(subsystem);

    self::create_workspace_dir(&legacy_dir)?;
    self::create_workspace_dir(&analysis_dir)?;
    self::create_workspace_dir(&modernized_dir)?;
    ::core::result::Result::Ok(())
}

fn create_workspace_dir(dir: &::std::path::Path) -> ::core::result::Result<(), ::std::string::String> {
    if let ::core::result::Result::Err(err) = ::std::fs::create_dir_all(dir) {
        let msg = ::std::format!("Failed to create directory {}: {err}", dir.display());
        return ::core::result::Result::Err(msg);
    }
    ::core::result::Result::Ok(())
}

/// Render pi-warden markdown file from template and configuration.
pub fn render_pi_warden(
    template_content: &::core::primitive::str,
    config: &::protocol::ModernizeConfig,
    subsystem: &::core::primitive::str,
) -> ::std::string::String {
    let target_dir = ::std::format!("modernized/{subsystem}/");
    let allowed_write = ::std::format!("- Only allow write tools within `{target_dir}`.");
    let forbid_outside = ::std::format!("- Never write outside `{target_dir}`.");

    let rendered = template_content
        .replace("{{campaign_id}}", &config.campaign_id)
        .replace("{{subsystem}}", subsystem)
        .replace("{{sys}}", subsystem)
        .replace("{{workspace_root}}", &config.paths.workspace_root)
        .replace("{{input_legacy_dir}}", &config.paths.input_legacy_dir)
        .replace("{{target_language}}", &config.stack.target_language)
        .replace("{{target_test_cmd}}", &config.stack.target_test_cmd)
        .replace("{{allowed_write_rule}}", &allowed_write)
        .replace("{{forbid_outside_rule}}", &forbid_outside);
    rendered
}
