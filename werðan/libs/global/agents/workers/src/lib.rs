//! Stateless macro-phase worker agents and activation envelopes for the
//! global orchestrator per METHODOLOGY.md Algorithm 1 substate S_m.2.
//!
//! Workers observe the global blackboard in read-only mode. Each execution
//! receives one hydrated activation envelope and returns one typed
//! [`PhaseWorkOutput`] for the orchestrator engine. The engine commits that
//! output to the board.
//! The deterministic defaults in this crate implement every discovery,
//! specification, and verification role (S0 through S6 except the S5 module
//! solver) without an LLM. Empirical harness execution stays with the daemon
//! bench harness. `global-critics` wires Rig-backed LLM crews onto the same
//! [`PhaseWorker`] contract.

/// Read-only source extensions recognized by the deterministic assessor.
const KNOWN_SOURCE_EXTENSIONS: &[&::core::primitive::str] = &[
    ".rs", ".c", ".h", ".cpp", ".hpp", ".cc", ".py", ".java", ".ts", ".js", ".go", ".md", ".toml",
    ".yml", ".yaml", ".json", ".lock",
];

/// Token scanned inside hardened target code by the deterministic security audit.
const UNSAFE_TOKEN: &::core::primitive::str = "unsafe ";

/// Typed activation envelope hydrated by the deterministic context assembler
/// per METHODOLOGY.md Definition 4.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct TaskActivationEnvelope {
    /// Activation mode selecting the worker prompt template.
    pub mode: ::holon_fsm::ActivationMode,
    /// One-hop blackboard slice holding module identifiers, parent identifiers, rule card summaries, and contract preconditions.
    pub slice_summary: ::std::string::String,
    /// Failure critique or contract conflict directive injected on retry.
    pub delta_directive: ::core::option::Option<::std::string::String>,
    /// Read-only tool names bound to the worker.
    pub bound_tools: ::std::vec::Vec<::std::string::String>,
}

/// One stubbed contract output paired with its directed edge.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct EdgeStub {
    /// Parent vertex of the dependency edge.
    pub parent_id: ::std::string::String,
    /// Child vertex of the dependency edge.
    pub child_id: ::std::string::String,
    /// Stubbed interface contract for the edge.
    pub contract: ::holon_blackboard::EdgeContract,
}

/// Risk report for one inventory shard.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct AssessmentReport {
    /// Shard subject the report covers.
    pub shard_subject: ::std::string::String,
    /// Count of inventoried files covered by the shard.
    pub file_count: ::core::primitive::usize,
    /// Dialect and debt risk notes raised by the assessor.
    pub risk_notes: ::std::vec::Vec<::std::string::String>,
}

/// Reduced risk matrix produced by the assessment synthesizer.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct RiskMatrix {
    /// Total inventoried file count across shards.
    pub file_count: ::core::primitive::usize,
    /// Merged risk notes across shard reports.
    pub risk_notes: ::std::vec::Vec<::std::string::String>,
}

/// Per-vertex integration and security verdict.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub struct VerifyVerdict {
    /// Vertex the verdict covers.
    pub module_id: ::std::string::String,
    /// Report whether the artifact carries a nonempty test suite.
    pub tests_pass: ::core::primitive::bool,
    /// Static security findings raised against the target code.
    pub security_findings: ::std::vec::Vec<::std::string::String>,
}

/// Typed output of one stateless worker execution, committed by the engine.
#[derive(::core::clone::Clone, ::core::fmt::Debug)]
pub enum PhaseWorkOutput {
    /// S0 preflight probe report.
    Preflight {
        /// Report whether the harness and inventory pass the intent gate.
        intent_valid: ::core::primitive::bool,
        /// Verified harness configuration observed by the probe.
        harness: ::global_blackboard::HarnessConfig,
    },
    /// S1 per-shard assessment reports plus the synthesized risk matrix.
    Assessment {
        /// Per-shard assessor reports.
        reports: ::std::vec::Vec<self::AssessmentReport>,
        /// Reduced risk matrix across shards.
        synthesized: self::RiskMatrix,
    },
    /// S2 hidden coupling edges discovered on the whole-repository shard.
    Coupling {
        /// Newly discovered dependency edges as parent and child pairs.
        discovered_edges: ::std::vec::Vec<(::std::string::String, ::std::string::String)>,
    },
    /// S3 rule cards with their owning vertex plus the cross-module link count.
    Rules {
        /// Rule cards paired with their owning vertex identifier.
        cards: ::std::vec::Vec<(::std::string::String, ::holon_blackboard::BusinessRuleCard)>,
        /// Count of cross-vertex link copies emitted by the linker.
        linked_count: ::core::primitive::usize,
    },
    /// S4 target idioms plus stubbed edge contracts.
    Contracts {
        /// Target idiom summary emitted by the macro architect.
        idioms: ::std::string::String,
        /// Stubbed contracts paired with their edges.
        stubs: ::std::vec::Vec<self::EdgeStub>,
    },
    /// S6 per-vertex verification verdicts plus the shard parity bit.
    Verification {
        /// Per-vertex integration and security verdicts.
        verdicts: ::std::vec::Vec<self::VerifyVerdict>,
        /// Report whether every verdict in this output greenlights release.
        sign_off: ::core::primitive::bool,
    },
    /// Phase carries no worker workload (S5 dispatches module solvers).
    Skipped,
}

/// Stateless macro-phase worker contract per Algorithm 1 substate S_m.2.
pub trait PhaseWorker: ::core::fmt::Debug + ::core::marker::Send + ::core::marker::Sync {
    /// Execute one phase shard and return the typed output for the engine to commit.
    fn execute(
        &self,
        phase: ::global_fsm::MacroState,
        shard_subject: &::core::primitive::str,
        envelope: &self::TaskActivationEnvelope,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> self::PhaseWorkOutput;
}

/// Deterministic zero-token worker crew covering every discovery and
/// verification role.
///
/// The defaults derive board-truth outputs without spawning processes or
/// touching the filesystem. A failed gate shows up as a finding in the
/// output or as an unchanged board. The engine structural guard then
/// observes the real gap instead of a fabricated pass.
#[derive(::core::clone::Clone, ::core::fmt::Debug, ::core::default::Default)]
pub struct DeterministicWorkers;

impl self::PhaseWorker for self::DeterministicWorkers {
    /// Route one phase shard to its role implementation.
    fn execute(
        &self,
        phase: ::global_fsm::MacroState,
        shard_subject: &::core::primitive::str,
        envelope: &self::TaskActivationEnvelope,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> self::PhaseWorkOutput {
        let _ = envelope;
        match phase {
            ::global_fsm::MacroState::S0IntentPreflight => self.probe_preflight(board),
            ::global_fsm::MacroState::S1Assess => {
                let report = self.assess_shard(shard_subject, board);
                let reports = ::std::vec![::core::clone::Clone::clone(&report)];
                let synthesized = self.synthesize_assessment(&reports);
                self::PhaseWorkOutput::Assessment {
                    reports,
                    synthesized,
                }
            }
            ::global_fsm::MacroState::S2MapDag => self.scan_coupling(),
            ::global_fsm::MacroState::S3ExtractRules => {
                let mut cards = ::std::vec::Vec::new();
                for vertex in &board.dag.vertices {
                    let subject_match = vertex == shard_subject;
                    if !subject_match {
                        continue;
                    }
                    let existing = board.rules.get(vertex);
                    let already_mined = ::core::matches!(
                        existing,
                        ::core::option::Option::Some(entries) if !entries.is_empty()
                    );
                    if already_mined {
                        continue;
                    }
                    let vertex_owned = ::core::clone::Clone::clone(vertex);
                    cards.push((vertex_owned, self.mine_rules(vertex)));
                }
                let (linked, linked_count) = self.link_cross_module_rules(board, &cards);
                cards.extend(linked);
                self::PhaseWorkOutput::Rules {
                    cards,
                    linked_count,
                }
            }
            ::global_fsm::MacroState::S4BriefContracts => {
                let idioms = self.architect_idioms(board);
                let mut stubs = ::std::vec::Vec::new();
                let edge_parts = shard_subject.split("->").collect::<::std::vec::Vec<_>>();
                if let [parent_id, child_id] = edge_parts.as_slice() {
                    let edge_key = ((*parent_id).to_string(), (*child_id).to_string());
                    let contract_absent = !board.contracts.contains_edge(&edge_key);
                    if contract_absent {
                        let stub = self.stub_contract(parent_id, child_id);
                        stubs.push(stub);
                    }
                }
                self::PhaseWorkOutput::Contracts { idioms, stubs }
            }
            ::global_fsm::MacroState::S6SystemVerify => {
                let mut verdicts = ::std::vec::Vec::new();
                for vertex in &board.dag.vertices {
                    let subject_match = vertex == shard_subject;
                    if !subject_match {
                        continue;
                    }
                    verdicts.push(self.verify_module(vertex, board));
                }
                let sign_off = self.sign_off(&verdicts);
                self::PhaseWorkOutput::Verification {
                    verdicts,
                    sign_off,
                }
            }
            ::global_fsm::MacroState::S5IterativeDag | ::global_fsm::MacroState::Terminated => {
                self::PhaseWorkOutput::Skipped
            }
        }
    }
}

impl self::DeterministicWorkers {
    /// Construct the deterministic worker crew.
    pub fn new() -> Self {
        Self
    }

    /// Probe the S0 intent gate from the inventory harness configuration.
    fn probe_preflight(&self, board: &::global_blackboard::GlobalBlackboard) -> self::PhaseWorkOutput {
        let harness = ::core::clone::Clone::clone(&board.inventory.harness);
        let build_present = !harness.build_cmd.is_empty();
        let test_present = !harness.test_cmd.is_empty();
        let inventory_present = !board.inventory.is_empty();
        let intent_valid = build_present && test_present && inventory_present;
        self::PhaseWorkOutput::Preflight {
            intent_valid,
            harness,
        }
    }

    /// Assess one whole-repository shard by scanning inventory entries for
    /// dialect coverage and unmapped file extensions.
    fn assess_shard(
        &self,
        shard_subject: &::core::primitive::str,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> self::AssessmentReport {
        let mut risk_notes = ::std::vec::Vec::new();
        let dialect_empty = board.inventory.dialect.source_language.is_empty();
        if dialect_empty {
            risk_notes.push("source language unset".to_string());
        }
        for file in &board.inventory.files {
            let mut mapped = false;
            for extension in self::KNOWN_SOURCE_EXTENSIONS {
                let extension_match = file.ends_with(extension);
                if extension_match {
                    mapped = true;
                }
            }
            let file_unmapped = !mapped;
            if file_unmapped {
                let note = ::std::format!("unmapped dialect: {file}");
                risk_notes.push(note);
            }
        }
        let shard_subject = shard_subject.to_string();
        let file_count = board.inventory.len();
        self::AssessmentReport {
            shard_subject,
            file_count,
            risk_notes,
        }
    }

    /// Reduce shard reports into one merged risk matrix.
    fn synthesize_assessment(&self, reports: &[self::AssessmentReport]) -> self::RiskMatrix {
        let mut matrix = self::RiskMatrix::default();
        for report in reports {
            matrix.file_count += report.file_count;
            let notes = ::core::clone::Clone::clone(&report.risk_notes);
            matrix.risk_notes.extend(notes);
        }
        matrix
    }

    /// Scan the board for hidden coupling. The static graph is already
    /// condensed upstream, so the deterministic default emits no speculative
    /// edges. Rig-backed scanners extend this derivation per shard.
    fn scan_coupling(&self) -> self::PhaseWorkOutput {
        self::PhaseWorkOutput::Coupling {
            discovered_edges: ::std::vec::Vec::new(),
        }
    }

    /// Mine one deterministic rule card per vertex naming its own module.
    fn mine_rules(&self, vertex: &::core::primitive::str) -> ::holon_blackboard::BusinessRuleCard {
        ::holon_blackboard::BusinessRuleCard {
            rule_id: ::std::format!("{vertex}_rule_1"),
            given: ::std::format!("legacy module {vertex} exists"),
            when: ::std::format!("the engine analyzes module {vertex}"),
            then: ::std::format!("module {vertex} preserves observed behavior"),
            citation: ::std::format!("legacy/{vertex}:1"),
        }
    }

    /// Link rule cards across vertices by exact identifier tokens in card
    /// clauses. A card whose clauses name another vertex registers a copy of
    /// the card on that vertex so cross-module obligations surface in S5
    /// problem models. Token matching is exact-word and never substring to
    /// avoid spurious links through unrelated clause prose.
    fn link_cross_module_rules(
        &self,
        board: &::global_blackboard::GlobalBlackboard,
        cards: &[(::std::string::String, ::holon_blackboard::BusinessRuleCard)],
    ) -> (
        ::std::vec::Vec<(::std::string::String, ::holon_blackboard::BusinessRuleCard)>,
        ::core::primitive::usize,
    ) {
        let mut linked = ::std::vec::Vec::new();
        for (owner, card) in cards {
            for vertex in &board.dag.vertices {
                let self_link = vertex == owner;
                if self_link {
                    continue;
                }
                let references = self::card_references_vertex(card, vertex);
                if references {
                    let card_copy = ::core::clone::Clone::clone(card);
                    let copy = (vertex.clone(), card_copy);
                    linked.push(copy);
                }
            }
        }
        let linked_count = linked.len();
        (linked, linked_count)
    }

    /// Emit the global target idiom line from the dialect profile.
    fn architect_idioms(&self, board: &::global_blackboard::GlobalBlackboard) -> ::std::string::String {
        let dialect = &board.inventory.dialect;
        ::std::format!(
            "target idioms: {} edition {}",
            dialect.target_language,
            dialect.target_edition
        )
    }

    /// Stub one interface contract for the edge.
    fn stub_contract(
        &self,
        parent_id: &::core::primitive::str,
        child_id: &::core::primitive::str,
    ) -> self::EdgeStub {
        let parent_id = parent_id.to_string();
        let child_id = child_id.to_string();
        let contract = ::holon_blackboard::EdgeContract {
            parent_id: ::core::clone::Clone::clone(&parent_id),
            interface_spec: ::std::format!("interface {parent_id} provides to {child_id}"),
            preconditions: ::std::vec::Vec::new(),
        };
        self::EdgeStub {
            parent_id,
            child_id,
            contract,
        }
    }

    /// Verify one vertex artifact for test coverage and unsafe code blocks.
    fn verify_module(
        &self,
        vertex: &::core::primitive::str,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> self::VerifyVerdict {
        let artifact_option = match board.artifacts.get(vertex) {
            ::core::option::Option::Some(inner) => ::core::clone::Clone::clone(inner),
            ::core::option::Option::None => ::core::option::Option::None,
        };
        let mut security_findings = ::std::vec::Vec::new();
        let mut tests_pass = false;
        if let ::core::option::Option::Some(artifact) = artifact_option {
            tests_pass = !artifact.test_suite.is_empty();
            let mut cursor = 0;
            while let ::core::option::Option::Some(offset) =
                artifact.target_code[cursor..].find(self::UNSAFE_TOKEN)
            {
                let finding = ::std::format!("unsafe block in {vertex}");
                security_findings.push(finding);
                cursor += offset + self::UNSAFE_TOKEN.len();
            }
        }
        let module_id = vertex.to_string();
        self::VerifyVerdict {
            module_id,
            tests_pass,
            security_findings,
        }
    }

    /// Reduce verdicts into the shard parity bit.
    fn sign_off(&self, verdicts: &[self::VerifyVerdict]) -> ::core::primitive::bool {
        let mut all_green = true;
        for verdict in verdicts {
            let verdict_green = verdict.tests_pass && verdict.security_findings.is_empty();
            if !verdict_green {
                all_green = false;
            }
        }
        all_green
    }
}

/// Deterministic context assembler hydrating activation envelopes from the global blackboard.
pub struct ContextAssembler;

impl self::ContextAssembler {
    /// Hydrate one activation envelope for the given role and target vertex from board data.
    pub fn hydrate(
        role: &::core::primitive::str,
        mode: ::holon_fsm::ActivationMode,
        target_ref: &::core::primitive::str,
        board: &::global_blackboard::GlobalBlackboard,
        delta_directive: ::core::option::Option<::std::string::String>,
    ) -> self::TaskActivationEnvelope {
        let slice_summary = self::ContextAssembler::render_slice(role, target_ref, board);
        let bound_tools = self::ContextAssembler::read_only_tools();
        self::TaskActivationEnvelope {
            mode,
            slice_summary,
            delta_directive,
            bound_tools,
        }
    }

    /// Render the compact label and value slice summary for one target.
    fn render_slice(
        role: &::core::primitive::str,
        target_ref: &::core::primitive::str,
        board: &::global_blackboard::GlobalBlackboard,
    ) -> ::std::string::String {
        let mut lines = ::std::vec::Vec::new();
        lines.push(::std::format!("role: {role}"));
        lines.push(::std::format!("target: {target_ref}"));
        let inventory_slice = self::inventory_lines(board, target_ref);
        let parent_slice = self::parent_lines(board, target_ref);
        let rule_slice = self::rule_lines(board, target_ref);
        let contract_slice = self::contract_lines(board, target_ref);
        lines.extend(inventory_slice);
        lines.extend(parent_slice);
        lines.extend(rule_slice);
        lines.extend(contract_slice);
        lines.join("\n")
    }

    /// Return the read-only deterministic tool surface bound to every worker.
    fn read_only_tools() -> ::std::vec::Vec<::std::string::String> {
        ::std::vec![
            "lsp".to_string(),
            "read".to_string(),
            "grep".to_string(),
            "glob".to_string(),
            "ast_grep".to_string(),
        ]
    }
}

/// Collect inventory lines for files under the target or the full repository.
fn inventory_lines(
    board: &::global_blackboard::GlobalBlackboard,
    target_ref: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let mut lines = ::std::vec::Vec::new();
    for file in &board.inventory.files {
        let under_target = file.contains(target_ref);
        let all_scope = self::targets_all(target_ref);
        if !(under_target || all_scope) {
            continue;
        }
        lines.push(::std::format!("inventory: {file}"));
    }
    lines
}

/// Collect parent identifier lines for the target vertex.
fn parent_lines(
    board: &::global_blackboard::GlobalBlackboard,
    target_ref: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let mut lines = ::std::vec::Vec::new();
    for parent in board.dag.parents_of(target_ref) {
        lines.push(::std::format!("parent: {parent}"));
    }
    lines
}

/// Collect rule card summary lines registered for the target vertex.
fn rule_lines(
    board: &::global_blackboard::GlobalBlackboard,
    target_ref: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let mut lines = ::std::vec::Vec::new();
    let all_scope = self::targets_all(target_ref);
    if all_scope {
        for (vertex, cards) in &board.rules.rules {
            for card in cards {
                lines.push(::std::format!(
                    "rule {} in {}: given={} when={} then={} citation={}",
                    card.rule_id,
                    vertex,
                    card.given,
                    card.when,
                    card.then,
                    card.citation
                ));
            }
        }
        return lines;
    }
    if let ::core::option::Option::Some(cards) = board.rules.get(target_ref) {
        for card in cards {
            lines.push(::std::format!(
                "rule {}: given={} when={} then={} citation={}",
                card.rule_id,
                card.given,
                card.when,
                card.then,
                card.citation
            ));
        }
    }
    lines
}

/// Collect contract lines for edges touching the target vertex.
fn contract_lines(
    board: &::global_blackboard::GlobalBlackboard,
    target_ref: &::core::primitive::str,
) -> ::std::vec::Vec<::std::string::String> {
    let mut lines = ::std::vec::Vec::new();
    for edge_pair in &board.dag.edges {
        let touches_target = edge_pair.0 == target_ref || edge_pair.1 == target_ref;
        let all_scope = self::targets_all(target_ref);
        if !(touches_target || all_scope) {
            continue;
        }
        let key = (edge_pair.0.clone(), edge_pair.1.clone());
        if let ::core::option::Option::Some(contract) = board.contracts.get(&key) {
            lines.push(::std::format!(
                "contract {}->{}: spec={} preconditions={}",
                edge_pair.0,
                edge_pair.1,
                contract.interface_spec,
                contract.preconditions.join(", ")
            ));
        }
    }
    lines
}

/// Report whether the target references the whole repository.
fn targets_all(target_ref: &::core::primitive::str) -> ::core::primitive::bool {
    target_ref == "repository"
}

/// Map a macro phase to the worker role name for its prompt template.
pub fn phase_role_name(phase: ::global_fsm::MacroState) -> &'static ::core::primitive::str {
    match phase {
        ::global_fsm::MacroState::S0IntentPreflight => "PreflightProbe",
        ::global_fsm::MacroState::S1Assess => "ShardAssessor",
        ::global_fsm::MacroState::S2MapDag => "ImplicitCouplingScanner",
        ::global_fsm::MacroState::S3ExtractRules => "RuleMiner",
        ::global_fsm::MacroState::S4BriefContracts => "ContractStubber",
        ::global_fsm::MacroState::S5IterativeDag => "ModuleSynthesizer",
        ::global_fsm::MacroState::S6SystemVerify => "IntegrationVerifier",
        ::global_fsm::MacroState::Terminated => "TerminalAuditor",
    }
}

/// Report whether any card clause names the vertex by exact word token.
fn card_references_vertex(
    card: &::holon_blackboard::BusinessRuleCard,
    vertex: &::core::primitive::str,
) -> ::core::primitive::bool {
    let clauses = [
        card.given.as_str(),
        card.when.as_str(),
        card.then.as_str(),
    ];
    for clause in clauses {
        let split = clause.split(&is_word_char);
        for token in split {
            let token_match = token == vertex;
            if token_match {
                return true;
            }
        }
    }
    false
}

/// Report whether the clause character breaks the token scan.
fn is_word_char(symbol: ::core::primitive::char) -> ::core::primitive::bool {
    let alphanumeric = symbol.is_alphanumeric();
    let underscore = symbol == '_';
    !(alphanumeric || underscore)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preflight_accepts_full_harness_and_rejects_empty_harness() {
        let workers = self::DeterministicWorkers::new();
        let board = self::sample_board();
        let envelope = self::sample_envelope();
        let output = workers.execute(
            ::global_fsm::MacroState::S0IntentPreflight,
            "repository",
            &envelope,
            &board,
        );
        let valid = match output {
            self::PhaseWorkOutput::Preflight { intent_valid, .. } => intent_valid,
            _ => ::std::panic!("expected preflight output"),
        };
        ::std::assert!(valid);
        let mut empty_board = ::global_blackboard::GlobalBlackboard::default();
        empty_board.inventory.files = ::std::vec!["src/main.rs".to_string()];
        let rejected = match workers.execute(
            ::global_fsm::MacroState::S0IntentPreflight,
            "repository",
            &envelope,
            &empty_board,
        ) {
            self::PhaseWorkOutput::Preflight { intent_valid, .. } => intent_valid,
            _ => ::std::panic!("expected preflight output"),
        };
        ::std::assert!(!rejected);
    }

    #[test]
    fn rules_miner_emits_template_card_for_unmined_vertex() {
        let workers = self::DeterministicWorkers::new();
        let board = self::sample_board();
        let envelope = self::sample_envelope();
        let output = workers.execute(
            ::global_fsm::MacroState::S3ExtractRules,
            "beta",
            &envelope,
            &board,
        );
        match output {
            self::PhaseWorkOutput::Rules { cards, linked_count } => {
                ::std::assert_eq!(cards.len(), 1);
                ::std::assert_eq!(cards[0].0, "beta");
                ::std::assert_eq!(cards[0].1.rule_id, "beta_rule_1");
                ::std::assert_eq!(linked_count, 0);
            }
            _ => ::std::panic!("expected rules output"),
        }
    }

    #[test]
    fn rules_miner_skips_already_mined_vertex() {
        let workers = self::DeterministicWorkers::new();
        let mut board = self::sample_board();
        board.rules.insert(
            "beta",
            ::std::vec![::holon_blackboard::BusinessRuleCard {
                rule_id: "seeded".to_string(),
                given: "given clause".to_string(),
                when: "when clause".to_string(),
                then: "then clause".to_string(),
                citation: "legacy/beta.c:7".to_string(),
            }],
        );
        let envelope = self::sample_envelope();
        let output = workers.execute(
            ::global_fsm::MacroState::S3ExtractRules,
            "beta",
            &envelope,
            &board,
        );
        match output {
            self::PhaseWorkOutput::Rules { cards, .. } => {
                ::std::assert!(cards.is_empty());
            }
            _ => ::std::panic!("expected rules output"),
        }
    }

    #[test]
    fn linker_copies_cards_that_name_other_vertices() {
        let workers = self::DeterministicWorkers::new();
        let board = self::sample_board();
        let card = ::holon_blackboard::BusinessRuleCard {
            rule_id: "alpha_rule_1".to_string(),
            given: "legacy module alpha feeds beta".to_string(),
            when: "the engine analyzes module alpha".to_string(),
            then: "module alpha preserves observed behavior".to_string(),
            citation: "legacy/alpha:1".to_string(),
        };
        let card_copy = ::core::clone::Clone::clone(&card);
        let cards = ::std::vec![("alpha".to_string(), card_copy)];
        let (linked, linked_count) = workers.link_cross_module_rules(&board, &cards);
        ::std::assert_eq!(linked_count, 1);
        ::std::assert_eq!(linked[0].0, "beta");
        ::std::assert_eq!(linked[0].1.rule_id, "alpha_rule_1");
    }

    #[test]
    fn contracts_stubber_fills_missing_edge_only() {
        let workers = self::DeterministicWorkers::new();
        let board = self::sample_board();
        let envelope = self::sample_envelope();
        let output = workers.execute(
            ::global_fsm::MacroState::S4BriefContracts,
            "alpha->beta",
            &envelope,
            &board,
        );
        match output {
            self::PhaseWorkOutput::Contracts { idioms, stubs } => {
                ::std::assert!(idioms.contains("rust"));
                ::std::assert_eq!(stubs.len(), 1);
                ::std::assert_eq!(stubs[0].contract.parent_id, "alpha");
                ::std::assert!(stubs[0].contract.interface_spec.contains("beta"));
            }
            _ => ::std::panic!("expected contracts output"),
        }
    }

    #[test]
    fn verification_flags_unsafe_blocks_and_empty_tests() {
        let workers = self::DeterministicWorkers::new();
        let mut board = self::sample_board();
        board.artifacts.commit(
            "alpha",
            ::global_blackboard::ModuleArtifact {
                target_code: "fn a() { unsafe { 1 } }".to_string(),
                test_suite: ::std::vec!["test_alpha".to_string()],
            },
        );
        board.artifacts.commit(
            "beta",
            ::global_blackboard::ModuleArtifact {
                target_code: "fn b() -> u32 { 1 }".to_string(),
                test_suite: ::std::vec::Vec::new(),
            },
        );
        let envelope = self::sample_envelope();
        let output = workers.execute(
            ::global_fsm::MacroState::S6SystemVerify,
            "alpha",
            &envelope,
            &board,
        );
        match output {
            self::PhaseWorkOutput::Verification { verdicts, sign_off } => {
                ::std::assert_eq!(verdicts.len(), 1);
                ::std::assert!(verdicts[0].tests_pass);
                ::std::assert_eq!(verdicts[0].security_findings.len(), 1);
                ::std::assert!(!sign_off);
            }
            _ => ::std::panic!("expected verification output"),
        }
    }

    #[test]
    fn card_references_vertex_uses_exact_tokens() {
        let card = ::holon_blackboard::BusinessRuleCard {
            rule_id: "r1".to_string(),
            given: "legacy module beta exists".to_string(),
            when: "the engine analyzes module alpha".to_string(),
            then: "module alpha preserves observed behavior".to_string(),
            citation: "legacy/alpha:1".to_string(),
        };
        let matches_beta = self::card_references_vertex(&card, "beta");
        let matches_eta = self::card_references_vertex(&card, "eta");
        ::std::assert!(matches_beta);
        ::std::assert!(!matches_eta);
    }

    /// Build a board with two vertices, one edge, a full harness, and no contracts.
    fn sample_board() -> ::global_blackboard::GlobalBlackboard {
        let mut board = ::global_blackboard::GlobalBlackboard::default();
        board.inventory.files = ::std::vec!["src/main.rs".to_string()];
        board.inventory.harness.build_cmd = "cargo build".to_string();
        board.inventory.harness.test_cmd = "cargo test".to_string();
        board.inventory.dialect.source_language = "c".to_string();
        board.inventory.dialect.target_language = "rust".to_string();
        board.inventory.dialect.target_edition = "2021".to_string();
        board.dag.vertices = ::std::vec!["alpha".to_string(), "beta".to_string()];
        board.dag.edges = ::std::vec![("alpha".to_string(), "beta".to_string())];
        board
    }

    /// Build an initial-mode envelope with an empty slice summary.
    fn sample_envelope() -> self::TaskActivationEnvelope {
        self::TaskActivationEnvelope {
            mode: ::holon_fsm::ActivationMode::Initial,
            slice_summary: ::std::string::String::new(),
            delta_directive: ::core::option::Option::None,
            bound_tools: ::std::vec::Vec::new(),
        }
    }
}
