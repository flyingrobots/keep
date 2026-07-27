//! This module owns canonical mutation-field parsing and bounded application.

use super::GoldenError;
use super::canonical_value::{EmptyHex, decimal, decoded_hex};
use super::corpus_protocol::{MAX_MUTATION_VALUE_BYTES, MAX_SOURCE_BYTES};

#[derive(Clone, Copy)]
enum FixedWidthOperation {
    SetU8,
    SetU16Be,
    XorByte,
}

pub(super) fn mutation_offset(
    value: &str,
    target_length: usize,
    name: &str,
) -> Result<usize, GoldenError> {
    let maximum = u64::try_from(target_length).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: target length cannot be represented: {source}"
        ))
    })?;
    let offset = decimal(value, &format!("{name} offset"), maximum)?;
    usize::try_from(offset).map_err(|source| {
        GoldenError::violation(format!(
            "{name}: mutation offset cannot fit this platform: {source}"
        ))
    })
}

pub(super) fn mutate(
    target: &[u8],
    operation: &str,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<Vec<u8>, GoldenError> {
    let mut changed = target.to_vec();
    match operation {
        "truncate" => truncate(&mut changed, offset, value_field, name)?,
        "append" => append(&mut changed, offset, value_field, name)?,
        "xor-byte" | "set-u8" | "set-u16-be" => {
            apply_fixed_width(&mut changed, operation, offset, value_field, name)?;
        }
        _ => {
            return Err(GoldenError::violation(format!(
                "{name}: unknown mutation operation {operation:?}"
            )));
        }
    }
    let maximum = MAX_SOURCE_BYTES
        .checked_add(MAX_MUTATION_VALUE_BYTES)
        .ok_or_else(|| GoldenError::violation("mutation bound overflow"))?;
    if changed == target || changed.len() > maximum {
        Err(GoldenError::violation(format!(
            "{name}: mutation is a no-op or exceeds its bound"
        )))
    } else {
        Ok(changed)
    }
}

fn truncate(
    changed: &mut Vec<u8>,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    if value_field != "-" || offset >= changed.len() {
        return Err(GoldenError::violation(format!(
            "{name}: invalid truncation"
        )));
    }
    changed.truncate(offset);
    Ok(())
}

fn append(
    changed: &mut Vec<u8>,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    let value = decoded_hex(
        value_field,
        &format!("{name} value"),
        MAX_MUTATION_VALUE_BYTES,
        EmptyHex::Refuse,
    )?;
    if offset != changed.len() {
        return Err(GoldenError::violation(format!(
            "{name}: append offset does not equal target length"
        )));
    }
    changed.extend_from_slice(&value);
    Ok(())
}

pub(super) fn apply_fixed_width(
    changed: &mut [u8],
    operation: &str,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    let operation = FixedWidthOperation::admit(operation, name)?;
    let width = operation.width();
    let expected_digits = width
        .checked_mul(2)
        .ok_or_else(|| GoldenError::violation(format!("{name}: mutation width overflow")))?;
    if value_field.len() != expected_digits {
        return Err(GoldenError::violation(format!(
            "{name}: mutation value must be exactly {width} bytes"
        )));
    }
    let value = decoded_hex(
        value_field,
        &format!("{name} value"),
        width,
        EmptyHex::Refuse,
    )?;
    let end = offset
        .checked_add(width)
        .ok_or_else(|| GoldenError::violation(format!("{name}: mutation offset overflow")))?;
    let destination = changed.get_mut(offset..end).ok_or_else(|| {
        GoldenError::violation(format!("{name}: mutation width or offset is invalid"))
    })?;
    match operation {
        FixedWidthOperation::XorByte => xor_byte(destination, &value, name)?,
        FixedWidthOperation::SetU8 | FixedWidthOperation::SetU16Be => {
            destination.copy_from_slice(&value);
        }
    }
    Ok(())
}

impl FixedWidthOperation {
    fn admit(value: &str, name: &str) -> Result<Self, GoldenError> {
        match value {
            "set-u8" => Ok(Self::SetU8),
            "set-u16-be" => Ok(Self::SetU16Be),
            "xor-byte" => Ok(Self::XorByte),
            _ => Err(GoldenError::violation(format!(
                "{name}: unknown fixed-width mutation operation {value:?}"
            ))),
        }
    }

    const fn width(self) -> usize {
        match self {
            Self::SetU8 | Self::XorByte => 1,
            Self::SetU16Be => 2,
        }
    }
}

fn xor_byte(destination: &mut [u8], value: &[u8], name: &str) -> Result<(), GoldenError> {
    let [destination] = destination else {
        return Err(GoldenError::violation(format!(
            "{name}: mutation width is invalid"
        )));
    };
    let [value] = value else {
        return Err(GoldenError::violation(format!(
            "{name}: mutation value is invalid"
        )));
    };
    *destination ^= *value;
    Ok(())
}
