//! Ask tool implementation.
//! Surfaces typed questions to the human operator through a pluggable answer sink.

use crate::DisplayExt;

/// A single typed question for the human operator.
#[derive(::core::fmt::Debug, ::serde::Deserialize)]
pub struct AskQuestion {
    pub id: ::std::string::String,
    pub question: ::std::string::String,
    pub options: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
    pub recommended: ::core::option::Option<::core::primitive::usize>,
}

/// Input arguments for ask tool.
#[derive(::core::fmt::Debug, ::serde::Deserialize)]
pub struct AskArgs {
    pub questions: ::std::vec::Vec<self::AskQuestion>,
}

/// One collected answer keyed by question id.
#[derive(
    ::core::clone::Clone, ::core::fmt::Debug, ::serde::Deserialize, ::serde::Serialize,
)]
pub struct AskAnswer {
    pub id: ::std::string::String,
    pub value: ::std::string::String,
}

/// Output results for ask tool.
#[derive(::core::fmt::Debug, ::serde::Serialize)]
pub struct AskOutput {
    pub answers: ::std::vec::Vec<self::AskAnswer>,
}

/// Tool for asking the human operator typed questions and collecting answers.
#[derive(::core::clone::Clone, ::core::default::Default)]
pub struct AskTool;

impl ::rig::tool::Tool for self::AskTool {
    const NAME: &'static ::core::primitive::str = "ask";

    type Error = ::rig::tool::ToolError;
    type Args = self::AskArgs;
    type Output = self::AskOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "description": "Questions to present to the human operator.",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": {
                                "type": "string",
                                "description": "Stable identifier that links the answer to its question."
                            },
                            "question": {
                                "type": "string",
                                "description": "Question text shown to the operator."
                            },
                            "options": {
                                "type": "array",
                                "items": {
                                    "type": "string"
                                },
                                "description": "Answer choices for the operator. Empty or omitted means free text."
                            },
                            "recommended": {
                                "type": "integer",
                                "description": "Index of the recommended option inside options."
                            }
                        },
                        "required": ["id", "question"]
                    }
                }
            },
            "required": ["questions"]
        });
        let name = Self::NAME.to_string();
        let description = "Ask the human operator typed questions and collect answers.".to_string();
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_ask(args)
    }
}

/// Process-global sink holding canned answers preloaded by tests and frontends.
static ANSWER_SINK: ::std::sync::LazyLock<
    ::std::sync::RwLock<::std::collections::VecDeque<self::AskAnswer>>,
> = ::std::sync::LazyLock::new(|| {
    let queue = ::std::collections::VecDeque::new();
    let lock = ::std::sync::RwLock::new(queue);
    lock
});

/// Preload canned answers that call() consumes one per matching question id.
pub fn push_canned_answers(answers: ::std::vec::Vec<self::AskAnswer>) {
    let mut guard = self::ANSWER_SINK.write().unwrap_or_else(|poisoned| poisoned.into_inner());
    for answer in answers {
        guard.push_back(answer);
    }
}

/// Drop every queued canned answer so the next call() fails without fresh input.
pub fn clear_canned_answers() {
    let mut guard = self::ANSWER_SINK.write().unwrap_or_else(|poisoned| poisoned.into_inner());
    guard.clear();
}

fn execute_ask(args: self::AskArgs) -> ::core::result::Result<self::AskOutput, ::rig::tool::ToolError> {
    let mut answers = ::std::vec::Vec::new();
    for question in args.questions {
        let answer = self::take_answer(&question.id).ok_or_else(|| {
            let message = ::std::format!(
                "no operator available to answer question {}",
                question.id
            );
            message.into_tool_error()
        })?;
        answers.push(answer);
    }
    let output = self::AskOutput { answers };
    ::core::result::Result::Ok(output)
}

fn take_answer(question_id: &::core::primitive::str) -> ::core::option::Option<self::AskAnswer> {
    let mut guard = self::ANSWER_SINK.write().unwrap_or_else(|poisoned| poisoned.into_inner());
    let position = guard
        .iter()
        .position(|answer| answer.id == question_id)?;
    guard.remove(position)
}
