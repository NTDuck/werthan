//! Unit tests for config.

#[test]
fn test_config_scaffold_and_warden() {
    let config = ::protocol::ModernizeConfig::default();
    let schema_json = "{}";
    let val_res = ::mcp::config::validate_config(&config, schema_json);
    ::std::assert!(val_res.is_ok());

    let template = "Campaign: {{campaign_id}}\nRule: {{allowed_write_rule}}";
    let rendered = ::mcp::config::render_pi_warden(template, &config, "core");
    ::std::assert!(rendered.contains("default-campaign"));
    ::std::assert!(rendered.contains("modernized/core/"));
}
