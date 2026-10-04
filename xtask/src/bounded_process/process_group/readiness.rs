//! This test-support module owns the subprocess readiness signal protocol.

use std::io::{self, Read};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::process::{Child, ExitStatus};

pub(super) fn readiness_listener(path: &Path) -> io::Result<UnixListener> {
    let listener = UnixListener::bind(path)?;
    listener.set_nonblocking(true)?;
    Ok(listener)
}

pub(super) fn wait_for_ready(listener: &UnixListener, child: &mut Child) -> io::Result<()> {
    wait_for_signal(listener, || child.try_wait())
}

/// The lifecycle port lets calibration choose a reproducible observation order.
pub(super) fn wait_for_signal(
    listener: &UnixListener,
    mut poll_exit: impl FnMut() -> io::Result<Option<ExitStatus>>,
) -> io::Result<()> {
    loop {
        match receive_signal(listener) {
            Err(source) if source.kind() == io::ErrorKind::WouldBlock => {}
            result => return result,
        }
        if let Some(status) = poll_exit()? {
            return receive_after_exit(listener, status);
        }
        std::thread::yield_now();
    }
}

/// A sender can queue its signal between the empty poll and observed exit.
fn receive_after_exit(listener: &UnixListener, status: ExitStatus) -> io::Result<()> {
    match receive_signal(listener) {
        Err(source) if source.kind() == io::ErrorKind::WouldBlock => Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("child exited before descendant readiness: {status}"),
        )),
        result => result,
    }
}

fn receive_signal(listener: &UnixListener) -> io::Result<()> {
    let (mut stream, _address) = listener.accept()?;
    stream.set_nonblocking(false)?;
    let mut signal = [0_u8; 1];
    stream.read_exact(&mut signal)?;
    if signal == [b'r'] {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "descendant sent an invalid readiness signal",
        ))
    }
}
