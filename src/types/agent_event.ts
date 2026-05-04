// TypeScript discriminated union mirroring runtime_core::AgentEvent.
//
// Hand-mirrored from `crates/runtime-core/src/event.rs` (v0.1 subset M02
// Stage D emits). The full union has more variants but Stage E only needs
// what the smoke session emits + a few related types the renderer pattern-
// matches against.
//
// **M03 carry-forward (per CLAUDE.md §14):** This file should be regenerated
// from `schemas/event.v1.json` via `cargo xtask regenerate-types` once the
// schema lands. Hand-mirrored types drift; codegen is the source-of-truth
// pattern the project uses for `runtime-core` types and frontend types alike.

export type ToolSource = 'builtin' | 'mcp' | 'generated';

export type AgentEvent =
  | { type: 'session_start'; session_id: string; framework: string; model: string }
  | {
      type: 'agent_spawned';
      agent_id: string;
      agent_name: string;
      parent_id: string | null;
      session_id: string;
    }
  | { type: 'agent_complete'; agent_id: string; result: string }
  | { type: 'agent_error'; agent_id: string; error: string }
  | {
      type: 'tool_invoked';
      agent_id: string;
      tool_name: string;
      source: ToolSource;
      server: string | null;
      input: unknown;
    }
  | {
      type: 'tool_result';
      agent_id: string;
      tool_name: string;
      output: unknown;
      duration_ms: number;
    }
  | { type: 'stream_text'; agent_id: string; text: string }
  | {
      type: 'decision_record';
      agent_id: string;
      decision: string;
      rationale: string;
      tool_used: string | null;
    }
  | { type: 'task_started'; plan_id: string; task_id: string; agent_id: string }
  | { type: 'task_completed'; plan_id: string; task_id: string; duration_ms: number };
