# Retention recovery successor entries

This decision owns preservation of unrelated namespace entries when recovery admits a staged successor manifest. A valid checksum, successor generation, predecessor digest, and matching staged root do not establish that the remaining entries preserve the published state.

Test commit `f036229` records real-filesystem failures on unfixed production `514a521`: canonical staged manifests dropped or altered an unrelated namespace, and recovery linked the manifest or finalized the head. The retained-byte preservation assertion failed in each production path.

Recovery now compares the canonically ordered entry streams after excluding the staged root's namespace. Every remaining namespace, root generation, and root digest must agree exactly. The comparison allocates nothing and rejects additions, omissions, and coordinate changes with `ManifestNotSuccessor` before publication effects. With no published state, no unrelated entries are admitted.

Manifest-link planning and head-finalization planning enforce the same rule. Already-committed cleanup retains its existing handling: the observed current manifest is the staged manifest itself, and this comparison cannot reconstruct an unavailable predecessor. Reopening predecessor roots and verifying staged closure remain separate obligations.

Checking only that the candidate root appears was rejected because that permits unrelated retention changes. Reconstructing a replacement manifest was rejected because recovery must refuse contradictory evidence rather than silently repair it. Durable encodings, logical identities, and public error variants remain unchanged.

The filesystem regressions observe exact retained bytes and the typed refusal for both production paths. They do not establish exhaustive generated-input coverage, all historical-state admission, concurrency safety, or physical power-loss durability.
