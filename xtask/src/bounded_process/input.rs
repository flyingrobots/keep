//! This module owns deadline-bounded child-process input streaming.

use std::io::{self, Write};
use std::process::ChildStdin;
use std::thread;
use std::time::Duration;

use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};

use super::{InterruptGuard, ProcessDeadline, ProcessError};

const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// Streams admitted slices to child stdin without a combined allocation.
///
/// The pipe is placed in nonblocking mode before the first write. Every retry
/// observes the complete process deadline and terminal-signal guard.
pub(super) fn write_input(
    program: &'static str,
    stdin: &mut ChildStdin,
    parts: &[&[u8]],
    deadline: &ProcessDeadline,
    interrupts: &InterruptGuard,
) -> Result<(), ProcessError> {
    let flags = fcntl_getfl(&*stdin)
        .map_err(|source| process_io(program, "read child input flags", source.into()))?;
    fcntl_setfl(&*stdin, flags | OFlags::NONBLOCK).map_err(|source| {
        process_io(program, "configure nonblocking child input", source.into())
    })?;
    for part in parts {
        write_part(program, stdin, part, deadline, interrupts)?;
    }
    Ok(())
}

fn write_part(
    program: &'static str,
    writer: &mut impl Write,
    part: &[u8],
    deadline: &ProcessDeadline,
    interrupts: &InterruptGuard,
) -> Result<(), ProcessError> {
    let mut remaining = part;
    while !remaining.is_empty() {
        observe_input_boundary(program, deadline, interrupts)?;
        match writer.write(remaining) {
            Ok(0) => {
                return Err(process_io(
                    program,
                    "write child input",
                    io::Error::new(io::ErrorKind::WriteZero, "child input made no progress"),
                ));
            }
            Ok(written) => {
                remaining = remaining.get(written..).ok_or_else(|| {
                    process_io(
                        program,
                        "account child input",
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            "child input write exceeded the admitted slice",
                        ),
                    )
                })?;
            }
            Err(source) if source.kind() == io::ErrorKind::Interrupted => {}
            Err(source) if source.kind() == io::ErrorKind::WouldBlock => {
                wait_for_input(program, deadline, interrupts)?;
            }
            Err(source) => return Err(process_io(program, "write child input", source)),
        }
    }
    Ok(())
}

fn observe_input_boundary(
    program: &'static str,
    deadline: &ProcessDeadline,
    interrupts: &InterruptGuard,
) -> Result<(), ProcessError> {
    if let Some(error) = interrupts.refusal(program) {
        return Err(error);
    }
    deadline.remaining(program)?;
    Ok(())
}

fn wait_for_input(
    program: &'static str,
    deadline: &ProcessDeadline,
    interrupts: &InterruptGuard,
) -> Result<(), ProcessError> {
    if let Some(error) = interrupts.refusal(program) {
        return Err(error);
    }
    let interval = match deadline.remaining(program)? {
        Some((remaining, _duration)) => INPUT_POLL_INTERVAL.min(remaining),
        None => INPUT_POLL_INTERVAL,
    };
    thread::sleep(interval);
    Ok(())
}

const fn process_io(
    program: &'static str,
    action: &'static str,
    source: io::Error,
) -> ProcessError {
    ProcessError::Io {
        program,
        action,
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PROGRAM: &str = "input-progress-test";

    struct OneByteWriter;

    impl Write for OneByteWriter {
        fn write(&mut self, bytes: &[u8]) -> Result<usize, io::Error> {
            Ok(usize::from(!bytes.is_empty()))
        }

        fn flush(&mut self) -> Result<(), io::Error> {
            Ok(())
        }
    }

    #[test]
    fn continuous_input_progress_still_obeys_the_complete_deadline() -> Result<(), ProcessError> {
        let deadline = ProcessDeadline::new(TEST_PROGRAM, Some(Duration::ZERO))?;
        let interrupts = InterruptGuard::begin(TEST_PROGRAM)?;
        let result = write_part(
            TEST_PROGRAM,
            &mut OneByteWriter,
            b"deadline",
            &deadline,
            &interrupts,
        );

        assert!(matches!(
            result,
            Err(ProcessError::Timeout {
                program: TEST_PROGRAM,
                duration: Duration::ZERO,
            })
        ));
        Ok(())
    }
}
