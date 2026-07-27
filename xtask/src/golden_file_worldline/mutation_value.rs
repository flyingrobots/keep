//! This module owns canonical mutation-field parsing and bounded application.

use super::GoldenError;
use super::canonical_value::{EmptyHex, decimal, decoded_hex};
use super::corpus_protocol::{MAX_MUTATION_VALUE_BYTES, MAX_SOURCE_BYTES};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum MutationOperation {
    Truncate,
    Append,
    XorByte,
    SetU8,
    SetU16Be,
}

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
    operation: MutationOperation,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<Vec<u8>, GoldenError> {
    let mut changed = target.to_vec();
    match operation {
        MutationOperation::Truncate => truncate(&mut changed, offset, value_field, name)?,
        MutationOperation::Append => append(&mut changed, offset, value_field, name)?,
        MutationOperation::XorByte | MutationOperation::SetU8 | MutationOperation::SetU16Be => {
            apply_fixed_width(&mut changed, operation, offset, value_field, name)?;
        }
    }
    let maximum = MAX_SOURCE_BYTES
        .checked_add(MAX_MUTATION_VALUE_BYTES)
        .ok_or_else(|| GoldenError::violation("mutation bound overflow"))?;
    if changed == target {
        return Err(GoldenError::violation(format!(
            "{name}: mutation is a no-op"
        )));
    }
    if changed.len() > maximum {
        return Err(GoldenError::violation(format!(
            "{name}: mutation produced {} bytes, exceeding {maximum}",
            changed.len()
        )));
    }
    Ok(changed)
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
    operation: MutationOperation,
    offset: usize,
    value_field: &str,
    name: &str,
) -> Result<(), GoldenError> {
    let operation = operation.fixed_width(name)?;
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
            copy_fixed_width(destination, &value, name)?;
        }
    }
    Ok(())
}

pub(super) fn copy_fixed_width(
    destination: &mut [u8],
    value: &[u8],
    name: &str,
) -> Result<(), GoldenError> {
    if destination.len() == value.len() {
        destination.copy_from_slice(value);
        Ok(())
    } else {
        Err(GoldenError::violation(format!(
            "{name}: mutation value width does not match its destination"
        )))
    }
}

impl MutationOperation {
    pub(super) fn admit(value: &str, name: &str) -> Result<Self, GoldenError> {
        match value {
            "truncate" => Ok(Self::Truncate),
            "append" => Ok(Self::Append),
            "xor-byte" => Ok(Self::XorByte),
            "set-u8" => Ok(Self::SetU8),
            "set-u16-be" => Ok(Self::SetU16Be),
            _ => Err(GoldenError::violation(format!(
                "{name}: unknown mutation operation {value:?}"
            ))),
        }
    }

    fn fixed_width(self, name: &str) -> Result<FixedWidthOperation, GoldenError> {
        match self {
            Self::SetU8 => Ok(FixedWidthOperation::SetU8),
            Self::SetU16Be => Ok(FixedWidthOperation::SetU16Be),
            Self::XorByte => Ok(FixedWidthOperation::XorByte),
            Self::Truncate | Self::Append => Err(GoldenError::violation(format!(
                "{name}: mutation operation is not fixed-width"
            ))),
        }
    }
}

impl FixedWidthOperation {
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
