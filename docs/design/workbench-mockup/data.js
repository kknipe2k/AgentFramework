/* =====================================================================
   data.js — ARIA-style framework graph + a scripted live-run event
   stream. Plain JS, attached to window.ARData. Consumed by the
   live-graph execution view and the Builder/Tester surfaces.

   The event objects mirror schemas/event.v1.json variants (session_start,
   agent_spawned, stream_text, tool_invoked, tool_result, skill_loaded,
   verify_started/passed, plan_created, hitl_requested/resolved,
   capability_violation, tool_missing, token_usage, budget_warn,
   budget_suspended). The reducer folds events[0..i] so scrubbing is
   reversible.
   ===================================================================== */
(function () {
  // ---- Framework graph (positions on a 1180 x 560 field) -------------
  const NODES = [
    { id: 'orchestrator', kind: 'agent', label: 'orchestrator', sub: 'agent · sonnet-4-6', x: 410, y: 24 },
    { id: 'planner',      kind: 'agent', label: 'planner',      sub: 'agent · sonnet-4-6', x: 150, y: 165 },
    { id: 'implementer',  kind: 'agent', label: 'implementer',  sub: 'agent · sonnet-4-6', x: 410, y: 165 },
    { id: 'verifier',     kind: 'agent', label: 'verify-app',   sub: 'agent · haiku-4-5',  x: 680, y: 165 },

    { id: 'plan_review',  kind: 'hitl',  label: 'plan approval', sub: 'on_plan_approval',  x: 120, y: 322 },
    { id: 'read_file',    kind: 'tool',  label: 'read_file',    sub: 'builtin',            x: 280, y: 322 },
    { id: 'write_file',   kind: 'tool',  label: 'write_file',   sub: 'builtin',            x: 430, y: 322 },
    { id: 'rag',          kind: 'skill', label: 'retrieval',    sub: 'skill · auto',       x: 575, y: 322 },
    { id: 'verify_std',   kind: 'hook',  label: 'verify_standard', sub: 'verify · post',   x: 700, y: 322 },

    { id: 'fetch_prs',    kind: 'tool',  label: 'fetch_prs',    sub: 'mcp · github',       x: 360, y: 462 },
    { id: 'secret_scan',  kind: 'hook',  label: 'secret_scan',  sub: 'verify · pre_commit',x: 700, y: 462 },
  ];

  // ---- Capability edges (kind drives styling — rule 4 / IRL #26) -----
  const EDGES = [
    { from: 'orchestrator', to: 'planner',     kind: 'agent' },
    { from: 'orchestrator', to: 'implementer', kind: 'agent' },
    { from: 'orchestrator', to: 'verifier',    kind: 'agent' },
    { from: 'planner',      to: 'plan_review', kind: 'hitl'  },
    { from: 'implementer',  to: 'read_file',   kind: 'tool'  },
    { from: 'implementer',  to: 'write_file',  kind: 'tool'  },
    { from: 'implementer',  to: 'rag',         kind: 'skill' },
    { from: 'implementer',  to: 'fetch_prs',   kind: 'tool'  },
    { from: 'verifier',     to: 'verify_std',  kind: 'hook'  },
    { from: 'verifier',     to: 'secret_scan', kind: 'hook'  },
  ];

  // ---- Scripted event stream (t = ms offset from session start) ------
  const EVENTS = [
    { t: 0,     ev: { type: 'session_start', session_id: 's-7f3a', framework: 'aria', model: 'claude-sonnet-4-6' } },
    { t: 120,   ev: { type: 'agent_spawned', agent_id: 'orchestrator', agent_name: 'orchestrator', parent_id: null } },
    { t: 240,   ev: { type: 'stream_text', agent_id: 'orchestrator', text: 'Decomposing the task and delegating to specialist agents. Routing to STANDARD mode.' } },
    { t: 900,   ev: { type: 'agent_spawned', agent_id: 'planner', agent_name: 'planner', parent_id: 'orchestrator' } },
    { t: 1100,  ev: { type: 'stream_text', agent_id: 'planner', text: 'Drafting a 4-task plan: (1) read the existing config, (2) implement the parser, (3) add tests, (4) verify the build.' } },
    { t: 1800,  ev: { type: 'plan_created', plan_id: 'p-1', title: 'Implement config parser', task_count: 4, approval_required: true } },
    { t: 2000,  ev: { type: 'token_usage', input: 1840, output: 420, model: 'claude-sonnet-4-6', cost_usd: 0.039 } },
    { t: 2100,  ev: { type: 'hitl_requested', prompt_id: 'h-1', trigger: 'on_plan_approval', agent_id: 'planner', question: 'Approve this 4-task plan before execution?', options: ['approve', 'revise', 'abort'], ui_variant: 'panel', timeout_at_unix_ms: 0 }, _node: 'plan_review' },
    { t: 4200,  ev: { type: 'hitl_resolved', prompt_id: 'h-1', choice: 'approve', duration_ms: 2100 }, _node: 'plan_review' },
    { t: 4350,  ev: { type: 'plan_approved', plan_id: 'p-1', approved_by: 'user' } },
    { t: 4800,  ev: { type: 'agent_spawned', agent_id: 'implementer', agent_name: 'implementer', parent_id: 'orchestrator', narrowed_from: ['read:src:glob:src/**', 'write:src:glob:src/**'] } },
    { t: 5000,  ev: { type: 'skill_loaded', agent_id: 'implementer', skill_name: 'retrieval', mode: 'auto' }, _node: 'rag' },
    { t: 5200,  ev: { type: 'stream_text', agent_id: 'implementer', text: 'Loaded the retrieval skill. Reading the current config to match existing conventions before writing.' } },
    { t: 5600,  ev: { type: 'tool_invoked', agent_id: 'implementer', tool_name: 'read_file', source: 'builtin', input: { path: 'src/config/parser.toml' } } },
    { t: 6400,  ev: { type: 'tool_result', agent_id: 'implementer', tool_name: 'read_file', output: '# parser config\nstyle = "snake_case"\nstrict = true\nmax_depth = 6', duration_ms: 142, tokens_in: 86, tokens_out: 0 } },
    { t: 6700,  ev: { type: 'stream_text', agent_id: 'implementer', text: 'Config uses snake_case keys with strict mode on. Writing the parser module to honour that convention.' } },
    { t: 7200,  ev: { type: 'tool_invoked', agent_id: 'implementer', tool_name: 'write_file', source: 'builtin', input: { path: 'src/config/parser.rs', bytes: 2480 } } },
    { t: 7900,  ev: { type: 'tool_result', agent_id: 'implementer', tool_name: 'write_file', output: 'wrote 2,480 bytes to src/config/parser.rs', duration_ms: 88, tokens_in: 0, tokens_out: 0 } },
    { t: 8200,  ev: { type: 'token_usage', input: 5120, output: 1980, model: 'claude-sonnet-4-6', cost_usd: 0.121 } },
    { t: 8600,  ev: { type: 'agent_spawned', agent_id: 'verifier', agent_name: 'verify-app', parent_id: 'orchestrator' } },
    { t: 8900,  ev: { type: 'verify_started', hook_id: 'verify_std', category: 'verify', firing_point: 'post_task', level: 'standard' } },
    { t: 9100,  ev: { type: 'stream_text', agent_id: 'verifier', text: 'Running the standard verify hook: typecheck, lint, unit tests.' } },
    { t: 13200, ev: { type: 'verify_passed', hook_id: 'verify_std', duration_ms: 4180, output_preview: '12 passed · 0 failed · clippy clean' } },
    { t: 13600, ev: { type: 'verify_started', hook_id: 'secret_scan', category: 'verify', firing_point: 'pre_commit', level: null } },
    { t: 14200, ev: { type: 'capability_violation', agent_id: 'verifier', capability_kind: 'network', requested_action: 'secret_scan attempted to POST scan results to an external endpoint', declared_scope: 'grants cover read:src/** only; no network egress' }, _node: 'secret_scan' },
    { t: 14400, ev: { type: 'budget_warn', spent_usd: 0.16, cap_usd: 0.30, percent: 53 } },
    { t: 15000, ev: { type: 'stream_text', agent_id: 'implementer', text: 'Tests pass. Fetching the related pull requests to cross-link the change before committing.' } },
    { t: 15400, ev: { type: 'tool_invoked', agent_id: 'implementer', tool_name: 'fetch_prs', source: 'mcp', server: 'github', input: { repo: 'agent-runtime', state: 'open' } } },
    { t: 15900, ev: { type: 'tool_missing', agent_id: 'implementer', tool_name: 'fetch_prs', severity: 'critical', suggested_action: "Install an MCP server that provides 'fetch_prs', then click Resume.", requested_via: 'loader' } },
    { t: 16000, ev: { type: 'budget_suspended', spent_usd: 0.16, cap_usd: 0.30 } },
  ];

  // ---- JSON for the Builder JSON tab (trimmed framework excerpt) -----
  const FRAMEWORK_JSON = `{
  "name": "aria",
  "version": "1.0.0",
  "model": { "provider": "anthropic", "id": "claude-sonnet-4-6" },
  "agents": [
    {
      "id": "orchestrator@1.0.0",
      "spawns": ["planner@1.0.0", "implementer@1.0.0", "verify-app@1.0.0"],
      "capabilities": { "read": ["src/**"], "write": ["src/**"], "exec": ["bash"] }
    },
    {
      "id": "implementer@1.0.0",
      "allowed_tools": ["read_file", "write_file", "fetch_prs"],
      "allowed_skills": ["retrieval"],
      "capabilities": { "read": ["src/**"], "write": ["src/**"] }
    }
  ],
  "tools": [
    { "id": "read_file",  "source": "builtin" },
    { "id": "write_file", "source": "builtin" },
    { "id": "fetch_prs",  "source": "mcp", "server": "github" }
  ],
  "skills": [{ "id": "retrieval", "path": "skills/retrieval.md" }],
  "hooks": { "post_task": [{ "id": "verify_standard" }], "pre_commit": [{ "id": "secret_scan" }] }
}`;

  // Plain-English error translations (rules 6 + 7; IRL #5/#15) ----------
  const ERROR_TRANSLATIONS = [
    {
      raw: '(root): data did not match any variant of untagged enum FrameworkAgentsItem',
      plain: 'Agent “demo-agent@1.0.0” is missing its required capabilities block, and its id uses characters the schema rejects.',
      node: 'demo-agent',
      fix: 'Add a capabilities block and rename the id to letters, digits and hyphens (e.g. demo-agent), then re-validate.',
    },
  ];

  window.ARData = { NODES, EDGES, EVENTS, FRAMEWORK_JSON, ERROR_TRANSLATIONS };
})();
