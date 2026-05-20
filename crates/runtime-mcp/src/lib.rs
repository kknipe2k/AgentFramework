//! MCP client crate — protocol-layer boundary for the runtime's external-
//! tooling surface (spec §5).
//!
//! Wraps the official `rmcp` crate (Model Context Protocol Rust SDK,
//! `modelcontextprotocol/rust-sdk`) with a small `Transport` + `Connection`
//! trait pair so downstream consumers (Stage C lifecycle + Stage D namespace
//! resolution + Stage D dispatch) program against runtime-mcp's stable
//! surface instead of rmcp's evolving API.
//!
//! Two production transport implementations:
//!
//! - [`transport::StdioTransport`] — spawns a child process speaking
//!   JSON-RPC over stdin/stdout via `rmcp::transport::TokioChildProcess`.
//!   Used for local MCP servers (`@modelcontextprotocol/server-filesystem`,
//!   `@modelcontextprotocol/server-git`, …).
//! - [`transport::HttpTransport`] — connects to a remote streamable-HTTP
//!   MCP server via `rmcp::transport::StreamableHttpClientTransport` per
//!   MCP specification 2025-11-25.
//!
//! Plus an in-process mock for unit testing, gated behind the
//! `test-helpers` cargo feature:
//!
//! - `transport::MockTransport` — scripted tool list + scripted
//!   call-result responses; no rmcp involvement. Used by this crate's own
//!   unit tests + downstream consumers' tests for deterministic behavior.
//!
//! See `agent-runtime-spec.md` §5 (MCP Manager) + §5a (Tool Namespace
//! Resolution) for the architectural role; see `docs/build-prompts/
//! M06-mcp-basic.md` Stage B for the staging plan.

pub mod client;
pub mod dispatch;
pub mod error;
pub mod namespace;
pub mod transport;

pub use client::{
    InMemorySecretStore, KeyringSecretStore, LifecycleError, McpClient, McpServerRecord,
    McpServerSummary, Registry, SecretStore, ServerStatus, MCP_KEYRING_SERVICE,
};
pub use dispatch::{mcp_tool_capability, ConnectionResolver, McpDispatcher};
pub use error::McpError;
pub use namespace::{
    AliasError, Aliases, NamespaceError, NamespaceResolver, NewAmbiguity, ResolvedTool,
};
pub use transport::{Connection, McpTool, Transport};
