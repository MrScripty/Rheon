//! Versioned binary draft and immutable constructor-zero input for the new
//! voxel method. No native execution/dispatch or old-format fallback lives here.
#[path = "voxel_material/run_set.rs"]
pub mod run_set;

use super::{
    ArchiveBudget, RunSetError, configured::ExecutorDeclaration, digest, publish_new,
    read_versioned,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use rheon::{
    GridGeometry, PressureSettings, SceneObjectId,
    initial_contents::{ContentsEditor, InitialContents},
    voxel_material::{
        MaterialPair, MaterialProperties, VoxelMaterialConfig, VoxelMaterialSimulation,
    },
};
use serde::{Deserialize, Serialize};
use std::path::Path;
const MAX_BYTES: u64 = 1024 * 1024;
const DRAFT_VERSION: u32 = 1;
const INPUT_VERSION: u32 = 3;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MaterialRole {
    Water,
    Air,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PaletteEntry {
    id: u16,
    role: MaterialRole,
    color_srgb: [u8; 3],
    density_kg_m3: f64,
    dynamic_viscosity_pa_s: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InitialDocument {
    document_id: u64,
    revision: u64,
    counts_xyz: [u64; 3],
    spacing_m: [f64; 3],
    origin_m: [f64; 3],
    palette_version: u32,
    palette: [PaletteEntry; 2],
    /// Version one encodes one byte per ID (0/1/2), although the editor stores
    /// u16. This does not authorize unknown/truncated larger material IDs.
    cell_ids_base64: String,
}
impl InitialDocument {
    fn capture(editor: &ContentsEditor) -> Result<Self, RunSetError> {
        if editor.active() {
            return Err(RunSetError::Invalid("active contents stroke"));
        }
        let d = editor.document();
        let pair = d.materials();
        let ids = d
            .cells()
            .iter()
            .map(|&x| u8::try_from(x).map_err(|_| RunSetError::Invalid("initial material ID")))
            .collect::<Result<Vec<_>, _>>()?;
        let result = Self {
            document_id: d.id().0,
            revision: d.revision(),
            counts_xyz: d.geometry().counts().map(|n| n as u64),
            spacing_m: d.geometry().spacing(),
            origin_m: d.geometry().origin(),
            palette_version: 1,
            palette: [
                PaletteEntry {
                    id: 1,
                    role: MaterialRole::Water,
                    color_srgb: [45, 125, 215],
                    density_kg_m3: pair.liquid().density(),
                    dynamic_viscosity_pa_s: pair.liquid().viscosity(),
                },
                PaletteEntry {
                    id: 2,
                    role: MaterialRole::Air,
                    color_srgb: [135, 103, 49],
                    density_kg_m3: pair.gas().density(),
                    dynamic_viscosity_pa_s: pair.gas().viscosity(),
                },
            ],
            cell_ids_base64: STANDARD.encode(ids),
        };
        result.restore()?;
        Ok(result)
    }
    fn grid(&self) -> Result<GridGeometry, RunSetError> {
        if self.counts_xyz.iter().any(|&n| !(4..=64).contains(&n)) {
            return Err(RunSetError::Unsupported("initial voxel counts"));
        }
        GridGeometry::new(self.counts_xyz, self.spacing_m, self.origin_m)
            .map_err(|_| RunSetError::Invalid("initial voxel geometry"))
    }
    fn materials(&self) -> Result<MaterialPair, RunSetError> {
        if self.palette_version != 1
            || self.palette[0].id != 1
            || self.palette[0].role != MaterialRole::Water
            || self.palette[0].color_srgb != [45, 125, 215]
            || self.palette[1].id != 2
            || self.palette[1].role != MaterialRole::Air
            || self.palette[1].color_srgb != [135, 103, 49]
        {
            return Err(RunSetError::Unsupported("initial voxel palette"));
        }
        let p = self.palette.each_ref().map(|p| {
            MaterialProperties::new(p.density_kg_m3, p.dynamic_viscosity_pa_s)
                .map_err(|_| RunSetError::Invalid("captured material coefficients"))
        });
        let [liquid, gas] = p;
        Ok(MaterialPair::new(liquid?, gas?))
    }
    fn restore(&self) -> Result<InitialContents, RunSetError> {
        let grid = self.grid()?;
        let n = grid.cell_len();
        // Protect decoded output before allocating. The bounded file owner also
        // limits the encoded JSON/string; no caller length grants more cells.
        if self.cell_ids_base64.len() != n.div_ceil(3) * 4 {
            return Err(RunSetError::Invalid("initial payload encoded length"));
        }
        let mut decoded = Vec::new();
        decoded
            .try_reserve_exact(n + 2)
            .map_err(|_| RunSetError::Invalid("initial payload allocation"))?;
        decoded.resize(n + 2, 0);
        let length = STANDARD
            .decode_slice(&self.cell_ids_base64, &mut decoded)
            .map_err(|_| RunSetError::Invalid("initial payload base64"))?;
        if length != n || decoded[..n].iter().any(|&x| x > 2) {
            return Err(RunSetError::Invalid("initial payload IDs/length"));
        }
        let mut cells = Vec::new();
        cells
            .try_reserve_exact(n)
            .map_err(|_| RunSetError::Invalid("initial ID allocation"))?;
        cells.extend(decoded[..n].iter().map(|&x| u16::from(x)));
        InitialContents::restore(
            SceneObjectId(self.document_id),
            self.revision,
            grid,
            self.materials()?,
            &cells,
        )
        .map_err(|_| RunSetError::Invalid("initial document/palette"))
    }
}
/// Nonallocating method-specific representability admission, after strict
/// document restoration. This does not predict timestep arithmetic or guarantee
/// that numerical/recording allocations will succeed. Both endpoint phases are
/// supported, including their half-cell contributions and binary interface rho.
fn numerical_geometry(document: &InitialContents) -> Result<(), RunSetError> {
    let fail = || RunSetError::Invalid("voxel numerical geometry representability");
    let positive = |x: f64| x.is_normal() && x > 0.0;
    let grid = document.geometry();
    let v = grid.cell_volume();
    let pair = document.materials();
    let densities = [pair.liquid().density(), pair.gas().density()];
    let domain_volume = v * grid.cell_len() as f64;
    if !positive(v) || !positive(0.5 * v) || !positive(domain_volume) {
        return Err(fail());
    }
    for rho in densities {
        if !positive(rho * v) || !positive((0.5 * rho) * v) || !positive(rho * domain_volume) {
            return Err(fail());
        }
    }
    for (count, rho) in document.counts()[1..].iter().zip(densities) {
        if *count > 0 {
            let volume = *count as f64 * v;
            if !positive(volume) || !positive(volume * rho) {
                return Err(fail());
            }
        }
    }
    let h = grid.spacing();
    let areas = [h[1] * h[2], h[0] * h[2], h[0] * h[1]];
    let face_densities = [
        densities[0],
        densities[1],
        0.5 * densities[0] + 0.5 * densities[1],
    ];
    let mut diagonal = 0.0;
    for d in 0..3 {
        let mut largest_weight = 0.0_f64;
        for rho in face_densities {
            // Constructor checks reciprocal/product; actual operator weights
            // use direct division. Check both distinct rounded expressions.
            let acceleration = 1.0 / (rho * h[d]);
            let constructor_weight = areas[d] * acceleration;
            let weight = areas[d] / (rho * h[d]);
            if !positive(areas[d])
                || !positive(acceleration)
                || !positive(constructor_weight)
                || !positive(weight)
                || !positive(rho * v)
            {
                return Err(fail());
            }
            largest_weight = largest_weight.max(weight);
        }
        // Interior cell has two faces per axis; every possible binary row is
        // bounded by this positive sum, in the operator's X/Y/Z order.
        diagonal += largest_weight;
        diagonal += largest_weight;
    }
    if !positive(diagonal) {
        return Err(fail());
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DraftKind {
    VoxelInitialContents,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct DraftEnvelope {
    schema_version: u32,
    kind: DraftKind,
    document: InitialDocument,
    sha256: [u8; 32],
}
/// An immutable saved draft revision; Vacuum is legal and session history is not saved.
#[derive(Debug, Clone, PartialEq)]
pub struct DraftRevision {
    document: InitialDocument,
    sha256: [u8; 32],
}
impl DraftRevision {
    pub fn capture(editor: &ContentsEditor) -> Result<Self, RunSetError> {
        let document = InitialDocument::capture(editor)?;
        let sha256 = digest(&document)?;
        Ok(Self { document, sha256 })
    }
    pub fn restore(&self) -> Result<ContentsEditor, RunSetError> {
        self.validate()?;
        Ok(ContentsEditor::new(self.document.restore()?))
    }
    pub fn digest(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn validate(&self) -> Result<(), RunSetError> {
        if digest(&self.document)? != self.sha256 {
            return Err(RunSetError::Integrity("initial draft digest"));
        }
        self.document.restore()?;
        Ok(())
    }
    pub fn publish(&self, path: &Path) -> Result<(), RunSetError> {
        self.validate()?;
        publish_new(
            path,
            &serde_json::to_vec(&DraftEnvelope {
                schema_version: DRAFT_VERSION,
                kind: DraftKind::VoxelInitialContents,
                document: self.document.clone(),
                sha256: self.sha256,
            })?,
            MAX_BYTES,
        )
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        let e: DraftEnvelope = read_versioned(path, DRAFT_VERSION, MAX_BYTES)?;
        let result = Self {
            document: e.document,
            sha256: e.sha256,
        };
        result.validate()?;
        Ok(result)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InputKind {
    VoxelMaterialConstructorZero,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Method {
    MatchedDonorStrainJacobiV1,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Walls {
    ClosedFreeSlip,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum InitialVelocity {
    Zero,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PressurePolicy {
    pub relative_residual: f64,
    pub absolute_residual: f64,
    pub divergence_limit: f64,
    pub max_iterations: usize,
}
impl PressurePolicy {
    fn settings(&self) -> PressureSettings {
        PressureSettings {
            relative_residual: self.relative_residual,
            absolute_residual: self.absolute_residual,
            divergence_limit: self.divergence_limit,
            max_iterations: self.max_iterations,
        }
    }
}
/// Future declarations are copied into FrozenInput once; callers cannot mutate
/// the stored policy by changing their original value afterward.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionPolicy {
    pub requested_dt_s: f64,
    pub requested_steps: usize,
    pub gravity_m_s2: [f64; 3],
    pub pressure: PressurePolicy,
    pub numerical_memory_limit_bytes: usize,
    pub recording: ArchiveBudget,
    pub executor: ExecutorDeclaration,
}
impl ExecutionPolicy {
    fn validate(&self, grid: &GridGeometry) -> Result<(), RunSetError> {
        if !self.requested_dt_s.is_normal()
            || self.requested_dt_s <= 0.0
            || !(1..=10_000).contains(&self.requested_steps)
            || !(self.requested_dt_s * self.requested_steps as f64).is_normal()
            || !self.gravity_m_s2.iter().all(|x| x.is_finite())
            || !(1..=20_000).contains(&self.pressure.max_iterations)
            || ![
                self.pressure.relative_residual,
                self.pressure.absolute_residual,
                self.pressure.divergence_limit,
            ]
            .iter()
            .all(|x| x.is_normal() && *x > 0.0)
        {
            return Err(RunSetError::Invalid(
                "voxel interval/pressure/gravity policy",
            ));
        }
        self.recording.limits().validate()?;
        if self.recording.max_records < self.requested_steps + 1 {
            return Err(RunSetError::Invalid(
                "voxel constructor/step retention count",
            ));
        }
        self.executor.validate()?;
        let faces = [rheon::Axis::X, rheon::Axis::Y, rheon::Axis::Z]
            .iter()
            .map(|&a| grid.face_len(a))
            .sum::<usize>();
        let minimum = 8 * (14 * grid.cell_len() + 9 * faces);
        if self.numerical_memory_limit_bytes < minimum
            || self.numerical_memory_limit_bytes > 192 * 1024 * 1024
        {
            return Err(RunSetError::Invalid("voxel numerical payload budget"));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputPayload {
    kind: InputKind,
    method: Method,
    walls: Walls,
    initial_velocity: InitialVelocity,
    initial: InitialDocument,
    execution: ExecutionPolicy,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InputEnvelope {
    schema_version: u32,
    input: InputPayload,
    sha256: [u8; 32],
}
/// Validated owned constructor-zero recipe. This is not a run-set manifest or
/// restart record and does not enter SavedRunSet dispatch until real integration.
#[derive(Debug, Clone, PartialEq)]
pub struct FrozenInput {
    input: InputPayload,
    sha256: [u8; 32],
}
impl FrozenInput {
    pub fn capture(
        editor: &ContentsEditor,
        execution: ExecutionPolicy,
    ) -> Result<Self, RunSetError> {
        let initial = InitialDocument::capture(editor)?;
        let input = InputPayload {
            kind: InputKind::VoxelMaterialConstructorZero,
            method: Method::MatchedDonorStrainJacobiV1,
            walls: Walls::ClosedFreeSlip,
            initial_velocity: InitialVelocity::Zero,
            initial,
            execution,
        };
        let sha256 = digest(&input)?;
        let result = Self { input, sha256 };
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), RunSetError> {
        if digest(&self.input)? != self.sha256 {
            return Err(RunSetError::Integrity("voxel frozen input digest"));
        }
        let document = self.input.initial.restore()?;
        // Occupancy inspection does not allocate a frozen fraction/ID pair or
        // misclassify a resource error as a request to fill a Vacuum hole.
        document.validate().map_err(|_| {
            RunSetError::Unsupported("Vacuum initial cells; fill with Air or Water")
        })?;
        numerical_geometry(&document)?;
        self.input.execution.validate(document.geometry())
    }
    pub fn digest(&self) -> [u8; 32] {
        self.sha256
    }
    pub fn execution(&self) -> &ExecutionPolicy {
        &self.input.execution
    }
    pub fn initial_editor(&self) -> Result<ContentsEditor, RunSetError> {
        self.validate()?;
        Ok(ContentsEditor::new(self.input.initial.restore()?))
    }
    pub fn construct(&self) -> Result<VoxelMaterialSimulation, RunSetError> {
        self.validate()?;
        let editor = ContentsEditor::new(self.input.initial.restore()?);
        let initial = editor
            .freeze()
            .map_err(|_| RunSetError::Invalid("filled initial input"))?;
        VoxelMaterialSimulation::new(
            initial.geometry().clone(),
            initial.materials(),
            initial.liquid_fraction(),
            VoxelMaterialConfig {
                pressure: self.input.execution.pressure.settings(),
                gravity: self.input.execution.gravity_m_s2,
                memory_limit: self.input.execution.numerical_memory_limit_bytes,
            },
        )
        .map_err(|_| RunSetError::Invalid("voxel numerical constructor admission"))
    }
    pub fn publish(&self, path: &Path) -> Result<(), RunSetError> {
        self.validate()?;
        publish_new(
            path,
            &serde_json::to_vec(&InputEnvelope {
                schema_version: INPUT_VERSION,
                input: self.input.clone(),
                sha256: self.sha256,
            })?,
            MAX_BYTES,
        )
    }
    pub fn open(path: &Path) -> Result<Self, RunSetError> {
        let e: InputEnvelope = read_versioned(path, INPUT_VERSION, MAX_BYTES)?;
        let result = Self {
            input: e.input,
            sha256: e.sha256,
        };
        result.validate()?;
        Ok(result)
    }
}

#[cfg(test)]
#[path = "../voxel_material_persistence_tests.rs"]
mod tests;
