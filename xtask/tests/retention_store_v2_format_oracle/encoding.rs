// This included source owns primitive binary construction and fixture transport.

use std::fmt::Write as _;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn hash(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain);
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn array_32(bytes: &[u8], offset: usize) -> Result<[u8; 32], String> {
    let end = offset
        .checked_add(32)
        .ok_or_else(|| "32-byte field offset overflow".to_owned())?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| format!("missing 32-byte field at offset {offset}"))?;
    <[u8; 32]>::try_from(field).map_err(|_| "32-byte field conversion failed".to_owned())
}

fn u64_at(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let end = offset
        .checked_add(8)
        .ok_or_else(|| "u64 field offset overflow".to_owned())?;
    let field = bytes
        .get(offset..end)
        .ok_or_else(|| format!("missing u64 field at offset {offset}"))?;
    let encoded =
        <[u8; 8]>::try_from(field).map_err(|_| "u64 field conversion failed".to_owned())?;
    Ok(u64::from_be_bytes(encoded))
}

fn decode_hex(source: &str) -> Result<Vec<u8>, String> {
    let encoded = source
        .strip_suffix('\n')
        .ok_or_else(|| "hex fixture lacks one final newline".to_owned())?;
    if encoded.contains('\n') || encoded.contains('\r') {
        return Err("hex fixture contains embedded line ending".to_owned());
    }
    if encoded.len() % 2 != 0 {
        return Err("hex fixture has odd encoded length".to_owned());
    }
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let [high_byte, low_byte] = <[u8; 2]>::try_from(pair)
                .map_err(|_| "hex pair conversion failed".to_owned())?;
            let high = hex_nibble(high_byte)?;
            let low = hex_nibble(low_byte)?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Result<u8, String> {
    match byte {
        b'0'..=b'9' => byte
            .checked_sub(b'0')
            .ok_or_else(|| "decimal hex nibble underflow".to_owned()),
        b'a'..=b'f' => byte
            .checked_sub(b'a')
            .and_then(|value| value.checked_add(10))
            .ok_or_else(|| "alphabetic hex nibble overflow".to_owned()),
        _ => Err("hex fixture contains a non-lowercase hexadecimal byte".to_owned()),
    }
}

fn encode_hex(bytes: &[u8]) -> Result<String, String> {
    let capacity = bytes
        .len()
        .checked_mul(2)
        .and_then(|length| length.checked_add(1))
        .ok_or_else(|| "hex output length overflow".to_owned())?;
    let mut encoded = String::with_capacity(capacity);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}")
            .map_err(|_| "hex output formatting failed".to_owned())?;
    }
    encoded.push('\n');
    Ok(encoded)
}

fn repository_root() -> Result<PathBuf, io::Error> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| io::Error::other("xtask manifest directory has no parent"))
}

fn corpus_path(relative: &str) -> Result<PathBuf, io::Error> {
    Ok(repository_root()?.join(CORPUS_ROOT).join(relative))
}

fn read_corpus_file(relative: &str) -> Result<String, io::Error> {
    fs::read_to_string(corpus_path(relative)?)
}

fn require_length(bytes: &[u8], expected: usize, name: &str) -> Result<(), String> {
    if bytes.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "{name} has {} bytes; expected {expected}",
            bytes.len()
        ))
    }
}
