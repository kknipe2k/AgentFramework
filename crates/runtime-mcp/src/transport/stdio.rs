//! Stdio child-process transport (spec §5).
//!
//! Wraps `rmcp::transport::TokioChildProcess` (rmcp 1.7.0 with the
//! `transport-child-process` feature). The child process speaks
//! JSON-RPC 2.0 over stdin/stdout; `rmcp::ServiceExt::serve` performs
//! the `initialize` handshake and returns a live
//! `rmcp::service::RunningService<RoleClient, ()>` we wrap in
//! [`StdioConnection`].
//!
//! Coverage holdout: [`StdioTransport::connect`]'s happy path (a real
//! subprocess speaking JSON-RPC successfully) is structurally
//! untestable from in-process unit tests on every CI platform — it
//! requires either a real MCP server binary or a feature-gated
//! integration test. The pure logic ([`StdioTransport::build_command`])
//! IS unit-testable and is the seam targeted by this crate's tests.
//! Same OS-call-holdout pattern as `runtime-main::providers::anthropic`
//! and `runtime-sandbox::seccomp`/`landlock` per CLAUDE.md §5.

use std::collections::BTreeMap;
use std::path::PathBuf;

use async_trait::async_trait;
use rmcp::model::CallToolRequestParams;
use rmcp::service::RunningService;
use rmcp::transport::TokioChildProcess;
use rmcp::ServiceExt;
use serde_json::Value;
use tokio::process::Command;

use super::{Connection, McpTool, Transport};
use crate::error::McpError;

/// Stdio (child-process) transport for a local MCP server.
#[derive(Debug, Clone)]
pub struct StdioTransport {
    command: String,
    args: Vec<String>,
    env: BTreeMap<String, String>,
    cwd: Option<PathBuf>,
}

impl StdioTransport {
    /// New transport spawning `command` with no args / no env / inherited
    /// cwd. Chain [`Self::with_args`] / [`Self::with_env`] / [`Self::with_cwd`]
    /// to configure further.
    #[must_use]
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
            cwd: None,
        }
    }

    /// Replace the args list.
    #[must_use]
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Replace the env map.
    #[must_use]
    pub fn with_env(mut self, env: BTreeMap<String, String>) -> Self {
        self.env = env;
        self
    }

    /// Set the subprocess working directory.
    #[must_use]
    pub fn with_cwd(mut self, cwd: PathBuf) -> Self {
        self.cwd = Some(cwd);
        self
    }

    /// Build a `tokio::process::Command` from this transport's config,
    /// without spawning. This is the pure-logic seam tested by the
    /// `build_command_*` test family.
    pub(crate) fn build_command(&self) -> Command {
        let mut cmd = Command::new(&self.command);
        for arg in &self.args {
            cmd.arg(arg);
        }
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        if let Some(c) = &self.cwd {
            cmd.current_dir(c);
        }
        cmd
    }
}

#[async_trait]
impl Transport for StdioTransport {
    async fn connect(&self) -> Result<Box<dyn Connection>, McpError> {
        let cmd = self.build_command();
        let proc = TokioChildProcess::new(cmd).map_err(McpError::connect_failed)?;
        let service = ().serve(proc).await.map_err(McpError::connect_failed)?;
        Ok(Box::new(StdioConnection { service }))
    }
}

/// Live MCP connection over stdin/stdout of a child process.
pub struct StdioConnection {
    service: RunningService<rmcp::RoleClient, ()>,
}

#[async_trait]
impl Connection for StdioConnection {
    async fn list_tools(&self) -> Result<Vec<McpTool>, McpError> {
        let result = self
            .service
            .list_all_tools()
            .await
            .map_err(McpError::transport)?;
        Ok(result.into_iter().map(rmcp_tool_to_mcp_tool).collect())
    }

    async fn call_tool(&self, name: &str, arguments: Value) -> Result<Value, McpError> {
        let mut params = CallToolRequestParams::new(name.to_string());
        if let Some(args) = value_to_object(arguments) {
            params = params.with_arguments(args);
        }
        let result = self
            .service
            .call_tool(params)
            .await
            .map_err(McpError::transport)?;
        serde_json::to_value(result).map_err(McpError::protocol)
    }

    async fn ping(&self) -> Result<(), McpError> {
        // rmcp 1.7.0's `Peer<RoleClient>` doesn't expose a dedicated
        // `ping` method (PingRequest exists in the MCP spec but rmcp
        // wraps it internally for the server→client direction only).
        // The simplest reliable liveness check is a `list_tools`
        // round-trip: the server must respond, which proves the
        // transport is alive. Stage C lifecycle's health-ping loop
        // can switch to a cheaper request if rmcp adds one upstream.
        self.service
            .list_tools(Option::default())
            .await
            .map_err(McpError::transport)?;
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), McpError> {
        // RunningService::cancel takes `self` by value — we hold by
        // shared reference (Connection trait is &self), so the best
        // we can do here is a no-op + rely on Drop to tear down the
        // subprocess cleanly. Stage C lifecycle owns the
        // StdioConnection by value and can call cancel directly via
        // a downcast path; the trait surface returns Ok.
        Ok(())
    }
}

fn rmcp_tool_to_mcp_tool(tool: rmcp::model::Tool) -> McpTool {
    let input_schema = serde_json::to_value(&*tool.input_schema).unwrap_or(Value::Null);
    McpTool {
        name: tool.name.to_string(),
        description: tool.description.map(|d| d.to_string()),
        input_schema,
    }
}

fn value_to_object(arguments: Value) -> Option<serde_json::Map<String, Value>> {
    match arguments {
        Value::Object(m) => Some(m),
        Value::Null => None,
        other => {
            // Wrap non-object payloads in {"value": other}; rmcp
            // requires Map for arguments. Stage D's dispatch layer
            // should be passing Map values; this is a defensive
            // shim with a single allocation.
            let mut m = serde_json::Map::new();
            m.insert("value".to_string(), other);
            Some(m)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_command_with_empty_args_env_cwd() {
        let t = StdioTransport::new("npx");
        assert_eq!(t.command, "npx");
        assert!(t.args.is_empty());
        assert!(t.env.is_empty());
        assert!(t.cwd.is_none());
    }

    #[test]
    fn with_args_replaces_arg_list() {
        let t = StdioTransport::new("npx").with_args(vec!["-y".into(), "server-filesystem".into()]);
        assert_eq!(t.args, vec!["-y", "server-filesystem"]);
    }

    #[test]
    fn with_env_replaces_env_map() {
        let mut env = BTreeMap::new();
        env.insert("RUST_LOG".to_string(), "debug".to_string());
        let t = StdioTransport::new("svr").with_env(env.clone());
        assert_eq!(t.env, env);
    }

    #[test]
    fn with_cwd_sets_working_directory() {
        let dir = PathBuf::from("/tmp/mcp");
        let t = StdioTransport::new("svr").with_cwd(dir.clone());
        assert_eq!(t.cwd, Some(dir));
    }

    #[test]
    fn build_command_program_matches_command_field() {
        let t = StdioTransport::new("echo");
        let cmd = t.build_command();
        let program = cmd.as_std().get_program();
        assert_eq!(program, std::ffi::OsStr::new("echo"));
    }

    #[test]
    fn build_command_includes_all_args_in_order() {
        let t = StdioTransport::new("echo").with_args(vec![
            "hello".into(),
            "world".into(),
            "-n".into(),
        ]);
        let cmd = t.build_command();
        let args: Vec<&std::ffi::OsStr> = cmd.as_std().get_args().collect();
        assert_eq!(
            args,
            vec![
                std::ffi::OsStr::new("hello"),
                std::ffi::OsStr::new("world"),
                std::ffi::OsStr::new("-n"),
            ]
        );
    }

    #[test]
    fn build_command_includes_all_env_keys() {
        let mut env = BTreeMap::new();
        env.insert("FOO".to_string(), "bar".to_string());
        env.insert("BAZ".to_string(), "qux".to_string());
        let t = StdioTransport::new("svr").with_env(env);
        let cmd = t.build_command();
        let envs: BTreeMap<&std::ffi::OsStr, Option<&std::ffi::OsStr>> =
            cmd.as_std().get_envs().collect();
        assert_eq!(
            envs.get(std::ffi::OsStr::new("FOO")).copied().flatten(),
            Some(std::ffi::OsStr::new("bar"))
        );
        assert_eq!(
            envs.get(std::ffi::OsStr::new("BAZ")).copied().flatten(),
            Some(std::ffi::OsStr::new("qux"))
        );
    }

    #[test]
    fn build_command_sets_cwd_when_configured() {
        let dir = std::env::temp_dir();
        let t = StdioTransport::new("svr").with_cwd(dir.clone());
        let cmd = t.build_command();
        assert_eq!(cmd.as_std().get_current_dir(), Some(dir.as_path()));
    }

    #[test]
    fn build_command_omits_cwd_when_unset_so_subprocess_inherits() {
        let t = StdioTransport::new("svr");
        let cmd = t.build_command();
        assert!(cmd.as_std().get_current_dir().is_none());
    }

    #[test]
    fn build_command_twice_in_sequence_both_succeed_with_same_program() {
        let t = StdioTransport::new("echo").with_args(vec!["hi".into()]);
        let cmd1 = t.build_command();
        let cmd2 = t.build_command();
        assert_eq!(cmd1.as_std().get_program(), cmd2.as_std().get_program());
    }

    #[tokio::test]
    async fn connect_returns_connect_failed_for_nonexistent_command() {
        let t = StdioTransport::new("this-command-definitely-does-not-exist-aaaa-bbbb-cccc");
        match t.connect().await {
            Ok(_) => panic!("expected connect to fail for nonexistent command"),
            Err(err) => assert!(
                err.is_connect_failure(),
                "expected ConnectFailed, got {err:?}"
            ),
        }
    }

    #[test]
    fn value_to_object_passes_through_object() {
        let v = serde_json::json!({"path": "/tmp"});
        let out = value_to_object(v).unwrap();
        assert_eq!(out.get("path").unwrap(), &serde_json::json!("/tmp"));
    }

    #[test]
    fn value_to_object_maps_null_to_none() {
        assert!(value_to_object(Value::Null).is_none());
    }

    #[test]
    fn value_to_object_wraps_non_object_under_value_key() {
        let v = serde_json::json!(42);
        let out = value_to_object(v).unwrap();
        assert_eq!(out.get("value").unwrap(), &serde_json::json!(42));
    }
}
