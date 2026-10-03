//! Specified record inputs and an in-memory serialization sink; no durability claim.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::error::Error;
use std::io::{self, Write};
use std::rc::Rc;

use crate::support::decode_hex;
use keep::{
    AdmittedLayout, AdmittedSegmentRecord, ChunkId, LayoutDecodePolicy, LayoutEntryLimit, LayoutId,
    SegmentRecordIdentity, SegmentRecordLimit, SegmentStage, StagedSegment,
};

type ResultOf<T> = Result<T, Box<dyn Error>>;
pub(super) type RecordMap = BTreeMap<SegmentRecordIdentity, Vec<u8>>;

pub(super) fn inputs() -> ResultOf<RecordMap> {
    let mut records = RecordMap::new();
    for bytes in [vec![0], vec![1, 2, 3]] {
        records.insert(
            SegmentRecordIdentity::Chunk(ChunkId::hash_bytes(&bytes)?),
            bytes,
        );
    }
    // Oracle: independently frozen layouts.tsv identities and literal fixture bytes,
    // never decoded segment records or catalog enumeration.
    for (id, hex) in [
        (
            "keep:layout:v1:flat-chunks-v1:blake3-256:176:539516cc52fe7433e3a984a4e2a4877676673bb62d6203a88dfe1ce253c2c0f8",
            include_str!("../../conformance/layout/v1/empty.layout.hex"),
        ),
        (
            "keep:layout:v1:flat-chunks-v1:blake3-256:220:887da23f1a7483359a78fc9a7fde80030ec2c4690603803f0ab7d0edb56575b8",
            include_str!("../../conformance/layout/v1/one-zero.layout.hex"),
        ),
    ] {
        records.insert(
            SegmentRecordIdentity::Layout(id.parse::<LayoutId>()?),
            decode_hex(hex.trim_end())?,
        );
    }
    Ok(records)
}

pub(super) fn select(inputs: &RecordMap, mask: u8) -> ResultOf<RecordMap> {
    let mut selected = RecordMap::new();
    for (index, (identity, bytes)) in inputs.iter().enumerate() {
        let bit = 1_u8
            .checked_shl(u32::try_from(index)?)
            .ok_or("input mask overflow")?;
        if mask & bit != 0 {
            selected.insert(*identity, bytes.clone());
        }
    }
    Ok(selected)
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Packing {
    Bundled,
    SeparateReversed,
}

pub(super) fn encode(records: &RecordMap, packing: Packing) -> ResultOf<Vec<Vec<u8>>> {
    let entries: Vec<_> = records.iter().collect();
    match packing {
        Packing::Bundled if entries.is_empty() => Ok(Vec::new()),
        Packing::Bundled => Ok(vec![segment(&entries)?]),
        Packing::SeparateReversed => entries
            .iter()
            .rev()
            .map(|entry| segment(&[*entry]))
            .collect(),
    }
}

fn segment(records: &[(&SegmentRecordIdentity, &Vec<u8>)]) -> ResultOf<Vec<u8>> {
    let output = Rc::new(RefCell::new(Vec::new()));
    let mut staged =
        StagedSegment::begin(MemoryStage(Rc::clone(&output)), SegmentRecordLimit::MAXIMUM)?;
    for (identity, bytes) in records {
        staged = match identity {
            SegmentRecordIdentity::Chunk(_) => {
                staged.append(AdmittedSegmentRecord::for_chunk(bytes)?)?
            }
            SegmentRecordIdentity::Layout(_) => {
                let layout = AdmittedLayout::decode_record(
                    bytes,
                    LayoutDecodePolicy::new(LayoutEntryLimit::MAXIMUM),
                )?
                .encode_record()?;
                staged.append(AdmittedSegmentRecord::for_layout(&layout)?)?
            }
        };
    }
    let _sealed = staged.seal()?;
    let bytes = output.borrow().clone();
    Ok(bytes)
}

struct MemoryStage(Rc<RefCell<Vec<u8>>>);

impl Write for MemoryStage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl SegmentStage for MemoryStage {
    fn synchronize(&mut self) -> io::Result<()> {
        Ok(())
    }
}
