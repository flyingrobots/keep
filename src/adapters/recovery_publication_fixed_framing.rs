//! This module owns fixed framing for incomplete recovery publication stages.

use super::recovery_fixed_field_prefix::observed_field;
use super::{
    CatalogDecodeError, PublicationHeadDecodeError, catalog_decoder, catalog_header_decoder,
    publication_head_decoder,
};

pub(super) fn catalog_header(encoded: &[u8]) -> Result<(), CatalogDecodeError> {
    let magic = observed_field(encoded, 0, catalog_decoder::MAGIC);
    if magic != catalog_decoder::MAGIC {
        return Err(CatalogDecodeError::InvalidMagic { observed: magic });
    }
    let version = u16::from_be_bytes(observed_field(
        encoded,
        16,
        catalog_decoder::VERSION.to_be_bytes(),
    ));
    if version != catalog_decoder::VERSION {
        return Err(CatalogDecodeError::UnsupportedVersion {
            expected: catalog_decoder::VERSION,
            observed: version,
        });
    }
    validate_catalog_widths(encoded)?;
    validate_catalog_coordinates(encoded)
}

fn validate_catalog_widths(encoded: &[u8]) -> Result<(), CatalogDecodeError> {
    let flags = u16::from_be_bytes(observed_field(
        encoded,
        18,
        catalog_decoder::FLAGS.to_be_bytes(),
    ));
    if flags != catalog_decoder::FLAGS {
        return Err(CatalogDecodeError::Flags {
            expected: catalog_decoder::FLAGS,
            observed: flags,
        });
    }
    let header = u16::from_be_bytes(observed_field(
        encoded,
        20,
        catalog_header_decoder::HEADER_LENGTH.to_be_bytes(),
    ));
    if header != catalog_header_decoder::HEADER_LENGTH {
        return Err(CatalogDecodeError::HeaderLength {
            expected: catalog_header_decoder::HEADER_LENGTH,
            observed: header,
        });
    }
    let entry = u16::from_be_bytes(observed_field(
        encoded,
        22,
        catalog_header_decoder::ENTRY_LENGTH.to_be_bytes(),
    ));
    if entry != catalog_header_decoder::ENTRY_LENGTH {
        return Err(CatalogDecodeError::EntryLength {
            expected: catalog_header_decoder::ENTRY_LENGTH,
            observed: entry,
        });
    }
    Ok(())
}

fn validate_catalog_coordinates(encoded: &[u8]) -> Result<(), CatalogDecodeError> {
    let checksum = u8::from_be_bytes(observed_field(encoded, 80, [catalog_decoder::ALGORITHM]));
    if checksum != catalog_decoder::ALGORITHM {
        return Err(CatalogDecodeError::ChecksumAlgorithm {
            expected: catalog_decoder::ALGORITHM,
            observed: checksum,
        });
    }
    let digest = u8::from_be_bytes(observed_field(encoded, 81, [catalog_decoder::ALGORITHM]));
    if digest != catalog_decoder::ALGORITHM {
        return Err(CatalogDecodeError::DigestAlgorithm {
            expected: catalog_decoder::ALGORITHM,
            observed: digest,
        });
    }
    let expected = [0_u8; 46];
    let observed = observed_field(encoded, 82, expected);
    if observed == expected {
        Ok(())
    } else {
        Err(CatalogDecodeError::Reserved { expected, observed })
    }
}

pub(super) fn next_head(encoded: &[u8]) -> Result<(), PublicationHeadDecodeError> {
    let magic = observed_field(encoded, 0, publication_head_decoder::MAGIC);
    if magic != publication_head_decoder::MAGIC {
        return Err(PublicationHeadDecodeError::InvalidMagic { observed: magic });
    }
    let version = u16::from_be_bytes(observed_field(
        encoded,
        16,
        publication_head_decoder::VERSION.to_be_bytes(),
    ));
    if version != publication_head_decoder::VERSION {
        return Err(PublicationHeadDecodeError::UnsupportedVersion {
            expected: publication_head_decoder::VERSION,
            observed: version,
        });
    }
    validate_next_head_coordinates(encoded)
}

fn validate_next_head_coordinates(encoded: &[u8]) -> Result<(), PublicationHeadDecodeError> {
    let flags = u16::from_be_bytes(observed_field(
        encoded,
        18,
        publication_head_decoder::FLAGS.to_be_bytes(),
    ));
    if flags != publication_head_decoder::FLAGS {
        return Err(PublicationHeadDecodeError::Flags {
            expected: publication_head_decoder::FLAGS,
            observed: flags,
        });
    }
    let length = u16::from_be_bytes(observed_field(
        encoded,
        20,
        publication_head_decoder::HEAD_LENGTH.to_be_bytes(),
    ));
    if length != publication_head_decoder::HEAD_LENGTH {
        return Err(PublicationHeadDecodeError::HeadLength {
            expected: publication_head_decoder::HEAD_LENGTH,
            observed: length,
        });
    }
    validate_next_head_algorithms(encoded)?;
    let expected = [0_u8; 24];
    let observed = observed_field(encoded, 72, expected);
    if observed == expected {
        Ok(())
    } else {
        Err(PublicationHeadDecodeError::Reserved { expected, observed })
    }
}

fn validate_next_head_algorithms(encoded: &[u8]) -> Result<(), PublicationHeadDecodeError> {
    let checksum = u8::from_be_bytes(observed_field(
        encoded,
        22,
        [publication_head_decoder::ALGORITHM],
    ));
    if checksum != publication_head_decoder::ALGORITHM {
        return Err(PublicationHeadDecodeError::ChecksumAlgorithm {
            expected: publication_head_decoder::ALGORITHM,
            observed: checksum,
        });
    }
    let digest = u8::from_be_bytes(observed_field(
        encoded,
        23,
        [publication_head_decoder::ALGORITHM],
    ));
    if digest == publication_head_decoder::ALGORITHM {
        Ok(())
    } else {
        Err(PublicationHeadDecodeError::DigestAlgorithm {
            expected: publication_head_decoder::ALGORITHM,
            observed: digest,
        })
    }
}
