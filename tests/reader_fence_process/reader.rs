//! Readiness channels and kernel-observed schedules for a real snapshot process.

use keep::{
    CatalogRestartByteLimit, CatalogRestartPolicy, FilesystemRetentionSnapshot, ReaderAttemptLimit,
    SegmentReadPolicy,
};
use std::error::Error;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::{
    fs::MetadataExt,
    net::{UnixListener, UnixStream},
    process::ExitStatusExt,
};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

const CHILD_ROOT: &str = "KEEP_FENCE_CHILD_ROOT";
const WATCHDOG: Duration = Duration::from_secs(20);

pub(super) fn is_child() -> bool {
    std::env::var_os(CHILD_ROOT).is_some()
}

pub(super) fn serve() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(std::env::var_os(CHILD_ROOT).ok_or("missing child root")?);
    let mut channel = UnixStream::connect(root.with_extension("sock"))?;
    channel.set_read_timeout(Some(WATCHDOG))?;
    let policy = CatalogRestartPolicy::new(
        SegmentReadPolicy::MAXIMUM,
        CatalogRestartByteLimit::new(1_048_576)?,
    );
    let snapshot = FilesystemRetentionSnapshot::load(&root, policy, ReaderAttemptLimit::DEFAULT)?;
    assert_eq!(
        snapshot.catalog().generation().get(),
        1,
        "child must load the golden migrated catalog"
    );
    channel.write_all(b"R")?;
    let mut stop = [0];
    channel.read_exact(&mut stop)?;
    assert_eq!(&stop, b"X", "parent must explicitly release the reader");
    drop(snapshot);
    Ok(())
}

pub(super) struct Reader {
    child: Child,
    channel: UnixStream,
    socket: PathBuf,
}

impl Reader {
    pub(super) fn spawn(root: &Path, law: &str) -> Result<Self, Box<dyn Error>> {
        let socket = root.with_extension("sock");
        let listener = UnixListener::bind(&socket)?;
        listener.set_nonblocking(true)?;
        let mut child = Command::new(std::env::current_exe()?)
            .args(["--exact", law, "--nocapture"])
            .env(CHILD_ROOT, root)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .spawn()?;
        let connected = connect(&listener, &mut child);
        let channel = match connected {
            Ok(channel) => channel,
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&socket);
                return Err(error);
            }
        };
        let reader = Self {
            child,
            channel,
            socket,
        };
        reader.channel.set_read_timeout(Some(WATCHDOG))?;
        reader.channel.set_write_timeout(Some(WATCHDOG))?;
        Ok(reader)
    }

    pub(super) fn await_snapshot(&mut self) -> Result<(), Box<dyn Error>> {
        let mut ready = [0];
        self.channel.read_exact(&mut ready)?;
        assert_eq!(&ready, b"R", "child must have a live admitted snapshot");
        Ok(())
    }

    pub(super) fn await_kernel_wait(&mut self, fence: &File) -> Result<(), Box<dyn Error>> {
        let pid = self.child.id().to_string();
        let inode = format!(":{}", fence.metadata()?.ino());
        let started = Instant::now();
        loop {
            let locks = fs::read_to_string("/proc/locks")?;
            if locks.lines().any(|line| queued_reader(line, &pid, &inode)) {
                return Ok(());
            }
            self.channel.set_nonblocking(true)?;
            let mut byte = [0];
            let observed = self.channel.read(&mut byte);
            self.channel.set_nonblocking(false)?;
            match observed {
                Ok(1) => return Err("snapshot escaped the exclusive collector fence".into()),
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
                result => {
                    return Err(
                        format!("reader failed before its kernel lock wait: {result:?}").into(),
                    );
                }
            }
            require_live(&mut self.child, started)?;
            std::thread::yield_now();
        }
    }

    pub(super) fn kill(&mut self) -> Result<(), Box<dyn Error>> {
        self.child.kill()?;
        let status = self.child.wait()?;
        assert_eq!(
            status.signal(),
            Some(9),
            "reader must die by SIGKILL, without Rust cleanup"
        );
        Ok(())
    }

    pub(super) fn finish(&mut self) -> Result<(), Box<dyn Error>> {
        self.channel.write_all(b"X")?;
        let started = Instant::now();
        loop {
            if let Some(status) = self.child.try_wait()? {
                assert!(
                    status.success(),
                    "reader failed after collector release: {status}"
                );
                return Ok(());
            }
            if started.elapsed() > WATCHDOG {
                return Err("reader exit watchdog expired".into());
            }
            std::thread::yield_now();
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_file(&self.socket);
    }
}

fn connect(listener: &UnixListener, child: &mut Child) -> Result<UnixStream, Box<dyn Error>> {
    let started = Instant::now();
    loop {
        match listener.accept() {
            Ok((channel, _address)) => return Ok(channel),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(error) => return Err(error.into()),
        }
        require_live(child, started)?;
        std::thread::yield_now();
    }
}

fn require_live(child: &mut Child, started: Instant) -> Result<(), Box<dyn Error>> {
    if let Some(status) = child.try_wait()? {
        return Err(format!("reader exited before readiness: {status}").into());
    }
    if started.elapsed() > WATCHDOG {
        return Err("reader readiness watchdog expired".into());
    }
    Ok(())
}

fn queued_reader(line: &str, pid: &str, inode: &str) -> bool {
    let mut fields = line.split_whitespace();
    let _row = fields.next();
    fields.next() == Some("->")
        && fields.next() == Some("FLOCK")
        && fields.next() == Some("ADVISORY")
        && fields.next() == Some("READ")
        && fields.next() == Some(pid)
        && fields.next().is_some_and(|file| file.ends_with(inode))
}
