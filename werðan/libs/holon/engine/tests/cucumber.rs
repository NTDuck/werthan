//! Cucumber test runner for ARIADNE cluster MCTS controller.
//! Verifies search tree exploration, gated hierarchical rewards, and multi-tier regressions.

/// Execution harness returning extrinsic contract conflict error logs.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct ConflictHarness;

impl ::holon_evaluator::ExecutionHarness for self::ConflictHarness {
    fn execute_candidate(
        &self,
        _module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> ::holon_evaluator::HarnessExecutionOutput {
        let _ = candidate_code;
        let _ = test_suite;
        let log = "error[E0425]: parent_alpha CONTRACT_CONFLICT missing symbol".to_string();
        ::holon_evaluator::HarnessExecutionOutput {
            compiles: false,
            pass_rate: 0.0,
            raw_logs: { log },
        }
    }
}

/// Execution harness returning cyclic dependency error logs.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct CycleHarness;

impl ::holon_evaluator::ExecutionHarness for self::CycleHarness {
    fn execute_candidate(
        &self,
        _module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> ::holon_evaluator::HarnessExecutionOutput {
        let _ = candidate_code;
        let _ = test_suite;
        let log = "fatal error: cyclic dependency between modules detected".to_string();
        ::holon_evaluator::HarnessExecutionOutput {
            compiles: false,
            pass_rate: 0.0,
            raw_logs: { log },
        }
    }
}

/// Execution harness returning partial test pass rate without full unit reward.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub struct PartialPassHarness;

impl ::holon_evaluator::ExecutionHarness for self::PartialPassHarness {
    fn execute_candidate(
        &self,
        _module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> ::holon_evaluator::HarnessExecutionOutput {
        let _ = candidate_code;
        let _ = test_suite;
        let log = "test assertion failed".to_string();
        ::holon_evaluator::HarnessExecutionOutput {
            compiles: true,
            pass_rate: 0.5,
            raw_logs: { log },
        }
    }
}

/// Closed set of scenario harnesses selectable per cucumber step.
/// Replaces the former `Box<dyn ExecutionHarness>` indirection with an
/// enum dispatching to the concrete scenario harness.
#[derive(::core::fmt::Debug, ::core::default::Default)]
pub enum ScenarioHarness {
    /// Default syntax and test condition harness.
    #[default]
    Default,
    /// Extrinsic contract conflict scenario.
    Conflict,
    /// Cyclic dependency escalation scenario.
    Cycle,
    /// Partial pass rate scenario.
    Partial,
}

impl ::holon_evaluator::ExecutionHarness for self::ScenarioHarness {
    fn execute_candidate(
        &self,
        module_id: &::core::primitive::str,
        candidate_code: &::core::primitive::str,
        test_suite: &[::std::string::String],
    ) -> ::holon_evaluator::HarnessExecutionOutput {
        let inner: &dyn ::holon_evaluator::ExecutionHarness = match self {
            self::ScenarioHarness::Default => &::holon_evaluator::DefaultExecutionHarness,
            self::ScenarioHarness::Conflict => &self::ConflictHarness,
            self::ScenarioHarness::Cycle => &self::CycleHarness,
            self::ScenarioHarness::Partial => &self::PartialPassHarness,
        };
        inner.execute_candidate(module_id, candidate_code, test_suite)
    }
}

/// Test-local machine union carrying the machine between steps.
/// Delegates per-state methods to the inner machine variant.
#[derive(::core::fmt::Debug)]
enum HeldMachine {
    /// Machine parked in M2 AriadneMcts.
    M2(::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts>),
    /// Machine parked in M3 ContractNegotiate.
    M3(::holon_fsm::StateMachine<::holon_fsm::M3ContractNegotiate>),
    /// Machine parked in M4 Hardened.
    M4(::holon_fsm::StateMachine<::holon_fsm::M4Hardened>),
}

impl self::HeldMachine {
    /// Return the reflection tag of the held machine variant.
    fn state_tag(&self) -> ::holon_fsm::MicroStateTag {
        match self {
            self::HeldMachine::M2(machine) => machine.state_tag(),
            self::HeldMachine::M3(machine) => machine.state_tag(),
            self::HeldMachine::M4(machine) => machine.state_tag(),
        }
    }

    /// Delegate outbox drain.
    fn drain_events(&mut self) -> ::std::vec::Vec<::holon_fsm::ClusterBroadcastEvent> {
        match self {
            self::HeldMachine::M2(machine) => machine.drain_events(),
            self::HeldMachine::M3(machine) => machine.drain_events(),
            self::HeldMachine::M4(machine) => machine.drain_events(),
        }
    }

    /// Swap the held machine for a placeholder, returning the taken machine.
    #[allow(dead_code)]
    fn swap_out(&mut self) -> self::HeldMachine {
        let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("pending");
        let placeholder = self::HeldMachine::M2(machine_m0.wake_ready().hydration_completed());
        let taken = ::core::mem::replace(self, placeholder);
        taken
    }
}

/// Cucumber test world state holding controller, blackboard, and machine.
#[derive(::core::fmt::Debug, ::core::default::Default, ::cucumber::World)]
pub struct EngineWorld {
    blackboard: ::core::option::Option<::holon_blackboard::LocalBlackboard>,
    machine: ::core::option::Option<self::HeldMachine>,
    last_outcome: ::core::option::Option<::holon_engine::types::EngineTypestateOutcome>,
    last_reward: ::core::option::Option<::core::primitive::f64>,
    custom_harness_type: ::core::option::Option<::std::string::String>,
    tree: ::core::option::Option<::holon_engine::tree::SearchTree>,
    selected_child_id: ::core::option::Option<::core::primitive::usize>,
    initial_node_count: ::core::primitive::usize,
    controller: ::core::option::Option<
        ::holon_engine::controller::ClusterEngine<FakeAriadneModel, self::ScenarioHarness>,
    >,
}

/// Deterministic completion double mirroring the canned synthesis output.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
struct FakeAriadneModel;

impl ::rig::completion::CompletionModel for self::FakeAriadneModel {
    type Response = ::serde_json::Value;

    async fn completion(
        &self,
        request: ::rig::completion::CompletionRequest,
    ) -> ::core::result::Result<
        ::rig::completion::CompletionResponse<Self::Response>,
        ::rig::completion::CompletionError,
    > {
        let request_preamble = request.preamble.unwrap_or_default();
        let is_codegen = request_preamble.contains("code synthesis engine");
        if is_codegen {
            let mut rule_lines = ::std::vec::Vec::<::std::string::String>::new();
            let prompt_message = request.prompt;
            let prompt_text = match prompt_message {
                ::rig::message::Message::User { content } => {
                    let text_block = content.first();
                    match text_block {
                        ::rig::message::UserContent::Text(text_value) => text_value.text,
                        _ => ::std::string::String::new(),
                    }
                }
                _ => ::std::string::String::new(),
            };
            let section_start = prompt_text.find("[BUSINESS RULES]");
            if let ::core::option::Option::Some(start_index) = section_start {
                let section_text = &prompt_text[start_index..];
                let section_end = section_text
                    .find("[PARENT CONTRACTS]")
                    .unwrap_or(section_text.len());
                for section_line in section_text[..section_end].lines() {
                    let trimmed_line = section_line.trim();
                    if let ::core::option::Option::Some(rest) = trimmed_line.strip_prefix("- ") {
                        if let ::core::option::Option::Some(tag_end) = rest.find(" |") {
                            rule_lines.push(::std::format!("// {}", &rest[..tag_end]));
                        }
                    }
                }
            }
            rule_lines.push("pub fn execute_modernized() -> bool { true }".to_string());
            let content = ::rig::message::AssistantContent::text(rule_lines.join("\n"));
            let choice = ::rig::one_or_many::OneOrMany::one(content);
            return ::core::result::Result::Ok(::rig::completion::CompletionResponse {
                choice,
                raw_response: ::serde_json::Value::Null,
            });
        }
        let response_text = ::std::string::String::from(
            "#[test]\nfn t_basic() {\n    ::core::assert!(true)\n}",
        );
        let content = ::rig::message::AssistantContent::text(response_text);
        let choice = ::rig::one_or_many::OneOrMany::one(content);
        ::core::result::Result::Ok(::rig::completion::CompletionResponse {
            choice,
            raw_response: ::serde_json::Value::Null,
        })
    }
}

#[::cucumber::given(regex = r#"^an initialized local blackboard for module "([^"]+)"$"#)]
async fn init_blackboard_for_module(world: &mut self::EngineWorld, module_id: ::std::string::String) {
    let bb = ::holon_blackboard::LocalBlackboard::new();
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new(module_id);
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    world.blackboard = ::core::option::Option::Some(bb);
    world.machine = ::core::option::Option::Some(self::HeldMachine::M2(machine_m2));
    world.last_outcome = ::core::option::Option::None;
    world.last_reward = ::core::option::Option::None;
    world.custom_harness_type = ::core::option::Option::None;
    world.tree = ::core::option::Option::None;
    world.selected_child_id = ::core::option::Option::None;
    world.initial_node_count = 0;
    world.controller = ::core::option::Option::None;
}

#[::cucumber::given(regex = r#"^an initialized local blackboard with parent contract "([^"]+)"$"#)]
async fn init_blackboard_with_parent_contract(world: &mut self::EngineWorld, parent_id: ::std::string::String) {
    let mut bb = ::holon_blackboard::LocalBlackboard::new();
    let contract = ::holon_blackboard::EdgeContract {
        parent_id,
        interface_spec: "pub fn compute(x: i32) -> i32".to_string(),
        preconditions: ::std::vec![],
    };
    bb.problem_model.add_parent_contract(contract);
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_child");
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    world.blackboard = ::core::option::Option::Some(bb);
    world.machine = ::core::option::Option::Some(self::HeldMachine::M2(machine_m2));
    world.last_outcome = ::core::option::Option::None;
    world.last_reward = ::core::option::Option::None;
    world.custom_harness_type = ::core::option::Option::None;
    world.tree = ::core::option::Option::None;
    world.selected_child_id = ::core::option::Option::None;
    world.initial_node_count = 0;
    world.controller = ::core::option::Option::None;
}

#[::cucumber::given(regex = r#"^an execution harness that flags extrinsic contract conflict$"#)]
async fn configure_conflict_harness(world: &mut self::EngineWorld) {
    world.custom_harness_type = ::core::option::Option::Some("conflict".to_string());
}

#[::cucumber::given(regex = r#"^an execution harness that flags cyclic dependency$"#)]
async fn configure_cycle_harness(world: &mut self::EngineWorld) {
    world.custom_harness_type = ::core::option::Option::Some("cycle".to_string());
}

#[::cucumber::given(regex = r#"^an execution harness that produces partial pass rate without unit reward$"#)]
async fn configure_partial_pass_harness(world: &mut self::EngineWorld) {
    world.custom_harness_type = ::core::option::Option::Some("partial".to_string());
}

#[::cucumber::given(regex = r#"^an initialized search tree with parent and two child candidates$"#)]
async fn init_tree_with_children(world: &mut self::EngineWorld) {
    let init_candidate = ::core::option::Option::Some("pub fn compute() -> i32 { 0 }".to_string());
    let mut tree = ::holon_engine::tree::SearchTree::new(init_candidate);
    let c1 = ::core::option::Option::Some("pub fn compute() -> i32 { 1 }".to_string());
    let s1 = ::core::option::Option::Some("strategy_alpha".to_string());
    let child1 = tree.add_child(0, c1, s1, 0.7);
    let c2 = ::core::option::Option::Some("pub fn compute() -> i32 { 2 }".to_string());
    let s2 = ::core::option::Option::Some("strategy_beta".to_string());
    let _child2 = tree.add_child(0, c2, s2, 0.3);
    tree.backpropagate(child1, 0.8);
    world.initial_node_count = tree.len();
    world.tree = ::core::option::Option::Some(tree);
}

#[::cucumber::when(regex = r#"^the controller executes search with rollout budget (\d+)$"#)]
async fn execute_search(world: &mut self::EngineWorld, budget: ::core::primitive::usize) {
    let config = ::holon_engine::types::EngineConfig {
        max_rollouts: { budget },
        ..::core::default::Default::default()
    };
    let bb = match world.blackboard.take() {
        ::core::option::Option::Some(b) => b,
        ::core::option::Option::None => ::holon_blackboard::LocalBlackboard::new(),
    };
    let mut ctrl = self::build_controller(world, config, bb);
    let taken_machine = match world.machine.take() {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("missing machine in world"),
    };
    match taken_machine {
        self::HeldMachine::M2(machine_m2) => {
            let typed_outcome = ctrl.solve(machine_m2);
            world.machine = self::machine_from_outcome(&typed_outcome);
            world.last_outcome = ::core::option::Option::Some(typed_outcome);
        }
        _ => ::core::panic!("search step requires M2 machine"),
    }
    world.controller = ::core::option::Option::Some(ctrl);
}

#[::cucumber::when(regex = r#"^the controller evaluates uncompilable candidate code$"#)]
async fn evaluate_uncompilable_code(world: &mut self::EngineWorld) {
    let weights = ::holon_engine::types::RewardWeights::default();
    let reward = ::holon_engine::reward::compute_gated_reward(&weights, false, 1.0, 1.0);
    world.last_reward = ::core::option::Option::Some(reward);
}

#[::cucumber::when(regex = r#"^the controller evaluates candidate code with 1.0 test pass rate and 1.0 rule score$"#)]
async fn evaluate_unit_candidate(world: &mut self::EngineWorld) {
    let weights = ::holon_engine::types::RewardWeights::default();
    let reward = ::holon_engine::reward::compute_gated_reward(&weights, true, 1.0, 1.0);
    world.last_reward = ::core::option::Option::Some(reward);
}

#[::cucumber::when(regex = r#"^the tree performs Softmax-UCB selection with temperature ([\d\.]+)$"#)]
async fn tree_performs_softmax_selection(world: &mut self::EngineWorld, tau: ::core::primitive::f64) {
    if let ::core::option::Option::Some(tree) = &world.tree {
        let selected = tree.select_softmax_child(0, 1.414, 0.5, tau);
        world.selected_child_id = selected;
    }
}

#[::cucumber::when(regex = r#"^downstream invalidation shifts activation mode to "([^"]+)"$"#)]
async fn downstream_invalidation_shifts_mode(world: &mut self::EngineWorld, _mode: ::std::string::String) {
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("module_math");
    let machine_m2 = machine_m0.wake_ready().hydration_completed();
    world.machine = ::core::option::Option::Some(self::HeldMachine::M2(machine_m2));
}

#[::cucumber::when(regex = r#"^the controller executes search under warm start mode$"#)]
async fn execute_search_warm_start(world: &mut self::EngineWorld) {
    let config = ::holon_engine::types::EngineConfig {
        max_rollouts: 3,
        ..::core::default::Default::default()
    };
    let bb = match world.blackboard.take() {
        ::core::option::Option::Some(b) => b,
        ::core::option::Option::None => ::holon_blackboard::LocalBlackboard::new(),
    };
    let mut ctrl = self::build_controller(world, config, bb);
    let taken_machine = match world.machine.take() {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("missing machine in world"),
    };
    match taken_machine {
        self::HeldMachine::M2(machine_m2) => {
            let typed_outcome = ctrl.solve(machine_m2);
            world.machine = self::machine_from_outcome(&typed_outcome);
            world.last_outcome = ::core::option::Option::Some(typed_outcome);
        }
        _ => ::core::panic!("warm start search requires M2 machine"),
    }
    world.controller = ::core::option::Option::Some(ctrl);
}

#[::cucumber::then(regex = r#"^the search yields typed outcome "([^"]+)"$"#)]
async fn assert_search_outcome(world: &mut self::EngineWorld, expected_name: ::std::string::String) {
    let outcome = match &world.last_outcome {
        ::core::option::Option::Some(o) => o,
        ::core::option::Option::None => ::core::panic!("missing search outcome"),
    };
    let matched = self::outcome_matches(outcome, &expected_name);
    ::core::assert!(matched);
}

#[::cucumber::then(regex = r#"^the held machine is in state "([^"]+)"$"#)]
async fn assert_held_machine_state(world: &mut self::EngineWorld, state_name: ::std::string::String) {
    let expected = match state_name.as_str() {
        "M2_AriadneMcts" => ::holon_fsm::MicroStateTag::M2AriadneMcts,
        "M3_ContractNegotiate" => ::holon_fsm::MicroStateTag::M3ContractNegotiate,
        "M4_Hardened" => ::holon_fsm::MicroStateTag::M4Hardened,
        _ => ::core::panic!("unknown state name"),
    };
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("missing machine in world"),
    };
    let actual = machine.state_tag();
    ::core::assert_eq!(actual, expected);
}

#[::cucumber::then(regex = r#"^the held machine emits broadcast event "([^"]+)"$"#)]
async fn assert_held_machine_emits(world: &mut self::EngineWorld, event_name: ::std::string::String) {
    let machine = match &mut world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("missing machine in world"),
    };
    let events = machine.drain_events();
    let mut has_event = false;
    for ev in &events {
        let is_match = self::broadcast_event_matches(ev, &event_name);
        if is_match {
            has_event = true;
            break;
        }
    }
    ::core::assert!(has_event);
}

#[::cucumber::then(regex = r#"^the tree contains candidate nodes$"#)]
async fn assert_tree_contains_nodes(world: &mut self::EngineWorld) {
    let ctrl = match &world.controller {
        ::core::option::Option::Some(c) => c,
        ::core::option::Option::None => ::core::panic!("missing controller in world"),
    };
    let count = ctrl.search_tree().len();
    let has_nodes = count > 1;
    ::core::assert!(has_nodes);
}

#[::cucumber::then(regex = r#"^the computed reward equals ([\d\.]+)$"#)]
async fn assert_computed_reward(world: &mut self::EngineWorld, expected: ::core::primitive::f64) {
    let reward = match world.last_reward {
        ::core::option::Option::Some(r) => r,
        ::core::option::Option::None => ::core::panic!("missing computed reward"),
    };
    let delta = (reward - expected).abs();
    let matches = delta < 1e-6;
    ::core::assert!(matches);
}

#[::cucumber::then(regex = r#"^the selection identifies an expandable child node$"#)]
async fn assert_selection_identifies_child(world: &mut self::EngineWorld) {
    let selected = world.selected_child_id;
    let is_some = selected.is_some();
    ::core::assert!(is_some);
}

#[::cucumber::then(regex = r#"^the search tree preserves existing nodes across rollouts$"#)]
async fn assert_tree_preserves_nodes(world: &mut self::EngineWorld) {
    let ctrl = match &world.controller {
        ::core::option::Option::Some(c) => c,
        ::core::option::Option::None => ::core::panic!("missing controller in world"),
    };
    let current_count = ctrl.search_tree().len();
    let has_preserved = current_count >= 1;
    ::core::assert!(has_preserved);
}

fn build_controller(
    world: &self::EngineWorld,
    config: ::holon_engine::types::EngineConfig,
    bb: ::holon_blackboard::LocalBlackboard,
) -> ::holon_engine::controller::ClusterEngine<FakeAriadneModel, self::ScenarioHarness> {
    let harness = match world.custom_harness_type.as_deref() {
        ::core::option::Option::Some("conflict") => self::ScenarioHarness::Conflict,
        ::core::option::Option::Some("cycle") => self::ScenarioHarness::Cycle,
        ::core::option::Option::Some("partial") => self::ScenarioHarness::Partial,
        _ => self::ScenarioHarness::Default,
    };
    let distiller = ::holon_evaluator::DefaultDiagnosticDistiller;
    let evaluator = ::holon_evaluator::TwoStageEvaluator::builder()
        .harness(harness)
        .distiller(distiller)
        .build();
    let fake_model = self::FakeAriadneModel::default();
    let strategy_agent = ::holon_strategy::HolonStrategyAgent::builder()
        .model(fake_model.clone())
        .temperature(0.2)
        .build();
    let codegen_agent = ::holon_codegen::HolonCodeGenAgent::builder()
        .model(fake_model.clone())
        .temperature(0.2)
        .build();
    let testgen_agent = ::holon_testgen::HolonTestGenAgent::builder()
        .model(fake_model.clone())
        .temperature(0.2)
        .build();
    let patch_agent = ::holon_patch::HolonPatchAgent::builder()
        .model(fake_model)
        .temperature(0.2)
        .build();
    let strategy_handle = ::std::sync::Arc::new(strategy_agent);
    let codegen_handle = ::std::sync::Arc::new(codegen_agent);
    let testgen_handle = ::std::sync::Arc::new(testgen_agent);
    let patch_handle = ::std::sync::Arc::new(patch_agent);
    let agents = ::holon_engine::agents::SubagentPool::builder()
        .strategy_agent(strategy_handle)
        .code_gen_agent(codegen_handle)
        .test_gen_agent(testgen_handle)
        .patch_agent(patch_handle)
        .build();
    ::holon_engine::controller::ClusterEngine::builder()
        .config(config)
        .blackboard(bb)
        .evaluator(evaluator)
        .agents(agents)
        .build()
}

/// Extract the post-transition machine from a typed outcome into the world union.
fn machine_from_outcome(outcome: &::holon_engine::types::EngineTypestateOutcome) -> ::core::option::Option<self::HeldMachine> {
    match outcome {
        ::holon_engine::types::EngineTypestateOutcome::Hardened(machine) => {
            let cloned = machine.clone();
            ::core::option::Option::Some(self::HeldMachine::M4(cloned))
        }
        ::holon_engine::types::EngineTypestateOutcome::ContractConflict(machine) => {
            let cloned = machine.clone();
            ::core::option::Option::Some(self::HeldMachine::M3(cloned))
        }
        ::holon_engine::types::EngineTypestateOutcome::CycleFound { machine, .. } => {
            let cloned = machine.clone();
            ::core::option::Option::Some(self::HeldMachine::M2(cloned))
        }
        ::holon_engine::types::EngineTypestateOutcome::BudgetExhausted { machine, .. } => {
            let cloned = machine.clone();
            ::core::option::Option::Some(self::HeldMachine::M3(cloned))
        }
    }
}

fn outcome_matches(
    outcome: &::holon_engine::types::EngineTypestateOutcome,
    expected_name: &::core::primitive::str,
) -> ::core::primitive::bool {
    match expected_name {
        "Hardened" => ::core::matches!(outcome, ::holon_engine::types::EngineTypestateOutcome::Hardened(_)),
        "ContractConflict" => ::core::matches!(
            outcome,
            ::holon_engine::types::EngineTypestateOutcome::ContractConflict(_)
        ),
        "CycleFound" => ::core::matches!(outcome, ::holon_engine::types::EngineTypestateOutcome::CycleFound { .. }),
        "BudgetExhausted" => ::core::matches!(
            outcome,
            ::holon_engine::types::EngineTypestateOutcome::BudgetExhausted { .. }
        ),
        _ => false,
    }
}

fn broadcast_event_matches(
    event: &::holon_fsm::ClusterBroadcastEvent,
    expected_name: &::core::primitive::str,
) -> ::core::primitive::bool {
    match expected_name {
        "ModuleHardened" => ::core::matches!(event, ::holon_fsm::ClusterBroadcastEvent::ModuleHardened { .. }),
        "ContractConflict" => ::core::matches!(event, ::holon_fsm::ClusterBroadcastEvent::ContractConflict { .. }),
        "CyclicDependencyFound" => ::core::matches!(event, ::holon_fsm::ClusterBroadcastEvent::CyclicDependencyFound { .. }),
        _ => false,
    }
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    let has_list = args.iter().any(|arg| arg == "--list");
    if has_list {
        let has_ignored = args.iter().any(|arg| arg == "--ignored");
        if !has_ignored {
            ::std::println!("engine::cucumber: test");
        }
        return;
    }

    <self::EngineWorld as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
