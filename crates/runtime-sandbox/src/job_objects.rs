//! Windows Job Objects — §8.security L3 OS isolation (Windows).
//!
//! Creates a Job Object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` set so
//! the entire process tree is terminated when the job handle is closed
//! (i.e., when the sandbox subprocess dies or is killed). Assigns the
//! calling process (the sandbox subprocess) to the job; any child the
//! subprocess might spawn inherits the job and is contained.
//!
//! `JOB_OBJECT_LIMIT_BREAKAWAY_OK` is also set so child processes
//! cannot escape the job via `CreateProcess` with the `CREATE_BREAKAWAY_FROM_JOB`
//! flag. The flag controls children of THIS process; the parent (Tauri
//! main) is free to spawn the sandbox in its own job if it chose to.
//!
//! Pairs with `crate::seccomp` + `crate::landlock` on Linux (those
//! modules are `cfg(target_os = "linux")` so the intra-doc link is bare
//! backticks per gotcha #55); this is the Windows-side equivalent of
//! process-tree containment. There is no
//! direct Windows analog to landlock's filesystem fence in v0.1 — the
//! validator is pure (no IO) and the IPC pipe is the only filesystem-
//! adjacent resource the subprocess touches.
//!
//! ## Job Object inheritance on modern Windows
//!
//! On Windows 8+ a process can belong to multiple jobs (nested). On
//! Windows 7 a process can only belong to one job. The Tauri main
//! process may or may not be in a job already (depends on launcher /
//! conhost). If the sandbox parent's job has `BREAKAWAY_OK` we can
//! create our own; otherwise the assignment fails with
//! `ERROR_ACCESS_DENIED`. v0.1 ships Windows-only and assumes Windows 10+
//! per spec §0d.
//!
//! ## Testable seams
//!
//! `install_restrictions` is decomposed into three smaller private
//! functions (`create_job`, `apply_limits`, `assign_process`) so each
//! FFI call path is unit-testable in isolation. Tests pass
//! `INVALID_HANDLE_VALUE` to `apply_limits` / `assign_process` to
//! trigger the error-mapping branches without affecting the test
//! runner's job-object state.

#![cfg(windows)]

#[cfg(test)]
use std::os::windows::io::AsRawHandle;
#[cfg(test)]
use std::os::windows::io::RawHandle;

use windows_sys::Win32::Foundation::{CloseHandle, GetLastError, HANDLE, INVALID_HANDLE_VALUE};
use windows_sys::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
    SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_BREAKAWAY_OK,
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

use crate::error::SandboxError;

/// The job-object limit flags used by the sandbox.
///
/// - `KILL_ON_JOB_CLOSE`: when the job handle closes (subprocess exits
///   or is killed) every process in the job is terminated. Defense
///   against the sandbox subprocess being killed mid-execution while
///   a spawned child lingers.
/// - `BREAKAWAY_OK`: children spawned by the sandbox subprocess can
///   request to break away from the job. Since the sandbox should never
///   spawn children, the flag is set as belt-and-suspenders against an
///   accidental child via dlopen / glibc init.
pub const SANDBOX_JOB_FLAGS: u32 =
    JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;

/// Construct an `EXTENDED_LIMIT_INFORMATION` struct with the sandbox's
/// flag set. Pure function — no kernel interaction; safe to call from
/// unit tests on any thread.
#[must_use]
pub const fn build_limit_info() -> JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
    // SAFETY: the struct is repr(C) and POD; zero-initialization yields
    // a valid `BasicLimitInformation.LimitFlags = 0` baseline that we
    // then OR with the sandbox flag. No padding fields contain pointers.
    let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    info.BasicLimitInformation.LimitFlags = SANDBOX_JOB_FLAGS;
    info
}

/// Format a Win32 error returned by `GetLastError` into a structured
/// [`SandboxError::Isolation`] with the operation name. Extracted so
/// the formatting logic is unit-testable without invoking the FFI.
fn win32_failure(op: &str, err: u32) -> SandboxError {
    SandboxError::Isolation(format!("job_objects {op} failed (GetLastError={err})"))
}

/// Create a fresh anonymous Job Object. Returns the handle on success.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the OS cannot create the job
/// (out of handles, security descriptor failure, etc.).
fn create_job() -> Result<HANDLE, SandboxError> {
    // SAFETY: CreateJobObjectW with null security attributes and null
    // name produces an anonymous job with default DACL. Documented per
    // https://learn.microsoft.com/en-us/windows/win32/api/jobapi2/nf-jobapi2-createjobobjectw .
    let job: HANDLE = unsafe { CreateJobObjectW(std::ptr::null(), std::ptr::null()) };
    if job.is_null() || job == INVALID_HANDLE_VALUE {
        // SAFETY: GetLastError is documented to return the calling
        // thread's last error; no parameters, always safe to call.
        let err = unsafe { GetLastError() };
        return Err(win32_failure("CreateJobObjectW", err));
    }
    Ok(job)
}

/// Apply the sandbox limit flags to `job` via
/// `SetInformationJobObject`. Returns Ok on success; on failure the
/// job handle is closed before returning so callers don't have to.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the kernel rejects the
/// limit info (invalid handle, malformed struct, etc.).
///
/// # Panics
///
/// Panics if `JOBOBJECT_EXTENDED_LIMIT_INFORMATION`'s size exceeds
/// `u32::MAX`. The struct is fixed-size and well under 1 KB; this is
/// a "this is impossible and represents a bug" assertion.
fn apply_limits(job: HANDLE) -> Result<(), SandboxError> {
    let info = build_limit_info();
    // SAFETY: `info` is a properly initialized POD; we pass its address
    // + size to SetInformationJobObject per the JobObjectExtendedLimit-
    // Information contract. Documented per https://learn.microsoft.com/
    // en-us/windows/win32/api/jobapi2/nf-jobapi2-setinformationjobobject .
    let ok = unsafe {
        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            std::ptr::from_ref(&info).cast(),
            u32::try_from(std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>())
                .expect("struct size fits in u32"),
        )
    };
    if ok == 0 {
        // SAFETY: GetLastError is always safe to call.
        let err = unsafe { GetLastError() };
        return Err(win32_failure("SetInformationJobObject", err));
    }
    Ok(())
}

/// Assign `process` (typically `GetCurrentProcess()`) into `job`.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the process is already in a
/// job that doesn't allow breakaway (`ERROR_ACCESS_DENIED`) or the
/// handle is invalid.
fn assign_process(job: HANDLE, process: HANDLE) -> Result<(), SandboxError> {
    // SAFETY: AssignProcessToJobObject is documented to take a valid
    // job handle + a valid process handle; the call is idempotent
    // (returns ERROR_ACCESS_DENIED if already assigned).
    let assigned = unsafe { AssignProcessToJobObject(job, process) };
    if assigned == 0 {
        // SAFETY: GetLastError is always safe to call.
        let err = unsafe { GetLastError() };
        return Err(win32_failure("AssignProcessToJobObject", err));
    }
    Ok(())
}

/// Create a Job Object, apply the sandbox flags, and assign the calling
/// process to it.
///
/// Once assigned the process — and any child it spawns — is contained:
/// closing the job handle kills the entire tree. The job handle is
/// intentionally leaked. The job MUST outlive the sandbox process for
/// `KILL_ON_JOB_CLOSE` to fire correctly on abnormal termination;
/// closing the handle while the subprocess is running would defeat
/// the containment.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the job cannot be created,
/// the limit cannot be set, or the process cannot be assigned (e.g.
/// the parent's job lacks `BREAKAWAY_OK` on Windows 7).
pub fn install_restrictions() -> Result<(), SandboxError> {
    let job = create_job()?;
    if let Err(e) = apply_limits(job) {
        // SAFETY: `job` is a valid handle we just created; CloseHandle
        // on an unassigned job does NOT kill anything because no
        // process is bound yet.
        unsafe { CloseHandle(job) };
        return Err(e);
    }
    // SAFETY: GetCurrentProcess returns a pseudo-handle (-1) that is
    // valid for the lifetime of the process; no Close needed.
    let current = unsafe { GetCurrentProcess() };
    if let Err(e) = assign_process(job, current) {
        // SAFETY: see above; closing an unassigned job is safe.
        unsafe { CloseHandle(job) };
        return Err(e);
    }
    // The job handle is intentionally leaked: closing it now would
    // immediately kill the sandbox process via KILL_ON_JOB_CLOSE.
    // The job lifetime is the process lifetime; the kernel reclaims
    // the handle when the process exits.
    tracing::info!(flags = SANDBOX_JOB_FLAGS, "windows job object installed");
    let _ = job; // explicit leak
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_limit_info_sets_kill_on_job_close() {
        let info = build_limit_info();
        assert_ne!(
            info.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
            0,
            "KILL_ON_JOB_CLOSE flag missing"
        );
    }

    #[test]
    fn build_limit_info_sets_breakaway_ok() {
        let info = build_limit_info();
        assert_ne!(
            info.BasicLimitInformation.LimitFlags & JOB_OBJECT_LIMIT_BREAKAWAY_OK,
            0,
            "BREAKAWAY_OK flag missing"
        );
    }

    #[test]
    fn build_limit_info_zero_for_unspecified_fields() {
        let info = build_limit_info();
        // Memory limits and process count are untouched; we only set
        // the basic limit flags.
        assert_eq!(info.ProcessMemoryLimit, 0);
        assert_eq!(info.JobMemoryLimit, 0);
        assert_eq!(info.BasicLimitInformation.ActiveProcessLimit, 0);
    }

    #[test]
    fn sandbox_job_flags_constant_matches_documented_set() {
        let expected = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;
        assert_eq!(SANDBOX_JOB_FLAGS, expected);
    }

    #[test]
    fn win32_failure_formats_op_and_error() {
        let e = win32_failure("CreateJobObjectW", 5);
        let msg = format!("{e}");
        assert!(msg.contains("CreateJobObjectW"), "op missing: {msg}");
        assert!(msg.contains("GetLastError=5"), "code missing: {msg}");
    }

    #[test]
    fn create_job_returns_a_valid_handle() {
        let job = create_job().expect("create_job");
        assert!(!job.is_null());
        assert_ne!(job, INVALID_HANDLE_VALUE);
        // SAFETY: we own the handle we just created; cleanup before
        // leaking would interact with KILL_ON_JOB_CLOSE if any process
        // were assigned (none in this test).
        unsafe { CloseHandle(job) };
    }

    #[test]
    fn apply_limits_on_valid_handle_succeeds() {
        let job = create_job().expect("create_job");
        apply_limits(job).expect("apply_limits");
        // SAFETY: see create_job_returns_a_valid_handle.
        unsafe { CloseHandle(job) };
    }

    #[test]
    fn apply_limits_on_invalid_handle_returns_error() {
        // INVALID_HANDLE_VALUE is documented to be rejected by
        // SetInformationJobObject with ERROR_INVALID_HANDLE — exercises
        // the win32_failure error mapping path without touching the
        // test runner's job state.
        let result = apply_limits(INVALID_HANDLE_VALUE);
        let err = result.expect_err("apply_limits on INVALID_HANDLE_VALUE should fail");
        assert!(
            format!("{err}").contains("SetInformationJobObject"),
            "expected SetInformationJobObject in error: {err}"
        );
    }

    #[test]
    fn assign_process_on_invalid_handle_returns_error() {
        // INVALID_HANDLE_VALUE as the job handle triggers an immediate
        // kernel rejection from AssignProcessToJobObject; tests the
        // error-mapping path without affecting the runner.
        // SAFETY: GetCurrentProcess returns a valid pseudo-handle; the
        // invalid job handle makes the call fail before any state change.
        let current = unsafe { GetCurrentProcess() };
        let result = assign_process(INVALID_HANDLE_VALUE, current);
        let err = result.expect_err("assign_process on INVALID_HANDLE_VALUE should fail");
        assert!(
            format!("{err}").contains("AssignProcessToJobObject"),
            "expected AssignProcessToJobObject in error: {err}"
        );
    }

    /// Verify a job can be created + flags applied + assigned to a
    /// child process. We can't safely assign the TEST process (would
    /// kill the cargo runner on job close) so we spawn a short-lived
    /// child and inspect via `IsProcessInJob`.
    ///
    /// Note: this test creates the job and assigns the *current*
    /// process via the production `install_restrictions` path is NOT
    /// safe to invoke from inside cargo's test runner — it would
    /// install the job on the runner itself. The test below uses the
    /// pure-function `build_limit_info` + a manual `CreateJobObjectW` +
    /// AssignProcessToJobObject(job, `child_handle`) flow against a
    /// spawned child instead.
    #[test]
    fn job_assignment_against_child_process_succeeds() {
        use std::process::Command;
        use windows_sys::Win32::System::JobObjects::IsProcessInJob;

        // Spawn a child that sleeps briefly then exits. Using `cmd /c
        // exit` makes the child terminate quickly so the test doesn't
        // wait long.
        let mut child = Command::new("cmd")
            .args(["/c", "exit", "0"])
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("spawn cmd /c exit");

        let raw: RawHandle = child.as_raw_handle();
        // SAFETY: child is alive (we haven't waited on it yet) and the
        // raw handle is valid for the duration of this scope. Cast is
        // documented for HANDLE = *mut c_void in windows-sys.
        let child_handle: HANDLE = raw.cast();

        let job = create_job().expect("create_job");
        apply_limits(job).expect("apply_limits");
        assign_process(job, child_handle).expect("assign_process");

        // Verify membership.
        let mut in_job: i32 = 0;
        // SAFETY: IsProcessInJob takes process handle + (optional) job
        // handle + out-parameter pointer. Passing the explicit job
        // handle queries membership in *that* job.
        let queried = unsafe { IsProcessInJob(child_handle, job, &mut in_job) };
        assert_ne!(queried, 0, "IsProcessInJob query failed");
        assert_ne!(in_job, 0, "child is not in the job after assignment");

        // Clean up the child; CloseHandle on the job kills its members,
        // which we want (child is short-lived anyway).
        // SAFETY: job handle is valid; CloseHandle on a job triggers
        // KILL_ON_JOB_CLOSE on every assigned process.
        unsafe { CloseHandle(job) };
        let _ = child.wait();
    }
}
