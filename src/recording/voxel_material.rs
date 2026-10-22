//! Exact inspection publications for the filled two-material MAC method.
//! These fields are neither binary authoring IDs nor an integrator checkpoint.
use super::*;
use crate::{
    Axis, PressureSettings,
    voxel_material::{
        MaterialPair, MaterialProperties, VoxelMaterialConfig, VoxelMaterialSimulation,
    },
};

pub const VOXEL_MATERIAL_SOLVER: &str =
    "rheon-voxel-two-material-mac-v1/matched-donor-strain-jacobi-v1";
const PREFIX: &str = "RVM1:";
const CELL: &str = "cell_center; offset [0.5,0.5,0.5]; Cartesian XYZ; x-fast";
const FACE: [&str; 3] = [
    "x_face; offset [0,0.5,0.5]; Cartesian XYZ; x-fast",
    "y_face; offset [0.5,0,0.5]; Cartesian XYZ; x-fast",
    "z_face; offset [0.5,0.5,0]; Cartesian XYZ; x-fast",
];
const PRESSURE: &str = "accepted_interval_projection; cell0 gauge=0; not endpoint pressure";
const VELOCITY: [&str; 3] = ["velocity_x", "velocity_y", "velocity_z"];

/// Captured coefficients and initial-input identity. Version one fixes filled
/// occupancy, shared incompressible velocity, closed free-slip walls, no surface
/// tension and matched-donor/strain/Jacobi evolution. Gas occupies `1-F`.
#[derive(Debug, Clone, Copy)]
pub struct VoxelMaterialMetadata {
    pub materials: MaterialPair,
    pub config: VoxelMaterialConfig,
    pub initial_input_sha256: [u8; 32],
}
impl VoxelMaterialMetadata {
    fn validate(&self, grid: &GridGeometry) -> Result<(), RecordingError> {
        if grid.counts().iter().any(|n| !(4..=64).contains(n)) {
            return Err(RecordingError::Unsupported("voxel recording profile"));
        }
        let config = self.config;
        let faces = [Axis::X, Axis::Y, Axis::Z].map(|a| grid.face_len(a));
        let minimum = 8 * (14 * grid.cell_len() + 9 * faces.into_iter().sum::<usize>());
        if !config.gravity.iter().all(|v| v.is_finite())
            || ![
                config.pressure.relative_residual,
                config.pressure.absolute_residual,
                config.pressure.divergence_limit,
            ]
            .iter()
            .all(|v| v.is_normal() && *v > 0.0)
            || !(1..=20_000).contains(&config.pressure.max_iterations)
            || config.memory_limit < minimum
            || config.memory_limit > 192 * 1024 * 1024
        {
            return Err(RecordingError::Unsupported("voxel recording profile"));
        }
        Ok(())
    }
    fn encode(self) -> String {
        let p = self.materials;
        let c = self.config;
        let values = [
            p.liquid().density(),
            p.liquid().viscosity(),
            p.gas().density(),
            p.gas().viscosity(),
            c.gravity[0],
            c.gravity[1],
            c.gravity[2],
            c.pressure.relative_residual,
            c.pressure.absolute_residual,
            c.pressure.divergence_limit,
        ];
        let mut bytes = [0u8; 128];
        for (chunk, value) in bytes[..80].chunks_exact_mut(8).zip(values) {
            chunk.copy_from_slice(&value.to_le_bytes());
        }
        bytes[80..88].copy_from_slice(&(c.pressure.max_iterations as u64).to_le_bytes());
        bytes[88..96].copy_from_slice(&(c.memory_limit as u64).to_le_bytes());
        bytes[96..].copy_from_slice(&self.initial_input_sha256);
        let mut out = String::with_capacity(PREFIX.len() + 256);
        out.push_str(PREFIX);
        for byte in bytes {
            use std::fmt::Write;
            write!(&mut out, "{byte:02x}").unwrap();
        }
        out
    }
    fn decode(run: &RecordingRun) -> Result<Self, RecordingError> {
        if run.solver != VOXEL_MATERIAL_SOLVER {
            return Err(RecordingError::Unsupported("voxel recording method"));
        }
        let text = run
            .configuration
            .strip_prefix(PREFIX)
            .ok_or(RecordingError::Unsupported("voxel configuration version"))?;
        if text.len() != 256 {
            return Err(RecordingError::Invalid("voxel configuration length"));
        }
        let mut bytes = [0u8; 128];
        for (out, chunk) in bytes.iter_mut().zip(text.as_bytes().chunks_exact(2)) {
            let digit = |c| match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                _ => None,
            };
            *out = digit(chunk[0])
                .zip(digit(chunk[1]))
                .map(|(a, b)| a * 16 + b)
                .ok_or(RecordingError::Invalid("voxel configuration encoding"))?;
        }
        let words = std::array::from_fn::<_, 12, _>(|i| {
            u64::from_le_bytes(bytes[i * 8..i * 8 + 8].try_into().unwrap())
        });
        let floats = std::array::from_fn::<_, 10, _>(|i| f64::from_bits(words[i]));
        let property = |rho, mu| {
            MaterialProperties::new(rho, mu)
                .map_err(|_| RecordingError::Invalid("voxel captured coefficients"))
        };
        Ok(Self {
            materials: MaterialPair::new(
                property(floats[0], floats[1])?,
                property(floats[2], floats[3])?,
            ),
            config: VoxelMaterialConfig {
                gravity: [floats[4], floats[5], floats[6]],
                pressure: PressureSettings {
                    relative_residual: floats[7],
                    absolute_residual: floats[8],
                    divergence_limit: floats[9],
                    max_iterations: usize::try_from(words[10])
                        .map_err(|_| RecordingError::Limit("voxel iterations"))?,
                },
                memory_limit: usize::try_from(words[11])
                    .map_err(|_| RecordingError::Limit("voxel memory"))?,
            },
            initial_input_sha256: bytes[96..].try_into().unwrap(),
        })
    }
}
/// Check the typed header without allocating numerical or selected-field owners.
/// This exposes captured declarations, not proof of an external recipe or run.
pub fn voxel_material_metadata(
    run: &RecordingRun,
) -> Result<VoxelMaterialMetadata, RecordingError> {
    run.validate()?;
    let grid = GridGeometry::new(run.counts, run.spacing, run.origin)
        .map_err(|_| RecordingError::Invalid("voxel geometry"))?;
    let metadata = VoxelMaterialMetadata::decode(run)?;
    metadata.validate(&grid)?;
    Ok(metadata)
}
/// Actual numerical constructor/configuration; source/build provenance is declared
/// by the caller. The digest binds an external immutable recipe, not restart state.
pub fn voxel_material_run(
    identity: SessionIdentity,
    sim: &VoxelMaterialSimulation,
    initial_input_sha256: [u8; 32],
    provenance: &str,
) -> Result<RecordingRun, RecordingError> {
    let grid = sim.state().geometry();
    let metadata = VoxelMaterialMetadata {
        materials: sim.state().materials(),
        config: sim.config(),
        initial_input_sha256,
    };
    metadata.validate(grid)?;
    let run = RecordingRun {
        identity,
        solver: VOXEL_MATERIAL_SOLVER.into(),
        configuration: metadata.encode(),
        counts: grid.counts().map(|n| n as u64),
        spacing: grid.spacing(),
        origin: grid.origin(),
        provenance: provenance.into(),
    };
    run.validate()?;
    Ok(run)
}
/// Conservative admission for one producer publication including metadata and
/// Vec/String overhead. Consumers must additionally budget decoded/preview owners.
pub fn voxel_material_publication_bytes(
    grid: &GridGeometry,
    accepted: bool,
) -> Result<usize, RecordingError> {
    let cells = grid
        .cell_len()
        .checked_mul(1 + usize::from(accepted))
        .ok_or(RecordingError::Limit("voxel publication shape"))?;
    [Axis::X, Axis::Y, Axis::Z]
        .into_iter()
        .try_fold(cells, |n, a| n.checked_add(grid.face_len(a)))
        .and_then(|n| n.checked_mul(8))
        .and_then(|n| n.checked_add(4096 + MAX_TEXT))
        .ok_or(RecordingError::Limit("voxel publication shape"))
}
/// Exact accepted solver fields. Failed/cancelled evolution retains the last
/// accepted state/report; no rejected scratch or constructor pressure is recorded.
pub fn voxel_material_publication(
    sim: &VoxelMaterialSimulation,
    ordinal: usize,
    limit: usize,
) -> Result<RecordedStep, RecordingError> {
    let state = sim.state();
    let grid = state.geometry();
    let initial = state.generation() == 0 && state.time() == 0.0;
    let report = sim.last_report();
    if !initial
        && report.is_none_or(|r| r.generation != state.generation() || r.time != state.time())
    {
        return Err(RecordingError::Invalid("voxel accepted report stamp"));
    }
    let controls = if initial {
        "constructor; no accepted interval".into()
    } else {
        format!(
            "matched-donor-strain-jacobi-v1; report={:?}",
            report.unwrap()
        )
    };
    valid_text(&controls)?;
    if voxel_material_publication_bytes(grid, !initial)? > limit {
        return Err(RecordingError::Limit("voxel publication payload"));
    }
    let stage = if initial { "constructor" } else { "accepted" };
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(if initial { 4 } else { 5 })
        .map_err(|_| RecordingError::Limit("voxel publication allocation"))?;
    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
        fields.push(field(
            VELOCITY[d],
            "m/s",
            FACE[d],
            stage,
            None,
            grid.face_counts(axis),
            state.velocity(axis),
        )?);
    }
    fields.push(field(
        "liquid_fraction",
        "1",
        CELL,
        stage,
        None,
        grid.counts(),
        state.liquid_fraction(),
    )?);
    if !initial {
        let start = report.unwrap().interval_start;
        fields.push(field(
            "interval_pressure",
            "Pa",
            CELL,
            PRESSURE,
            Some(start),
            grid.counts(),
            state
                .pressure()
                .ok_or(RecordingError::Invalid("voxel accepted pressure"))?,
        )?);
    }
    let step = RecordedStep {
        ordinal,
        generation: state.generation(),
        time: state.time(),
        dt: report.map(|r| r.dt),
        stage: if initial {
            SnapshotStage::Constructor
        } else {
            SnapshotStage::Accepted
        },
        controls,
        fields,
    };
    if publication_capacity(&step)? > limit || step.payload_bytes()? > limit {
        return Err(RecordingError::Limit("voxel publication capacity"));
    }
    Ok(step)
}
fn publication_capacity(step: &RecordedStep) -> Result<usize, RecordingError> {
    let fail = || RecordingError::Limit("voxel publication capacity");
    let base = step
        .fields
        .capacity()
        .checked_mul(std::mem::size_of::<RecordedField>())
        .and_then(|n| n.checked_add(std::mem::size_of::<RecordedStep>()))
        .and_then(|n| n.checked_add(step.controls.capacity()))
        .ok_or_else(fail)?;
    step.fields.iter().try_fold(base, |sum, f| {
        [
            f.data.capacity(),
            f.name.capacity(),
            f.units.capacity(),
            f.association.capacity(),
            f.stage.capacity(),
            f.scalar_type.capacity(),
        ]
        .into_iter()
        .try_fold(sum, |n, c| n.checked_add(c).ok_or_else(fail))
    })
}
#[allow(clippy::too_many_arguments)]
fn field(
    name: &str,
    units: &str,
    association: &str,
    stage: &str,
    evaluation_time: Option<f64>,
    dimensions: [usize; 3],
    values: &[f64],
) -> Result<RecordedField, RecordingError> {
    let mut data = Vec::new();
    data.try_reserve_exact(values.len() * 8)
        .map_err(|_| RecordingError::Limit("voxel publication allocation"))?;
    for value in values {
        data.extend_from_slice(&value.to_le_bytes());
    }
    Ok(RecordedField {
        name: name.into(),
        units: units.into(),
        association: association.into(),
        stage: stage.into(),
        evaluation_time,
        dimensions: dimensions.map(|n| n as u64),
        components: 1,
        scalar_type: "f64".into(),
        data,
    })
}
/// Owned selected publication, with exact cell fraction and staggered f64 fields.
/// Gas is occupied `1-F`; density is derived from captured endpoint coefficients.
pub struct RecordedVoxelMaterial {
    pub metadata: VoxelMaterialMetadata,
    pub grid: GridGeometry,
    pub generation: u64,
    pub time: f64,
    pub dt: Option<f64>,
    pub liquid_fraction: Vec<f64>,
    pub velocity: [Vec<f64>; 3],
    pub interval_pressure: Option<(f64, Vec<f64>)>,
}
impl RecordedVoxelMaterial {
    pub fn decode(run: &RecordingRun, step: &RecordedStep) -> Result<Self, RecordingError> {
        run.validate()?;
        step.payload_bytes()?;
        let metadata = VoxelMaterialMetadata::decode(run)?;
        let grid = GridGeometry::new(run.counts, run.spacing, run.origin)
            .map_err(|_| RecordingError::Invalid("voxel geometry"))?;
        metadata.validate(&grid)?;
        let initial = step.stage == SnapshotStage::Constructor;
        if step.fields.len() != if initial { 4 } else { 5 } {
            return Err(RecordingError::Invalid("voxel field inventory"));
        }
        let capacity_limit = voxel_material_publication_bytes(&grid, !initial)?;
        if publication_capacity(step)? > capacity_limit {
            return Err(RecordingError::Limit("voxel raw selection capacity"));
        }
        let stage = if initial { "constructor" } else { "accepted" };
        let mut velocity = [Vec::new(), Vec::new(), Vec::new()];
        for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
            velocity[d] = values(
                step,
                VELOCITY[d],
                "m/s",
                FACE[d],
                stage,
                None,
                grid.face_counts(axis),
            )?;
            if initial && velocity[d].iter().any(|&v| v != 0.0) {
                return Err(RecordingError::Invalid("voxel constructor velocity"));
            }
            let dims = grid.face_counts(axis);
            for (index, &v) in velocity[d].iter().enumerate() {
                let p = [
                    index % dims[0],
                    (index / dims[0]) % dims[1],
                    index / (dims[0] * dims[1]),
                ];
                if (p[d] == 0 || p[d] == grid.counts()[d]) && v != 0.0 {
                    return Err(RecordingError::Invalid("voxel closed wall velocity"));
                }
            }
        }
        let liquid_fraction = values(
            step,
            "liquid_fraction",
            "1",
            CELL,
            stage,
            None,
            grid.counts(),
        )?;
        if liquid_fraction.iter().any(|v| !(0.0..=1.0).contains(v)) {
            return Err(RecordingError::Invalid("voxel fraction range"));
        }
        let interval_pressure = if initial {
            None
        } else {
            let p = step
                .field("interval_pressure")
                .ok_or(RecordingError::Invalid("voxel field inventory"))?;
            let start = p
                .evaluation_time
                .ok_or(RecordingError::Invalid("voxel pressure interval"))?;
            let dt = step
                .dt
                .ok_or(RecordingError::Invalid("voxel pressure interval"))?;
            if !start.is_finite()
                || start < 0.0
                || start >= step.time
                || (step.time - start - dt).abs() > 8.0 * f64::EPSILON * step.time.max(dt)
            {
                return Err(RecordingError::Invalid("voxel pressure interval"));
            }
            let p = values(
                step,
                "interval_pressure",
                "Pa",
                CELL,
                PRESSURE,
                Some(start),
                grid.counts(),
            )?;
            if p[0] != 0.0 {
                return Err(RecordingError::Invalid("voxel pressure gauge"));
            }
            Some((start, p))
        };
        let decoded_bytes = velocity
            .iter()
            .map(Vec::capacity)
            .chain(std::iter::once(liquid_fraction.capacity()))
            .chain(interval_pressure.iter().map(|(_, p)| p.capacity()))
            .try_fold(std::mem::size_of::<Self>(), |n, capacity| {
                capacity.checked_mul(8).and_then(|c| n.checked_add(c))
            })
            .ok_or(RecordingError::Limit("voxel decoded selection capacity"))?;
        if decoded_bytes > capacity_limit {
            return Err(RecordingError::Limit("voxel decoded selection capacity"));
        }
        Ok(Self {
            metadata,
            grid,
            generation: step.generation,
            time: step.time,
            dt: step.dt,
            liquid_fraction,
            velocity,
            interval_pressure,
        })
    }
}
#[allow(clippy::too_many_arguments)]
fn values(
    step: &RecordedStep,
    name: &str,
    units: &str,
    association: &str,
    stage: &str,
    evaluation: Option<f64>,
    dims: [usize; 3],
) -> Result<Vec<f64>, RecordingError> {
    let f = step
        .field(name)
        .ok_or(RecordingError::Invalid("voxel field inventory"))?;
    if f.scalar_type != "f64"
        || f.units != units
        || f.association != association
        || f.stage != stage
        || f.evaluation_time != evaluation
        || f.dimensions != dims.map(|n| n as u64)
        || f.components != 1
    {
        return Err(RecordingError::Invalid("voxel field schema"));
    }
    let mut out = Vec::new();
    out.try_reserve_exact(f.data.len() / 8)
        .map_err(|_| RecordingError::Limit("voxel decode allocation"))?;
    out.extend(
        f.data
            .chunks_exact(8)
            .map(|b| f64::from_le_bytes(b.try_into().unwrap())),
    );
    Ok(out)
}
