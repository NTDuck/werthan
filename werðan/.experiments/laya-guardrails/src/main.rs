fn main() {
    init_tracing();
    ::tracing::info!("Starting Laya tool guardrails with sys1 typed decisions.");

    let clean_cmd = "cargo check --workspace";
    evaluate_command_guardrail(clean_cmd);

    let dangerous_cmd = "rm -rf /";
    evaluate_command_guardrail(dangerous_cmd);

    ::tracing::info!("Laya tool guardrails finished successfully.");
}

fn init_tracing() {
    ::tracing_subscriber::fmt()
        .with_max_level(::tracing::Level::INFO)
        .init();
}

fn evaluate_command_guardrail(command: &::core::primitive::str) {
    let request = create_guardrail_request(command);
    let response = execute_decision(&request);
    report_guardrail_verdict(command, &response);
}

fn create_guardrail_request(command: &::core::primitive::str) -> ::sys1::schema::DecisionRequest {
    let mut questions = ::serde_json::Map::new();

    let action_query = ::serde_json::json!({
        "type": "choice",
        "instructions": "Select safety action for tool command",
        "criteria": {
            "allow": "Safe command execution in workspace",
            "block": "Destructive or dangerous operation"
        }
    });
    questions.insert("action".to_string(), action_query);

    let blast_query = ::serde_json::json!({
        "type": "score",
        "instructions": "Score blast radius of tool command",
        "criteria": [
            "isolated workspace",
            "repository scoped",
            "system wide host"
        ]
    });
    questions.insert("blast_radius".to_string(), blast_query);

    let destruct_query = ::serde_json::json!({
        "type": "noul",
        "instructions": "Is this tool call destructive to the system?"
    });
    questions.insert("is_destructive".to_string(), destruct_query);
    let mut state_map = ::serde_json::Map::new();
    state_map.insert("tool".to_string(), ::serde_json::json!("bash"));
    state_map.insert("command".to_string(), ::serde_json::json!(command));
    let state = ::serde_json::Value::Object(state_map);

    let model_id = ::sys1::models::LAYA_TYPED_DECISIONS_MODEL_ID.to_string();
    ::sys1::schema::DecisionRequest {
        model: ::core::option::Option::Some(model_id),
        state,
        questions,
    }
}

fn execute_decision(request: &::sys1::schema::DecisionRequest) -> ::sys1::schema::DecisionResponse {
    let mut answers = ::serde_json::Map::new();
    let state_cmd = request.state.get("command").and_then(|v| v.as_str()).unwrap_or("");
    let is_dangerous = state_cmd.contains("rm -rf") || state_cmd.contains("/etc/shadow");

    if is_dangerous {
        answers.insert(
            "action".to_string(),
            ::serde_json::json!({
                "type": "choice",
                "choice": "block",
                "confidence": 0.99
            }),
        );
        answers.insert(
            "blast_radius".to_string(),
            ::serde_json::json!({
                "type": "score",
                "score": 2.0,
                "confidence": 0.95
            }),
        );
        answers.insert(
            "is_destructive".to_string(),
            ::serde_json::json!({
                "type": "noul",
                "probability": 0.99
            }),
        );
    } else {
        answers.insert(
            "action".to_string(),
            ::serde_json::json!({
                "type": "choice",
                "choice": "allow",
                "confidence": 0.98
            }),
        );
        answers.insert(
            "blast_radius".to_string(),
            ::serde_json::json!({
                "type": "score",
                "score": 0.0,
                "confidence": 0.99
            }),
        );
        answers.insert(
            "is_destructive".to_string(),
            ::serde_json::json!({
                "type": "noul",
                "probability": 0.01
            }),
        );
    }

    let usage = ::sys1::schema::Usage {
        input_tokens: 32,
        output_tokens: 3,
    };
    ::sys1::schema::DecisionResponse {
        model: ::sys1::models::LAYA_TYPED_DECISIONS_MODEL_ID.to_string(),
        answers,
        usage,
    }
}

fn report_guardrail_verdict(
    command: &::core::primitive::str,
    response: &::sys1::schema::DecisionResponse,
) {
    let action_val = response.answers.get("action");
    let action_str = action_val.and_then(|a| a.get("choice")).and_then(|c| c.as_str()).unwrap_or("unknown");
    let is_allowed = action_str == "allow";
    let model = &response.model;

    if is_allowed {
        ::tracing::info!(
            model = %model,
            command = command,
            action = action_str,
            "Tool call approved by guardrail."
        );
    } else {
        ::tracing::warn!(
            model = %model,
            command = command,
            action = action_str,
            "Tool call blocked by guardrail."
        );
    }
}
