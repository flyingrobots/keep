//! Exact semantic oracles shared by Keep fuzz targets.

use keep::{ChunkHashError, ChunkId, ChunkSpan};

/// Failure while validating the semantic coverage of emitted chunk spans.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpanOracleError {
    /// The platform input length cannot be represented as a stream coordinate.
    InputLengthOutOfRange {
        /// Platform slice length that could not be represented.
        observed: usize,
    },
    /// A span did not begin at the end of its predecessor.
    NonContiguous {
        /// Required inclusive start coordinate.
        expected_start: u64,
        /// Emitted inclusive start coordinate.
        observed_start: u64,
    },
    /// A span ended at or before its start coordinate.
    NonAdvancing {
        /// Emitted inclusive start coordinate.
        start: u64,
        /// Emitted exclusive end coordinate.
        end: u64,
    },
    /// A span's coordinates disagreed with its identity length.
    LengthMismatch {
        /// Length committed by the span identity.
        expected: u32,
        /// Length calculated from the span coordinates.
        observed: u64,
    },
    /// A span coordinate could not be represented as a platform slice index.
    CoordinateOutOfRange {
        /// Stream coordinate that could not be represented.
        observed: u64,
    },
    /// A span's coordinates escaped the fuzz input.
    SpanEscapedInput {
        /// Inclusive platform slice start.
        start: usize,
        /// Exclusive platform slice end.
        end: usize,
        /// Exact fuzz input length.
        input_length: usize,
    },
    /// Rehashing the exact span bytes failed.
    ChunkHash(ChunkHashError),
    /// A span identity did not name its exact emitted bytes.
    IdentityMismatch {
        /// Identity calculated from the exact span bytes.
        expected: ChunkId,
        /// Identity emitted by the detector.
        observed: ChunkId,
    },
    /// Emitted spans did not cover the complete fuzz input.
    IncompleteCoverage {
        /// Exact input length that required coverage.
        expected: usize,
        /// End coordinate reached by the emitted spans.
        observed: u64,
    },
}

/// Validates that emitted spans cover the complete fuzz input.
///
/// # Errors
///
/// Returns a typed failure when a span is discontinuous, malformed, outside
/// the input, misidentified, or leaves any input byte uncovered.
pub fn validate_spans(input: &[u8], spans: &[ChunkSpan]) -> Result<(), SpanOracleError> {
    let input_length =
        u64::try_from(input.len()).map_err(|_source| SpanOracleError::InputLengthOutOfRange {
            observed: input.len(),
        })?;
    let mut expected_start = 0_u64;
    for span in spans {
        let start = span.offset().get();
        let end = span.end().get();
        if start != expected_start {
            return Err(SpanOracleError::NonContiguous {
                expected_start,
                observed_start: start,
            });
        }
        let observed_length = end
            .checked_sub(start)
            .filter(|length| *length != 0)
            .ok_or(SpanOracleError::NonAdvancing { start, end })?;
        let expected_length = span.length().get();
        if observed_length != u64::from(expected_length) {
            return Err(SpanOracleError::LengthMismatch {
                expected: expected_length,
                observed: observed_length,
            });
        }
        let platform_start = usize::try_from(start)
            .map_err(|_source| SpanOracleError::CoordinateOutOfRange { observed: start })?;
        let platform_end = usize::try_from(end)
            .map_err(|_source| SpanOracleError::CoordinateOutOfRange { observed: end })?;
        let chunk =
            input
                .get(platform_start..platform_end)
                .ok_or(SpanOracleError::SpanEscapedInput {
                    start: platform_start,
                    end: platform_end,
                    input_length: input.len(),
                })?;
        let expected_id = ChunkId::hash_bytes(chunk).map_err(SpanOracleError::ChunkHash)?;
        if expected_id != span.id() {
            return Err(SpanOracleError::IdentityMismatch {
                expected: expected_id,
                observed: span.id(),
            });
        }
        expected_start = end;
    }
    if expected_start != input_length {
        return Err(SpanOracleError::IncompleteCoverage {
            expected: input.len(),
            observed: expected_start,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use keep::{ChunkSpan, FastCdc};

    use super::{SpanOracleError, validate_spans};

    #[test]
    fn nonempty_input_requires_at_least_one_covering_span() {
        assert_eq!(
            validate_spans(b"x", &[]),
            Err(SpanOracleError::IncompleteCoverage {
                expected: 1,
                observed: 0,
            })
        );
    }

    #[test]
    fn detector_output_covers_the_exact_input() -> Result<(), SpanOracleError> {
        let input = b"exact input";
        let spans = detect(input);
        validate_spans(input, &spans)
    }

    fn detect(input: &[u8]) -> Vec<ChunkSpan> {
        let mut detector = FastCdc::new();
        let mut spans = Vec::new();
        let feed = detector.feed(input, |span| spans.push(span));
        assert_eq!(feed, Ok(()));
        let finish = detector.finish();
        assert!(finish.is_ok());
        if let Ok(Some(span)) = finish {
            spans.push(span);
        }
        spans
    }
}
