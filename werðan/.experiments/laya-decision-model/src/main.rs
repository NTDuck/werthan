fn main() {
    init_tracing();
    ::tracing::info!("Starting Laya decision model with sys1 typed decisions.");

    evaluate_modernization_decision();

    ::tracing::info!("Laya decision model finished successfully.");
}

fn init_tracing() {
    ::tracing_subscriber::fmt()
        .with_max_level(::tracing::Level::INFO)
        .init();
}

fn evaluate_modernization_decision() {
    let request = create_modernization_request();
    let response = execute_decision(&request);
    report_decision_verdict(&response);
}

fn create_modernization_request() -> ::sys1::schema::DecisionRequest {
    let mut questions = ::serde_json::Map::new();
    let profile_query = ::serde_json::json!({
        "type": "choice",
        "instructions": "Route modernization execution profile.",
        "criteria": {
            "fast_moe_ast": "Low complexity AST.",
            "deep_chain_of_thought": "High complexity deep tree.",
            "adversarial_audit": "Security audit."
        }
    });
    questions.insert("execution_profile".to_string(), profile_query);

    let blocker_query = ::serde_json::json!({
        "type": "choice",
        "instructions": "Classify compiler failure trace.",
        "criteria": {
            "intrinsic_bug": "Local implementation bug.",
            "extrinsic_contract_conflict": "Parent interface mismatch."
        }
    });
    questions.insert("blocker_classification".to_string(), blocker_query);

    let stuck_query = ::serde_json::json!({
        "type": "noul",
        "instructions": "Is the repair agent trapped in a loop?"
    });
    questions.insert("is_stuck_loop".to_string(), stuck_query);

    let mut state_map = ::serde_json::Map::new();
    state_map.insert("module".to_string(), ::serde_json::json!("payment_processor.cbl"));
    state_map.insert("ast_depth".to_string(), ::serde_json::json!(26));
    state_map.insert("loc_ratio".to_string(), ::serde_json::json!(3.4));
    let trace_msg = "error[E0308]: mismatched types in call to compute_interest";
    state_map.insert("compiler_trace".to_string(), ::serde_json::json!(trace_msg));
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
    let depth = request.state.get("ast_depth").and_then(|v| v.as_u64()).unwrap_or(0);
    let chosen_profile = if depth > 20 {
        "deep_chain_of_thought"
    } else {
        "fast_moe_ast"
    };

    let mut choice_map = ::serde_json::Map::new();
    choice_map.insert("type".to_string(), ::serde_json::json!("choice"));
    choice_map.insert("choice".to_string(), ::serde_json::json!(chosen_profile));
    choice_map.insert("confidence".to_string(), ::serde_json::json!(0.96));
    let profile_answer = ::serde_json::Value::Object(choice_map);
    answers.insert("execution_profile".to_string(), profile_answer);

    let blocker_answer = ::serde_json::json!({
        "type": "choice",
        "choice": "extrinsic_contract_conflict",
        "confidence": 0.94
    });
    answers.insert("blocker_classification".to_string(), blocker_answer);

    let stuck_answer = ::serde_json::json!({
        "type": "noul",
        "probability": 0.05
    });
    answers.insert("is_stuck_loop".to_string(), stuck_answer);

    let usage = ::sys1::schema::Usage {
        input_tokens: 48,
        output_tokens: 3,
    };

    ::sys1::schema::DecisionResponse {
        model: ::sys1::models::LAYA_TYPED_DECISIONS_MODEL_ID.to_string(),
        answers,
        usage,
    }
}

fn report_decision_verdict(response: &::sys1::schema::DecisionResponse) {
    let profile_val = response.answers.get("execution_profile");
    let profile_str = profile_val
        .and_then(|p| p.get("choice"))
        .and_then(|c| c.as_str())
        .unwrap_or("unknown");

    let blocker_val = response.answers.get("blocker_classification");
    let blocker_str = blocker_val
        .and_then(|b| b.get("choice"))
        .and_then(|c| c.as_str())
        .unwrap_or("unknown");

    let stuck_val = response.answers.get("is_stuck_loop");
    let stuck_prob = stuck_val
        .and_then(|s| s.get("probability"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0);

    let model = &response.model;

    ::tracing::info!(
        model = %model,
        profile = profile_str,
        blocker = blocker_str,
        stuck_probability = stuck_prob,
        "Modernization pipeline decision evaluated."
    );
}
