//! Cucumber test runner for ARIADNE cluster micro-FSM.
//! Verifies typestate transitions, event emissions, and warm start regressions
//! through the per-state methods of the strongly-typed state machine.

/// Cucumber test world state holding the active typestate machine.
/// Each scenario restarts from a fresh M0 machine. The world stores the
/// latest machine in a test-local enum over the six typestates, with
/// per-state transition methods matching on the held variant.
#[derive(::core::fmt::Debug, ::core::default::Default, ::cucumber::World)]
pub struct FsmWorld {
    machine: ::core::option::Option<self::HeldMachine>,
    drained_events: ::std::vec::Vec<::holon_fsm::ClusterBroadcastEvent>,
}

/// Test-local machine union carrying the machine between steps.
/// Delegates per-state methods to the inner machine variant.
#[derive(::core::fmt::Debug)]
enum HeldMachine {
    /// Machine parked in M0 Blocked.
    M0(::holon_fsm::StateMachine<::holon_fsm::M0Blocked>),
    /// Machine parked in M1 ContextHydration.
    M1(::holon_fsm::StateMachine<::holon_fsm::M1ContextHydration>),
    /// Machine parked in M2 AriadneMcts.
    M2(::holon_fsm::StateMachine<::holon_fsm::M2AriadneMcts>),
    /// Machine parked in M3 ContractNegotiate.
    M3(::holon_fsm::StateMachine<::holon_fsm::M3ContractNegotiate>),
    /// Machine parked in M4 Hardened.
    M4(::holon_fsm::StateMachine<::holon_fsm::M4Hardened>),
    /// Machine parked in M5 Invalidated.
    M5(::holon_fsm::StateMachine<::holon_fsm::M5Invalidated>),
}

impl self::HeldMachine {
    /// Return the reflection tag of the held machine variant.
    fn state_tag(&self) -> ::holon_fsm::MicroStateTag {
        match self {
            self::HeldMachine::M0(machine) => machine.state_tag(),
            self::HeldMachine::M1(machine) => machine.state_tag(),
            self::HeldMachine::M2(machine) => machine.state_tag(),
            self::HeldMachine::M3(machine) => machine.state_tag(),
            self::HeldMachine::M4(machine) => machine.state_tag(),
            self::HeldMachine::M5(machine) => machine.state_tag(),
        }
    }

    /// Delegate blocked status predicate.
    fn is_blocked(&self) -> ::core::primitive::bool {
        match self {
            self::HeldMachine::M0(machine) => machine.is_blocked(),
            self::HeldMachine::M1(machine) => machine.is_blocked(),
            self::HeldMachine::M2(machine) => machine.is_blocked(),
            self::HeldMachine::M3(machine) => machine.is_blocked(),
            self::HeldMachine::M4(machine) => machine.is_blocked(),
            self::HeldMachine::M5(machine) => machine.is_blocked(),
        }
    }

    /// Delegate hardened status predicate.
    fn is_hardened(&self) -> ::core::primitive::bool {
        match self {
            self::HeldMachine::M0(machine) => machine.is_hardened(),
            self::HeldMachine::M1(machine) => machine.is_hardened(),
            self::HeldMachine::M2(machine) => machine.is_hardened(),
            self::HeldMachine::M3(machine) => machine.is_hardened(),
            self::HeldMachine::M4(machine) => machine.is_hardened(),
            self::HeldMachine::M5(machine) => machine.is_hardened(),
        }
    }

    /// Delegate active status predicate.
    fn is_active(&self) -> ::core::primitive::bool {
        match self {
            self::HeldMachine::M0(machine) => machine.is_active(),
            self::HeldMachine::M1(machine) => machine.is_active(),
            self::HeldMachine::M2(machine) => machine.is_active(),
            self::HeldMachine::M3(machine) => machine.is_active(),
            self::HeldMachine::M4(machine) => machine.is_active(),
            self::HeldMachine::M5(machine) => machine.is_active(),
        }
    }

    /// Delegate activation mode getter.
    fn activation_mode(&self) -> ::holon_fsm::ActivationMode {
        match self {
            self::HeldMachine::M0(machine) => machine.activation_mode(),
            self::HeldMachine::M1(machine) => machine.activation_mode(),
            self::HeldMachine::M2(machine) => machine.activation_mode(),
            self::HeldMachine::M3(machine) => machine.activation_mode(),
            self::HeldMachine::M4(machine) => machine.activation_mode(),
            self::HeldMachine::M5(machine) => machine.activation_mode(),
        }
    }

    /// Delegate history getter.
    fn history(&self) -> &::holon_fsm::FsmHistory {
        match self {
            self::HeldMachine::M0(machine) => machine.history(),
            self::HeldMachine::M1(machine) => machine.history(),
            self::HeldMachine::M2(machine) => machine.history(),
            self::HeldMachine::M3(machine) => machine.history(),
            self::HeldMachine::M4(machine) => machine.history(),
            self::HeldMachine::M5(machine) => machine.history(),
        }
    }

    /// Delegate outbox drain, appending into the world event log.
    fn drain_events_into(&mut self, drained: &mut ::std::vec::Vec<::holon_fsm::ClusterBroadcastEvent>) {
        match self {
            self::HeldMachine::M0(machine) => drained.append(&mut machine.drain_events()),
            self::HeldMachine::M1(machine) => drained.append(&mut machine.drain_events()),
            self::HeldMachine::M2(machine) => drained.append(&mut machine.drain_events()),
            self::HeldMachine::M3(machine) => drained.append(&mut machine.drain_events()),
            self::HeldMachine::M4(machine) => drained.append(&mut machine.drain_events()),
            self::HeldMachine::M5(machine) => drained.append(&mut machine.drain_events()),
        }
    }

    /// Apply the wake ready transition from M0 into M1.
    fn wake_ready(&mut self) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M0(machine) => {
                *self = self::HeldMachine::M1(machine.wake_ready());
            }
            _ => ::core::panic!("wake ready step requires M0 machine"),
        }
    }

    /// Apply the hydration completed transition from M1 into M2.
    fn hydration_completed(&mut self) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M1(machine) => {
                *self = self::HeldMachine::M2(machine.hydration_completed());
            }
            _ => ::core::panic!("hydration step requires M1 machine"),
        }
    }

    /// Apply the hardened transition from M2 into M4.
    fn into_hardened(
        &mut self,
        target_code: ::std::string::String,
        test_suite: ::std::vec::Vec<::std::string::String>,
        reward: ::core::primitive::f64,
    ) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M2(machine) => {
                *self = self::HeldMachine::M4(machine.into_hardened(target_code, test_suite, reward));
            }
            _ => ::core::panic!("hardened step requires M2 machine"),
        }
    }

    /// Apply the contract negotiate transition from M2 into M3.
    fn into_contract_negotiate(
        &mut self,
        parent_id: ::std::string::String,
        delta_spec: ::std::string::String,
        proof_trace: ::std::string::String,
    ) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M2(machine) => {
                let machine_m3 = machine.into_contract_negotiate(parent_id, delta_spec, proof_trace);
                *self = self::HeldMachine::M3(machine_m3);
            }
            _ => ::core::panic!("conflict step requires M2 machine"),
        }
    }

    /// Apply the conflict dispatched transition from M3 into M0.
    fn conflict_dispatched(&mut self) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M3(machine) => {
                *self = self::HeldMachine::M0(machine.conflict_dispatched());
            }
            _ => ::core::panic!("dispatch step requires M3 machine"),
        }
    }

    /// Apply the invalidated transition from M4 into M5.
    fn into_invalidated(&mut self, delta_spec: ::std::string::String) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M4(machine) => {
                *self = self::HeldMachine::M5(machine.into_invalidated(delta_spec));
            }
            _ => ::core::panic!("invalidation step requires M4 machine"),
        }
    }

    /// Apply the warm start ready transition from M5 into M2.
    fn warm_start_ready(&mut self) {
        let taken = self.swap_out();
        match taken {
            self::HeldMachine::M5(machine) => {
                *self = self::HeldMachine::M2(machine.warm_start_ready());
            }
            _ => ::core::panic!("warm start step requires M5 machine"),
        }
    }

    /// Swap the held machine for a placeholder, returning the taken machine.
    fn swap_out(&mut self) -> self::HeldMachine {
        let placeholder_machine = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new("pending");
        let placeholder = self::HeldMachine::M0(placeholder_machine);
        let taken = ::core::mem::replace(self, placeholder);
        taken
    }

    fn emit_cycle_regression(
        &mut self,
        cycle_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
        proof_trace: ::std::string::String,
    ) {
        if let self::HeldMachine::M2(machine) = self {
            machine.emit_cycle_regression(cycle_edges, proof_trace);
        }
    }
}

#[::cucumber::given(regex = r#"^a new module cluster for "([^"]+)"$"#)]
async fn create_cluster(world: &mut self::FsmWorld, module_id: ::std::string::String) {
    let machine_m0 = ::holon_fsm::StateMachine::<::holon_fsm::M0Blocked>::new(module_id);
    world.machine = ::core::option::Option::Some(self::HeldMachine::M0(machine_m0));
    world.drained_events.clear();
}

#[::cucumber::when(regex = r#"^the cluster advances through wake ready$"#)]
async fn advance_wake_ready(world: &mut self::FsmWorld) {
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.wake_ready();
    }
}

#[::cucumber::when(regex = r#"^the cluster advances through hydration completed$"#)]
async fn advance_hydration_completed(world: &mut self::FsmWorld) {
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.hydration_completed();
    }
}

#[::cucumber::when(regex = r#"^a candidate achieves unit reward with code "([^"]+)"$"#)]
async fn candidate_achieves_unit_reward(world: &mut self::FsmWorld, target_code: ::std::string::String) {
    let test_suite = ::std::vec!["assert!(true)".to_string()];
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.into_hardened(target_code, test_suite, 1.0);
    }
}

#[::cucumber::when(regex = r#"^the evaluator flags extrinsic contract conflict on parent "([^"]+)"$"#)]
async fn extrinsic_contract_conflict_detected(world: &mut self::FsmWorld, parent_id: ::std::string::String) {
    let delta_spec = "requires param timeout: u64".to_string();
    let proof_trace = "error[E0425]: missing param in call".to_string();
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.into_contract_negotiate(parent_id, delta_spec, proof_trace);
    }
}

#[::cucumber::when(regex = r#"^downstream invalidation occurs with delta "([^"]+)"$"#)]
async fn downstream_invalidation_occurs(world: &mut self::FsmWorld, delta_spec: ::std::string::String) {
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.into_invalidated(delta_spec);
    }
}

#[::cucumber::when(regex = r#"^the evaluator flags cyclic dependency involving parent "([^"]+)"$"#)]
async fn cyclic_dependency_detected(world: &mut self::FsmWorld, parent_id: ::std::string::String) {
    let module_id = match &world.machine {
        ::core::option::Option::Some(machine) => match machine {
            self::HeldMachine::M2(m) => m.module_id().to_string(),
            _ => ::core::panic!("cycle step requires M2 machine"),
        },
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    let edge_pair = (parent_id, module_id);
    let edges = ::std::vec![edge_pair];
    let proof = "cyclic dependency graph loop".to_string();
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.emit_cycle_regression(edges, proof);
    }
}

#[::cucumber::when(regex = r#"^the cluster advances through conflict dispatched$"#)]
async fn advance_conflict_dispatched(world: &mut self::FsmWorld) {
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.conflict_dispatched();
    }
}

#[::cucumber::when(regex = r#"^the cluster advances through warm start ready$"#)]
async fn advance_warm_start_ready(world: &mut self::FsmWorld) {
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.warm_start_ready();
    }
}

#[::cucumber::then(regex = r#"^the micro-state is "([^"]+)"$"#)]
async fn assert_micro_state(world: &mut self::FsmWorld, state_name: ::std::string::String) {
    let expected = self::parse_state_name(&state_name);
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    let actual = machine.state_tag();
    ::core::assert_eq!(actual, expected);
}

#[::cucumber::then(regex = r#"^the held machine type is "([^"]+)"$"#)]
async fn assert_held_machine_type(world: &mut self::FsmWorld, state_name: ::std::string::String) {
    let expected = self::parse_state_name(&state_name);
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    let matches = self::held_variant_matches(machine, &expected);
    ::core::assert!(matches);
}

#[::cucumber::then(regex = r#"^the module holds blocked status$"#)]
async fn assert_module_blocked(world: &mut self::FsmWorld) {
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    ::core::assert!(machine.is_blocked());
}

#[::cucumber::then(regex = r#"^the module holds hardened status$"#)]
async fn assert_module_hardened(world: &mut self::FsmWorld) {
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    ::core::assert!(machine.is_hardened());
}

#[::cucumber::then(regex = r#"^the module holds active status$"#)]
async fn assert_module_active(world: &mut self::FsmWorld) {
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    ::core::assert!(machine.is_active());
}

#[::cucumber::then(regex = r#"^the outbox contains broadcast event "([^"]+)"$"#)]
async fn assert_outbox_event(world: &mut self::FsmWorld, broadcast_name: ::std::string::String) {
    if let ::core::option::Option::Some(machine) = &mut world.machine {
        machine.drain_events_into(&mut world.drained_events);
    }
    let mut found = false;
    for event in &world.drained_events {
        let matched = self::broadcast_matches(event, &broadcast_name);
        if matched {
            found = true;
            break;
        }
    }
    ::core::assert!(found);
}

#[::cucumber::then(regex = r#"^the activation mode is "([^"]+)"$"#)]
async fn assert_activation_mode(world: &mut self::FsmWorld, mode_name: ::std::string::String) {
    let expected = self::parse_activation_mode(&mode_name);
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    let actual = machine.activation_mode();
    ::core::assert_eq!(actual, expected);
}

#[::cucumber::then(regex = r#"^the warm start count is (\d+)$"#)]
async fn assert_warm_start_count(world: &mut self::FsmWorld, expected_count: ::core::primitive::usize) {
    let machine = match &world.machine {
        ::core::option::Option::Some(machine) => machine,
        ::core::option::Option::None => ::core::panic!("cluster machine not instantiated"),
    };
    let count = machine.history().warm_start_count;
    ::core::assert_eq!(count, expected_count);
}

fn parse_state_name(name: &::core::primitive::str) -> ::holon_fsm::MicroStateTag {
    match name {
        "M0_Blocked" => ::holon_fsm::MicroStateTag::M0Blocked,
        "M1_ContextHydration" => ::holon_fsm::MicroStateTag::M1ContextHydration,
        "M2_AriadneMcts" => ::holon_fsm::MicroStateTag::M2AriadneMcts,
        "M3_ContractNegotiate" => ::holon_fsm::MicroStateTag::M3ContractNegotiate,
        "M4_Hardened" => ::holon_fsm::MicroStateTag::M4Hardened,
        "M5_Invalidated" => ::holon_fsm::MicroStateTag::M5Invalidated,
        _ => ::core::panic!("unknown state name"),
    }
}

fn held_variant_matches(
    machine: &self::HeldMachine,
    expected: &::holon_fsm::MicroStateTag,
) -> ::core::primitive::bool {
    let is_match = match machine {
        self::HeldMachine::M0(_) => ::core::matches!(expected, ::holon_fsm::MicroStateTag::M0Blocked),
        self::HeldMachine::M1(_) => ::core::matches!(expected, ::holon_fsm::MicroStateTag::M1ContextHydration),
        self::HeldMachine::M2(_) => ::core::matches!(expected, ::holon_fsm::MicroStateTag::M2AriadneMcts),
        self::HeldMachine::M3(_) => ::core::matches!(expected, ::holon_fsm::MicroStateTag::M3ContractNegotiate),
        self::HeldMachine::M4(_) => ::core::matches!(expected, ::holon_fsm::MicroStateTag::M4Hardened),
        self::HeldMachine::M5(_) => ::core::matches!(expected, ::holon_fsm::MicroStateTag::M5Invalidated),
    };
    is_match
}

fn broadcast_matches(
    event: &::holon_fsm::ClusterBroadcastEvent,
    name: &::core::primitive::str,
) -> ::core::primitive::bool {
    match name {
        "ModuleHardened" => ::core::matches!(event, ::holon_fsm::ClusterBroadcastEvent::ModuleHardened { .. }),
        "ContractConflict" => ::core::matches!(event, ::holon_fsm::ClusterBroadcastEvent::ContractConflict { .. }),
        "CyclicDependencyFound" => {
            ::core::matches!(event, ::holon_fsm::ClusterBroadcastEvent::CyclicDependencyFound { .. })
        }
        _ => false,
    }
}

fn parse_activation_mode(name: &::core::primitive::str) -> ::holon_fsm::ActivationMode {
    match name {
        "Initial" => ::holon_fsm::ActivationMode::Initial,
        "WarmStartRegression" => ::holon_fsm::ActivationMode::WarmStartRegression,
        _ => ::core::panic!("unknown activation mode"),
    }
}

#[::tokio::main]
async fn main() {
    let args = ::std::env::args().collect::<::std::vec::Vec<::std::string::String>>();
    let has_list = args.iter().any(|arg| arg == "--list");
    if has_list {
        let has_ignored = args.iter().any(|arg| arg == "--ignored");
        if !has_ignored {
            ::std::println!("fsm::cucumber: test");
        }
        return;
    }

    <self::FsmWorld as ::cucumber::World>::cucumber()
        .with_default_cli()
        .fail_on_skipped()
        .run_and_exit("tests/features")
        .await;
}
