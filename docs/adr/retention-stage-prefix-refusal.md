# Refuse contradictory fixed bytes in interrupted retention stages

This decision owns the fixed-field admission rule for incomplete `root.next`, `manifest.next`, and `head.next` records in PR #99. It implements the existing requirement that corruption refuses recovery without mutation.

A length-first decoder correctly reports that a record is incomplete, but length alone cannot distinguish an interrupted canonical write from garbage. The former implementation admitted `invalid` as truncation, planned a discard, removed the file, and reported successful recovery. The public filesystem regression observes this failure on unfixed production code; the generated public-assessment laws also fail on unfixed commit `4513a776d1c901c659d20a75bcf436c96eedf43a`.

After a decoder reports truncation, recovery compares every available byte of the format's fixed magic, version, header or record width, flags, anchor or entry width, and reserved fields against its canonical value. Contradiction produces the record-specific `PrefixByteMismatch` decode error and a `StageCorrupt` planning refusal. Its diagnostic contains the actual byte offset, expected byte, and observed byte; missing bytes are neither padded nor presented as observations.

The public assessment, filesystem observation, and recovery-context reopening paths use the same stage assessment. Publication invokes explicit recovery before creating its stages, so forward publication and direct restart recovery use the same refusal rule.

Accepting every short byte string was rejected because it destroys corrupt evidence. Rejecting every interrupted write was rejected because the protocol requires recovery of canonical pre-effect prefixes. Checking magic alone was rejected because a short record can already contradict another available fixed field. Completing an incomplete field with fabricated bytes was rejected because that would misrepresent the evidence.

The comparison allocates no memory and examines only bounded fixed-field bytes. Complete-record decoding retains its existing admission and failure order. Durable bytes, identity, publication order, and sync behavior do not change. The public decode-error enums gain a variant, so consumers with exhaustive matches must update their matches.

The filesystem regression requires the exact stage-specific refusal and equality of every retained file's bytes before and after refusal. Generated laws verify strict canonical prefixes and contradictory fixed bytes across interrupted lengths. Diagnostic, valid-prefix, and byte-preservation assertions were separately calibrated by mutations that changed production behavior and failed the named checks. The retention fuzz target now exercises stage assessment as well as complete decoding.

This decision does not claim complete semantic admission of incomplete variable fields, validation of every recovery transition, or physical power-loss evidence. Those remain separate review obligations; the PR is not approved by this focused fix alone.

## Complete generation fields

A complete generation field in an incomplete record is independently decidable. Root generations and global liveness generations must be positive, so recovery uses the existing domain constructors as soon as all eight generation bytes are present. Zero produces the existing record-specific generation error and `StageCorrupt`; a shorter field remains unknown and is not padded or rejected as zero.

Public laws sweep every later strict prefix after a complete zero generation, and the filesystem law requires corruption refusal and equality of every retained file's bytes. They fail on unfixed production `ce54ae50eb99ba6b44d45c43f97b80e64b1e55c4`. A separate production mutation deleting evidence before planning confirms the byte-preservation assertion independently of the refusal. This extends the fixed-field correction without claiming admission of other incomplete semantic fields.
