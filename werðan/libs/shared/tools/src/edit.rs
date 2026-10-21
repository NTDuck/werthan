// Pinned commit: d5cd750e41fede277f9eb933455da92a0738915e
// Reference: https://github.com/can1357/oh-my-pi/blob/d5cd750e41fede277f9eb933455da92a0738915e/packages/coding-agent/src/tools/edit.ts

use crate::DisplayExt;

/// Arguments for hashline edit tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct EditArgs {
    pub input: ::std::string::String,
}

/// Output from hashline edit tool.
#[derive(::serde::Serialize, ::serde::Deserialize, ::core::fmt::Debug, ::core::clone::Clone)]
pub struct EditOutput {
    pub summary: ::std::string::String,
}

/// Tool for applying hashline surgical edits to files.
#[derive(::core::fmt::Debug, ::core::clone::Clone, ::core::default::Default)]
pub struct EditTool;

#[derive(::core::clone::Clone, ::core::fmt::Debug)]
enum HunkAction {
    ReplaceRange {
        start: ::core::primitive::usize,
        end: ::core::primitive::usize,
    },
    InsertBefore {
        line: ::core::primitive::usize,
    },
    InsertAfter {
        line: ::core::primitive::usize,
    },
    InsertAtEnd,
}

#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct ParsedHunk {
    action: self::HunkAction,
    lines: ::std::vec::Vec<::std::string::String>,
    order_index: ::core::primitive::usize,
}

#[derive(::core::clone::Clone, ::core::fmt::Debug)]
struct FilePatch {
    path: ::std::string::String,
    hunks: ::std::vec::Vec<self::ParsedHunk>,
}

impl ::rig::tool::Tool for self::EditTool {
    const NAME: &'static ::core::primitive::str = "edit";

    type Error = ::rig::tool::ToolError;
    type Args = self::EditArgs;
    type Output = self::EditOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "input": {
                    "type": "string",
                    "description": "Hashline patch text containing file sections and hunks."
                }
            },
            "required": ["input"]
        });
        let name = Self::NAME.to_string();
        let description = "Apply hashline patches to files.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_edit(args).await
    }
}

async fn execute_edit(args: self::EditArgs) -> ::core::result::Result<self::EditOutput, ::rig::tool::ToolError> {
    let patches = self::parse_hashline_patches(&args.input)?;
    let mut total_edits = 0;
    for patch in &patches {
        let edits = self::apply_file_patch(patch).await?;
        total_edits += edits;
    }

    let file_count = patches.len();
    let summary = ::std::format!("Applied {total_edits} edits across {file_count} files.");
    let output = self::EditOutput { summary };
    ::core::result::Result::Ok(output)
}

fn parse_hashline_patches(
    input: &::core::primitive::str,
) -> ::core::result::Result<::std::vec::Vec<self::FilePatch>, ::rig::tool::ToolError> {
    let mut patches = ::std::vec::Vec::new();
    let mut current_patch: ::core::option::Option<self::FilePatch> = ::core::option::Option::None;
    let mut current_hunk: ::core::option::Option<self::ParsedHunk> = ::core::option::Option::None;
    let mut order_counter = 0;

    for raw_line in input.lines() {
        let trimmed = raw_line.trim();
        let file_header_opt = self::parse_file_header(trimmed);
        if let ::core::option::Option::Some(file_path) = file_header_opt {
            if let ::core::option::Option::Some(hunk) = current_hunk.take() {
                if let ::core::option::Option::Some(patch) = &mut current_patch {
                    patch.hunks.push(hunk);
                }
            }
            if let ::core::option::Option::Some(patch) = current_patch.take() {
                patches.push(patch);
            }
            let path = file_path;
            let hunks = ::std::vec::Vec::new();
            let new_patch = self::FilePatch {
                path,
                hunks,
            };
            current_patch = ::core::option::Option::Some(new_patch);
            continue;
        }

        let hunk_action_opt = self::parse_hunk_header(trimmed);
        if let ::core::option::Option::Some(action) = hunk_action_opt {
            if let ::core::option::Option::Some(hunk) = current_hunk.take() {
                if let ::core::option::Option::Some(patch) = &mut current_patch {
                    patch.hunks.push(hunk);
                }
            }
            let lines = ::std::vec::Vec::new();
            let order_index = order_counter;
            let new_hunk = self::ParsedHunk {
                action,
                lines,
                order_index,
            };
            order_counter += 1;
            current_hunk = ::core::option::Option::Some(new_hunk);
            continue;
        }

        let is_body_line = raw_line.starts_with('+');
        if is_body_line {
            if let ::core::option::Option::Some(hunk) = &mut current_hunk {
                let body_content = &raw_line[1..];
                let owned_content = body_content.to_string();
                hunk.lines.push(owned_content);
            }
        }
    }

    if let ::core::option::Option::Some(hunk) = current_hunk.take() {
        if let ::core::option::Option::Some(patch) = &mut current_patch {
            patch.hunks.push(hunk);
        }
    }
    if let ::core::option::Option::Some(patch) = current_patch.take() {
        patches.push(patch);
    }

    let is_empty = patches.is_empty();
    if is_empty {
        let msg = "No valid hashline file sections found in patch input.".to_string();
        let err = msg.into_tool_error();
        return ::core::result::Result::Err(err);
    }

    ::core::result::Result::Ok(patches)
}

fn parse_file_header(line: &::core::primitive::str) -> ::core::option::Option<::std::string::String> {
    let starts_bracket = line.starts_with('[');
    let ends_bracket = line.ends_with(']');
    if !starts_bracket || !ends_bracket {
        return ::core::option::Option::None;
    }

    let inner_len = line.len().saturating_sub(1);
    if inner_len <= 1 {
        return ::core::option::Option::None;
    }

    let inner = &line[1..inner_len];
    let trimmed = inner.trim();
    let hash_pos = trimmed.find('#');
    let path_str = match hash_pos {
        ::core::option::Option::Some(idx) => {
            let raw = &trimmed[..idx];
            raw.trim()
        },
        ::core::option::Option::None => trimmed,
    };

    if path_str.is_empty() {
        return ::core::option::Option::None;
    }

    let owned = path_str.to_string();
    ::core::option::Option::Some(owned)
}

fn parse_hunk_header(line: &::core::primitive::str) -> ::core::option::Option<self::HunkAction> {
    let is_put = line.starts_with("PUT ");
    if is_put {
        let rest = &line[4..];
        let action_opt = self::parse_put_header(rest);
        return action_opt;
    }

    let is_cut = line.starts_with("CUT ");
    if is_cut {
        let rest = &line[4..];
        let action_opt = self::parse_cut_header(rest);
        return action_opt;
    }

    ::core::option::Option::None
}

fn parse_put_header(content: &::core::primitive::str) -> ::core::option::Option<self::HunkAction> {
    let trimmed = content.trim();
    let is_before = trimmed.starts_with('<');
    if is_before {
        let raw_target = &trimmed[1..];
        let cleaned = raw_target.trim_end_matches(':').trim();
        let parsed_line = cleaned.parse::<::core::primitive::usize>();
        let line_num = match parsed_line {
            ::core::result::Result::Ok(line_val) => line_val.max(1),
            ::core::result::Result::Err(_) => return ::core::option::Option::None,
        };
        let line = line_num;
        let action = self::HunkAction::InsertBefore { line };
        return ::core::option::Option::Some(action);
    }

    let is_after = trimmed.starts_with('>');
    if is_after {
        let raw_target = &trimmed[1..];
        let cleaned = raw_target.trim_end_matches(':').trim();
        if cleaned == "$" {
            let action = self::HunkAction::InsertAtEnd;
            return ::core::option::Option::Some(action);
        }
        let parsed_line = cleaned.parse::<::core::primitive::usize>();
        let line_num = match parsed_line {
            ::core::result::Result::Ok(line_val) => line_val,
            ::core::result::Result::Err(_) => return ::core::option::Option::None,
        };
        let line = line_num;
        let action = self::HunkAction::InsertAfter { line };
        return ::core::option::Option::Some(action);
    }

    let without_colon = trimmed.trim_end_matches(':').trim();
    let is_block = without_colon.ends_with('*');
    if is_block {
        let cleaned = without_colon.trim_end_matches('*').trim();
        let parsed_line = cleaned.parse::<::core::primitive::usize>();
        let line_num = match parsed_line {
            ::core::result::Result::Ok(line_val) => line_val,
            ::core::result::Result::Err(_) => return ::core::option::Option::None,
        };
        let start = line_num;
        let end = line_num;
        let action = self::HunkAction::ReplaceRange {
            start,
            end,
        };
        return ::core::option::Option::Some(action);
    }

    let range_opt = self::parse_line_range(without_colon);
    if let ::core::option::Option::Some((start, end)) = range_opt {
        let action = self::HunkAction::ReplaceRange { start, end };
        return ::core::option::Option::Some(action);
    }

    ::core::option::Option::None
}

fn parse_cut_header(content: &::core::primitive::str) -> ::core::option::Option<self::HunkAction> {
    let trimmed = content.trim();
    let without_colon = trimmed.trim_end_matches(':').trim();
    let is_block = without_colon.ends_with('*');
    if is_block {
        let cleaned = without_colon.trim_end_matches('*').trim();
        let parsed_line = cleaned.parse::<::core::primitive::usize>();
        let line_num = match parsed_line {
            ::core::result::Result::Ok(line_val) => line_val,
            ::core::result::Result::Err(_) => return ::core::option::Option::None,
        };
        let start = line_num;
        let end = line_num;
        let action = self::HunkAction::ReplaceRange {
            start,
            end,
        };
        return ::core::option::Option::Some(action);
    }

    let range_opt = self::parse_line_range(without_colon);
    if let ::core::option::Option::Some((start, end)) = range_opt {
        let action = self::HunkAction::ReplaceRange { start, end };
        return ::core::option::Option::Some(action);
    }

    ::core::option::Option::None
}

fn parse_line_range(
    raw: &::core::primitive::str,
) -> ::core::option::Option<(::core::primitive::usize, ::core::primitive::usize)> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return ::core::option::Option::None;
    }

    let eq_pos = trimmed.find(".=");
    if let ::core::option::Option::Some(idx) = eq_pos {
        let s_part = &trimmed[..idx];
        let e_part = &trimmed[idx + 2..];
        let s_res = s_part.parse::<::core::primitive::usize>();
        let e_res = e_part.parse::<::core::primitive::usize>();
        if let (::core::result::Result::Ok(start_val), ::core::result::Result::Ok(end_val)) = (s_res, e_res) {
            return ::core::option::Option::Some((start_val, end_val));
        }
    }

    let dot_dot_pos = trimmed.find("..");
    if let ::core::option::Option::Some(idx) = dot_dot_pos {
        let s_part = &trimmed[..idx];
        let e_part = &trimmed[idx + 2..];
        let s_res = s_part.parse::<::core::primitive::usize>();
        let e_res = e_part.parse::<::core::primitive::usize>();
        if let (::core::result::Result::Ok(start_val), ::core::result::Result::Ok(end_val)) = (s_res, e_res) {
            return ::core::option::Option::Some((start_val, end_val));
        }
    }

    let hyphen_pos = trimmed.find('-');
    if let ::core::option::Option::Some(idx) = hyphen_pos {
        let s_part = &trimmed[..idx];
        let e_part = &trimmed[idx + 1..];
        let s_res = s_part.parse::<::core::primitive::usize>();
        let e_res = e_part.parse::<::core::primitive::usize>();
        if let (::core::result::Result::Ok(start_val), ::core::result::Result::Ok(end_val)) = (s_res, e_res) {
            return ::core::option::Option::Some((start_val, end_val));
        }
    }

    let single_res = trimmed.parse::<::core::primitive::usize>();
    if let ::core::result::Result::Ok(num) = single_res {
        return ::core::option::Option::Some((num, num));
    }

    ::core::option::Option::None
}

async fn apply_file_patch(patch: &self::FilePatch) -> ::core::result::Result<::core::primitive::usize, ::rig::tool::ToolError> {
    let file_path = ::std::path::Path::new(&patch.path);
    let file_exists = file_path.exists();
    let mut file_lines = ::std::vec::Vec::new();
    if file_exists {
        let raw_bytes_res = ::tokio::fs::read(&file_path).await;
        let raw_bytes = match raw_bytes_res {
            ::core::result::Result::Ok(bytes) => bytes,
            ::core::result::Result::Err(io_err) => {
                let err = io_err.into_tool_error();
                return ::core::result::Result::Err(err);
            },
        };
        let lossy = ::std::string::String::from_utf8_lossy(&raw_bytes);
        let full_text = lossy.to_string();
        for line in full_text.lines() {
            let owned_line = line.to_string();
            file_lines.push(owned_line);
        }
    }

    let sorted_hunks = self::sort_hunks_descending(&patch.hunks);
    for hunk in &sorted_hunks {
        self::apply_single_hunk(&mut file_lines, hunk);
    }

    let parent_opt = file_path.parent();
    if let ::core::option::Option::Some(parent_dir) = parent_opt {
        let parent_empty = parent_dir.as_os_str().is_empty();
        if !parent_empty {
            let mkdir_res = ::tokio::fs::create_dir_all(parent_dir).await;
            if let ::core::result::Result::Err(io_err) = mkdir_res {
                let err = io_err.into_tool_error();
                return ::core::result::Result::Err(err);
            }
        }
    }

    let joined_lines = file_lines.join("\n");
    let mut output_content = joined_lines;
    let has_content = !output_content.is_empty();
    if has_content {
        output_content.push('\n');
    }

    let output_bytes = output_content.as_bytes();
    let write_res = ::tokio::fs::write(&file_path, output_bytes).await;
    if let ::core::result::Result::Err(io_err) = write_res {
        let err = io_err.into_tool_error();
        return ::core::result::Result::Err(err);
    }

    let hunk_count = patch.hunks.len();
    ::core::result::Result::Ok(hunk_count)
}

fn sort_hunks_descending(hunks: &[self::ParsedHunk]) -> ::std::vec::Vec<self::ParsedHunk> {
    let mut sorted = hunks.to_vec();
    sorted.sort_by(|a, b| {
        let key_a = self::hunk_sort_key(a);
        let key_b = self::hunk_sort_key(b);
        let anchor_cmp = key_b.0.cmp(&key_a.0);
        if anchor_cmp != ::core::cmp::Ordering::Equal {
            return anchor_cmp;
        }
        key_b.1.cmp(&key_a.1)
    });
    sorted
}

fn hunk_sort_key(hunk: &self::ParsedHunk) -> (::core::primitive::usize, ::core::primitive::usize) {
    let anchor = match hunk.action {
        self::HunkAction::ReplaceRange { start, .. } => start,
        self::HunkAction::InsertBefore { line } => line,
        self::HunkAction::InsertAfter { line } => line.saturating_add(1),
        self::HunkAction::InsertAtEnd => ::core::primitive::usize::MAX,
    };
    (anchor, hunk.order_index)
}

fn apply_single_hunk(file_lines: &mut ::std::vec::Vec<::std::string::String>, hunk: &self::ParsedHunk) {
    let total_lines = file_lines.len();
    match hunk.action {
        self::HunkAction::ReplaceRange { start, end } => {
            let start_idx = start.max(1).saturating_sub(1);
            let end_idx = end.min(total_lines);
            let s_clamped = start_idx.min(total_lines);
            let e_clamped = end_idx.max(s_clamped);
            let drain_range = s_clamped..e_clamped;
            let lines_to_insert = hunk.lines.clone();
            let _ = file_lines.splice(drain_range, lines_to_insert);
        },
        self::HunkAction::InsertBefore { line } => {
            let start_idx = line.max(1).saturating_sub(1);
            let insert_idx = start_idx.min(total_lines);
            let insert_range = insert_idx..insert_idx;
            let lines_to_insert = hunk.lines.clone();
            let _ = file_lines.splice(insert_range, lines_to_insert);
        },
        self::HunkAction::InsertAfter { line } => {
            let insert_idx = line.min(total_lines);
            let insert_range = insert_idx..insert_idx;
            let lines_to_insert = hunk.lines.clone();
            let _ = file_lines.splice(insert_range, lines_to_insert);
        },
        self::HunkAction::InsertAtEnd => {
            let lines_to_insert = hunk.lines.clone();
            file_lines.extend(lines_to_insert);
        },
    }
}
