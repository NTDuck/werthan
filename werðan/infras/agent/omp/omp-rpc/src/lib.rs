//! Rust client for the omp RPC protocol: JSON lines over the stdio of `omp
//! --mode rpc`.
//!
//! [`wire`] is generated from the wire schema by `bun run gen:rpc`; [`client`]
//! is a blocking transport (protocol v2 chunking, prompt waits, host tools,
//! host URIs).

#[rustfmt::skip]
pub mod wire;
pub mod client;
mod frame;
pub mod host_tool;
pub mod host_uri;
pub mod turn;

pub use client::encode_command;
pub use client::Client;
pub use client::ClientOptions;
pub use client::Error;
pub use client::Event;
pub use client::DEFAULT_PROMPT_TIMEOUT;
pub use host_tool::HostTool;
pub use host_tool::HostToolContext;
pub use host_tool::HostToolError;
pub use host_tool::HostToolOutput;
pub use host_uri::HostUri;
pub use host_uri::HostUriContext;
pub use host_uri::HostUriError;
pub use host_uri::HostUriRead;
pub use turn::PromptTurn;
pub use wire::*;
