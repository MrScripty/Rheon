//! Versioned, bit-preserving force metadata inside the existing archive envelope.
//! Legacy SQL schema, legacy field inventories and hash algorithms are unchanged.
use super::*;
use crate::{
    ForceKind, ForceObject, ForceScene, ForceSnapshot, ForceSupport, ForceTime, ForceTransform,
    ForceUnits, SceneObjectId,
};

pub const FORCE_METADATA_VERSION: u32 = 1;
pub const MAX_FORCE_METADATA_BYTES: usize = 256 * 1024;
const MAGIC: &[u8; 8] = b"RHEONFF\0";
pub(super) const FIELD: &str = "external_force_scene_v1";
const MARKER: &str = "\nexternal-force-metadata-v1-sha256=";
const ASSOCIATION: &str = "immutable_run_input; world_m; activation_at_interval_start_s";
const STAGE: &str = "external_force_document_v1";

/// Little-endian binary encoding with explicit codec and authoring versions.
/// Floats retain exact bits, including signed zero; input is already validated.
pub fn encode_force_metadata(fields: &ForceSnapshot) -> Result<Vec<u8>, RecordingError> {
    let scene = fields.scene();
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORCE_METADATA_VERSION.to_le_bytes());
    out.extend_from_slice(&scene.version.to_le_bytes());
    out.extend_from_slice(&(scene.objects.len() as u32).to_le_bytes());
    for o in &scene.objects {
        out.extend_from_slice(&o.id.0.to_le_bytes());
        out.extend_from_slice(&(o.name.len() as u32).to_le_bytes());
        out.extend_from_slice(o.name.as_bytes());
        out.push(u8::from(o.enabled));
        floats(&mut out, o.transform.translation);
        floats(&mut out, o.transform.rotation);
        out.push(match o.units {
            ForceUnits::Acceleration => 0,
            ForceUnits::ForceDensity => 1,
        });
        match o.kind {
            ForceKind::Directional { vector } => {
                out.push(0);
                floats(&mut out, vector);
            }
            ForceKind::Radial { strength } => {
                out.push(1);
                floats(&mut out, [strength]);
            }
            ForceKind::Vortex { strength } => {
                out.push(2);
                floats(&mut out, [strength]);
            }
        }
        match o.support {
            ForceSupport::Global => out.push(0),
            ForceSupport::Sphere {
                radius,
                edge_fraction,
            } => {
                out.push(1);
                floats(&mut out, [radius, edge_fraction]);
            }
            ForceSupport::Box {
                half_extent,
                edge_fraction,
            } => {
                out.push(2);
                floats(&mut out, half_extent);
                floats(&mut out, [edge_fraction]);
            }
        }
        floats(&mut out, [o.time.start_seconds]);
        out.push(u8::from(o.time.end_seconds.is_some()));
        if let Some(end) = o.time.end_seconds {
            floats(&mut out, [end]);
        }
    }
    if out.len() > MAX_FORCE_METADATA_BYTES {
        return Err(RecordingError::Limit("force metadata bytes"));
    }
    Ok(out)
}
fn floats<const N: usize>(out: &mut Vec<u8>, values: [f64; N]) {
    for value in values {
        out.extend_from_slice(&value.to_le_bytes());
    }
}
struct Input<'a> {
    bytes: &'a [u8],
}
impl Input<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], RecordingError> {
        let (head, tail) = self
            .bytes
            .split_at_checked(n)
            .ok_or(RecordingError::Invalid("truncated force metadata"))?;
        self.bytes = tail;
        Ok(head)
    }
    fn byte(&mut self) -> Result<u8, RecordingError> {
        Ok(self.take(1)?[0])
    }
    fn boolean(&mut self) -> Result<bool, RecordingError> {
        match self.byte()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(RecordingError::Invalid("force boolean")),
        }
    }
    fn u32(&mut self) -> Result<u32, RecordingError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> Result<f64, RecordingError> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn vector<const N: usize>(&mut self) -> Result<[f64; N], RecordingError> {
        let mut out = [0.0; N];
        for v in &mut out {
            *v = self.f64()?;
        }
        Ok(out)
    }
}
/// Bounds input before parsing, validates all objects and rejects trailing bytes.
pub fn decode_force_metadata(bytes: &[u8]) -> Result<ForceSnapshot, RecordingError> {
    if bytes.len() > MAX_FORCE_METADATA_BYTES {
        return Err(RecordingError::Limit("force metadata bytes"));
    }
    let mut input = Input { bytes };
    if input.take(8)? != MAGIC || input.u32()? != FORCE_METADATA_VERSION {
        return Err(RecordingError::Unsupported("force metadata codec version"));
    }
    let version = input.u32()?;
    let count = input.u32()? as usize;
    if count > crate::MAX_FORCE_OBJECTS {
        return Err(RecordingError::Limit("force object count"));
    }
    let mut objects = Vec::new();
    for _ in 0..count {
        let id = SceneObjectId(u64::from_le_bytes(input.take(8)?.try_into().unwrap()));
        let len = input.u32()? as usize;
        if len > 256 {
            return Err(RecordingError::Limit("force name bytes"));
        }
        let name = std::str::from_utf8(input.take(len)?)
            .map_err(|_| RecordingError::Invalid("force name UTF-8"))?
            .to_owned();
        let enabled = input.boolean()?;
        let transform = ForceTransform {
            translation: input.vector()?,
            rotation: input.vector()?,
        };
        let units = match input.byte()? {
            0 => ForceUnits::Acceleration,
            1 => ForceUnits::ForceDensity,
            _ => return Err(RecordingError::Unsupported("force units")),
        };
        let kind = match input.byte()? {
            0 => ForceKind::Directional {
                vector: input.vector()?,
            },
            1 => ForceKind::Radial {
                strength: input.f64()?,
            },
            2 => ForceKind::Vortex {
                strength: input.f64()?,
            },
            _ => return Err(RecordingError::Unsupported("force kind")),
        };
        let support = match input.byte()? {
            0 => ForceSupport::Global,
            1 => ForceSupport::Sphere {
                radius: input.f64()?,
                edge_fraction: input.f64()?,
            },
            2 => ForceSupport::Box {
                half_extent: input.vector()?,
                edge_fraction: input.f64()?,
            },
            _ => return Err(RecordingError::Unsupported("force support")),
        };
        let start_seconds = input.f64()?;
        let end_seconds = if input.boolean()? {
            Some(input.f64()?)
        } else {
            None
        };
        objects.push(ForceObject {
            id,
            name,
            enabled,
            transform,
            units,
            kind,
            support,
            time: ForceTime {
                start_seconds,
                end_seconds,
            },
        });
    }
    if !input.bytes.is_empty() {
        return Err(RecordingError::Invalid("trailing force metadata"));
    }
    ForceScene { version, objects }
        .freeze()
        .map_err(|_| RecordingError::Invalid("force document"))
}

impl RecordingRun {
    /// Explicit opt-in. Bind the complete scene's versioned binary digest to the
    /// existing run header. Does not alter any historical encoding/hash algorithm.
    pub fn with_force_fields(mut self, fields: &ForceSnapshot) -> Result<Self, RecordingError> {
        self.validate()?;
        if self.configuration.contains(MARKER) {
            return Err(RecordingError::Invalid("force run already bound"));
        }
        let digest = Sha256::digest(encode_force_metadata(fields)?);
        self.configuration.push_str(MARKER);
        for byte in digest {
            self.configuration.push_str(&format!("{byte:02x}"));
        }
        self.validate()?;
        Ok(self)
    }
    pub fn force_fields_digest(&self) -> Result<Option<[u8; 32]>, RecordingError> {
        let Some((prefix, hex)) = self.configuration.rsplit_once(MARKER) else {
            return Ok(None);
        };
        if prefix.contains(MARKER)
            || hex.len() != 64
            || !hex
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(RecordingError::Invalid("force run digest"));
        }
        let mut digest = [0; 32];
        for (i, out) in digest.iter_mut().enumerate() {
            *out = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
                .map_err(|_| RecordingError::Invalid("force run digest"))?;
        }
        Ok(Some(digest))
    }
}

/// Decode a selected publication's actual full force document. Absence means
/// legacy metadata, not proof that an arbitrary producer applied zero force.
pub fn recorded_force_fields(step: &RecordedStep) -> Result<Option<ForceSnapshot>, RecordingError> {
    let Some(field) = step.field(FIELD) else {
        return Ok(None);
    };
    if field.units != "1"
        || field.association != ASSOCIATION
        || field.stage != STAGE
        || field.scalar_type != "u8"
        || field.components != 1
        || field.evaluation_time.is_some()
        || field.dimensions != [field.data.len() as u64, 1, 1]
        || step.fields.iter().filter(|f| f.name == FIELD).count() != 1
    {
        return Err(RecordingError::Invalid("force metadata field schema"));
    }
    Ok(Some(decode_force_metadata(&field.data)?))
}
/// Pure association check shared by append/read and available to codec callers.
pub fn validate_run_force_fields(
    run: &RecordingRun,
    step: &RecordedStep,
) -> Result<(), RecordingError> {
    let expected = run.force_fields_digest()?;
    let fields = recorded_force_fields(step)?;
    match (expected, fields) {
        (None, None) => Ok(()),
        (Some(expected), Some(fields))
            if expected.as_slice()
                == Sha256::digest(encode_force_metadata(&fields)?).as_slice() =>
        {
            Ok(())
        }
        _ => Err(RecordingError::Integrity(
            "run/step force document mismatch",
        )),
    }
}
/// Attach run input metadata to a constructor publication, or require that the
/// accepted publication already contains exactly the supplied actual document.
pub(super) fn require_fields(
    step: &mut RecordedStep,
    fields: &ForceSnapshot,
    limit: usize,
) -> Result<(), RecordingError> {
    if let Some(actual) = recorded_force_fields(step)? {
        if encode_force_metadata(&actual)? != encode_force_metadata(fields)? {
            return Err(RecordingError::Integrity(
                "accepted force document mismatch",
            ));
        }
    } else if step.stage == SnapshotStage::Constructor {
        append_fields(step, fields, limit)?;
    } else {
        return Err(RecordingError::Invalid("accepted force custody absent"));
    }
    Ok(())
}
pub(super) fn append_fields(
    step: &mut RecordedStep,
    fields: &ForceSnapshot,
    limit: usize,
) -> Result<(), RecordingError> {
    if step.field(FIELD).is_some() {
        return Err(RecordingError::Invalid("duplicate force metadata"));
    }
    let data = encode_force_metadata(fields)?;
    step.fields.push(RecordedField {
        name: FIELD.into(),
        units: "1".into(),
        association: ASSOCIATION.into(),
        stage: STAGE.into(),
        evaluation_time: None,
        dimensions: [data.len() as u64, 1, 1],
        components: 1,
        scalar_type: "u8".into(),
        data,
    });
    if step.payload_bytes()? > limit {
        return Err(RecordingError::Limit("force record payload"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fields() -> ForceSnapshot {
        ForceScene {
            version: crate::FORCE_SCENE_VERSION,
            objects: vec![
                ForceObject {
                    id: SceneObjectId(1),
                    name: "directional".into(),
                    enabled: true,
                    transform: ForceTransform {
                        translation: [-0.0, 2.0, 3.0],
                        rotation: [-0.0, 0.0, 0.0, 1.0],
                    },
                    units: ForceUnits::Acceleration,
                    kind: ForceKind::Directional {
                        vector: [-0.0, -2.0, 3.0],
                    },
                    support: ForceSupport::Global,
                    time: ForceTime::default(),
                },
                ForceObject {
                    id: SceneObjectId(u64::MAX),
                    name: "attraction λ".into(),
                    enabled: false,
                    transform: ForceTransform::default(),
                    units: ForceUnits::ForceDensity,
                    kind: ForceKind::Radial { strength: -4.0 },
                    support: ForceSupport::Sphere {
                        radius: 2.0,
                        edge_fraction: 0.25,
                    },
                    time: ForceTime {
                        start_seconds: 1.0,
                        end_seconds: Some(2.0),
                    },
                },
                ForceObject {
                    id: SceneObjectId(3),
                    name: "vortex".into(),
                    enabled: true,
                    transform: ForceTransform::default(),
                    units: ForceUnits::Acceleration,
                    kind: ForceKind::Vortex { strength: 5.0 },
                    support: ForceSupport::Box {
                        half_extent: [1.0, 2.0, 3.0],
                        edge_fraction: 1.0,
                    },
                    time: ForceTime::default(),
                },
            ],
        }
        .freeze()
        .unwrap()
    }
    fn run() -> RecordingRun {
        RecordingRun {
            identity: crate::SessionIdentity {
                run_id: 7,
                source_revision: "legacy-source".into(),
            },
            solver: "legacy-solver".into(),
            configuration: "legacy-config".into(),
            counts: [1; 3],
            spacing: [1.0; 3],
            origin: [0.0; 3],
            provenance: "legacy-provenance".into(),
        }
    }
    fn step() -> RecordedStep {
        RecordedStep {
            ordinal: 0,
            generation: 0,
            time: 0.0,
            dt: None,
            stage: SnapshotStage::Constructor,
            controls: "legacy-controls".into(),
            fields: vec![RecordedField {
                name: "sample".into(),
                units: "1".into(),
                association: "cell".into(),
                stage: "constructor".into(),
                evaluation_time: None,
                dimensions: [1; 3],
                components: 1,
                scalar_type: "u8".into(),
                data: vec![9],
            }],
        }
    }
    #[test]
    fn binary_roundtrip_preserves_all_variants_ids_disabled_and_float_bits() {
        let original = fields();
        let bytes = encode_force_metadata(&original).unwrap();
        assert_eq!(&bytes[..20], b"RHEONFF\0\x01\0\0\0\x01\0\0\0\x03\0\0\0");
        let decoded = decode_force_metadata(&bytes).unwrap();
        assert_eq!(encode_force_metadata(&decoded).unwrap(), bytes);
        assert_eq!(decoded.scene(), original.scene());
        assert_eq!(
            decoded.scene().objects[0].transform.translation[0].to_bits(),
            (-0.0_f64).to_bits()
        );
    }
    #[test]
    fn binary_rejects_truncation_trailing_unknown_versions_and_invalid_objects() {
        let bytes = encode_force_metadata(&fields()).unwrap();
        for n in 0..bytes.len() {
            assert!(decode_force_metadata(&bytes[..n]).is_err(), "prefix {n}");
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert!(decode_force_metadata(&bad).is_err());
        for offset in [8, 12] {
            let mut bad = bytes.clone();
            bad[offset] = 99;
            assert!(decode_force_metadata(&bad).is_err());
        }
        let mut bad = bytes.clone();
        bad[20..28].fill(0);
        assert!(decode_force_metadata(&bad).is_err());
        let mut bad = bytes.clone();
        bad[16..20].copy_from_slice(&257_u32.to_le_bytes());
        assert!(decode_force_metadata(&bad).is_err());
        assert!(decode_force_metadata(&vec![0; MAX_FORCE_METADATA_BYTES + 1]).is_err());
    }
    #[test]
    fn maximum_valid_scene_fits_bounded_codec() {
        let mut scene = fields().scene().clone();
        let mut object = scene.objects[2].clone();
        object.name = "x".repeat(256);
        scene.objects = (1..=crate::MAX_FORCE_OBJECTS)
            .map(|id| {
                let mut o = object.clone();
                o.id = SceneObjectId(id as u64);
                o
            })
            .collect();
        let fields = scene.freeze().unwrap();
        let bytes = encode_force_metadata(&fields).unwrap();
        assert!(bytes.len() <= MAX_FORCE_METADATA_BYTES);
        assert_eq!(
            encode_force_metadata(&decode_force_metadata(&bytes).unwrap()).unwrap(),
            bytes
        );
    }
    #[test]
    fn run_binding_rejects_absent_mismatched_and_unbound_metadata() {
        let original = fields();
        let bound = run().with_force_fields(&original).unwrap();
        let mut s = step();
        assert!(validate_run_force_fields(&bound, &s).is_err());
        append_fields(&mut s, &original, 1024 * 1024).unwrap();
        validate_run_force_fields(&bound, &s).unwrap();
        assert!(validate_run_force_fields(&run(), &s).is_err());
        let empty = ForceScene::default().freeze().unwrap();
        let wrong = run().with_force_fields(&empty).unwrap();
        assert!(validate_run_force_fields(&wrong, &s).is_err());
        assert!(bound.with_force_fields(&original).is_err());
    }
    #[test]
    fn field_schema_limits_and_accepted_custody_fail_closed() {
        let original = fields();
        let mut s = step();
        require_fields(&mut s, &original, 1024 * 1024).unwrap();
        assert!(
            require_fields(
                &mut s,
                &ForceScene::default().freeze().unwrap(),
                1024 * 1024
            )
            .is_err()
        );
        s.fields.last_mut().unwrap().association = "particle_index_within_run".into();
        assert!(recorded_force_fields(&s).is_err());
        let mut s = step();
        assert!(append_fields(&mut s, &original, 1).is_err());
        let mut s = step();
        s.generation = 1;
        s.time = 0.1;
        s.dt = Some(0.1);
        s.stage = SnapshotStage::Accepted;
        assert!(require_fields(&mut s, &original, 1024 * 1024).is_err());
    }
    #[test]
    fn malformed_run_digest_never_panics_and_is_not_rebound() {
        for text in ["λ".repeat(32), "x".repeat(64), "0".repeat(63)] {
            let mut r = run();
            r.configuration.push_str(MARKER);
            r.configuration.push_str(&text);
            assert!(r.force_fields_digest().is_err());
            assert!(r.validate().is_err());
        }
    }
    #[test]
    fn legacy_run_hash_and_absent_metadata_are_unchanged() {
        let r = run();
        let s = step();
        let limits = RecordingLimits {
            max_database_bytes: 1048576,
            max_record_bytes: 4096,
            max_records: 3,
        };
        let hash = r
            .digest(limits)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        // Independent byte-level SHA-256 calculation of the pre-change contract.
        assert_eq!(
            hash,
            "781b179b9818a048d50183f33244e73968dc7ec02e35a258f5bba3463886986b"
        );
        let step_hash = s
            .digest(&r.digest(limits))
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            step_hash,
            "579f8cc614789a49c128f54c90ed449e6bd09a35fda8b7c6958b449ceef39b76"
        );
        assert!(r.force_fields_digest().unwrap().is_none());
        assert!(recorded_force_fields(&s).unwrap().is_none());
        validate_run_force_fields(&r, &s).unwrap();
        assert_eq!(s.controls, "legacy-controls");
        assert_eq!(s.fields.len(), 1);
    }
}
