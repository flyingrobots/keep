//! This module owns the process-death gate at production protocol boundaries.

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;

use xtask::{DurabilityCrashCase, DurabilityCrashPoint, DurabilityCrashPosition};

const READY: u8 = b'r';

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum DuringTiming {
    Before,
    After,
}

pub(super) struct CrashControl {
    case: DurabilityCrashCase,
    readiness: UnixStream,
}

impl CrashControl {
    pub(super) const fn new(case: DurabilityCrashCase, readiness: UnixStream) -> Self {
        Self { case, readiness }
    }

    pub(super) fn before(
        &mut self,
        point: DurabilityCrashPoint,
        during: DuringTiming,
    ) -> io::Result<()> {
        let position = self.position(point);
        if position == Some(DurabilityCrashPosition::Before)
            || position == Some(DurabilityCrashPosition::During) && during == DuringTiming::Before
        {
            return self.await_process_death();
        }
        Ok(())
    }

    pub(super) fn after(
        &mut self,
        point: DurabilityCrashPoint,
        during: DuringTiming,
    ) -> io::Result<()> {
        let position = self.position(point);
        if position == Some(DurabilityCrashPosition::After)
            || position == Some(DurabilityCrashPosition::During) && during == DuringTiming::After
        {
            return self.await_process_death();
        }
        Ok(())
    }

    pub(super) fn position(&self, point: DurabilityCrashPoint) -> Option<DurabilityCrashPosition> {
        (self.case.point() == point).then(|| self.case.position())
    }

    pub(super) fn await_process_death(&mut self) -> io::Result<()> {
        self.readiness.write_all(&[READY])?;
        let mut unexpected = [0_u8; 1];
        self.readiness.read_exact(&mut unexpected)?;
        Err(io::Error::other(
            "crash controller resumed a child selected for process death",
        ))
    }
}
