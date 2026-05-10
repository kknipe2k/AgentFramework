-- Migration 001 — plans + tasks tables.
--
-- Spec §3a (Plan & Task primitive) + spec §10 (Persistence Layer DDL).
-- M04 Stage B authored these tables; the drone-internal
-- `plan_projector::project_signal` UPSERTs rows from plan/task signals.
-- The renderer's PlanNode + TaskNode read from these tables (Stage C
-- wires the visual surface).
--
-- Idempotent: every CREATE statement uses IF NOT EXISTS. The migration
-- runner's `_migrations` row is the authoritative "applied" signal.

CREATE TABLE IF NOT EXISTS plans (
  id TEXT PRIMARY KEY,
  session_id TEXT NOT NULL,
  title TEXT NOT NULL,
  description TEXT,
  status TEXT NOT NULL
    CHECK (status IN ('pending_approval', 'approved', 'in_progress', 'awaiting_replan', 'complete', 'aborted')),
  approval_required INTEGER NOT NULL,
  loop_policy TEXT NOT NULL
    CHECK (loop_policy IN ('one_shot', 'fresh_context_per_task', 'continuous')),
  hitl_checkpoints TEXT NOT NULL DEFAULT '[]',
  risks TEXT NOT NULL DEFAULT '[]',
  created_by TEXT,
  created_at INTEGER NOT NULL,
  approved_at INTEGER,
  completed_at INTEGER,
  FOREIGN KEY (session_id) REFERENCES sessions(id)
);
CREATE INDEX IF NOT EXISTS idx_plans_session ON plans(session_id);
CREATE INDEX IF NOT EXISTS idx_plans_status ON plans(status);

CREATE TABLE IF NOT EXISTS tasks (
  id TEXT PRIMARY KEY,
  plan_id TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL
    CHECK (status IN ('pending', 'running', 'done', 'failed', 'blocked', 'skipped', 'escalated')),
  hitl INTEGER NOT NULL DEFAULT 0,
  hitl_reason TEXT,
  failure_count INTEGER NOT NULL DEFAULT 0,
  max_failures INTEGER NOT NULL DEFAULT 3,
  files_affected TEXT,
  acceptance_criteria TEXT,
  created_at INTEGER NOT NULL,
  started_at INTEGER,
  completed_at INTEGER,
  estimated_minutes INTEGER,
  actual_minutes INTEGER,
  FOREIGN KEY (plan_id) REFERENCES plans(id)
);
CREATE INDEX IF NOT EXISTS idx_tasks_plan_id ON tasks(plan_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
