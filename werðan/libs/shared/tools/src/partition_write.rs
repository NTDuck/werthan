//! Partition-scoped mutation tool restricting writes to the declared target
//! partition root (VAL-TOOL-02). The plain write tool stays general. This
//! variant enforces the methodology rule that mutation tools write only
//! inside their declared target partition.

use crate::DisplayExt;

/// Partition-scoped write tool bounding every mutation to one partition root.
#[derive(::core::fmt::Debug, ::core::clone::Clone)]
pub struct PartitionWriteTool {
    /// Canonical partition root that bounds every accepted write.
    partition_root: ::std::path::PathBuf,
}

impl self::PartitionWriteTool {
    /// Construct the tool with one declared partition root.
    pub fn new(partition_root: ::std::path::PathBuf) -> Self {
        Self { partition_root }
    }
}

impl ::rig::tool::Tool for self::PartitionWriteTool {
    const NAME: &'static ::core::primitive::str = "partition_write";

    type Error = ::rig::tool::ToolError;
    type Args = crate::WriteArgs;
    type Output = crate::WriteOutput;

    async fn definition(&self, _prompt: ::std::string::String) -> ::rig::completion::ToolDefinition {
        let name = Self::NAME.to_string();
        let description = "Write content to a file inside the declared target partition.".to_string();
        let parameters = ::serde_json::json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Partition-relative path to the destination file."
                },
                "content": {
                    "type": "string",
                    "description": "Content to write into the file."
                }
            },
            "required": ["path"]
        });
        ::rig::completion::ToolDefinition {
            name,
            description,
            parameters,
        }
    }

    async fn call(&self, args: Self::Args) -> ::core::result::Result<Self::Output, Self::Error> {
        self::execute_partition_write(&self.partition_root, args).await
    }
}

async fn execute_partition_write(
    partition_root: &::std::path::Path,
    args: crate::WriteArgs,
) -> ::core::result::Result<crate::WriteOutput, ::rig::tool::ToolError> {
    let target_relative = ::std::path::Path::new(&args.path);
    let is_absolute = target_relative.is_absolute();
    let escapes = target_relative
        .components()
        .any(|component| ::core::matches!(component, ::std::path::Component::ParentDir));
    if is_absolute || escapes {
        let detail = ::std::format!(
            "write rejected: {} lies outside the declared partition root {}",
            args.path,
            partition_root.display()
        );
        let err = ::std::io::Error::other(detail);
        let boxed = ::std::boxed::Box::new(err);
        return ::core::result::Result::Err(::rig::tool::ToolError::ToolCallError(boxed));
    }
    let root_prepare_res = ::tokio::fs::create_dir_all(partition_root).await;
    if let ::core::result::Result::Err(io_err) = root_prepare_res {
        let err = io_err.into_tool_error();
        return ::core::result::Result::Err(err);
    }
    let root_canonical_res = ::tokio::fs::canonicalize(partition_root).await;
    let root_canonical = match root_canonical_res {
        ::core::result::Result::Ok(value) => value,
        ::core::result::Result::Err(io_err) => {
            let err = io_err.into_tool_error();
            return ::core::result::Result::Err(err);
        },
    };
    let full_target = root_canonical.join(target_relative);
    let parent_opt = full_target.parent();
    if let ::core::option::Option::Some(parent_dir) = parent_opt {
        let mkdir_res = ::tokio::fs::create_dir_all(parent_dir).await;
        if let ::core::result::Result::Err(io_err) = mkdir_res {
            let err = io_err.into_tool_error();
            return ::core::result::Result::Err(err);
        }
    }
    let payload = args.content.as_deref().unwrap_or("");
    let payload_bytes = payload.as_bytes();
    let bytes_written = payload_bytes.len();
    let write_res = ::tokio::fs::write(&full_target, payload_bytes).await;
    if let ::core::result::Result::Err(io_err) = write_res {
        let err = io_err.into_tool_error();
        return ::core::result::Result::Err(err);
    }
    // A symlink that later resolves outside the partition must also reject.
    let final_canonical_res = ::tokio::fs::canonicalize(&full_target).await;
    if let ::core::result::Result::Ok(final_canonical) = final_canonical_res {
        let contained = final_canonical.starts_with(&root_canonical);
        if !contained {
            let detail = ::std::format!(
                "write rejected: {} resolves outside the declared partition root {}",
                final_canonical.display(),
                root_canonical.display()
            );
            let err = ::std::io::Error::other(detail);
            let boxed = ::std::boxed::Box::new(err);
            return ::core::result::Result::Err(::rig::tool::ToolError::ToolCallError(boxed));
        }
    }
    let path = full_target.display().to_string();
    let output = crate::WriteOutput {
        path,
        bytes_written,
    };
    ::core::result::Result::Ok(output)
}
