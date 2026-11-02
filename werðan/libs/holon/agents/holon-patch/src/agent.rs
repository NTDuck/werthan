//! Surgical code repair agent implementation for the ARIADNE cluster.
//!
//! Exposes the repair agent builder and execution logic, alongside
//! prompt rendering, diagnostic compaction, and code extraction routines.

/// Maximum character budget permitted for compacted diagnostic traces.
///
/// ADR-0018 caps diagnostic payloads for Small Language Models.
/// Small models drift into the dumb zone above 12k tokens.
/// The 2048 character limit keeps prompts inside the smart zone.
/// This preserves focus on line numbers and error tokens.
pub const MAX_CHAR_BUDGET: ::core::primitive::usize = 2048;

/// Maximum number of diagnostic lines retained before truncation.
///
/// ADR-0018 bounds fault representations to at most 20 lines.
/// This keeps guidance under 300 tokens for effective attention.
pub const MAX_DIAGNOSTIC_LINES: ::core::primitive::usize = 20;

/// Namespace for constructing the surgical repair agent and invoking it.
pub struct HolonPatchAgent;

impl self::HolonPatchAgent {
    /// Start building the surgical repair Rig agent.
    pub fn builder<M: ::rig::completion::CompletionModel>() -> self::HolonPatchAgentBuilder<M> {
        self::HolonPatchAgentBuilder {
            model: ::core::option::Option::None,
            temperature: ::core::option::Option::None,
            max_tokens_override: ::core::option::Option::None,
            target_language: "rust".to_string(),
        }
    }

    /// Prompt the wired agent and extract the repaired module source.
    pub fn repair_code<M: ::rig::completion::CompletionModel + 'static>(
        agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
        candidate_code: &::core::primitive::str,
        fault: &::holon_blackboard::FaultRecord,
        problem_model: &::holon_blackboard::ProblemModelSlice,
    ) -> ::std::string::String {
        // A candidate that is already a fault sentinel carries no repairable
        // code. Re-prompting would nest sentinels and churn the repair loop,
        // so the existing marker propagates unchanged.
        let candidate_is_sentinel = candidate_code.contains("SYNTAX_ERROR:")
            || candidate_code.contains("compile_error!");
        if candidate_is_sentinel {
            return candidate_code.to_string();
        }
        let user_prompt = self::render_repair_prompt(candidate_code, fault, problem_model);
        let response_text = self::prompt_blocking(agent, user_prompt);
        match self::fault_sentinel_of(&response_text) {
            ::core::option::Option::Some(sentinel_text) => sentinel_text,
            ::core::option::Option::None => self::extract_code(&response_text),
        }
    }
}

/// Builder assembling the surgical repair Rig agent with role defaults wired.
#[derive(::core::fmt::Debug)]
pub struct HolonPatchAgentBuilder<M: ::rig::completion::CompletionModel> {
    model: ::core::option::Option<M>,
    temperature: ::core::option::Option<::core::primitive::f64>,
    max_tokens_override: ::core::option::Option<::core::primitive::u64>,
    target_language: ::std::string::String,
}

impl<M: ::rig::completion::CompletionModel> self::HolonPatchAgentBuilder<M> {
    /// Wire the completion model backing the agent.
    pub fn model(mut self, wired_model: M) -> Self {
        self.model = ::core::option::Option::Some(wired_model);
        self
    }

    /// Override the sampling temperature.
    pub fn temperature(mut self, temperature_value: ::core::primitive::f64) -> Self {
        self.temperature = ::core::option::Option::Some(temperature_value);
        self
    }

    /// Override the completion token budget.
    pub fn max_tokens(mut self, token_budget: ::core::primitive::u64) -> Self {
        self.max_tokens_override = ::core::option::Option::Some(token_budget);
        self
    }

    /// Set the target language the repair preamble advertises.
    pub fn target_language(mut self, language: ::std::string::String) -> Self {
        self.target_language = language;
        self
    }

    /// Build the Rig agent with the repair preamble and token budget wired.
    pub fn build(self) -> ::rig::agent::Agent<M> {
        let wired_model = match self.model {
            ::core::option::Option::Some(found_model) => found_model,
            ::core::option::Option::None => ::core::panic!("patch model must be wired before build"),
        };
        let preamble_text = self::patch_system_prompt(&self.target_language);
        let mut agent_builder = ::rig::agent::AgentBuilder::new(wired_model)
            .preamble(&preamble_text)
            .max_tokens(16384);
        if let ::core::option::Option::Some(temperature_value) = self.temperature {
            agent_builder = agent_builder.temperature(temperature_value);
        }
        if let ::core::option::Option::Some(token_budget) = self.max_tokens_override {
            agent_builder = agent_builder.max_tokens(token_budget);
        }
        agent_builder.build()
    }
}

/// Render the repair prompt from candidate code, fault record, and problem model.
pub fn render_repair_prompt(
    candidate_code: &::core::primitive::str,
    fault: &::holon_blackboard::FaultRecord,
    problem_model: &::holon_blackboard::ProblemModelSlice,
) -> ::std::string::String {
    let context_block = problem_model.render_prompt_context();
    let fault_summary = &fault.summary;
    let diagnostic_trace = self::compact_diagnostic_trace(&fault.diagnostic_trace, MAX_DIAGNOSTIC_LINES);
    let target_language = &problem_model.target_language;
    ::std::format!(
        "[PROBLEM CONTEXT]\n{context_block}\n\n[CANDIDATE CODE]\n{candidate_code}\n\n[FAULT SUMMARY]\n{fault_summary}\n\n[DIAGNOSTIC TRACE]\n{diagnostic_trace}\n\nReturn the repaired complete {target_language} module now.",
    )
}

/// Compact a diagnostic trace into a focused fault card within the SLM "Smart Zone" (<= 12k tokens).
/// Distills compiler errors, file/line spans, error codes, and suggestions, stripping noisy build boilerplate.
///
/// ADR-0018 filters compiler output down to key lines.
/// It enforces the 2048 character budget.
/// This keeps repair prompts inside the 12k token smart zone.
pub fn compact_diagnostic_trace(
    raw_trace: &::core::primitive::str,
    max_lines: ::core::primitive::usize,
) -> ::std::string::String {
    let lines = raw_trace.lines().collect::<::std::vec::Vec<&str>>();
    let predicate = |&line: &&str| self::is_diagnostic_signal(line);
    let relevant_lines = lines
        .iter()
        .copied()
        .filter(predicate)
        .take(max_lines)
        .collect::<::std::vec::Vec<&str>>();
    let compacted = if relevant_lines.is_empty() {
        let skip_count = lines.len().saturating_sub(max_lines);
        lines
            .iter()
            .skip(skip_count)
            .copied()
            .collect::<::std::vec::Vec<&str>>()
            .join("\n")
    } else {
        relevant_lines.join("\n")
    };
    self::truncate_diagnostics(&compacted)
}

/// Identify whether a diagnostic line represents an actionable compiler or runtime error signal.
fn is_diagnostic_signal(line_text: &::core::primitive::str) -> ::core::primitive::bool {
    let trimmed = line_text.trim();
    trimmed.starts_with("error")
        || trimmed.contains("error:")
        || trimmed.starts_with("-->")
        || trimmed.starts_with("help:")
        || trimmed.starts_with("note:")
        || trimmed.starts_with('|')
        || trimmed.split_once(" | ").is_some_and(|(prefix, _)| !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()))
        || trimmed.contains("TypeError")
        || trimmed.contains("SyntaxError")
        || trimmed.contains("AssertionError")
        || trimmed.contains("failed")
        || trimmed.contains("panic")
}

/// Truncate diagnostics to enforce the hard character ceiling of ADR-0018.
///
/// ADR-0018 truncates diagnostic payloads that exceed the budget.
/// The function appends a truncation sentinel notice.
/// This prevents stack traces from exceeding the smart zone.
pub fn truncate_diagnostics(compacted_text: &::core::primitive::str) -> ::std::string::String {
    if compacted_text.chars().count() > MAX_CHAR_BUDGET {
        let truncated = compacted_text
            .chars()
            .take(MAX_CHAR_BUDGET)
            .collect::<::std::string::String>();
        ::std::format!("{truncated}\n[...diagnostic truncated...]")
    } else {
        compacted_text.to_string()
    }
}

/// Run one blocking prompt exchange on a dedicated single-thread runtime.
/// The rig prompt API is async while the ARIADNE controller path is synchronous,
/// so the future is driven to completion on its own worker thread.
fn prompt_blocking<M: ::rig::completion::CompletionModel + 'static>(
    agent: &::std::sync::Arc<::rig::agent::Agent<M>>,
    user_prompt: ::std::string::String,
) -> ::std::string::String {
    let agent_for_call = ::std::sync::Arc::clone(agent);
    let (sender, receiver) = ::std::sync::mpsc::channel::<
        ::core::result::Result<::std::string::String, ::std::string::String>,
    >();
    let handle_result = ::std::thread::Builder::new()
        .name("ariadne-patch-call".to_string())
        .spawn(move || {
            let runtime = ::tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime builds");
            let outcome = runtime.block_on(async {
                ::rig::completion::Prompt::prompt(&*agent_for_call, user_prompt)
                    .await
                    .map_err(|prompt_error| prompt_error.to_string())
            });
            let _ = sender.send(outcome);
        });
    match handle_result {
        ::core::result::Result::Ok(handle) => {
            let _ = handle.join();
        }
        ::core::result::Result::Err(spawn_error) => {
            let failure_message = ::std::format!("repair inference unavailable: {spawn_error}");
            let sentinel = self::fault_candidate(&failure_message);
            return sentinel;
        }
    }
    match receiver.recv() {
        ::core::result::Result::Ok(::core::result::Result::Ok(response_text)) => response_text,
        ::core::result::Result::Ok(::core::result::Result::Err(prompt_error)) => {
            let failure_message = ::std::format!("repair inference failed: {prompt_error}");
            let sentinel = self::fault_candidate(&failure_message);
            sentinel
        }
        ::core::result::Result::Err(_) => {
            let sentinel = self::fault_candidate("repair thread panicked");
            sentinel
        }
    }
}

fn fault_sentinel_of(response_text: &::core::primitive::str) -> ::core::option::Option<::std::string::String> {
    let is_fault = response_text.contains("repair inference unavailable")
        || response_text.contains("repair inference failed")
        || response_text.contains("repair thread panicked");
    if !is_fault {
        return ::core::option::Option::None;
    }
    let sentinel_text = self::fault_candidate(response_text);
    ::core::option::Option::Some(sentinel_text)
}

/// Encode an inference failure as a language-neutral SYNTAX_ERROR sentinel candidate.
pub fn fault_candidate(failure_reason: &::core::primitive::str) -> ::std::string::String {
    // A reason that already embeds a sentinel marker would nest markers on
    // every retry cycle. Collapse to the existing sentinel text instead.
    let reason_is_sentinel = failure_reason.contains("SYNTAX_ERROR:")
        || failure_reason.contains("compile_error!");
    if reason_is_sentinel {
        return failure_reason.to_string();
    }
    let sanitized_reason = failure_reason
        .replace('"', "'")
        .replace('\n', " ")
        .replace(';', "");
    ::std::format!("SYNTAX_ERROR: ariadne llm repair fault: {sanitized_reason}")
}

/// Extract the first fenced Rust code block from a model response.
/// Unfenced responses come back trimmed so bare code still works.
pub fn extract_code(response_text: &::core::primitive::str) -> ::std::string::String {
    let fence_token = "```";
    let fence_start = match response_text.find(fence_token) {
        ::core::option::Option::Some(found_index) => found_index + fence_token.len(),
        ::core::option::Option::None => {
            let trimmed_text = response_text.trim();
            let unwrapped_code = self::unwrap_outer_mod(trimmed_text);
            return self::ensure_top_level_pub(&unwrapped_code);
        }
    };
    let after_language_line = match response_text[fence_start..].find('\n') {
        ::core::option::Option::Some(line_break) => fence_start + line_break + 1,
        ::core::option::Option::None => {
            let trimmed_text = response_text.trim();
            let unwrapped_code = self::unwrap_outer_mod(trimmed_text);
            return self::ensure_top_level_pub(&unwrapped_code);
        }
    };
    let remainder = &response_text[after_language_line..];
    let code_body = match remainder.find(fence_token) {
        ::core::option::Option::Some(close_index) => &remainder[..close_index],
        ::core::option::Option::None => remainder,
    };
    let trimmed_body = code_body.trim();
    let unwrapped_code = self::unwrap_outer_mod(trimmed_body);
    self::ensure_top_level_pub(&unwrapped_code)
}

/// Remove outer module declarations from source code.
///
/// This function finds top-level module blocks, extracts their inner contents,
/// reduces indentation, and joins them with other top-level code.
pub fn unwrap_outer_mod(code: &::core::primitive::str) -> ::std::string::String {
    let mut brace_depth: ::core::primitive::usize = 0;
    let mut in_line_comment: ::core::primitive::bool = false;
    let mut block_comment_depth: ::core::primitive::usize = 0;
    let mut in_string: ::core::primitive::bool = false;
    let mut in_char: ::core::primitive::bool = false;
    let mut is_escaped: ::core::primitive::bool = false;
    let mut cursor_idx: ::core::primitive::usize = 0;
    let mut last_copied_idx: ::core::primitive::usize = 0;
    let mut unwrapped_any: ::core::primitive::bool = false;
    let mut result_output = ::std::string::String::new();

    while cursor_idx < code.len() {
        let current_char = match code[cursor_idx..].chars().next() {
            ::core::option::Option::Some(char_val) => char_val,
            ::core::option::Option::None => break,
        };
        let char_len = current_char.len_utf8();

        if is_escaped {
            is_escaped = false;
            cursor_idx += char_len;
            continue;
        }

        if in_line_comment {
            if current_char == '\n' {
                in_line_comment = false;
            }
            cursor_idx += char_len;
            continue;
        }

        if block_comment_depth > 0 {
            if current_char == '*' && code[cursor_idx..].starts_with("*/") {
                block_comment_depth -= 1;
                cursor_idx += 2;
                continue;
            }
            if current_char == '/' && code[cursor_idx..].starts_with("/*") {
                block_comment_depth -= 1;
                cursor_idx += 2;
                continue;
            }
            cursor_idx += char_len;
            continue;
        }

        if in_string {
            if current_char == '\\' {
                is_escaped = true;
            } else if current_char == '"' {
                in_string = false;
            }
            cursor_idx += char_len;
            continue;
        }

        if in_char {
            if current_char == '\\' {
                is_escaped = true;
            } else if current_char == '\'' {
                in_char = false;
            }
            cursor_idx += char_len;
            continue;
        }

        if brace_depth == 0 {
            let is_preceded_by_ident = if cursor_idx == 0 {
                false
            } else {
                match code[..cursor_idx].chars().next_back() {
                    ::core::option::Option::Some(prev_char) => {
                        prev_char.is_alphanumeric() || prev_char == '_'
                    }
                    ::core::option::Option::None => false,
                }
            };
            if !is_preceded_by_ident {
                let candidate_slice = &code[cursor_idx..];
                if let ::core::option::Option::Some(rel_open_brace) = self::match_mod_header(candidate_slice) {
                    let open_brace_idx = cursor_idx + rel_open_brace;
                    let remaining_from_open = &code[open_brace_idx + 1..];
                    if let ::core::option::Option::Some(rel_close_brace) = self::find_matching_brace(remaining_from_open) {
                        let closing_brace_idx = open_brace_idx + 1 + rel_close_brace;
                        let prefix_between = &code[last_copied_idx..cursor_idx];
                        let prefix_trimmed = prefix_between.trim();
                        if prefix_trimmed.is_empty() {
                            if !result_output.is_empty() && !result_output.ends_with('\n') {
                                result_output.push('\n');
                            }
                        } else {
                            result_output.push_str(prefix_between);
                        }
                        let inner_content = &code[open_brace_idx + 1..closing_brace_idx];
                        let dedented_content = self::dedent_module_body(inner_content);
                        result_output.push_str(&dedented_content);
                        let mut post_close_idx = closing_brace_idx + 1;
                        while post_close_idx < code.len() {
                            let next_byte = code.as_bytes()[post_close_idx];
                            let is_whitespace_or_semi = next_byte == b';'
                                || next_byte == b' '
                                || next_byte == b'\t'
                                || next_byte == b'\r';
                            if is_whitespace_or_semi {
                                post_close_idx += 1;
                            } else {
                                break;
                            }
                        }
                        last_copied_idx = post_close_idx;
                        cursor_idx = post_close_idx;
                        unwrapped_any = true;
                        continue;
                    }
                }
            }
        }

        if current_char == '/' {
            if code[cursor_idx..].starts_with("//") {
                in_line_comment = true;
                cursor_idx += 2;
                continue;
            }
            if code[cursor_idx..].starts_with("/*") {
                block_comment_depth += 1;
                cursor_idx += 2;
                continue;
            }
        }

        if current_char == '"' {
            in_string = true;
            cursor_idx += 1;
            continue;
        }

        if current_char == '\'' {
            let after_quote = &code[cursor_idx + 1..];
            let is_char_lit = if after_quote.starts_with('\\') {
                true
            } else if let ::core::option::Option::Some(second_char) = after_quote.chars().next() {
                after_quote[second_char.len_utf8()..].starts_with('\'')
            } else {
                false
            };
            if is_char_lit {
                in_char = true;
            }
            cursor_idx += 1;
            continue;
        }

        if current_char == '{' {
            brace_depth += 1;
        } else if current_char == '}' && brace_depth > 0 {
            brace_depth -= 1;
        }

        cursor_idx += char_len;
    }

    if !unwrapped_any {
        return code.to_string();
    }

    let trailing_tail = &code[last_copied_idx..];
    let trimmed_tail = trailing_tail.trim();
    let is_ignorable = trimmed_tail.is_empty() || trimmed_tail == ";";
    if !is_ignorable {
        if !result_output.is_empty() && !result_output.ends_with('\n') {
            result_output.push('\n');
        }
        result_output.push_str(trimmed_tail);
    }
    let trimmed_output = result_output.trim();
    trimmed_output.to_string()
}

/// Match a module declaration header at the start of the code slice.
///
/// Returns the byte offset of the opening brace `{` if matched.
fn match_mod_header(code_slice: &::core::primitive::str) -> ::core::option::Option<::core::primitive::usize> {
    let mut cursor_offset: ::core::primitive::usize = 0;
    if let ::core::option::Option::Some(after_pub) = code_slice.strip_prefix("pub") {
        cursor_offset = 3;
        let mut chars_iter = after_pub.chars();
        let next_char = chars_iter.next()?;
        if next_char == '(' {
            let close_paren_rel = after_pub.find(')')?;
            cursor_offset += close_paren_rel + 1;
            let after_paren = &code_slice[cursor_offset..];
            let first_after_paren = after_paren.chars().next()?;
            if !first_after_paren.is_whitespace() {
                return ::core::option::Option::None;
            }
        } else if !next_char.is_whitespace() {
            return ::core::option::Option::None;
        }
    }

    let remaining_after_pub = &code_slice[cursor_offset..];
    let trimmed_leading = remaining_after_pub.trim_start();
    let whitespace_len = remaining_after_pub.len() - trimmed_leading.len();
    cursor_offset += whitespace_len;

    let after_mod = code_slice[cursor_offset..].strip_prefix("mod")?;
    cursor_offset += 3;

    let next_after_mod = after_mod.chars().next()?;
    if !next_after_mod.is_whitespace() {
        return ::core::option::Option::None;
    }

    let remaining_after_mod = &code_slice[cursor_offset..];
    let trimmed_ident = remaining_after_mod.trim_start();
    let ident_whitespace_len = remaining_after_mod.len() - trimmed_ident.len();
    cursor_offset += ident_whitespace_len;

    let first_ident_char = trimmed_ident.chars().next()?;
    if !first_ident_char.is_alphabetic() && first_ident_char != '_' {
        return ::core::option::Option::None;
    }

    let mut ident_len: ::core::primitive::usize = 0;
    for (char_offset, char_val) in trimmed_ident.char_indices() {
        if char_val.is_alphanumeric() || char_val == '_' {
            ident_len = char_offset + char_val.len_utf8();
        } else {
            break;
        }
    }
    cursor_offset += ident_len;

    let remaining_after_ident = &code_slice[cursor_offset..];
    let trimmed_brace = remaining_after_ident.trim_start();
    let brace_whitespace_len = remaining_after_ident.len() - trimmed_brace.len();
    cursor_offset += brace_whitespace_len;

    if trimmed_brace.starts_with('{') {
        ::core::option::Option::Some(cursor_offset)
    } else {
        ::core::option::Option::None
    }
}

/// Find the byte offset of the matching closing brace.
fn find_matching_brace(code_slice: &::core::primitive::str) -> ::core::option::Option<::core::primitive::usize> {
    let mut brace_depth: ::core::primitive::usize = 1;
    let mut in_string: ::core::primitive::bool = false;
    let mut in_char: ::core::primitive::bool = false;
    let mut is_escaped: ::core::primitive::bool = false;
    let mut in_line_comment: ::core::primitive::bool = false;
    let mut block_comment_depth: ::core::primitive::usize = 0;
    let mut cursor_idx: ::core::primitive::usize = 0;

    while cursor_idx < code_slice.len() {
        let current_char = match code_slice[cursor_idx..].chars().next() {
            ::core::option::Option::Some(char_val) => char_val,
            ::core::option::Option::None => break,
        };
        let char_len = current_char.len_utf8();

        if is_escaped {
            is_escaped = false;
            cursor_idx += char_len;
            continue;
        }

        if in_line_comment {
            if current_char == '\n' {
                in_line_comment = false;
            }
            cursor_idx += char_len;
            continue;
        }

        if block_comment_depth > 0 {
            if current_char == '*' && code_slice[cursor_idx..].starts_with("*/") {
                block_comment_depth -= 1;
                cursor_idx += 2;
                continue;
            }
            if current_char == '/' && code_slice[cursor_idx..].starts_with("/*") {
                block_comment_depth += 1;
                cursor_idx += 2;
                continue;
            }
            cursor_idx += char_len;
            continue;
        }

        if in_string {
            if current_char == '\\' {
                is_escaped = true;
            } else if current_char == '"' {
                in_string = false;
            }
            cursor_idx += char_len;
            continue;
        }

        if in_char {
            if current_char == '\\' {
                is_escaped = true;
            } else if current_char == '\'' {
                in_char = false;
            }
            cursor_idx += char_len;
            continue;
        }

        if current_char == '/' {
            if code_slice[cursor_idx..].starts_with("//") {
                in_line_comment = true;
                cursor_idx += 2;
                continue;
            }
            if code_slice[cursor_idx..].starts_with("/*") {
                block_comment_depth += 1;
                cursor_idx += 2;
                continue;
            }
        }

        if current_char == '"' {
            in_string = true;
            cursor_idx += 1;
            continue;
        }

        if current_char == '\'' {
            let after_quote = &code_slice[cursor_idx + 1..];
            let is_char_lit = if after_quote.starts_with('\\') {
                true
            } else if let ::core::option::Option::Some(second_char) = after_quote.chars().next() {
                after_quote[second_char.len_utf8()..].starts_with('\'')
            } else {
                false
            };
            if is_char_lit {
                in_char = true;
            }
            cursor_idx += 1;
            continue;
        }

        if current_char == '{' {
            brace_depth += 1;
        } else if current_char == '}' {
            brace_depth -= 1;
            if brace_depth == 0 {
                return ::core::option::Option::Some(cursor_idx);
            }
        }

        cursor_idx += char_len;
    }

    ::core::option::Option::None
}

/// Dedent each line of the module body by four spaces or one tab.
fn dedent_module_body(inner_content: &::core::primitive::str) -> ::std::string::String {
    let dedented_lines = inner_content
        .lines()
        .map(|line_str| {
            if let ::core::option::Option::Some(stripped_line) = line_str.strip_prefix("    ") {
                stripped_line
            } else if let ::core::option::Option::Some(stripped_line) = line_str.strip_prefix('\t') {
                stripped_line
            } else {
                line_str
            }
        })
        .collect::<::std::vec::Vec<&::core::primitive::str>>();
    let joined_content = dedented_lines.join("\n");
    let trimmed_result = joined_content.trim();
    trimmed_result.to_string()
}

/// Ensure top-level item declarations carry public visibility.
pub fn ensure_top_level_pub(source_code: &::core::primitive::str) -> ::std::string::String {
    const TOP_LEVEL_PREFIXES: [&::core::primitive::str; 5] = [
        "fn ",
        "struct ",
        "enum ",
        "const ",
        "type ",
    ];
    let transformed_lines = source_code
        .lines()
        .map(|line| {
            let matches_prefix = TOP_LEVEL_PREFIXES
                .iter()
                .any(|prefix| line.starts_with(prefix));
            let needs_pub = matches_prefix && !line.starts_with("pub ");
            if needs_pub {
                ::std::format!("pub {line}")
            } else {
                line.to_string()
            }
        })
        .collect::<::std::vec::Vec<::std::string::String>>();
    transformed_lines.join("\n")
}

/// System prompt steering the model toward repaired module code in the
/// configured target language.
pub fn patch_system_prompt(target_language: &::core::primitive::str) -> ::std::string::String {
    ::std::format!(
        "You are a {target_language} code repair engine for a legacy modernization pipeline. \
Output only the repaired complete {target_language} source for one module with no markdown and no commentary. \
Keep every public signature from the candidate unchanged unless a contract requires otherwise. \
Strictly adhere to the exact function signatures, argument counts, and parameter types defined in [EXPORT OBLIGATIONS]. If an exported function is \
specified with N arguments, you must define it with exactly those N arguments so that integration tests compile. \
Do NOT wrap the output in 'pub mod <name> {{ ... }}'. Output only top-level items (functions, structs, constants) for this single module - the module is already declared by the crate root. Do NOT output code for other modules. \
The repaired code must compile and must satisfy every rule card. Return only repaired source."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_code_strips_fences() {
        let sample = "```rust\npub fn repaired_probe() -> u8 { 1 }\n```";
        let extracted = self::extract_code(sample);
        ::std::assert!(extracted.starts_with("pub fn repaired_probe()"));
        ::std::assert!(!extracted.contains("```"));
    }

    #[test]
    fn test_extract_code_bare_response_passthrough() {
        let sample = "pub fn repaired_probe() -> u8 { 1 }";
        let extracted = self::extract_code(sample);
        ::std::assert_eq!(extracted, sample);
    }
    #[test]
    fn test_extract_code_unwraps_pub_mod_matrix() {
        let sample = "pub mod matrix {\n    pub fn generate_matrix() -> Vec<f64> {\n        vec![1.0, 2.0]\n    }\n}";
        let extracted = self::extract_code(sample);
        ::std::assert!(!extracted.contains("pub mod matrix"));
        ::std::assert!(extracted.contains("pub fn generate_matrix() -> Vec<f64>"));
        ::std::assert_eq!(
            extracted,
            "pub fn generate_matrix() -> Vec<f64> {\n    vec![1.0, 2.0]\n}"
        );
    }

    #[test]
    fn test_extract_code_unwraps_pub_mod_utils() {
        let sample = "```rust\npub mod utils {\n    pub fn helper() -> bool {\n        true\n    }\n}\n```";
        let extracted = self::extract_code(sample);
        ::std::assert!(!extracted.contains("pub mod utils"));
        ::std::assert!(extracted.contains("pub fn helper() -> bool"));
        ::std::assert_eq!(
            extracted,
            "pub fn helper() -> bool {\n    true\n}"
        );
    }

    #[test]
    fn test_patch_system_prompt_instructs_no_pub_mod() {
        let prompt = self::patch_system_prompt("rust");
        let instruction = "Do NOT wrap the output in 'pub mod <name> { ... }'. Output only top-level items (functions, structs, constants) for this single module - the module is already declared by the crate root. Do NOT output code for other modules.";
        ::std::assert!(prompt.contains(instruction));
    }

    #[test]
    fn test_patch_system_prompt_instructs_export_obligations_alignment() {
        let prompt = self::patch_system_prompt("rust");
        let instruction = ::core::concat!(
            "Strictly adhere to the exact function signatures, argument counts, and parameter types defined in [EXPORT OBLIGATIONS]. If an exported function is ",
            "specified with N arguments, you must define it with exactly those N arguments so that integration tests compile.",
        );
        ::std::assert!(prompt.contains(instruction));
    }

    #[test]
    fn test_fault_candidate_sanitizes_and_encodes_marker() {
        let sentinel = self::fault_candidate("boom\nnext \"line\"");
        ::std::assert!(sentinel.contains("SYNTAX_ERROR"));
        ::std::assert!(sentinel.contains("ariadne llm repair fault"));
        ::std::assert!(!sentinel.contains('\n'));
        ::std::assert!(!sentinel.contains('"'));
    }

    #[test]
    fn test_fault_sentinel_detects_transport_errors() {
        let failure_text = "repair inference failed: timeout";
        let sentinel = self::fault_sentinel_of(failure_text);
        ::std::assert!(sentinel.is_some());
        let normal_text = "pub fn repaired_probe() -> u8 { 1 }";
        let no_sentinel = self::fault_sentinel_of(normal_text);
        ::std::assert!(no_sentinel.is_none());
    }

    #[test]
    fn test_compact_diagnostic_trace_extracts_key_diagnostics() {
        let noisy_trace = "Compiling target v0.1.0\nwarning: unused variable\nerror[E0308]: mismatched types\n  --> src/lib.rs:10:5\nhelp: consider borrowing\nBuild failed with exit code 1";
        let compacted = self::compact_diagnostic_trace(noisy_trace, 3);
        ::std::assert!(compacted.contains("error[E0308]"));
        ::std::assert!(compacted.contains("--> src/lib.rs:10:5"));
        ::std::assert!(compacted.lines().count() <= 3);
    }

    #[test]
    fn test_truncate_diagnostics_enforces_budget() {
        let long_trace = "diagnostic error line\n".repeat(200);
        let truncated = self::truncate_diagnostics(&long_trace);
        ::std::assert!(truncated.contains("[...diagnostic truncated...]"));
        ::std::assert!(truncated.len() <= MAX_CHAR_BUDGET + 50);
    }

    /// Counting completion model: records every call and answers with a
    /// marker that would corrupt the candidate if a call ever happened.
    #[derive(::core::clone::Clone)]
    struct CountingFakeModel {
        call_count: ::std::sync::Arc<::std::sync::atomic::AtomicU64>,
    }

    impl ::rig::completion::CompletionModel for self::CountingFakeModel {
        type Response = ::std::string::String;

        fn completion(
            &self,
            _request: ::rig::completion::CompletionRequest,
        ) -> impl ::core::future::Future<
            Output = ::core::result::Result<
                ::rig::completion::CompletionResponse<Self::Response>,
                ::rig::completion::CompletionError,
            >,
        > + ::core::marker::Send {
            self.call_count
                .fetch_add(1, ::std::sync::atomic::Ordering::SeqCst);
            let content_item = ::rig::message::AssistantContent::text(
                "SYNTAX_ERROR: ariadne llm repair fault: nested marker would prove inference ran",
            );
            let outcome = ::rig::completion::CompletionResponse {
                choice: ::rig::OneOrMany::one(content_item),
                raw_response: ::std::string::String::from("counting-fake"),
            };
            ::core::future::ready(::core::result::Result::Ok(outcome))
        }
    }

    #[test]
    fn test_repair_code_on_sentinel_candidate_makes_zero_calls() {
        // A candidate that is already a fault marker carries no repairable
        // code. Re-prompting would nest sentinels and churn the repair
        // loop, so repair_code must propagate it without any inference.
        let zero_calls = ::std::sync::atomic::AtomicU64::new(0);
        let call_count = ::std::sync::Arc::new(zero_calls);
        let patch_agent = self::HolonPatchAgent::builder::<self::CountingFakeModel>()
            .model(self::CountingFakeModel {
                call_count: ::std::sync::Arc::clone(&call_count),
            })
            .build();
        let agent = ::std::sync::Arc::new(patch_agent);
        let sentinel_candidate = self::fault_candidate("prior synthesis failure");
        let fault = ::holon_blackboard::FaultRecord {
            summary: ::std::string::String::from("prior failure"),
            diagnostic_trace: ::std::string::String::new(),
            cause: ::core::option::Option::None,
        };
        let problem_model = ::holon_blackboard::ProblemModelSlice::new();
        let outcome = self::HolonPatchAgent::repair_code(
            &agent,
            &sentinel_candidate,
            &fault,
            &problem_model,
        );
        ::std::assert_eq!(outcome, sentinel_candidate);
        let observed = call_count.load(::std::sync::atomic::Ordering::SeqCst);
        ::std::assert_eq!(observed, 0, "sentinel candidate must not trigger inference");
    }
}
