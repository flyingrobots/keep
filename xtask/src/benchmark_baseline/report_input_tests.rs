//! Laws for report input bounds before decoding or allocation-heavy admission.

use super::super::report_input::ReportInputError;
use super::{BenchmarkBaselineError, artifact, environment};

#[test]
fn oversized_reports_refuse_before_encoding_admission() -> Result<(), Box<dyn std::error::Error>> {
    let maximum = super::super::REPORT_LIMIT;
    let observed = maximum
        .checked_add(1)
        .ok_or("input fixture size overflow")?;
    let oversized = vec![0xff; observed];
    assert!(matches!(artifact::validate(&oversized, &environment()),
        Err(BenchmarkBaselineError::ReportInput(ReportInputError::Bound {
            maximum: actual_maximum, observed: actual_observed,
        })) if actual_maximum == maximum && actual_observed == observed));
    Ok(())
}

#[test]
fn exact_input_limit_reaches_decoding_without_size_refusal()
-> Result<(), Box<dyn std::error::Error>> {
    let maximum = super::super::REPORT_LIMIT;
    let valid_encoding = vec![b'a'; maximum];
    let decoded = super::super::report_input::decode(&valid_encoding)?;
    assert_eq!(decoded.len(), maximum);
    let invalid_encoding = vec![0xff; maximum];
    assert!(
        matches!(super::super::report_input::decode(&invalid_encoding),
        Err(BenchmarkBaselineError::ReportInput(ReportInputError::Encoding { source }))
            if source.valid_up_to() == 0 && source.error_len() == Some(1))
    );
    Ok(())
}

#[test]
fn report_encoding_failure_retains_the_original_utf8_source()
-> Result<(), Box<dyn std::error::Error>> {
    use std::error::Error;
    let error = artifact::validate(&[0xff], &environment())
        .err()
        .ok_or("invalid encoding was admitted")?;
    assert!(matches!(&error,
        BenchmarkBaselineError::ReportInput(ReportInputError::Encoding { source })
            if source.valid_up_to() == 0 && source.error_len() == Some(1)));
    assert_eq!(
        error
            .source()
            .and_then(Error::source)
            .and_then(|source| source.downcast_ref::<std::str::Utf8Error>())
            .map(std::str::Utf8Error::valid_up_to),
        Some(0)
    );
    Ok(())
}
