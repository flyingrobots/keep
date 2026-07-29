//! This module owns deadline-bounded crash-child execution and termination.

use std::fs;
use std::io::Read;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use crate::bounded_process::ProcessGroup;
use crate::test_directory::TestDirectory;

use super::DurabilityCrashMatrixError;
use super::child::marker;
use xtask::DurabilityCrashCase;

const DEADLINE: Duration = Duration::from_secs(10);
const READY: u8 = b'r';

pub(super) fn run(
    repository_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let directory = TestDirectory::create("durability-crash")
        .map_err(|source| DurabilityCrashMatrixError::io("create crash-case directory", source))?;
    let socket_path = directory.path().join("ready.sock");
    let listener = listener(&socket_path)?;
    let deadline = Instant::now()
        .checked_add(DEADLINE)
        .ok_or(DurabilityCrashMatrixError::Timeout { duration: DEADLINE })?;
    let (mut child, group) = spawn(repository_root, directory.path(), &socket_path, case)?;
    let readiness_stream = wait_for_ready(&listener, &mut child, deadline);
    let termination = terminate(group, &mut child);
    let _readiness_stream = readiness_stream?;
    termination?;
    verify_marker(directory.path(), case)?;
    drop(listener);
    fs::remove_file(&socket_path)
        .map_err(|source| DurabilityCrashMatrixError::io("remove readiness socket", source))?;
    directory
        .close()
        .map_err(|source| DurabilityCrashMatrixError::io("remove crash-case directory", source))
}

fn listener(path: &Path) -> Result<UnixListener, DurabilityCrashMatrixError> {
    let listener = UnixListener::bind(path)
        .map_err(|source| DurabilityCrashMatrixError::io("bind readiness socket", source))?;
    listener
        .set_nonblocking(true)
        .map_err(|source| DurabilityCrashMatrixError::io("configure readiness socket", source))?;
    Ok(listener)
}

fn spawn(
    repository_root: &Path,
    case_root: &Path,
    socket_path: &Path,
    case: DurabilityCrashCase,
) -> Result<(Child, ProcessGroup), DurabilityCrashMatrixError> {
    let executable = std::env::current_exe()
        .map_err(|source| DurabilityCrashMatrixError::io("resolve xtask executable", source))?;
    let mut command = Command::new(executable);
    command
        .current_dir(repository_root)
        .arg("__durability-crash-child")
        .arg(case.point().identifier())
        .arg(case.position().identifier())
        .arg(case_root)
        .arg(socket_path);
    let mut child = crate::bounded_process::spawn_in_process_group(&mut command)
        .map_err(|source| DurabilityCrashMatrixError::io("spawn crash child", source))?;
    match ProcessGroup::for_child(&child) {
        Ok(group) => Ok((child, group)),
        Err(source) => {
            let admission =
                DurabilityCrashMatrixError::io("admit crash-child process group", source);
            child
                .kill()
                .map_err(|source| DurabilityCrashMatrixError::io("kill crash child", source))?;
            child
                .wait()
                .map_err(|source| DurabilityCrashMatrixError::io("reap crash child", source))?;
            Err(admission)
        }
    }
}

fn wait_for_ready(
    listener: &UnixListener,
    child: &mut Child,
    deadline: Instant,
) -> Result<UnixStream, DurabilityCrashMatrixError> {
    loop {
        match listener.accept() {
            Ok((stream, _address)) => return read_ready(stream, child, deadline),
            Err(source) if source.kind() == std::io::ErrorKind::WouldBlock => {
                admit_running(child)?;
                admit_deadline(deadline)?;
                std::thread::yield_now();
            }
            Err(source) => {
                return Err(DurabilityCrashMatrixError::io(
                    "accept crash-child readiness",
                    source,
                ));
            }
        }
    }
}

fn read_ready(
    mut stream: UnixStream,
    child: &mut Child,
    deadline: Instant,
) -> Result<UnixStream, DurabilityCrashMatrixError> {
    stream
        .set_nonblocking(true)
        .map_err(|source| DurabilityCrashMatrixError::io("configure child readiness", source))?;
    let mut signal = [0_u8; 1];
    loop {
        match stream.read_exact(&mut signal) {
            Ok(()) if signal == [READY] => return Ok(stream),
            Ok(()) => {
                return Err(DurabilityCrashMatrixError::InvalidReadinessSignal {
                    observed: signal[0],
                });
            }
            Err(source) if source.kind() == std::io::ErrorKind::WouldBlock => {
                admit_running(child)?;
                admit_deadline(deadline)?;
                std::thread::yield_now();
            }
            Err(source) => {
                return Err(DurabilityCrashMatrixError::io(
                    "read crash-child readiness",
                    source,
                ));
            }
        }
    }
}

fn admit_running(child: &mut Child) -> Result<(), DurabilityCrashMatrixError> {
    child
        .try_wait()
        .map_err(|source| DurabilityCrashMatrixError::io("poll crash child", source))?
        .map_or(Ok(()), |status| {
            Err(DurabilityCrashMatrixError::ChildExitedEarly {
                code: status.code(),
            })
        })
}

fn admit_deadline(deadline: Instant) -> Result<(), DurabilityCrashMatrixError> {
    if Instant::now() >= deadline {
        Err(DurabilityCrashMatrixError::Timeout { duration: DEADLINE })
    } else {
        Ok(())
    }
}

fn terminate(group: ProcessGroup, child: &mut Child) -> Result<(), DurabilityCrashMatrixError> {
    if let Err(group_error) = group.terminate() {
        child
            .kill()
            .map_err(|source| DurabilityCrashMatrixError::io("kill crash child", source))?;
        child
            .wait()
            .map_err(|source| DurabilityCrashMatrixError::io("reap crash child", source))?;
        return Err(DurabilityCrashMatrixError::io(
            "terminate crash-child process group",
            group_error,
        ));
    }
    let status = child
        .wait()
        .map_err(|source| DurabilityCrashMatrixError::io("reap crash child", source))?;
    if status.signal().is_some() {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::ChildSurvivedTermination {
            code: status.code(),
        })
    }
}

fn verify_marker(
    case_root: &Path,
    case: DurabilityCrashCase,
) -> Result<(), DurabilityCrashMatrixError> {
    let observed = fs::read(case_root.join("prepared-case"))
        .map_err(|source| DurabilityCrashMatrixError::io("read crash-case marker", source))?;
    if observed == marker(case) {
        Ok(())
    } else {
        Err(DurabilityCrashMatrixError::StateMismatch)
    }
}
