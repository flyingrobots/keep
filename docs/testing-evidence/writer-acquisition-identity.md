# Writer acquisition identity evidence

Change kind: test-oracle correction for [#169](https://github.com/flyingrobots/keep/issues/169), discovered while auditing T-13.1 under #131. Production is unchanged from main `6051abb25a9fd33ae7ee0de5614514b709a4d82a` and already propagates the identity refusal correctly. This is not an ordinary runtime bug fix; there is no claim that unmutated parent production should fail the strengthened law.

## Promise and boundary

`KEEP-RECOVERY-004` requires the authority-producing acquisition path to reject an opened lock file whose canonical pathname now names a different device/inode. The old test invoked `verify_current_identity` directly. It could establish the checker's refusal but could not establish that the acquisition path respects it. The public replacement test covers a different transition: replacement after a first guard is returned, with root-lock exclusion preventing a second writer.

The replacement law opens and retains the real root lock and original lock file, deterministically renames the latter, writes a distinct replacement, and invokes `FilesystemWriterLock::acquire`. This is the shared module capability boundary used by ordinary acquisition and initialization: it performs the real kernel file lock and can return the writer guard. The test requires no guard to escape, exact `VerifyFileIdentity/InvalidData`, and unchanged bytes in both evidence files. It uses no sleep, stress loop, global hook, source-text assertion or independently reimplemented identity checker.

This experiment enters after the outer public pathname-opening steps. It is not a public-entry-point race schedule or a proof about every possible concurrent raw namespace mutation. Existing public acquisition laws separately cover ordinary success, exclusion, missing-file refusal, no-follow refusal and unchanged evidence. The strengthened test checks the acquisition transition rather than asserting helper-call choreography.

## Calibration

On the inspected parent, replacing `verify_current_identity(&directory, expected_identity)?;` with `let _ = verify_current_identity(&directory, expected_identity);` leaves the old helper test, public lock tests, initialization port tests and filesystem initializer tests GREEN. The strengthened law on that same production mutant instead fails with `replacement received writer authority`. An earlier attempt removed the call and was rejected by dead-code lint; that compilation failure is excluded from runtime calibration.

Additional isolated producer mutations change the mismatch error phase, truncate the displaced original, or truncate the selected replacement before refusing. Each compiles and fails its intended assertion: exact identity-refusal boundary, original bytes, or replacement bytes. Original and mutant sources, replay commands and raw RED logs are retained as review artifacts. Mutations are removed and source timestamps invalidated before unmutated validation.

Deletion criterion: the helper-only test is subsumed by a stronger test of the authority-producing transition, including precise refusal and evidence preservation. No production contract or expected behavior is relaxed. The shared boundary remains a private implementation entry point, with the tested contract being whether writer authority is produced under the controlled filesystem state.

## Execution and limits

The test is medium-sized: it uses one owned filesystem sandbox and real kernel locking on Linux in a copied Docker source tree with a dedicated build target. The initial focused library and mutation runs use Docker overlay scratch space; public integration fixtures use ext4. Both execute actual files and kernel locks, not simulated syscalls. The broader all-feature suite additionally requires production ext4 admission; its first attempt correctly refused the source-local overlay scratch path in unrelated platform-admission laws. That failed setup run is preserved, and the corrected validation binds both library and integration scratch roots to ext4 without bypassing admission. There is no network dependency, generated input, randomized schedule, format change, benchmark, allocator experiment or physical power-loss claim. Per-test resource enforcement gaps remain those in the binding testing enforcement ledger.

Unmutated debug/release runs exercise the strengthened library law, public `catalog_writer_lock`, `store_initialization`, and filesystem initializer laws. Final required validation and independent review must name the pushed head; source inspection or earlier green CI alone does not certify that head.
