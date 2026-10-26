//! Laws for a writer that has stopped: a recorded failure outlives a read
//! core that ignores it, and no later write reaches the sink.

use std::io::{self, Write};

use super::{TransferBounds, run};
use crate::{NeverCancelled, TransferError, TransferSegment, TransferSink, TransferWindow};

/// A sink that refuses every segment and counts how often it was asked.
#[derive(Default)]
struct RefusingSink {
    applied: u32,
}

impl TransferSink for RefusingSink {
    type Error = io::Error;

    fn apply(&mut self, _segment: TransferSegment<'_>) -> Result<(), io::Error> {
        self.applied = self.applied.saturating_add(1);
        Err(io::Error::other("sink refused"))
    }

    fn acknowledge(&mut self) -> Result<(), io::Error> {
        Ok(())
    }

    fn complete(&mut self) -> Result<(), io::Error> {
        Ok(())
    }
}

fn bounds() -> Result<TransferBounds<'static, NeverCancelled>, &'static str> {
    Ok(TransferBounds::new(
        TransferWindow::new(1).ok_or("window")?,
        &NeverCancelled,
    ))
}

#[test]
fn a_recorded_sink_failure_outlives_a_read_core_that_ignores_it()
-> Result<(), Box<dyn std::error::Error>> {
    let mut sink = RefusingSink::default();
    let outcome = run(&mut sink, &bounds()?, |writer| {
        let _ignored = writer.write(b"bytes");
        Ok::<(), io::Error>(())
    });
    assert!(
        matches!(outcome, Err(TransferError::Sink(_))),
        "a swallowed sink failure produced {outcome:?}"
    );
    Ok(())
}

#[test]
fn no_write_after_a_stop_reaches_the_sink() -> Result<(), Box<dyn std::error::Error>> {
    let mut sink = RefusingSink::default();
    let _outcome = run(&mut sink, &bounds()?, |writer| {
        let _first = writer.write(b"first");
        writer.write(b"second").map(|_written| ())
    });
    assert_eq!(sink.applied, 1);
    Ok(())
}
