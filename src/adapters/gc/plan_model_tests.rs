//! Model law: over generated liveness universes, planning never collects a
//! live or named segment, classifies every inventoried segment exactly once,
//! and is a pure function of its snapshot.

use std::collections::BTreeSet;
use std::error::Error;

use super::{closure, coordinates, segment};
use crate::adapters::SegmentDigest;
use crate::adapters::gc::{
    GcLimits, GcLivenessSnapshot, GcPlannedCandidate, GcSegmentClassification, plan_gc,
};

const UNIVERSES: u32 = 512;

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0.wrapping_shl(13);
        self.0 ^= self.0.wrapping_shr(7);
        self.0 ^= self.0.wrapping_shl(17);
        self.0
    }

    fn below(&mut self, bound: u64) -> u64 {
        self.next().checked_rem(bound.max(1)).unwrap_or(0)
    }
}

struct Universe {
    snapshot: GcLivenessSnapshot,
    named: BTreeSet<SegmentDigest>,
    live: BTreeSet<SegmentDigest>,
    released: BTreeSet<SegmentDigest>,
    inventory: BTreeSet<SegmentDigest>,
}

fn universe(random: &mut XorShift) -> Result<Universe, Box<dyn Error>> {
    let mut snapshot = GcLivenessSnapshot::new(coordinates()?);
    let mut named = BTreeSet::new();
    let mut released = BTreeSet::new();
    let mut inventory = BTreeSet::new();
    let count = random.below(8).saturating_add(1);
    for seed in 1..=count {
        let seed = u8::try_from(seed)?;
        let digest = segment(seed);
        let present = random.below(8) != 0;
        if present {
            snapshot.inventory_segment(digest, u64::from(seed))?;
            inventory.insert(digest);
        }
        // Named only when present; superseded, disposed, or bare orphan
        // otherwise, so every classification appears across the universes.
        match random.below(5) {
            0 | 1 if present => {
                snapshot.name_segment(digest);
                named.insert(digest);
            }
            2 => {
                snapshot.supersede_segment(digest);
                released.insert(digest);
            }
            3 => {
                snapshot.dispose_segment(digest);
                released.insert(digest);
            }
            _ => {}
        }
    }
    let mut live = BTreeSet::new();
    let named_list: Vec<_> = named.iter().copied().collect();
    for root in 0..random.below(4) {
        let members: Vec<_> = named_list
            .iter()
            .copied()
            .filter(|_| random.below(2) == 0)
            .collect();
        live.extend(members.iter().copied());
        snapshot.retain(closure(u8::try_from(root.saturating_add(100))?, &members)?)?;
    }
    Ok(Universe {
        snapshot,
        named,
        live,
        released,
        inventory,
    })
}

#[test]
fn planning_never_collects_live_or_named_material_and_is_pure() -> Result<(), Box<dyn Error>> {
    let mut random = XorShift(0x9e37_79b9_7f4a_7c15);
    for _ in 0..UNIVERSES {
        let universe = universe(&mut random)?;
        let plan = plan_gc(&universe.snapshot, GcLimits::MAXIMUM)?;
        let again = plan_gc(&universe.snapshot, GcLimits::MAXIMUM)?;
        assert_eq!(plan, again, "planning is a pure function of its snapshot");

        let classified: BTreeSet<_> = plan.segments().keys().copied().collect();
        assert_eq!(classified, universe.inventory);
        assert_eq!(
            plan.live_segments().collect::<BTreeSet<_>>(),
            universe.live,
            "the live set is exactly the union of retained closures"
        );
        let candidates: Vec<_> = plan.candidates().map(GcPlannedCandidate::segment).collect();
        assert!(candidates.windows(2).all(|pair| pair.first() < pair.last()));
        assert_eq!(usize::try_from(plan.candidate_count())?, candidates.len());
        for candidate in &candidates {
            assert!(!universe.named.contains(candidate));
            assert!(!universe.live.contains(candidate));
            assert!(universe.released.contains(candidate));
            assert!(universe.inventory.contains(candidate));
        }
        for (digest, planned) in plan.segments() {
            let expected_candidate =
                universe.released.contains(digest) && !universe.named.contains(digest);
            assert_eq!(planned.classification().is_candidate(), expected_candidate);
            if universe.live.contains(digest) {
                assert!(matches!(
                    planned.classification(),
                    GcSegmentClassification::Live { .. }
                ));
            }
            if !universe.named.contains(digest) && !universe.released.contains(digest) {
                assert_eq!(
                    planned.classification(),
                    GcSegmentClassification::RecoveryProtected,
                    "an orphan without release evidence stays protected"
                );
            }
        }
        let retired: BTreeSet<_> = universe
            .released
            .difference(&universe.inventory)
            .copied()
            .collect();
        assert_eq!(plan.already_retired(), &retired);
    }
    Ok(())
}
