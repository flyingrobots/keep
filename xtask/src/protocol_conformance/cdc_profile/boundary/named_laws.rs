//! This module owns named witnesses for subtle `FastCDC` boundary semantics.

use super::{Boundaries, schedule::feed_sizes_with};
use crate::protocol_conformance::ConformanceError;
use crate::protocol_conformance::cdc_profile::scalar_fastcdc::{
    StreamingChunker, probe_fingerprint, reference_boundaries,
};
use crate::protocol_conformance::cdc_profile::source::Sources;
use crate::protocol_conformance::cdc_profile::{
    GearTable, LONG_MASK, MAXIMUM, MINIMUM, SHORT_MASK, TARGET,
};

pub(super) fn validate(
    sources: &Sources,
    boundaries: &Boundaries,
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    require_empty_and_runt_laws(sources, boundaries)?;
    require_probe_carry(sources, boundaries, gear)?;
    require_mask_witnesses(sources, boundaries, gear)?;
    require_natural_runt(sources, boundaries)?;
    require_empty_feed_invariance(sources, boundaries, gear)
}

fn require_empty_and_runt_laws(
    sources: &Sources,
    boundaries: &Boundaries,
) -> Result<(), ConformanceError> {
    if !boundaries.get("empty")?.is_empty() {
        return Err(ConformanceError::violation(
            "empty input must emit no chunks",
        ));
    }
    for name in ["tiny", "min-minus-one", "min-exact"] {
        if boundaries.get(name)? != [sources.get(name)?.len()] {
            return Err(ConformanceError::violation(format!(
                "{name}: EOF runt law moved"
            )));
        }
    }
    if boundaries.get("max-exact")? != [MAXIMUM] {
        return Err(ConformanceError::violation(
            "max-exact: maximum-size forced cut moved",
        ));
    }
    if boundaries.get("max-plus-one")? != [MAXIMUM, MAXIMUM + 1] {
        return Err(ConformanceError::violation(
            "max-plus-one: forced cut plus EOF runt law moved",
        ));
    }
    Ok(())
}

fn require_probe_carry(
    sources: &Sources,
    boundaries: &Boundaries,
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    let carry = boundaries.get("probe-byte-carry")?;
    let first = *carry
        .first()
        .ok_or_else(|| ConformanceError::violation("probe-byte-carry first boundary is absent"))?;
    let second = *carry
        .get(1)
        .ok_or_else(|| ConformanceError::violation("probe-byte-carry second boundary is absent"))?;
    let source = sources.get("probe-byte-carry")?;
    let carry_hash = probe_fingerprint(source, first, gear)?;
    let suffix = source
        .get(first..)
        .ok_or_else(|| ConformanceError::violation("probe-byte-carry suffix is absent"))?;
    let suffix_first = *reference_boundaries(suffix, gear)?
        .first()
        .ok_or_else(|| ConformanceError::violation("probe-byte-carry suffix boundary is absent"))?;
    let reset_boundary = first
        .checked_add(suffix_first)
        .ok_or_else(|| ConformanceError::violation("probe-byte-carry reset boundary overflow"))?;
    if carry_hash & LONG_MASK != 0 || reset_boundary != second {
        return Err(ConformanceError::violation(
            "probe-byte-carry: exclusive probe carry/reset witness moved",
        ));
    }
    Ok(())
}

fn require_mask_witnesses(
    sources: &Sources,
    boundaries: &Boundaries,
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    let short = first_boundary(boundaries, "short-mask-match")?;
    let short_hash = probe_fingerprint(sources.get("short-mask-match")?, short, gear)?;
    if !(MINIMUM < short && short < TARGET) || short_hash & SHORT_MASK != 0 {
        return Err(ConformanceError::violation(
            "short-mask-match: short-region witness moved",
        ));
    }
    let transition = first_boundary(boundaries, "target-long-transition")?;
    let transition_hash =
        probe_fingerprint(sources.get("target-long-transition")?, transition, gear)?;
    if transition != TARGET || transition_hash & LONG_MASK != 0 || transition_hash & SHORT_MASK == 0
    {
        return Err(ConformanceError::violation(
            "target-long-transition: exact mask transition witness moved",
        ));
    }
    Ok(())
}

fn require_natural_runt(
    sources: &Sources,
    boundaries: &Boundaries,
) -> Result<(), ConformanceError> {
    let runt = boundaries.get("natural-cut-runt")?;
    let [first, _second] = runt else {
        return Err(ConformanceError::violation(
            "natural-cut-runt: boundary count moved",
        ));
    };
    let remainder = sources
        .get("natural-cut-runt")?
        .len()
        .checked_sub(*first)
        .ok_or_else(|| ConformanceError::violation("natural-cut-runt remainder underflow"))?;
    if remainder == 0 || remainder >= MINIMUM {
        return Err(ConformanceError::violation(
            "natural-cut-runt: EOF remainder witness moved",
        ));
    }
    Ok(())
}

fn require_empty_feed_invariance(
    sources: &Sources,
    boundaries: &Boundaries,
    gear: &GearTable,
) -> Result<(), ConformanceError> {
    let source = sources.get("probe-byte-carry")?;
    let sizes = [1_usize, 4_093, 65_521];
    let mut chunker = StreamingChunker::new(gear);
    feed_sizes_with(
        &mut chunker,
        source,
        &sizes,
        "empty-feed schedule",
        |chunker| {
            let before = chunker.snapshot();
            chunker.feed(&[])?;
            if before != chunker.snapshot() {
                return Err(ConformanceError::violation(
                    "empty-interleaved: empty feed changed chunker state",
                ));
            }
            Ok(())
        },
    )?;
    chunker.finish()?;
    let first = chunker.boundaries()?;
    chunker.feed(&[])?;
    chunker.finish()?;
    if first != boundaries.get("probe-byte-carry")? || chunker.boundaries()? != first {
        return Err(ConformanceError::violation(
            "empty-interleaved: empty feed flushed or moved a boundary",
        ));
    }
    Ok(())
}

fn first_boundary(boundaries: &Boundaries, name: &str) -> Result<usize, ConformanceError> {
    boundaries
        .get(name)?
        .first()
        .copied()
        .ok_or_else(|| ConformanceError::violation(format!("{name}: first boundary is absent")))
}
