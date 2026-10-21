use ::blackboard::CondensedDAG;
use ::blackboard::FaultCause;
use ::blackboard::FaultRecord;
use ::blackboard::GlobalBlackboard;
use ::blackboard::HardenedArtifact;
use ::blackboard::InterfaceSpec;
use ::blackboard::InventoryManifest;
use ::blackboard::JournalDelta;
use ::blackboard::LocalBlackboard;
use ::blackboard::ProblemModelSlice;
use ::blackboard::RuleCard;
use ::blackboard::RuleTag;
use ::blackboard::StrategyHypothesis;

#[tokio::test]
async fn test_global_blackboard_journal_and_delta_commit() {
    let temp_dir = std::env::temp_dir();
    let journal_path = temp_dir.join(format!(
        "werthan_bb_journal_test_{}.redb",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));

    // 1. Initialize GlobalBlackboard
    let bb = GlobalBlackboard::open(&journal_path).expect("Failed to open GlobalBlackboard");

    // 2. Commit Inventory
    let inv = InventoryManifest::builder()
        .build_harness("cargo check".to_string())
        .build();
    let seq1 = bb
        .commit_delta(JournalDelta::SetInventory(inv))
        .await
        .expect("Failed to commit inventory");
    assert_eq!(seq1, 1);

    // 3. Commit DAG
    let mut dag = CondensedDAG::new();
    dag.add_edge("v_core", "v_api");
    let seq2 = bb
        .commit_delta(JournalDelta::UpdateDAG(dag))
        .await
        .expect("Failed to commit DAG");
    assert_eq!(seq2, 2);

    // 4. Commit Rule Card
    let card = RuleCard::builder()
        .id("RC-101")
        .title("Account Balance Invariant")
        .given("Account is active")
        .when("Withdrawal requested")
        .then("Balance must not drop below zero")
        .citation("accounts.c:42")
        .tag(RuleTag::Observed)
        .build();
    let seq3 = bb
        .commit_delta(JournalDelta::AddRuleCard {
            vertex_id: "v_core".to_string(),
            card,
        })
        .await
        .expect("Failed to commit rule card");
    assert_eq!(seq3, 3);

    // 5. Commit Contract Spec
    let spec = InterfaceSpec::builder()
        .provider_vertex("v_core")
        .consumer_vertex("v_api")
        .interface_stub("pub trait CoreService { fn ping(&self); }".to_string())
        .build();
    let seq4 = bb
        .commit_delta(JournalDelta::SetContract {
            edge_key: ("v_core".to_string(), "v_api".to_string()),
            spec,
        })
        .await
        .expect("Failed to commit contract");
    assert_eq!(seq4, 4);

    // 6. Commit Hardened Artifact
    let artifact = HardenedArtifact::builder()
        .vertex_id("v_core")
        .target_code("pub struct CoreServiceImpl;".to_string())
        .characterization_tests("#[test] fn test_core() {}".to_string())
        .reward(1.0)
        .hash_stamp("sha256_mock_abc123".to_string())
        .build();
    let seq5 = bb
        .commit_delta(JournalDelta::CommitArtifact {
            vertex_id: "v_core".to_string(),
            artifact,
        })
        .await
        .expect("Failed to commit artifact");
    assert_eq!(seq5, 5);

    drop(bb);
    // 7. Re-open and verify replay
    let bb_reopened = GlobalBlackboard::open(&journal_path).expect("Failed to re-open GlobalBlackboard");
    let data = bb_reopened.read_data().await;

    assert_eq!(data.dag.vertices.len(), 2);
    assert!(data.dag.edges.contains(&("v_core".to_string(), "v_api".to_string())));
    assert_eq!(data.rule_cards.get("v_core").unwrap().len(), 1);
    assert_eq!(data.rule_cards.get("v_core").unwrap()[0].id, "RC-101");
    assert_eq!(data.artifacts.get("v_core").unwrap().reward, 1.0);

    let _ = std::fs::remove_file(&journal_path);
}

#[test]
fn test_local_blackboard_4_slice_and_read_isolation() {
    let pm = ProblemModelSlice::builder()
        .vertex_id("v_payment")
        .legacy_ast_slices(vec!["void process_payment() {}".to_string()])
        .build();

    let mut local_bb = LocalBlackboard::new(pm);

    // Populate strategy slice
    local_bb.strategies.hypotheses.push(
        StrategyHypothesis::builder()
            .id("H1_Functional")
            .description("Pure functional pipeline")
            .prior_score(0.85)
            .build(),
    );

    // Populate patch record slice
    local_bb.patch_records.faults.push(
        FaultRecord::builder()
            .rollout_index(3)
            .localized_location("payment.rs:18")
            .diagnostic_summary("mismatched types: expected u64, found i32")
            .cause(FaultCause::Intrinsic)
            .build(),
    );

    // 1. Adversarial Test Generator view: can read PM, but compile-time isolated from candidate code
    let test_view = local_bb.test_generator_view();
    assert_eq!(test_view.problem_model.vertex_id, "v_payment");
    assert_eq!(test_view.problem_model.legacy_ast_slices.len(), 1);

    // 2. Code Generator view: reads PM and strategies
    let code_view = local_bb.code_generator_view();
    assert_eq!(code_view.strategies.hypotheses.len(), 1);
    assert_eq!(code_view.strategies.hypotheses[0].id, "H1_Functional");

    // 3. Patch Repairer view: reads PM and fault records
    let patch_view = local_bb.patch_repair_view();
    assert_eq!(patch_view.patch_records.faults.len(), 1);
    assert_eq!(patch_view.patch_records.faults[0].cause, FaultCause::Intrinsic);
}
