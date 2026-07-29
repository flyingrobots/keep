//! This module owns child process-group creation and termination.

use std::io;
use std::process::Child;

#[cfg(test)]
use std::io::Read;
#[cfg(test)]
use std::os::unix::net::UnixListener;
#[cfg(test)]
use std::path::Path;

use rustix::io::Errno;
use rustix::process::{Pid, Signal, kill_process_group};

/// The dedicated process-group identity established for one spawned child.
#[derive(Clone, Copy)]
pub(crate) struct ProcessGroup(Pid);

impl ProcessGroup {
    /// Admits the child's nonzero operating-system identifier as a group ID.
    ///
    /// Conversion fails when the unsigned child ID does not fit the platform's
    /// signed PID representation or when the observed ID is zero.
    pub(crate) fn for_child(child: &Child) -> Result<Self, io::Error> {
        let raw = i32::try_from(child.id())
            .map_err(|source| io::Error::new(io::ErrorKind::InvalidData, source))?;
        let pid = Pid::from_raw(raw)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "child PID is zero"))?;
        Ok(Self(pid))
    }

    /// Sends `SIGKILL` to every member of the admitted process group.
    ///
    /// An absent group is already terminated and succeeds. Other operating
    /// system errors are preserved.
    pub(crate) fn terminate(self) -> Result<(), io::Error> {
        match kill_process_group(self.0, Signal::KILL) {
            Ok(()) | Err(Errno::SRCH) => Ok(()),
            Err(source) => Err(source.into()),
        }
    }
}

#[cfg(test)]
const DESCENDANT_CHILD: &str = "KEEP_XTASK_DESCENDANT_CHILD";
#[cfg(test)]
const DESCENDANT_CHILD_READY: &str = "KEEP_XTASK_DESCENDANT_CHILD_READY";
#[cfg(test)]
const DESCENDANT_PARENT: &str = "KEEP_XTASK_DESCENDANT_PARENT";
#[cfg(test)]
const DESCENDANT_READY: &str = "KEEP_XTASK_DESCENDANT_READY";
#[cfg(test)]
const DESCENDANT_SOCKET: &str = "KEEP_XTASK_DESCENDANT_SOCKET";
#[cfg(test)]
const INTERRUPT_SUPERVISOR: &str = "KEEP_XTASK_INTERRUPT_SUPERVISOR";

#[cfg(test)]
fn readiness_listener(path: &Path) -> Result<UnixListener, io::Error> {
    let listener = UnixListener::bind(path)?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

#[cfg(test)]
fn wait_for_ready(listener: &UnixListener, child: &mut Child) -> Result<(), io::Error> {
    loop {
        match listener.accept() {
            Ok((mut stream, _address)) => {
                stream.set_nonblocking(false)?;
                let mut signal = [0_u8; 1];
                stream.read_exact(&mut signal)?;
                if signal != [b'r'] {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "descendant sent an invalid readiness signal",
                    ));
                }
                return Ok(());
            }
            Err(source) if source.kind() == io::ErrorKind::WouldBlock => {
                if let Some(status) = child.try_wait()? {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        format!("child exited before descendant readiness: {status}"),
                    ));
                }
                std::thread::yield_now();
            }
            Err(source) => return Err(source),
        }
    }
}

#[cfg(test)]
#[path = "process_group/child_tests.rs"]
mod child_tests;

#[cfg(test)]
#[path = "process_group/readiness_tests.rs"]
mod readiness_tests;

#[cfg(test)]
#[path = "process_group/tests.rs"]
mod tests;
