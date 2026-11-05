//! Exact retention records for retained-closure publication laws.

use std::error::Error;

use keep::{
    AdmittedRetentionManifest, AdmittedRetentionRoot, CanonicalRetentionHead,
    CanonicalRetentionManifest, CanonicalRetentionRoot, ChecksummedRetentionHead,
    LivenessGeneration, RetentionHead, RetentionManifest, RetentionManifestDigest,
    RetentionManifestEntry, RetentionManifestLength, RetentionNamespace, RetentionPolicy,
    RetentionRoot, RetentionRootDigest, RootGeneration,
};

use crate::support::decode_hex;

const ROOT_HEX: &str = include_str!("../../conformance/segment-store/v2/one-anchor-root.hex");
const MANIFEST_HEX: &str = include_str!("../../conformance/segment-store/v2/one-root-manifest.hex");
const HEAD_HEX: &str = include_str!("../../conformance/segment-store/v2/one-root-head.hex");

/// Whether a built root retains the one-zero blob or nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RootShape {
    /// Anchors the frozen one-zero blob and layout.
    Retaining,
    /// Publishes an empty anchor set.
    Released,
}

/// Exact retention records served by a storage model.
#[derive(Clone, Debug)]
pub struct RetentionRecords {
    /// Current `retention/HEAD` bytes, absent when unpublished.
    pub head: Option<Vec<u8>>,
    /// Manifest pool entry keyed by the digest storage resolves it with.
    pub manifest: Option<(RetentionManifestDigest, Vec<u8>)>,
    /// Root pool entries keyed by the digest storage resolves them with.
    pub roots: Vec<(RetentionRootDigest, Vec<u8>)>,
}

impl RetentionRecords {
    /// Records of a store that never published retention.
    pub const fn unretained() -> Self {
        Self {
            head: None,
            manifest: None,
            roots: Vec::new(),
        }
    }

    /// Frozen one-root generation-one records.
    ///
    /// # Errors
    ///
    /// Returns the exact fixture or decoder refusal.
    pub fn golden() -> Result<Self, Box<dyn Error>> {
        let root = fixture(ROOT_HEX)?;
        let manifest = fixture(MANIFEST_HEX)?;
        let root_digest = AdmittedRetentionRoot::decode(&root)?.digest();
        let manifest_digest = AdmittedRetentionManifest::decode(&manifest)?.digest();
        Ok(Self {
            head: Some(fixture(HEAD_HEX)?),
            manifest: Some((manifest_digest, manifest)),
            roots: vec![(root_digest, root)],
        })
    }

    /// Builds generation-one records naming one root per namespace.
    ///
    /// # Errors
    ///
    /// Returns the exact namespace, root, manifest, or head refusal.
    pub fn built(roots: &[(&[u8], RootShape)]) -> Result<Self, Box<dyn Error>> {
        let mut entries = Vec::new();
        let mut encoded_roots = Vec::new();
        for (namespace, shape) in roots {
            let canonical = CanonicalRetentionRoot::from_root(&root(namespace, *shape)?)?;
            let namespace_digest = RetentionNamespace::try_from(*namespace)?.digest();
            entries.push(RetentionManifestEntry::new(
                namespace_digest,
                RootGeneration::INITIAL,
                canonical.digest(),
            ));
            encoded_roots.push((canonical.digest(), canonical.encoded().to_vec()));
        }
        let manifest = RetentionManifest::new(LivenessGeneration::INITIAL, None, entries)?;
        let canonical = CanonicalRetentionManifest::from_manifest(&manifest)?;
        let length = RetentionManifestLength::new(u64::try_from(canonical.encoded().len())?)?;
        let head = RetentionHead::new(
            LivenessGeneration::INITIAL,
            length,
            canonical.digest(),
            None,
        )?;
        Ok(Self {
            head: Some(encode_head(&head)),
            manifest: Some((canonical.digest(), canonical.encoded().to_vec())),
            roots: encoded_roots,
        })
    }

    /// Returns the admitted current head.
    ///
    /// # Errors
    ///
    /// Returns a missing-head or decoder refusal.
    pub fn current_head(&self) -> Result<RetentionHead, Box<dyn Error>> {
        let encoded = self.head.as_ref().ok_or("records publish no head")?;
        Ok(*ChecksummedRetentionHead::decode(encoded)?.head())
    }

    /// Returns manifest entries in canonical order.
    ///
    /// # Errors
    ///
    /// Returns a missing-manifest or decoder refusal.
    pub fn entries(&self) -> Result<Vec<RetentionManifestEntry>, Box<dyn Error>> {
        let (_digest, encoded) = self.manifest.as_ref().ok_or("records hold no manifest")?;
        Ok(AdmittedRetentionManifest::decode(encoded)?
            .manifest()
            .entries()
            .to_vec())
    }
}

/// Encodes one semantic head with a correct checksum.
pub fn encode_head(head: &RetentionHead) -> Vec<u8> {
    CanonicalRetentionHead::from_head(head).encoded().to_vec()
}

/// Decodes one LF-terminated lowercase hexadecimal fixture.
///
/// # Errors
///
/// Returns a framing or hexadecimal transport refusal.
pub fn fixture(hex: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    decode_hex(hex.strip_suffix('\n').ok_or("fixture must end in one LF")?).map_err(Into::into)
}

fn root(namespace: &[u8], shape: RootShape) -> Result<RetentionRoot, Box<dyn Error>> {
    let template_bytes = fixture(ROOT_HEX)?;
    let template = AdmittedRetentionRoot::decode(&template_bytes)?;
    let anchors = match shape {
        RootShape::Retaining => template.root().anchors().to_vec(),
        RootShape::Released => Vec::new(),
    };
    Ok(RetentionRoot::new(
        RetentionNamespace::try_from(namespace)?,
        RootGeneration::INITIAL,
        RetentionPolicy::new(template.root().profile(), template.root().limits()),
        None,
        anchors,
    )?)
}
