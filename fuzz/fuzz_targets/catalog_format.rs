#![no_main]

//! This target owns canonical catalog and publication-head parser fuzzing.

use keep::{ChecksummedCatalog, ChecksummedPublicationHead};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|bytes: &[u8]| {
    let Some((&selector, input)) = bytes.split_first() else {
        return;
    };
    if selector == 0 {
        catalog(input);
    } else {
        publication_head(input);
    }
});

fn catalog(input: &[u8]) {
    if let Ok(catalog) = ChecksummedCatalog::decode(input) {
        assert_eq!(catalog.encoded(), input);
        assert_eq!(
            u64::try_from(catalog.encoded().len()),
            Ok(catalog.length().get())
        );
    }
}

fn publication_head(input: &[u8]) {
    if let Ok(head) = ChecksummedPublicationHead::decode(input) {
        assert_eq!(head.encoded(), input);
    }
}
