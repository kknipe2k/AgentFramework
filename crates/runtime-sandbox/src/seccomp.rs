//! Linux seccomp filter installation — §8.security L3 OS isolation (Linux).
//!
//! Installs a curated allowlist of syscalls at sandbox subprocess startup
//! via the BPF-backed seccomp filter. Default action is `KillProcess`:
//! any syscall outside [`ALLOWED_SYSCALLS`] terminates the subprocess
//! (the main process spawns a fresh one in a known-good state).
//!
//! The allowlist is conservative-but-pragmatic. A tokio multi-thread
//! runtime + LinesCodec + Unix-socket IPC needs roughly 55 syscalls;
//! `execve`/`ptrace`/`mount`/`fork`/`clone3`/`kexec_load` and friends
//! are implicitly denied via the default action.
//!
//! v0.1 scope is "establish the L3 boundary mechanism"; the allowlist
//! will tighten in M09 (generators) when production callers wire
//! validation into the dispatch path. The list MUST stay conservative
//! enough that startup never spuriously kills the subprocess; every
//! addition needs a justification in the retrospective.
//!
//! This module is `cfg(target_os = "linux")`-only: seccomp is a Linux
//! kernel feature. On macOS / Windows the corresponding restrictions
//! live in `crate::landlock` (also Linux, filesystem-only) and
//! `crate::job_objects` (Windows, process-tree containment) — bare
//! backticks because the linked modules are cfg-gated per gotcha #55.

#![cfg(target_os = "linux")]

use crate::error::SandboxError;
use libseccomp::{ScmpAction, ScmpArch, ScmpFilterContext, ScmpSyscall};

/// Curated allowlist of syscalls permitted inside the sandbox subprocess.
///
/// Adding a syscall here requires (a) inline justification on the entry
/// AND (b) a retrospective entry naming what surfaced the need. The list
/// covers tokio's multi-thread runtime + LinesCodec framed-JSON I/O over
/// Unix domain sockets + serde construction. Forbidden syscalls
/// (`execve`, `ptrace`, `mount`, `fork`, `clone3`, etc.) are NOT in this
/// list; the default `KillProcess` action terminates the subprocess on
/// any disallowed syscall.
pub const ALLOWED_SYSCALLS: &[&str] = &[
    // --- Process lifecycle ---
    // exit / exit_group: graceful and abrupt termination.
    "exit",
    "exit_group",
    // Signal handling — tokio's signal driver + libc unwind handlers.
    "rt_sigreturn",
    "rt_sigaction",
    "rt_sigprocmask",
    "sigaltstack",
    // tokio thread-local init / glibc bookkeeping.
    "set_tid_address",
    "set_robust_list",
    "rseq",
    // --- Synchronization ---
    // futex: tokio + std::sync + glibc all depend on it.
    "futex",
    // --- Memory management ---
    // tokio's allocator + glibc heap growth + thread stack allocation.
    "mmap",
    "munmap",
    "mremap",
    "mprotect",
    "brk",
    "madvise",
    // membarrier: tokio uses it for cross-thread fence emulation.
    "membarrier",
    // sched_yield: tokio worker stealing falls back on it under contention.
    "sched_yield",
    // --- Time ---
    // tokio's time driver + tracing-subscriber timestamps.
    "clock_gettime",
    "clock_nanosleep",
    "nanosleep",
    "clock_getres",
    "gettimeofday",
    // --- Randomness ---
    // serde_json's nonce / uuid v4 + glibc stack canary seeding.
    "getrandom",
    // --- Process / user identity (read-only) ---
    // libc init + tracing payloads.
    "getpid",
    "gettid",
    "getuid",
    "geteuid",
    "getgid",
    "getegid",
    "prlimit64",
    "getrlimit",
    // --- Arch-specific ---
    // x86_64 TLS register install.
    "arch_prctl",
    // glibc setname / no-new-privs queries.
    "prctl",
    // --- Polling / events (tokio reactor) ---
    "epoll_create1",
    "epoll_ctl",
    "epoll_wait",
    "epoll_pwait",
    "epoll_pwait2",
    "poll",
    "ppoll",
    "eventfd2",
    "pipe2",
    // --- Sockets / network I/O (Unix domain sockets for IPC) ---
    "socket",
    "bind",
    "listen",
    "accept",
    "accept4",
    "getsockname",
    "getpeername",
    "getsockopt",
    "setsockopt",
    "recvfrom",
    "sendto",
    "recvmsg",
    "sendmsg",
    "shutdown",
    // --- File / stream I/O ---
    // Reading from IPC, serde buffers, glibc locale, /proc lookups.
    "read",
    "write",
    "readv",
    "writev",
    "pread64",
    "pwrite64",
    "close",
    "fcntl",
    "ioctl",
    // Open syscalls — restricted scope by landlock, syscall itself
    // allowed so libc dlopen / serde tmp + /proc reads work.
    "openat",
    "openat2",
    "newfstatat",
    "fstat",
    "statx",
    "lseek",
    "getdents64",
    "readlink",
    "readlinkat",
    "getcwd",
    // --- Filesystem operations needed by ipc::serve's bind path ---
    // unlink: remove stale socket file before bind.
    // mkdir / mkdirat: ensure socket parent directory exists.
    "unlink",
    "unlinkat",
    "mkdir",
    "mkdirat",
];

/// Construct (but do not install) a seccomp filter with the curated
/// allowlist. Returns the assembled [`ScmpFilterContext`] ready for
/// [`ScmpFilterContext::load`].
///
/// Pure function — does not interact with the kernel. Production calls
/// [`install`] which builds and loads in one step; tests call this and
/// inspect / drop the resulting filter without affecting global state.
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the libseccomp library reports
/// an error constructing the filter or resolving a syscall name.
pub fn build_filter() -> Result<ScmpFilterContext, SandboxError> {
    let mut filter = ScmpFilterContext::new_filter(ScmpAction::KillProcess)
        .map_err(|e| SandboxError::Isolation(format!("seccomp new_filter: {e}")))?;
    filter
        .add_arch(ScmpArch::X8664)
        .map_err(|e| SandboxError::Isolation(format!("seccomp add_arch: {e}")))?;
    for name in ALLOWED_SYSCALLS {
        let syscall = ScmpSyscall::from_name(name)
            .map_err(|e| SandboxError::Isolation(format!("seccomp resolve {name}: {e}")))?;
        filter
            .add_rule(ScmpAction::Allow, syscall)
            .map_err(|e| SandboxError::Isolation(format!("seccomp add_rule {name}: {e}")))?;
    }
    Ok(filter)
}

/// Build + load the seccomp filter on the calling process.
///
/// Once loaded the filter is permanent for the process lifetime; the
/// `PR_SET_NO_NEW_PRIVS` flag is set automatically by libseccomp so
/// suid binaries cannot escape via `execve` (which is itself blocked by
/// the default action).
///
/// # Errors
///
/// Returns [`SandboxError::Isolation`] if the filter cannot be
/// constructed (libseccomp version mismatch / unresolved syscall) or
/// loaded (kernel rejected the BPF program).
pub fn install() -> Result<(), SandboxError> {
    let filter = build_filter()?;
    filter
        .load()
        .map_err(|e| SandboxError::Isolation(format!("seccomp load: {e}")))?;
    tracing::info!(rules = ALLOWED_SYSCALLS.len(), "seccomp filter loaded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_filter_constructs_successfully() {
        let result = build_filter();
        assert!(
            result.is_ok(),
            "build_filter should succeed against the host libseccomp"
        );
    }

    #[test]
    fn allowlist_includes_io_syscalls_needed_by_tokio() {
        // Tokio's IPC loop fails immediately if any of these is blocked.
        for required in &["read", "write", "futex", "epoll_wait", "mmap"] {
            assert!(
                ALLOWED_SYSCALLS.contains(required),
                "tokio cannot run without {required}"
            );
        }
    }

    #[test]
    fn allowlist_includes_socket_syscalls_needed_by_ipc_serve() {
        for required in &["socket", "bind", "listen", "accept4", "recvfrom", "sendto"] {
            assert!(
                ALLOWED_SYSCALLS.contains(required),
                "ipc::serve cannot bind/accept without {required}"
            );
        }
    }

    #[test]
    fn allowlist_excludes_dangerous_syscalls() {
        // These MUST be blocked — they are the §8.security L3 escape
        // routes the sandbox exists to prevent.
        for forbidden in &[
            "execve",
            "execveat",
            "fork",
            "vfork",
            "clone3",
            "ptrace",
            "mount",
            "umount2",
            "kexec_load",
            "init_module",
            "reboot",
            "swapon",
            "settimeofday",
            "sethostname",
        ] {
            assert!(
                !ALLOWED_SYSCALLS.contains(forbidden),
                "{forbidden} must NOT be in the allowlist"
            );
        }
    }

    #[test]
    fn allowlist_size_is_documented() {
        // The allowlist count is documented in the M05.C2 retrospective.
        // If this changes, update the retrospective so audit trail
        // captures why we added or removed a syscall.
        assert!(
            ALLOWED_SYSCALLS.len() >= 50 && ALLOWED_SYSCALLS.len() <= 100,
            "allowlist size drifted outside the documented range: {}",
            ALLOWED_SYSCALLS.len()
        );
    }

    #[test]
    fn all_syscall_names_resolve_on_host_arch() {
        // Every entry must be a valid libseccomp syscall name on x86_64.
        // A typo would make build_filter fail at runtime — catch it here
        // at compile-time-of-tests instead.
        for name in ALLOWED_SYSCALLS {
            assert!(
                ScmpSyscall::from_name(name).is_ok(),
                "libseccomp could not resolve syscall {name}"
            );
        }
    }
}
