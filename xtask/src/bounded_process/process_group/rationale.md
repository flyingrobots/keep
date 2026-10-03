# Readiness across sender exit

This page owns test-support readiness observation for subprocess cleanup experiments. It calibrates the driver and does not establish a Keep storage or durability claim.

A fixture sends the readiness byte before exiting. The observer first polls the listener and then observes the child's lifecycle. If the sender connects, writes, and exits between those polls, a successful exit does not invalidate the queued signal. The previous observer incorrectly returned `UnexpectedEof`, producing a false alarm before the process-group cleanup assertion could run.

After observing exit, the observer now attempts to receive the queued signal before reporting absence. It accepts only `r`, preserves `InvalidData` for other bytes, and returns `UnexpectedEof` when the listener remains empty. The readiness module owns this protocol separately from production process-group termination. Its lifecycle callback is the real substitution boundary that permits a controlled observation schedule; ordinary callers still use `Child::try_wait`.

Increasing a timeout or retrying CI was rejected because neither fixes the incorrect relationship between the observations. Holding every sender alive until an acknowledgment was rejected because the inherited-pipe experiments deliberately require a sender that exits while a descendant remains. The real process-group cleanup, deadline, and interruption laws retain their existing assertions.

The controlled reproducer holds a real child on a socket gate until after the observer's empty poll, releases it during lifecycle observation, and waits for its actual exit after the readiness write. This records the race without sleeps or random scheduling. The signal admission law also sweeps the invalid byte domain in ascending order. These experiments do not establish exhaustive process interleavings or bounded response to a peer that connects and never supplies a byte.

The deleted source-substring check prohibited the spelling `std::time::` in one file; it did not prove runtime determinism or a deadline and could be bypassed by an import or moved implementation. Rule 18's non-contract deletion criterion applies. The replacement calibration observes signals and lifecycle outcomes; production process-group laws continue to observe actual descendant disconnection. No current runtime latency claim relies on the removed spelling check.
