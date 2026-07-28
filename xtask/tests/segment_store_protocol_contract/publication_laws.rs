//! Physical namespace and publication laws.

use super::SPECIFICATION;

#[test]
fn physical_namespace_refuses_aliasing_filesystems() {
    for required in [
        "case-sensitive, byte-preserving directory names",
        "case-folding or normalization aliases",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing physical-namespace capability: {required}"
        );
    }
}

#[test]
fn immutable_pool_links_are_verified_after_namespace_resolution() {
    for required in [
        "After either a new link or an existing-name result",
        "reopens the pool entry without following links",
        "against the pre-link verified bytes and digest",
        "Only that post-link verification advances the protocol",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing post-link verification law: {required}"
        );
    }
}

#[test]
fn leftover_next_head_has_explicit_finalization_or_discard() {
    for required in [
        "Recovery finalizes it only when it",
        "exactly extends the verified current head",
        "never rewrites a retained `head.next`",
        "unlinks the fingerprint-bound `head.next` (`KEEP-CRASH-029`)",
        "synchronizes the store root (`KEEP-CRASH-030`)",
    ] {
        assert!(
            SPECIFICATION.contains(required),
            "missing leftover next-head law: {required}"
        );
    }
}
