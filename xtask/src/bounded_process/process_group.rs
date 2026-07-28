//! This module owns child process-group creation and termination.

use std::io;
use std::process::Child;

use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group};

pub(super) struct ProcessGroup(Pid);

impl ProcessGroup {
    pub(super) fn for_child(child: &Child) -> Result<Self, io::Error> {
        let raw = i32::try_from(child.id())
            .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
        let pid = Pid::from_raw(raw)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "child PID is zero"))?;
        Ok(Self(pid))
    }

    pub(super) fn terminate(self) -> Result<(), io::Error> {
        match kill_process_group(self.0, Signal::KILL) {
            Ok(()) | Err(Errno::SRCH) => Ok(()),
            Err(source) => Err(source.into()),
        }
    }
}

#[cfg(test)]
const DESCENDANT_CHILD: &str = "KEEP_XTASK_DESCENDANT_CHILD";
#[cfg(test)]
const DESCENDANT_PARENT: &str = "KEEP_XTASK_DESCENDANT_PARENT";
#[cfg(test)]
const DESCENDANT_READY: &str = "KEEP_XTASK_DESCENDANT_READY";
#[cfg(test)]
const DESCENDANT_SOCKET: &str = "KEEP_XTASK_DESCENDANT_SOCKET";
#[cfg(test)]
const INTERRUPT_SUPERVISOR: &str = "KEEP_XTASK_INTERRUPT_SUPERVISOR";

#[cfg(test)]
fn wait_for_ready(path: &std::path::Path) -> Result<(), io::Error> {
    let expires = std::time::Instant::now()
        .checked_add(std::time::Duration::from_secs(2))
        .ok_or_else(|| io::Error::other("descendant readiness deadline overflow"))?;
    while !path.is_file() {
        if std::time::Instant::now() >= expires {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "descendant did not become ready",
            ));
        }
        std::thread::yield_now();
    }
    Ok(())
}

#[cfg(test)]
#[path = "process_group/child_tests.rs"]
mod child_tests;

#[cfg(test)]
#[path = "process_group/tests.rs"]
mod tests;
