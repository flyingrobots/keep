# Durable locator process isolation

Change kind: test-isolation bug fix for #178. Owner: `@flyingrobots`. The product oracles remain exact durable bytes, catalogued layout identity, typed refusal and original I/O sources; no expected product outcome changes. This record distinguishes observed runtime failure, a controlled kernel experiment, and the scheduling boundary introduced by the correction.

## Observed RED

Main `7a21faebdbed38c386db14873966d433754c6eb4` failed the release golden worldline law `suite::durable_layout_laws::supplied_range_layouts_cannot_replace_the_catalogued_target_binding` with `WriterLock { source: Busy }` in [run 37164344603](https://github.com/flyingrobots/keep/actions/runs/37164344603). The [failure excerpt](durable-locator-isolation/hosted-red.txt) retains the original diagnostics and timestamps with line-end whitespace normalized. Earlier green runs are not substituted for this observed RED.

The failed law calls the real store fixture before asserting layout refusal. That fixture relinquishes and reacquires writer authority between catalog publication, migration and retention publication. Two sibling locator laws launched child processes from the same test executable. A child can inherit another thread's open flock descriptions until exec, extending their lifetime past the parent guard's drop. The log does not identify which handoff refused or prove that this schedule caused the hosted failure.

## Controlled mechanism and limits

On parent `3165890e9291cfb5fe10e81a9d7cd151f3e59464`, a separate diagnostic crate opened production Keep authority, held a child before exec with pipe handshakes, dropped the parent authority and observed exact public `WriterLockAcquireError::Busy`. After releasing and reaping the child, public acquisition succeeded. The [receipt](durable-locator-isolation/controlled-inheritance.txt) and [diagnostic source](durable-locator-isolation/probe-source.txt) preserve the experiment. No sleep or probabilistic workload determines that schedule.

The diagnostic's isolated unsafe pre-exec hook performs only raw pipe reads and writes; it is not linked into Keep, added to the test suite, or used as a merge gate. It demonstrates an actual inherited-descriptor lifetime, not the exact unobserved CI trace. A separate strace run delaying syscall completion passed the original suite; it found no failing schedule and supplies no proof of absence. Keep production source and the failing test's assertions are identical between the observed RED revision and this parent.

## Correction and retained contracts

The two existing locator laws now run in `tests/durable_locator.rs`, a separate integration-test executable. Its parent creates no store or writer authority; each admitted child runs exactly one locator law serially and creates its own stores. The golden worldline executable no longer launches those children. Parallel execution of the two executables does not share their descriptor tables, so a locator child cannot inherit the golden executable's store locks.

The locator laws still check that a relative handle keeps its original store after a working-directory change and that a deleted working directory preserves its exact NotFound cause. The golden layout-binding law retains its exact LayoutMissing checks and unchanged output sentinel. No law is deleted, ignored, retried or globally serialized. The locator module and its exact child selectors remain byte-identical to the original. Shared fixture imports allow unused partial-record constructors and explicit sandbox removal only in the new locator executable; other filesystem laws still exercise those operations.

Keep's authority handoffs, flock semantics, public errors, APIs, formats and recovery protocols do not change. This correction removes the demonstrated interference mechanism from this suite without asserting that every possible cause of Busy has been eliminated. Any new failure remains a defect to diagnose, not a retry instruction.

## Validation profile

These unchanged laws remain medium tests using owned Linux ext4 scratch and real filesystem/process operations. The locator child retains its existing 20-second watchdog; that is an execution ceiling, not a latency promise. Focused validation runs both executables in debug and release inside copied Docker source, followed by the required stable-candidate chain and independent exact-head review. Commands and terminal results are recorded on the corrective PR; no pending execution is described as passing here.

The original ordinary-Cargo resource enforcement gaps remain disclosed in the [enforcement profile](../testing/enforcement.md). Moving existing assertions does not establish new per-test memory limits, egress isolation, suite latency measurements or exhaustive scheduler exploration. #106's unrelated version-update waiver does not cover this change. No new assertion calibration or artificial fixture-count test is substituted for the existing runtime failure and retained product laws. Retire the separation if subprocess creation is removed or another verified process-isolation boundary makes inherited lock interference impossible.
