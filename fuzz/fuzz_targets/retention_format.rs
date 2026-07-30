#![no_main]

//! This target owns canonical retention-record parser fuzzing.

use keep::{AdmittedRetentionManifest, AdmittedRetentionRoot, ChecksummedRetentionHead};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let Some((&selector, input)) = bytes.split_first() else {
        return;
    };
    match selector {
        0 => root(input),
        1 => manifest(input),
        _ => head(input),
    }
});

fn root(input: &[u8]) {
    if let Ok(root) = AdmittedRetentionRoot::decode(input) {
        assert_eq!(root.encoded(), input);
    }
}

fn manifest(input: &[u8]) {
    if let Ok(manifest) = AdmittedRetentionManifest::decode(input) {
        assert_eq!(manifest.encoded(), input);
    }
}

fn head(input: &[u8]) {
    if let Ok(head) = ChecksummedRetentionHead::decode(input) {
        assert_eq!(head.encoded(), input);
    }
}
