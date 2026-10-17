//! Lifetime management for finite CLI invocations.
//!
//! On Unix, each CLI gets a separate process group. Keep its guard until the
//! invocation finishes: dropping it also kills descendants after a timeout,
//! cancellation, error, or a successful parent exit. This is not a sandbox; a
//! process that deliberately leaves the group is outside this guarantee.
//! Other platforms retain Tokio's direct-child termination only.

#![forbid(unsafe_code)]

#[cfg(unix)]
use std::num::NonZeroI32;

#[cfg(unix)]
use nix::{
    errno::Errno,
    sys::signal::{Signal, killpg},
    unistd::Pid,
};
use tokio::process::Command;

/// Call before spawning the command. Unix children inherit their parent's group
/// unless this is set; never call `killpg` on the gateway's own process group.
pub(crate) fn prepare(command: &mut Command) {
    #[cfg(unix)]
    {
        command.process_group(0);
    }
    #[cfg(not(unix))]
    {
        let _ = command;
    }
}

/// Owns only the process group of a child spawned after [`prepare`]. Construct
/// immediately after `spawn`, before awaiting or checking the child's exit.
/// Missing/invalid IDs are inert, and no process group is managed on non-Unix.
#[derive(Debug)]
pub(crate) struct ProcessGroup {
    #[cfg(unix)]
    group_id: Option<NonZeroI32>,
}

impl ProcessGroup {
    pub(crate) fn new(child_id: Option<u32>) -> Self {
        #[cfg(unix)]
        let group_id = child_id
            .and_then(|id| i32::try_from(id).ok())
            .and_then(NonZeroI32::new);
        #[cfg(not(unix))]
        let _ = child_id;
        Self {
            #[cfg(unix)]
            group_id,
        }
    }
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(group_id) = self.group_id.take() {
            match killpg(Pid::from_raw(group_id.get()), Signal::SIGKILL) {
                Ok(()) | Err(Errno::ESRCH) => {}
                Err(error) => eprintln!(
                    "getcraft-mcp: could not terminate CLI process group {group_id}: {error}"
                ),
            }
        }
    }
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::{io, process::Stdio, time::Duration};
    use tokio::{
        io::{AsyncBufReadExt, BufReader},
        process::Child,
        time::{sleep, timeout},
    };

    type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

    struct Fixture {
        guard: Option<ProcessGroup>,
        child: Child,
        child_id: Pid,
        descendant_id: Option<Pid>,
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            drop(self.guard.take());
            // Independent cleanup also runs if a regression makes the guard
            // fail, an assertion panics, or reading the worker's PID fails.
            let _ = killpg(self.child_id, Signal::SIGKILL);
            if let Some(descendant) = self.descendant_id {
                let _ = nix::sys::signal::kill(descendant, Signal::SIGKILL);
            }
            let _ = self.child.start_kill();
        }
    }

    async fn spawn_fixture(parent_exits: bool) -> TestResult<Fixture> {
        let script = if parent_exits {
            r#"/bin/sleep 30 & descendant=$!; printf '%s\n' "$descendant""#
        } else {
            r#"/bin/sleep 30 & descendant=$!; printf '%s\n' "$descendant"; wait "$descendant""#
        };
        let mut command = Command::new("/bin/sh");
        command
            .args(["-c", script])
            .kill_on_drop(true)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        prepare(&mut command);
        let child = command.spawn()?;
        let guard = ProcessGroup::new(child.id());
        let child_id = child.id().ok_or("newly spawned child has no PID")?;
        let child_id = Pid::from_raw(i32::try_from(child_id)?);
        let mut fixture = Fixture {
            guard: Some(guard),
            child,
            child_id,
            descendant_id: None,
        };
        let stdout = fixture.child.stdout.take().ok_or("missing child stdout")?;
        let mut line = String::new();
        timeout(
            Duration::from_secs(3),
            BufReader::new(stdout).read_line(&mut line),
        )
        .await??;
        let descendant_id: i32 = line.trim().parse()?;
        if descendant_id <= 0 {
            return Err("worker returned an invalid PID".into());
        }
        fixture.descendant_id = Some(Pid::from_raw(descendant_id));
        Ok(fixture)
    }

    fn is_running(pid: Pid) -> io::Result<bool> {
        match std::fs::read_to_string(format!("/proc/{}/stat", pid.as_raw())) {
            Ok(stat) => {
                let state = stat
                    .rsplit_once(") ")
                    .and_then(|(_, fields)| fields.chars().next())
                    .ok_or_else(|| io::Error::other("invalid process stat"))?;
                // A killed orphan may briefly remain a zombie until its new
                // parent reaps it. It is terminated and cannot execute work.
                Ok(!matches!(state, 'Z' | 'X'))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    async fn assert_stopped(pid: Pid) -> TestResult {
        timeout(Duration::from_secs(3), async {
            while is_running(pid)? {
                sleep(Duration::from_millis(10)).await;
            }
            Ok::<(), io::Error>(())
        })
        .await??;
        Ok(())
    }

    #[test]
    fn absent_zero_and_out_of_range_ids_do_not_target_any_group() {
        for id in [None, Some(0), Some(u32::MAX)] {
            assert!(ProcessGroup::new(id).group_id.is_none());
        }
    }

    #[tokio::test]
    async fn dropping_guard_terminates_child_and_descendant() -> TestResult {
        let mut fixture = spawn_fixture(false).await?;
        let descendant = fixture.descendant_id.ok_or("missing descendant PID")?;
        assert_eq!(nix::unistd::getpgid(Some(descendant))?, fixture.child_id);
        assert!(is_running(descendant)?);
        drop(fixture.guard.take());
        assert_stopped(fixture.child_id).await?;
        assert_stopped(descendant).await?;
        assert!(
            !timeout(Duration::from_secs(3), fixture.child.wait())
                .await??
                .success()
        );
        Ok(())
    }

    #[tokio::test]
    async fn successful_parent_does_not_leave_its_descendant_running() -> TestResult {
        let mut fixture = spawn_fixture(true).await?;
        let descendant = fixture.descendant_id.ok_or("missing descendant PID")?;
        assert!(
            timeout(Duration::from_secs(3), fixture.child.wait())
                .await??
                .success()
        );
        assert!(is_running(descendant)?);
        drop(fixture.guard.take());
        assert_stopped(descendant).await?;
        Ok(())
    }

    #[tokio::test]
    async fn timeout_drops_guard_and_terminates_descendants() -> TestResult {
        let mut fixture = spawn_fixture(false).await?;
        let descendant = fixture.descendant_id.ok_or("missing descendant PID")?;
        let guard = fixture.guard.take().ok_or("missing group guard")?;
        let result = timeout(Duration::from_millis(30), async move {
            let _guard = guard;
            std::future::pending::<()>().await;
        })
        .await;
        assert!(result.is_err());
        assert_stopped(fixture.child_id).await?;
        assert_stopped(descendant).await?;
        let _ = timeout(Duration::from_secs(3), fixture.child.wait()).await??;
        Ok(())
    }

    #[tokio::test]
    async fn task_cancellation_drops_guard_and_terminates_descendants() -> TestResult {
        let mut fixture = spawn_fixture(false).await?;
        let descendant = fixture.descendant_id.ok_or("missing descendant PID")?;
        let guard = fixture.guard.take().ok_or("missing group guard")?;
        let task = tokio::spawn(async move {
            let _guard = guard;
            std::future::pending::<()>().await;
        });
        task.abort();
        assert!(task.await.is_err_and(|error| error.is_cancelled()));
        assert_stopped(fixture.child_id).await?;
        assert_stopped(descendant).await?;
        let _ = timeout(Duration::from_secs(3), fixture.child.wait()).await??;
        Ok(())
    }

    #[tokio::test]
    async fn gateway_timeout_terminates_the_native_cli_descendant() -> TestResult {
        struct RecordedPids(std::path::PathBuf);

        impl RecordedPids {
            fn read(&self) -> io::Result<Vec<Pid>> {
                Ok(std::fs::read_to_string(&self.0)?
                    .lines()
                    .filter_map(|line| line.parse::<i32>().ok())
                    .filter(|id| *id > 0)
                    .map(Pid::from_raw)
                    .collect())
            }
        }

        impl Drop for RecordedPids {
            fn drop(&mut self) {
                if let Ok(ids) = self.read() {
                    if let Some(parent) = ids.first() {
                        let _ = killpg(*parent, Signal::SIGKILL);
                    }
                    for id in ids {
                        let _ = nix::sys::signal::kill(id, Signal::SIGKILL);
                    }
                }
            }
        }

        let directory = tempfile::tempdir()?;
        let cleanup = RecordedPids(directory.path().join("pids"));
        // Record the parent before spawning its worker, so cleanup still finds
        // the whole group if an error occurs before the second PID is written.
        let script = r#"printf '%s\n' "$$" > "$1"; /bin/sleep 30 & worker=$!; printf '%s\n' "$worker" >> "$1"; wait "$worker""#;
        let result = crate::invoke_program(
            crate::lookup_app("pdfcraft")?,
            std::path::PathBuf::from("/bin/sh"),
            vec![
                "-c".to_string(),
                script.to_string(),
                "getcraft-process-test".to_string(),
                cleanup.0.to_string_lossy().into_owned(),
            ],
            500,
        )
        .await;
        assert!(result.is_err_and(|error| error.contains("exceeded")));
        let ids = cleanup.read()?;
        assert_eq!(ids.len(), 2, "the fixture must create a descendant");
        for id in ids {
            assert_stopped(id).await?;
        }
        Ok(())
    }
}
