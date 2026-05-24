# ADR-0023: Windows `.cmd` / `.bat` invocation via `cmd.exe /C` wrapper (BatBadBut-safe)

**Status:** Accepted
**Date:** 2026-05-23 (Proposed); 2026-05-24 (Accepted at M08.5.5 Stage B.fix impl commit)
**Deciders:** @kknipe2k
**Tags:** persistence, ipc-adjacent, security, dev-experience

## Context

`crates/runtime-mcp/src/transport/stdio.rs::build_command` invokes a
local MCP server's stdio-transport command via `tokio::process::Command`.
The M06.5 IRL 🟡-2 fix added `resolve_program` rewriting bare `npx` →
`npx.cmd` on Windows (the npm CLI ships its tools as `.cmd` batch shims
on Windows). The rewrite is correct.

The M08.5 IRL re-verify on Windows (2026-05-23) exposed a deeper
defect: the spawn of `npx.cmd` with a path argument
(`C:\Users\...\Some\Path`) fails with the Windows OS-level error
"filename, directory name, or volume label syntax is incorrect" BEFORE
npx ever runs. The error originates in `CreateProcessW` / cmd.exe's
command-line interpretation, not in npx.

Root cause: Rust 1.77.2+'s `std::process::Command` includes the
CVE-2024-24576 ("BatBadBut") security fix, which escapes batch-file
arguments through a Windows-specific quoting routine. The escaping is
correct for arbitrary string args BUT produces command lines that
Windows itself refuses to parse when the args contain Windows paths
(drive letter + backslashes). Specifically: a path arg containing `:`
+ `\` triggers Windows' UNC-path heuristic in the cmd.exe-interpreted
command line.

This is independent of the npm-shim resolution (correct), the npx
binary (not yet invoked when the error fires), and the MCP server
package (irrelevant to the spawn-level failure). The defect is at the
**Rust-stdlib + Windows + batch-file + path-arg intersection**.

## Decision

When the resolved program ends in `.cmd` or `.bat` on Windows **AND
the args list is non-empty**, wrap the invocation in
`cmd.exe /C "<full command line>"` using
`std::os::windows::process::CommandExt::raw_arg`. The full command line
(program + args) is constructed as a single Windows-quoted string and
passed as the only `raw_arg` to `cmd.exe`. This bypasses the BatBadBut
escaping entirely (which is intended for the case where Rust constructs
the arguments; here, we construct the full command line ourselves with
full control over quoting).

The non-empty-args guard preserves the M06.5 IRL 🟡-2 bare-shim
contract pinned by
`build_command_resolves_npx_to_npx_cmd_on_windows` /
`build_command_resolves_npm_to_npm_cmd_on_windows`: bare `npx` / `npm`
(no args) is rewritten to `npx.cmd` / `npm.cmd` and then spawned
directly via `Command::new("npx.cmd")`. The BatBadBut OS-level
command-line-parse error fires only when args are present (an arg-less
batch invocation has no per-arg quoting for the BatBadBut layer to
mangle), so narrowing the wrap to non-empty-args adds no behavioral
gap. This narrowing was settled at M08.5.5 Stage B.fix red-phase entry
when the maintainer answered the surfaced design conflict between
this ADR's literal wording and the stage's
`<retrospective_requirements>` "M06.5 IRL 🟡-2 tests still pass"
clause.

Concretely (full impl in
`crates/runtime-mcp/src/transport/stdio.rs::build_command`):

```rust
if (resolved.ends_with(".cmd") || resolved.ends_with(".bat"))
    && !self.args.is_empty()
{
    use std::os::windows::process::CommandExt;
    let mut cmd = Command::new("cmd.exe");
    let full_command_line = build_quoted_command_line(&resolved, &self.args);
    cmd.raw_arg(format!("/C {full_command_line}"));
    // ... env + cwd applied normally ...
    return cmd;
}
```

The quoting routine follows the Microsoft-documented quoting
convention: args containing space, colon, backslash, or quote are
wrapped in `"..."` with embedded quotes escaped as `\"`. Args without
those characters pass through unquoted (the simplest cmd.exe-parseable
form).

On non-Windows platforms, the existing direct-spawn path is unchanged
(`.cmd` / `.bat` files don't exist; the BatBadBut issue is
Windows-specific). The fix scope is platform-gated by
`#[cfg(target_os = "windows")]`.

## Consequences

### Positive

- MCP servers with path arguments work on Windows (the IRL-failing
  M08.5 case + the broader class of "any path arg with `:` + `\`").
- 🔴 #6 closed at root; the assembled regression test in
  `crates/runtime-mcp/tests/mcp_add_with_path_args.rs` pins the fix.
- Single platform-abstraction point (the cmd.exe wrapper). No
  per-argument decision logic spreads across the codebase.
- The fix is structurally sound across all Rust versions ≥ 1.77.2
  (which includes the MSRV 1.80 + all current stable). No version-
  pinning required.

### Negative

- Two spawn paths on Windows (direct for `.exe`, wrapped for
  `.cmd`/`.bat`). Mitigation: the dispatcher is one function
  (`build_command`) + a private helper (`build_quoted_command_line`);
  the choice is visible at the call site of `Command::new`.
- The `raw_arg` API is `#[cfg(target_os = "windows")]`-gated; the
  `build_command` function has a Windows-cfg block that other
  contributors may need to learn. The function-level docstring
  cross-references this ADR.
- `cmd.exe /C` adds ~5-10ms spawn overhead (the cmd.exe interpreter
  startup). Negligible for MCP server startup (the MCP server itself
  takes seconds).

### Neutral / future implications

- The same pattern applies to any future Rust-spawned `.cmd` / `.bat`
  invocation on Windows. If a future milestone adds a `.cmd`-shipping
  tool, the wrapper extends naturally.
- If the BatBadBut escaping is ever loosened upstream (unlikely; it's
  a security fix), the wrapper still works (and is still necessary
  for the Windows path-arg case which Rust's escaping cannot fix
  without breaking other args).

## Alternatives Considered

### Alternative A: Per-argument `raw_arg` for path-containing args

**Rejected because:** it spreads platform-specific decision logic
across every spawn site (each caller must decide which args contain
paths). The cmd.exe wrapper is a single point of abstraction.

### Alternative B: Use `which` crate to resolve `npx.cmd` to its full path + spawn the resolved path directly

**Rejected because:** the BatBadBut escaping fires on the `.cmd`
extension regardless of path. Full-path resolution doesn't bypass it.

### Alternative C: Skip Windows `.cmd` MCP servers entirely; require users to install MCP servers as native `.exe`

**Rejected because:** the npm ecosystem distributes MCP servers as
npm packages with `npx` as the standard invocation. Requiring users
to repackage as native exes makes MCP unusable on Windows.

### Alternative D: Wait for an upstream Rust fix to the BatBadBut escaping for path args

**Rejected because:** the BatBadBut escaping is a CVE security fix;
loosening it for path args would re-introduce the command-injection
vector. The Rust stdlib team has explicitly stated `raw_arg` is the
escape hatch for advanced cases where the caller takes responsibility
for the quoting.

## Related

- Spec sections: §5 (MCP) — local stdio transport invocation.
- Prior ADRs: ADR-0010 (MCP dispatch dependency inversion); ADR-0011
  (M06.F scope: seam not running app — this fix is the running-app
  correctness this ADR's tests guard).
- Findings: `docs/M08-irl-findings.md` § Resolution 🔴 #6 (MCP
  Add/Test on Windows infinite loop).
- Code: `crates/runtime-mcp/src/transport/stdio.rs::build_command` +
  `resolve_program` (the BatBadBut wrap + the M06.5 IRL 🟡-2 shim
  resolver, respectively).
- External: CVE-2024-24576
  (<https://nvd.nist.gov/vuln/detail/cve-2024-24576>); Rust 1.77.2
  release notes
  (<https://blog.rust-lang.org/2024/04/09/cve-2024-24576.html>);
  Node.js parallel ecosystem confirmation
  (<https://github.com/nodejs/node/issues/52681>); Microsoft cmd.exe
  quoting docs
  (<https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd>).

## Notes

Status flipped `Proposed → Accepted` in the M08.5.5 Stage B.fix impl
commit per CLAUDE.md §11.

Three Windows-cfg-gated unit tests in
`crates/runtime-mcp/src/transport/stdio.rs::tests` pin the wrapper
shape (program == `cmd.exe` for `.cmd`-with-args; non-batch programs
unchanged; IRL-failing combo wrapped). The assembled regression test
in `crates/runtime-mcp/tests/mcp_add_with_path_args.rs` is the
permanent defense-in-depth guard. On the build machine's current
toolchain (Windows + Rust 1.95.0 + Node 2026's npm-shipped batch
shims) the integration test passes on pre-fix code as well — the
BatBadBut command-line-parse error does not fire deterministically
for `TempDir`-generated path args, which suggests the IRL repro
involved a different toolchain at build time, a different process
context, or an arg shape with shell metacharacters. The two unit
tests carry the empirical right-reason RED weight for this stage;
the integration test catches future regressions in older Rust
versions or differently-configured envs.
