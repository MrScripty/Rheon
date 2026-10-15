//! Native, read-only comparison of exact accepted-v1 archive selections.
//! Recipe/version semantics and limitations are documented in
//! `docs/api/recording-comparison.md`. No live simulation or model is consulted.
use crate::recording::{
    RecordedField, RecordedStep, RecordingError, RecordingReader, RecordingRun,
};
use crate::{PressureImplementation, SnapshotStage};
use sha2::{Digest, Sha256};

/// Version of alignment, gauge and reduction semantics.
pub const ANALYSIS_VERSION: u32 = 1;

const CELL: &str = "cell_center; offset [0.5,0.5,0.5]; Cartesian XYZ; x-fast";
const FACES: [&str; 3] = [
    "x_face; offset [0,0.5,0.5]; Cartesian XYZ; x-fast",
    "y_face; offset [0.5,0,0.5]; Cartesian XYZ; x-fast",
    "z_face; offset [0.5,0.5,0]; Cartesian XYZ; x-fast",
];
const PRESSURE_STAGE: &str = "accepted_interval_projection; cell0 gauge=0; not endpoint pressure";

/// Identity of one exact publication, stable when its archive is appended or closed.
/// The digest is analysis-v1 SHA256, not an archive-file hash or authentication.
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionIdentity {
    pub run_id: u64,
    pub source_revision: String,
    pub ordinal: usize,
    pub generation: u64,
    pub sha256: [u8; 32],
}

/// Immutable owned, bounded reader selection. Construction verifies archive hashes.
pub struct Selection {
    run: RecordingRun,
    step: RecordedStep,
    identity: SelectionIdentity,
}
impl Selection {
    pub fn read(reader: &RecordingReader, ordinal: usize) -> Result<Self, RecordingError> {
        let step = reader.select(ordinal)?;
        let run = reader.run().clone();
        let identity = SelectionIdentity {
            run_id: run.identity.run_id,
            source_revision: run.identity.source_revision.clone(),
            ordinal,
            generation: step.generation,
            sha256: selection_digest(&run, &step),
        };
        Ok(Self {
            run,
            step,
            identity,
        })
    }
    pub fn identity(&self) -> &SelectionIdentity {
        &self.identity
    }
    pub fn digest(&self) -> [u8; 32] {
        self.identity.sha256
    }
    /// Run metadata identity; excludes lifecycle and storage budgets.
    pub fn run_digest(&self) -> [u8; 32] {
        run_digest(&self.run)
    }
    pub fn publication(&self) -> &RecordedStep {
        &self.step
    }
    pub fn run(&self) -> &RecordingRun {
        &self.run
    }
}

#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldRole {
    Tracer,
    IntervalPressure,
    VelocityX,
    VelocityY,
    VelocityZ,
    VelocityDivergence,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PressurePolicy {
    /// Both fields must actually have cell0=0; no transform is applied.
    Canonical,
    /// Diagnostic only: subtract mean(candidate-reference), retaining gauge failures.
    RemoveMeanOffsetDiagnostic,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Recipe {
    pub field: FieldRole,
    /// Absolute physical-time tolerance, seconds; finite and nonnegative.
    pub time_tolerance: f64,
    pub pressure: PressurePolicy,
}
impl Recipe {
    pub fn validate(self) -> Result<(), RecipeError> {
        validate_recipe(self)
    }
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(rename_all = "snake_case"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unavailable {
    UnsupportedProfile,
    Geometry,
    PublicationStage,
    PhysicalTime,
    MissingField,
    FieldSchema,
    NoncanonicalGauge,
    EmptySupport,
    ArithmeticRange,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeError {
    InvalidTolerance,
    AmbiguousAlignment,
}
impl std::fmt::Display for RecipeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "comparison recipe: {self:?}")
    }
}
impl std::error::Error for RecipeError {}

#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Metrics {
    pub support_count: usize,
    /// Euclidean norm of candidate-reference in field units.
    pub l2: f64,
    /// Uniform native-volume-weighted root mean square in field units.
    pub rmse: f64,
    pub linf: f64,
    /// None precisely when the reference norm is zero; no epsilon denominator.
    #[cfg_attr(
        feature = "desktop",
        serde(deserialize_with = "deserialize_explicit_option")
    )]
    pub relative_l2: Option<f64>,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorBounds {
    pub min: f64,
    pub max: f64,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaugeReport {
    pub reference_canonical: bool,
    pub candidate_canonical: bool,
    /// Mean(candidate-reference), subtracted only in the diagnostic policy.
    pub removed_offset_pa: f64,
    pub diagnostic: bool,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq)]
pub struct FieldComparison {
    pub metrics: Metrics,
    pub units: &'static str,
    /// Common bounds of the two compared fields after the declared transform.
    pub color_bounds: ColorBounds,
    pub gauge: Option<GaugeReport>,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq)]
pub struct PairComparison {
    pub analysis_version: u32,
    pub recipe: Recipe,
    pub reference: SelectionIdentity,
    pub candidate: SelectionIdentity,
    pub outcome: Result<FieldComparison, Unavailable>,
}
#[cfg_attr(feature = "desktop", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "desktop", serde(deny_unknown_fields))]
#[derive(Debug, Clone, PartialEq)]
pub struct Coverage {
    pub analysis_version: u32,
    pub recipe: Recipe,
    /// Physical-time/stage matches, including unavailable field comparisons.
    pub pairs: Vec<PairComparison>,
    pub reference_unmatched: Vec<SelectionIdentity>,
    pub candidate_unmatched: Vec<SelectionIdentity>,
}

/// Compare native values; ordinal and run-local generation do not align runs.
pub fn compare(
    reference: &Selection,
    candidate: &Selection,
    recipe: Recipe,
) -> Result<PairComparison, RecipeError> {
    recipe.validate()?;
    Ok(PairComparison {
        analysis_version: ANALYSIS_VERSION,
        recipe,
        reference: reference.identity.clone(),
        candidate: candidate.identity.clone(),
        outcome: compare_fields(reference, candidate, recipe),
    })
}

/// Actual native geometry, in metres. Field lattice dimensions are separate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeGeometry {
    pub counts_xyz: [u64; 3],
    pub spacing_m: [f64; 3],
    pub origin_m: [f64; 3],
}
/// Native sample locations; face values are never averaged onto cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleAssociation {
    CellCenter,
    XFace,
    YFace,
    ZFace,
}
/// Cartesian XYZ, x-fast scalar samples. Offsets are measured in cells from origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NativeSampling {
    pub association: SampleAssociation,
    pub dimensions: [u64; 3],
    pub offset_cells: [f64; 3],
}
/// Owned presentation values and the exact reduction that authorized them.
/// Pressure candidates include only the summary's declared gauge transform.
/// This is transient derived data, not accepted state or a persisted field store.
#[derive(Debug, Clone, PartialEq)]
pub struct ComparedValues {
    pub reference: Vec<f64>,
    pub candidate: Vec<f64>,
    pub comparison: PairComparison,
    pub geometry: NativeGeometry,
    pub sampling: NativeSampling,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparedValuesError {
    Recipe(RecipeError),
    Unavailable(Unavailable),
}
impl std::fmt::Display for ComparedValuesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Recipe(e) => write!(f, "native comparison values: {e}"),
            Self::Unavailable(reason) => {
                write!(f, "native comparison values unavailable: {reason:?}")
            }
        }
    }
}
impl std::error::Error for ComparedValuesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Recipe(e) => Some(e),
            Self::Unavailable(_) => None,
        }
    }
}
impl From<Unavailable> for ComparedValuesError {
    fn from(value: Unavailable) -> Self {
        Self::Unavailable(value)
    }
}

/// Synchronously materialize one already bounded immutable native pair.
/// Reuses the comparison/field validators and exact returned gauge offset. The
/// extra extraction intentionally avoids a second mean or reduction algorithm.
/// Recording limits bound source records, not peak RSS or promoted f64 allocations.
pub fn compared_values(
    reference: &Selection,
    candidate: &Selection,
    recipe: Recipe,
) -> Result<ComparedValues, ComparedValuesError> {
    let comparison = compare(reference, candidate, recipe).map_err(ComparedValuesError::Recipe)?;
    let summary = comparison
        .outcome
        .as_ref()
        .map_err(|reason| ComparedValuesError::Unavailable(*reason))?;
    let x = values(reference, recipe.field)?;
    let mut y = values(candidate, recipe.field)?;
    if let Some(gauge) = summary.gauge {
        for value in &mut y {
            *value -= gauge.removed_offset_pa;
        }
    }
    if x.iter().chain(&y).any(|value| !value.is_finite()) {
        return Err(Unavailable::ArithmeticRange.into());
    }
    let mut dimensions = reference.run.counts;
    let mut offset_cells = [0.5; 3];
    let association = if let Some(axis) = face_axis(recipe.field) {
        dimensions[axis] = dimensions[axis]
            .checked_add(1)
            .ok_or(Unavailable::FieldSchema)?;
        offset_cells[axis] = 0.0;
        [
            SampleAssociation::XFace,
            SampleAssociation::YFace,
            SampleAssociation::ZFace,
        ][axis]
    } else {
        SampleAssociation::CellCenter
    };
    Ok(ComparedValues {
        reference: x,
        candidate: y,
        comparison,
        geometry: NativeGeometry {
            counts_xyz: reference.run.counts,
            spacing_m: reference.run.spacing,
            origin_m: reference.run.origin,
        },
        sampling: NativeSampling {
            association,
            dimensions,
            offset_cells,
        },
    })
}

/// One-to-one alignment of caller-bounded selections. Multiple matches within
/// tolerance reject as ambiguous; input order never chooses a scientific match.
/// No interpolation, remapping or cross-run particle pairing is performed.
pub fn align(
    reference: &[Selection],
    candidate: &[Selection],
    recipe: Recipe,
) -> Result<Coverage, RecipeError> {
    recipe.validate()?;
    let mut result = Coverage {
        analysis_version: ANALYSIS_VERSION,
        recipe,
        pairs: Vec::new(),
        reference_unmatched: Vec::new(),
        candidate_unmatched: Vec::new(),
    };
    let mut used = vec![false; candidate.len()];
    for a in reference {
        let mut match_index = None;
        for (i, b) in candidate.iter().enumerate() {
            if temporal_compatibility(a, b, recipe).is_ok() {
                if match_index.is_some() || used[i] {
                    return Err(RecipeError::AmbiguousAlignment);
                }
                match_index = Some(i);
            }
        }
        if let Some(i) = match_index {
            used[i] = true;
            result.pairs.push(compare(a, &candidate[i], recipe)?);
        } else {
            result.reference_unmatched.push(a.identity.clone());
        }
    }
    for (b, used) in candidate.iter().zip(used) {
        if !used {
            result.candidate_unmatched.push(b.identity.clone());
        }
    }
    Ok(result)
}
fn validate_recipe(recipe: Recipe) -> Result<(), RecipeError> {
    if recipe.time_tolerance.is_finite() && recipe.time_tolerance >= 0.0 {
        Ok(())
    } else {
        Err(RecipeError::InvalidTolerance)
    }
}
fn temporal_compatibility(a: &Selection, b: &Selection, recipe: Recipe) -> Result<(), Unavailable> {
    if a.step.stage != b.step.stage {
        return Err(Unavailable::PublicationStage);
    }
    if (a.step.time - b.step.time).abs() > recipe.time_tolerance {
        return Err(Unavailable::PhysicalTime);
    }
    if recipe.field == FieldRole::IntervalPressure {
        let start = |s: &Selection| {
            let dt = s.step.dt?;
            // Invalid/missing fields cannot erase a temporal match. The step's
            // interval remains authoritative for pairing; values() rejects the field.
            s.step
                .field("interval_pressure")
                .and_then(|f| f.evaluation_time)
                .filter(|start| {
                    *start < s.step.time
                        && (s.step.time - start - dt).abs()
                            <= 8.0 * f64::EPSILON * s.step.time.max(dt)
                })
                .or(Some(s.step.time - dt))
        };
        match (start(a), start(b)) {
            (Some(x), Some(y)) if (x - y).abs() <= recipe.time_tolerance => {}
            (None, None) => {}
            _ => return Err(Unavailable::PhysicalTime),
        }
    }
    Ok(())
}
fn compare_fields(
    a: &Selection,
    b: &Selection,
    recipe: Recipe,
) -> Result<FieldComparison, Unavailable> {
    for s in [a, b] {
        if !s.run.solver.starts_with("rheon-carrier-full-box-v1/")
            || s.run
                .solver
                .rsplit('/')
                .next()
                .and_then(PressureImplementation::from_id)
                .is_none()
        {
            return Err(Unavailable::UnsupportedProfile);
        }
    }
    if a.run.counts != b.run.counts
        || !same_bits(a.run.spacing, b.run.spacing)
        || !same_bits(a.run.origin, b.run.origin)
    {
        return Err(Unavailable::Geometry);
    }
    temporal_compatibility(a, b, recipe)?;
    let mut x = values(a, recipe.field)?;
    let mut y = values(b, recipe.field)?;
    let gauge = if recipe.field == FieldRole::IntervalPressure {
        let reference_canonical = x.first() == Some(&0.0);
        let candidate_canonical = y.first() == Some(&0.0);
        if recipe.pressure == PressurePolicy::Canonical
            && !(reference_canonical && candidate_canonical)
        {
            return Err(Unavailable::NoncanonicalGauge);
        }
        let diagnostic = recipe.pressure == PressurePolicy::RemoveMeanOffsetDiagnostic;
        let offset = if diagnostic {
            // Scale before summing; compensate rounding in nearly constant offsets.
            let n = x.len() as f64;
            let mut mean = 0.0;
            let mut compensation = 0.0;
            for (x, y) in x.iter().zip(&y) {
                let term = (y - x) / n - compensation;
                let next = mean + term;
                compensation = (next - mean) - term;
                mean = next;
            }
            if !mean.is_finite() {
                return Err(Unavailable::ArithmeticRange);
            }
            mean
        } else {
            0.0
        };
        for v in &mut y {
            *v -= offset;
        }
        Some(GaugeReport {
            reference_canonical,
            candidate_canonical,
            removed_offset_pa: offset,
            diagnostic,
        })
    } else {
        None
    };
    if x.is_empty() {
        return Err(Unavailable::EmptySupport);
    }
    if x.iter().chain(&y).any(|v| !v.is_finite()) {
        return Err(Unavailable::ArithmeticRange);
    }
    let mut norm = 0.0_f64;
    let mut error = 0.0_f64;
    let mut linf = 0.0_f64;
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    for (x, y) in x.drain(..).zip(y) {
        let delta = y - x;
        norm = norm.hypot(x);
        error = error.hypot(delta);
        linf = linf.max(delta.abs());
        min = min.min(x).min(y);
        max = max.max(x).max(y);
    }
    // Number of samples follows the exact native schema, not display pixels.
    let support_count = sample_count(a, recipe.field)?;
    let rmse = error / (support_count as f64).sqrt();
    let relative_l2 = if norm == 0.0 {
        None
    } else {
        Some(error / norm)
    };
    if [norm, error, linf, rmse, min, max]
        .iter()
        .any(|v| !v.is_finite())
        || relative_l2.is_some_and(|v| !v.is_finite())
    {
        return Err(Unavailable::ArithmeticRange);
    }
    Ok(FieldComparison {
        metrics: Metrics {
            support_count,
            l2: error,
            rmse,
            linf,
            relative_l2,
        },
        units: match recipe.field {
            FieldRole::Tracer => "1",
            FieldRole::IntervalPressure => "Pa",
            FieldRole::VelocityDivergence => "1/s",
            _ => "m/s",
        },
        color_bounds: ColorBounds { min, max },
        gauge,
    })
}
fn same_bits(a: [f64; 3], b: [f64; 3]) -> bool {
    a.map(f64::to_bits) == b.map(f64::to_bits)
}
fn sample_count(s: &Selection, role: FieldRole) -> Result<usize, Unavailable> {
    let mut dims = s.run.counts;
    if let Some(d) = face_axis(role) {
        dims[d] = dims[d].checked_add(1).ok_or(Unavailable::FieldSchema)?;
    }
    dims.into_iter().try_fold(1usize, |n, d| {
        n.checked_mul(usize::try_from(d).map_err(|_| Unavailable::FieldSchema)?)
            .ok_or(Unavailable::FieldSchema)
    })
}
fn face_axis(role: FieldRole) -> Option<usize> {
    match role {
        FieldRole::VelocityX => Some(0),
        FieldRole::VelocityY => Some(1),
        FieldRole::VelocityZ => Some(2),
        _ => None,
    }
}
fn field_values(field: &RecordedField) -> Result<Vec<f64>, Unavailable> {
    let width = match field.scalar_type.as_str() {
        "f32" => 4,
        "f64" => 8,
        _ => return Err(Unavailable::FieldSchema),
    };
    let mut values = Vec::with_capacity(field.data.len() / width);
    for b in field.data.chunks_exact(width) {
        let value = if width == 4 {
            f32::from_le_bytes(b.try_into().unwrap()) as f64
        } else {
            f64::from_le_bytes(b.try_into().unwrap())
        };
        if !value.is_finite() {
            return Err(Unavailable::FieldSchema);
        }
        values.push(value);
    }
    Ok(values)
}
fn values(s: &Selection, role: FieldRole) -> Result<Vec<f64>, Unavailable> {
    if role == FieldRole::VelocityDivergence {
        return divergence(s);
    }
    let face = face_axis(role);
    let (name, units, association, kind) = match role {
        FieldRole::Tracer => ("tracer", "1", CELL, "f32"),
        FieldRole::IntervalPressure => ("interval_pressure", "Pa", CELL, "f64"),
        FieldRole::VelocityX => ("velocity_x", "m/s", FACES[0], "f32"),
        FieldRole::VelocityY => ("velocity_y", "m/s", FACES[1], "f32"),
        FieldRole::VelocityZ => ("velocity_z", "m/s", FACES[2], "f32"),
        FieldRole::VelocityDivergence => unreachable!(),
    };
    let f = s.step.field(name).ok_or(Unavailable::MissingField)?;
    let mut dims = s.run.counts;
    if let Some(d) = face {
        dims[d] = dims[d].checked_add(1).ok_or(Unavailable::FieldSchema)?;
    }
    let stage = if role == FieldRole::IntervalPressure {
        PRESSURE_STAGE
    } else if s.step.stage == SnapshotStage::Constructor {
        "constructor"
    } else {
        "accepted"
    };
    if f.units != units
        || f.association != association
        || f.scalar_type != kind
        || f.stage != stage
        || f.components != 1
        || f.dimensions != dims
    {
        return Err(Unavailable::FieldSchema);
    }
    if role == FieldRole::IntervalPressure {
        let (Some(start), Some(dt)) = (f.evaluation_time, s.step.dt) else {
            return Err(Unavailable::FieldSchema);
        };
        if s.step.stage != SnapshotStage::Accepted
            || start >= s.step.time
            || (s.step.time - start - dt).abs() > 8.0 * f64::EPSILON * s.step.time.max(dt)
        {
            return Err(Unavailable::FieldSchema);
        }
    } else if f.evaluation_time.is_some() {
        return Err(Unavailable::FieldSchema);
    }
    let result = field_values(f)?;
    if result.len() != sample_count(s, role)? {
        return Err(Unavailable::FieldSchema);
    }
    Ok(result)
}
fn divergence(s: &Selection) -> Result<Vec<f64>, Unavailable> {
    let x = values(s, FieldRole::VelocityX)?;
    let y = values(s, FieldRole::VelocityY)?;
    let z = values(s, FieldRole::VelocityZ)?;
    let [nx, ny, nz] = s.run.counts.map(|n| usize::try_from(n).unwrap());
    let [dx, dy, dz] = s.run.spacing;
    let mut result = Vec::with_capacity(sample_count(s, FieldRole::VelocityDivergence)?);
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let ix = i + (nx + 1) * (j + ny * k);
                let iy = i + nx * (j + (ny + 1) * k);
                let iz = i + nx * (j + ny * k);
                result.push(
                    (x[ix + 1] - x[ix]) / dx
                        + (y[iy + nx] - y[iy]) / dy
                        + (z[iz + nx * ny] - z[iz]) / dz,
                );
            }
        }
    }
    Ok(result)
}
fn selection_digest(run: &RecordingRun, step: &RecordedStep) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"rheon-native-selection-v1\0");
    h.update(run_digest(run));
    h.update((step.ordinal as u64).to_le_bytes());
    h.update(step.generation.to_le_bytes());
    h.update(step.time.to_le_bytes());
    hash_optional(&mut h, step.dt);
    h.update([if step.stage == SnapshotStage::Constructor {
        0
    } else {
        1
    }]);
    hash_text(&mut h, &step.controls);
    h.update((step.fields.len() as u64).to_le_bytes());
    for f in &step.fields {
        for t in [&f.name, &f.units, &f.association, &f.stage, &f.scalar_type] {
            hash_text(&mut h, t);
        }
        hash_optional(&mut h, f.evaluation_time);
        for n in f.dimensions {
            h.update(n.to_le_bytes());
        }
        h.update((f.components as u64).to_le_bytes());
        h.update((f.data.len() as u64).to_le_bytes());
        h.update(&f.data);
    }
    h.finalize().into()
}
/// Analysis-v1 identity of exact run metadata; does not validate caller-built metadata.
/// Use `RecordingReader::run()` for verified persisted source facts.
pub fn run_digest(run: &RecordingRun) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"rheon-native-run-v1\0");
    h.update(run.identity.run_id.to_le_bytes());
    for t in [
        &run.identity.source_revision,
        &run.solver,
        &run.configuration,
        &run.provenance,
    ] {
        hash_text(&mut h, t);
    }
    for v in run.counts {
        h.update(v.to_le_bytes());
    }
    for v in run.spacing.into_iter().chain(run.origin) {
        h.update(v.to_le_bytes());
    }
    h.finalize().into()
}
fn hash_text(h: &mut Sha256, text: &str) {
    h.update((text.len() as u64).to_le_bytes());
    h.update(text.as_bytes());
}
fn hash_optional(h: &mut Sha256, v: Option<f64>) {
    h.update([u8::from(v.is_some())]);
    if let Some(v) = v {
        h.update(v.to_le_bytes());
    }
}

#[cfg(feature = "desktop")]
impl<'de> serde::Deserialize<'de> for FieldComparison {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct WireField {
            metrics: Metrics,
            units: String,
            color_bounds: ColorBounds,
            #[serde(deserialize_with = "deserialize_explicit_option")]
            gauge: Option<GaugeReport>,
        }
        let field = WireField::deserialize(deserializer)?;
        let units = match field.units.as_str() {
            "1" => "1",
            "Pa" => "Pa",
            "m/s" => "m/s",
            "1/s" => "1/s",
            _ => {
                return Err(serde::de::Error::custom(
                    "unsupported native comparison units",
                ));
            }
        };
        Ok(Self {
            metrics: field.metrics,
            units,
            color_bounds: field.color_bounds,
            gauge: field.gauge,
        })
    }
}

#[cfg(feature = "desktop")]
fn deserialize_explicit_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    serde::Deserialize::deserialize(deserializer)
}
