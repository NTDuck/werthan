fn main() {
    init_tracing();
    ::tracing::info!("Starting Laya judge with sys1 typed decisions.");

    evaluate_transformation_gate();

    ::tracing::info!("Laya judge finished successfully.");
}

fn init_tracing() {
    ::tracing_subscriber::fmt()
        .with_max_level(::tracing::Level::INFO)
        .init();
}

fn evaluate_transformation_gate() {
    let request = create_judge_request();
    let response = execute_decision(&request);
    report_judge_verdict(&response);
}

fn create_judge_request() -> ::sys1::schema::DecisionRequest {
    let mut questions = ::serde_json::Map::new();

    let verdict_query = ::serde_json::json!({
        "type": "choice",
        "instructions": "Evaluate release gate decision.",
        "criteria": {
            "approve": "Passes all verification gates.",
            "reject": "Fails quality rubric.",
            "revise": "Requires targeted repair."
        }
    });
    questions.insert("gate_decision".to_string(), verdict_query);

    let theater_query = ::serde_json::json!({
        "type": "noul",
        "instructions": "Does diff contain coverage theater?"
    });
    questions.insert("coverage_theater".to_string(), theater_query);

    let hardening_query = ::serde_json::json!({
        "type": "score",
        "instructions": "Rate code hardening from tier 0 to tier 2.",
        "criteria": [
            "tier 0 - severe defects",
            "tier 1 - basic functionality",
            "tier 2 - hardened quality"
        ]
    });
    questions.insert("hardening_tier".to_string(), hardening_query);

    let mut state_map = ::serde_json::Map::new();
    state_map.insert("module".to_string(), ::serde_json::json!("interest_rate_calc"));
    state_map.insert("compiler_clean".to_string(), ::serde_json::json!(true));
    state_map.insert("test_pass_rate".to_string(), ::serde_json::json!(1.0));
    state_map.insert("dummy_assertions".to_string(), ::serde_json::json!(false));
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

    let has_dummy = request
        .state
        .get("dummy_assertions")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let decision_choice = if has_dummy {
        "reject"
    } else {
        "approve"
    };

    let mut choice_map = ::serde_json::Map::new();
    choice_map.insert("type".to_string(), ::serde_json::json!("choice"));
    choice_map.insert("choice".to_string(), ::serde_json::json!(decision_choice));
    choice_map.insert("confidence".to_string(), ::serde_json::json!(0.98));
    let verdict_answer = ::serde_json::Value::Object(choice_map);
    answers.insert("gate_decision".to_string(), verdict_answer);

    let theater_answer = ::serde_json::json!({
        "type": "noul",
        "probability": 0.02
    });
    answers.insert("coverage_theater".to_string(), theater_answer);

    let hardening_answer = ::serde_json::json!({
        "type": "score",
        "score": 2.0,
        "confidence": 0.96
    });
    answers.insert("hardening_tier".to_string(), hardening_answer);

    let usage = ::sys1::schema::Usage {
        input_tokens: 36,
        output_tokens: 3,
    };

    ::sys1::schema::DecisionResponse {
        model: ::sys1::models::LAYA_TYPED_DECISIONS_MODEL_ID.to_string(),
        answers,
        usage,
    }
}

fn report_judge_verdict(response: &::sys1::schema::DecisionResponse) {
    let decision_val = response.answers.get("gate_decision");
    let decision_str = decision_val
        .and_then(|d| d.get("choice"))
        .and_then(|c| c.as_str())
        .unwrap_or("unknown");

    let theater_val = response.answers.get("coverage_theater");
    let theater_prob = theater_val
        .and_then(|t| t.get("probability"))
        .and_then(|p| p.as_f64())
        .unwrap_or(0.0);

    let hardening_val = response.answers.get("hardening_tier");
    let hardening_score = hardening_val
        .and_then(|h| h.get("score"))
        .and_then(|s| s.as_f64())
        .unwrap_or(0.0);

    let model = &response.model;

    ::tracing::info!(
        model = %model,
        verdict = decision_str,
        coverage_theater_probability = theater_prob,
        hardening_tier = hardening_score,
        "Transformation gate release verdict evaluated."
    );
}
