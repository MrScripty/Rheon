//! Opt-in, immutable owned f64 snapshot data on one borrowed retained geometry.
//! Structural validity and caller provenance do not qualify physical dynamics.
use crate::{Axis, NO_FLUID_COMPONENT, StaticObstacleGeometry};
use std::{
    fmt,
    io::{Read, Write},
    mem::size_of,
};

pub const MAX_OBSTACLE_STATE_BYTES: usize = 16_000_000;
const MAGIC: &[u8; 8] = b"RHEONOS1";

#[derive(Debug)]
pub enum ObstacleStateError {
    MissingEvidence,
    UnsupportedGeometry,
    InvalidMaterial,
    InvalidChronology,
    InvalidErrorDeclaration,
    InvalidRest,
    InvalidBudget { limit: usize },
    ShapeMismatch { axis: Axis },
    InvalidVelocity { axis: Axis, face: usize },
    NonstationaryTrace { axis: Axis, face: usize },
    BufferLimit { required: usize, limit: usize },
    CapacityOverflow,
    AllocationFailure,
    GeometryMismatch,
    InvalidCheckpoint,
    Io(std::io::Error),
}
impl fmt::Display for ObstacleStateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "obstacle state refused: {self:?}")
    }
}
impl std::error::Error for ObstacleStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}
impl From<std::io::Error> for ObstacleStateError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Caller-declared content reference, not authentication or a verified source.
/// Fixed-size references cannot retain arbitrary strings/history buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObstacleEvidenceRef {
    id: u64,
    version: u64,
    sha256: [u8; 32],
}
impl ObstacleEvidenceRef {
    pub fn new(id: u64, version: u64, sha256: [u8; 32]) -> Result<Self, ObstacleStateError> {
        if id == 0 || sha256 == [0; 32] {
            return Err(ObstacleStateError::MissingEvidence);
        }
        Ok(Self {
            id,
            version,
            sha256,
        })
    }
    pub fn id(self) -> u64 {
        self.id
    }
    pub fn version(self) -> u64 {
        self.version
    }
    pub fn sha256(self) -> [u8; 32] {
        self.sha256
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleStateUnits {
    Si,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleStateModel {
    TransientNavierStokes,
    TransientStokes,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleStateBoundary {
    StationaryNoSlipSolidSealedFreeSlipOuter,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleStateMaterial {
    pub density: f64,
    pub dynamic_viscosity: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleInitialData {
    KnownRest(ObstacleEvidenceRef),
    Supplied(ObstacleEvidenceRef),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleForcingKind {
    ExplicitNoForcing,
    DeclaredExternalForcing,
}
/// Declared history covering precisely the represented initial-to-frame span.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleForcingHistory {
    pub kind: ObstacleForcingKind,
    pub evidence: ObstacleEvidenceRef,
    pub start_time: f64,
    pub end_time: f64,
}
/// All fields are required. There is no Default or missing-input fallback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstaclePhysicalInputs {
    pub units: ObstacleStateUnits,
    pub model: ObstacleStateModel,
    pub material: ObstacleStateMaterial,
    pub problem: ObstacleEvidenceRef,
    pub boundary_evidence: ObstacleEvidenceRef,
    pub boundary: ObstacleStateBoundary,
    pub initial: ObstacleInitialData,
    pub initial_time: f64,
    pub forcing: ObstacleForcingHistory,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleStateOrigin {
    InitialData,
    ExternalSnapshot(ObstacleEvidenceRef),
}
/// Units m/s: bounds are caller assertions, including declared0. Never certified.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObstacleVelocityError {
    Unknown,
    CallerDeclaredUpperBound {
        value: f64,
        evidence: ObstacleEvidenceRef,
    },
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleVelocityErrors {
    pub face_linf: [ObstacleVelocityError; 3],
    pub spatial: ObstacleVelocityError,
    pub temporal: ObstacleVelocityError,
    pub inputs: ObstacleVelocityError,
    pub boundary_transfer: ObstacleVelocityError,
    pub algebraic: ObstacleVelocityError,
    pub roundoff: ObstacleVelocityError,
}
impl ObstacleVelocityErrors {
    pub const fn unknown() -> Self {
        Self {
            face_linf: [ObstacleVelocityError::Unknown; 3],
            spatial: ObstacleVelocityError::Unknown,
            temporal: ObstacleVelocityError::Unknown,
            inputs: ObstacleVelocityError::Unknown,
            boundary_transfer: ObstacleVelocityError::Unknown,
            algebraic: ObstacleVelocityError::Unknown,
            roundoff: ObstacleVelocityError::Unknown,
        }
    }
    fn values(self) -> [ObstacleVelocityError; 9] {
        [
            self.face_linf[0],
            self.face_linf[1],
            self.face_linf[2],
            self.spatial,
            self.temporal,
            self.inputs,
            self.boundary_transfer,
            self.algebraic,
            self.roundoff,
        ]
    }
    fn from_values(v: [ObstacleVelocityError; 9]) -> Self {
        Self {
            face_linf: [v[0], v[1], v[2]],
            spatial: v[3],
            temporal: v[4],
            inputs: v[5],
            boundary_transfer: v[6],
            algebraic: v[7],
            roundoff: v[8],
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ObstacleStateFrame {
    pub time: f64,
    pub generation: u64,
    pub origin: ObstacleStateOrigin,
    pub errors: ObstacleVelocityErrors,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObstacleStateQualification {
    Unqualified,
}

/// Owns only values/metadata; borrows one immutable geometry for its lifetime.
/// No mutable state, pressure, solve, commit or timestep API is exposed.
#[derive(Debug)]
pub struct ObstacleFlowState<'g> {
    geometry: &'g StaticObstacleGeometry,
    inputs: ObstaclePhysicalInputs,
    frame: ObstacleStateFrame,
    velocity: [Vec<f64>; 3],
    owned_payload_bytes: usize,
}
fn normal_or_zero(v: f64) -> bool {
    v == 0. || v.is_normal()
}
fn nonnegative(v: f64) -> bool {
    v >= 0. && normal_or_zero(v)
}
fn validate_metadata(
    inputs: ObstaclePhysicalInputs,
    frame: ObstacleStateFrame,
) -> Result<(), ObstacleStateError> {
    let material = inputs.material;
    if !material.density.is_normal()
        || material.density <= 0.
        || !material.dynamic_viscosity.is_normal()
        || material.dynamic_viscosity <= 0.
    {
        return Err(ObstacleStateError::InvalidMaterial);
    }
    if !nonnegative(inputs.initial_time)
        || !nonnegative(frame.time)
        || frame.time < inputs.initial_time
        || !nonnegative(inputs.forcing.start_time)
        || !nonnegative(inputs.forcing.end_time)
        || inputs.forcing.start_time != inputs.initial_time
        || inputs.forcing.end_time != frame.time
        || (frame.origin == ObstacleStateOrigin::InitialData
            && (frame.time != inputs.initial_time || frame.generation != 0))
    {
        return Err(ObstacleStateError::InvalidChronology);
    }
    for error in frame.errors.values() {
        if let ObstacleVelocityError::CallerDeclaredUpperBound { value, .. } = error
            && !nonnegative(value)
        {
            return Err(ObstacleStateError::InvalidErrorDeclaration);
        }
    }
    Ok(())
}
fn validate_value(
    geometry: &StaticObstacleGeometry,
    inputs: ObstaclePhysicalInputs,
    frame: ObstacleStateFrame,
    axis: Axis,
    face: usize,
    v: f64,
) -> Result<(), ObstacleStateError> {
    if !normal_or_zero(v) {
        return Err(ObstacleStateError::InvalidVelocity { axis, face });
    }
    let g = geometry.grid();
    let coordinate = crate::obstacle_pressure::coordinate(g.face_counts(axis), face);
    let d = axis.index();
    if (coordinate[d] == 0
        || coordinate[d] == g.counts()[d]
        || geometry.open_areas(axis)[face] == 0.)
        && v != 0.
    {
        return Err(ObstacleStateError::NonstationaryTrace { axis, face });
    }
    if frame.origin == ObstacleStateOrigin::InitialData
        && matches!(inputs.initial, ObstacleInitialData::KnownRest(_))
        && v != 0.
    {
        return Err(ObstacleStateError::InvalidRest);
    }
    Ok(())
}
fn gate(bytes: usize, limit: usize) -> Result<(), ObstacleStateError> {
    if bytes > limit {
        Err(ObstacleStateError::BufferLimit {
            required: bytes,
            limit,
        })
    } else {
        Ok(())
    }
}
fn reserve(n: usize, used: &mut usize, limit: usize) -> Result<Vec<f64>, ObstacleStateError> {
    let planned = n
        .checked_mul(8)
        .and_then(|v| v.checked_add(*used))
        .ok_or(ObstacleStateError::CapacityOverflow)?;
    gate(planned, limit)?;
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| ObstacleStateError::AllocationFailure)?;
    *used = v
        .capacity()
        .checked_mul(8)
        .and_then(|b| b.checked_add(*used))
        .ok_or(ObstacleStateError::CapacityOverflow)?;
    gate(*used, limit)?;
    Ok(v)
}
impl<'g> ObstacleFlowState<'g> {
    /// Planned combined retained geometry + fixed owner + face payload. Actual
    /// Vec capacities are checked separately. Caller inputs/allocator/RSS excluded.
    pub fn planned_payload_bytes(
        geometry: &StaticObstacleGeometry,
    ) -> Result<usize, ObstacleStateError> {
        crate::aligned_strain::admission(geometry)
            .map_err(|_| ObstacleStateError::UnsupportedGeometry)?;
        let faces = Axis::ALL.into_iter().try_fold(0usize, |n, a| {
            n.checked_add(geometry.grid().face_len(a))
                .ok_or(ObstacleStateError::CapacityOverflow)
        })?;
        faces
            .checked_mul(8)
            .and_then(|n| n.checked_add(size_of::<Self>()))
            .and_then(|n| n.checked_add(geometry.allocation().retained_bytes))
            .ok_or(ObstacleStateError::CapacityOverflow)
    }
    fn preflight(
        geometry: &StaticObstacleGeometry,
        inputs: ObstaclePhysicalInputs,
        frame: ObstacleStateFrame,
        limit: usize,
    ) -> Result<usize, ObstacleStateError> {
        if limit == 0 || limit > MAX_OBSTACLE_STATE_BYTES {
            return Err(ObstacleStateError::InvalidBudget { limit });
        }
        validate_metadata(inputs, frame)?;
        gate(Self::planned_payload_bytes(geometry)?, limit)?;
        geometry
            .allocation()
            .retained_bytes
            .checked_add(size_of::<Self>())
            .ok_or(ObstacleStateError::CapacityOverflow)
    }
    pub fn new(
        geometry: &'g StaticObstacleGeometry,
        inputs: ObstaclePhysicalInputs,
        frame: ObstacleStateFrame,
        velocity: [&[f64]; 3],
        memory_limit: usize,
    ) -> Result<Self, ObstacleStateError> {
        let mut used = Self::preflight(geometry, inputs, frame, memory_limit)?;
        for axis in Axis::ALL {
            let a = axis.index();
            if velocity[a].len() != geometry.grid().face_len(axis) {
                return Err(ObstacleStateError::ShapeMismatch { axis });
            }
            for (face, &v) in velocity[a].iter().enumerate() {
                validate_value(geometry, inputs, frame, axis, face, v)?;
            }
        }
        let mut owned = [Vec::new(), Vec::new(), Vec::new()];
        for axis in Axis::ALL {
            let a = axis.index();
            owned[a] = reserve(velocity[a].len(), &mut used, memory_limit)?;
            owned[a].extend_from_slice(velocity[a]);
        }
        Ok(Self {
            geometry,
            inputs,
            frame,
            velocity: owned,
            owned_payload_bytes: used - geometry.allocation().retained_bytes,
        })
    }
    pub fn geometry(&self) -> &'g StaticObstacleGeometry {
        self.geometry
    }
    pub fn physical_inputs(&self) -> &ObstaclePhysicalInputs {
        &self.inputs
    }
    pub fn frame(&self) -> &ObstacleStateFrame {
        &self.frame
    }
    pub fn qualification(&self) -> ObstacleStateQualification {
        ObstacleStateQualification::Unqualified
    }
    pub fn pressure_available(&self) -> bool {
        false
    }
    pub fn velocity(&self) -> [&[f64]; 3] {
        [&self.velocity[0], &self.velocity[1], &self.velocity[2]]
    }
    /// Includes fixed owner/metadata and actual owned face capacities.
    pub fn owned_payload_bytes(&self) -> usize {
        self.owned_payload_bytes
    }
    /// Counts the shared geometry retained payload once, not process RSS.
    pub fn combined_payload_bytes(&self) -> usize {
        self.owned_payload_bytes + self.geometry.allocation().retained_bytes
    }
    /// Streamed version1 exact-bit checkpoint. A writer error may leave partial
    /// output; no state mutates. Caller owns atomic file/transport handling.
    pub fn write_checkpoint(&self, writer: &mut impl Write) -> Result<(), ObstacleStateError> {
        writer.write_all(MAGIC)?;
        put_u32(writer, 1)?;
        write_geometry(writer, self.geometry)?;
        write_metadata(writer, self.inputs, self.frame)?;
        for axis in Axis::ALL {
            put_u64(writer, self.velocity[axis.index()].len() as u64)?;
        }
        for axis in Axis::ALL {
            for &v in &self.velocity[axis.index()] {
                put_f64(writer, v)?;
            }
        }
        Ok(())
    }
    /// Restores a new, forced-unqualified owner. Exact geometry and fixed
    /// metadata are checked before reserving fields; all state publishes at EOF.
    pub fn read_checkpoint(
        geometry: &'g StaticObstacleGeometry,
        reader: &mut impl Read,
        memory_limit: usize,
    ) -> Result<Self, ObstacleStateError> {
        if memory_limit == 0 || memory_limit > MAX_OBSTACLE_STATE_BYTES {
            return Err(ObstacleStateError::InvalidBudget {
                limit: memory_limit,
            });
        }
        gate(Self::planned_payload_bytes(geometry)?, memory_limit)?;
        let mut magic = [0; 8];
        reader.read_exact(&mut magic)?;
        if &magic != MAGIC || get_u32(reader)? != 1 {
            return Err(ObstacleStateError::InvalidCheckpoint);
        }
        compare_geometry(reader, geometry)?;
        let (inputs, frame) = read_metadata(reader)?;
        let mut used = Self::preflight(geometry, inputs, frame, memory_limit)?;
        for axis in Axis::ALL {
            if get_u64(reader)? != geometry.grid().face_len(axis) as u64 {
                return Err(ObstacleStateError::InvalidCheckpoint);
            }
        }
        let mut owned = [Vec::new(), Vec::new(), Vec::new()];
        for axis in Axis::ALL {
            let a = axis.index();
            let n = geometry.grid().face_len(axis);
            owned[a] = reserve(n, &mut used, memory_limit)?;
            for face in 0..n {
                let v = get_f64(reader)?;
                validate_value(geometry, inputs, frame, axis, face, v)?;
                owned[a].push(v);
            }
        }
        let mut trailing = [0];
        loop {
            match reader.read(&mut trailing) {
                Ok(0) => break,
                Ok(_) => return Err(ObstacleStateError::InvalidCheckpoint),
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(Self {
            geometry,
            inputs,
            frame,
            velocity: owned,
            owned_payload_bytes: used - geometry.allocation().retained_bytes,
        })
    }
}
fn put_u8(w: &mut impl Write, v: u8) -> Result<(), ObstacleStateError> {
    Ok(w.write_all(&[v])?)
}
fn put_u32(w: &mut impl Write, v: u32) -> Result<(), ObstacleStateError> {
    Ok(w.write_all(&v.to_le_bytes())?)
}
fn put_u64(w: &mut impl Write, v: u64) -> Result<(), ObstacleStateError> {
    Ok(w.write_all(&v.to_le_bytes())?)
}
fn put_f64(w: &mut impl Write, v: f64) -> Result<(), ObstacleStateError> {
    put_u64(w, v.to_bits())
}
fn get_u8(r: &mut impl Read) -> Result<u8, ObstacleStateError> {
    let mut b = [0];
    r.read_exact(&mut b)?;
    Ok(b[0])
}
fn get_u32(r: &mut impl Read) -> Result<u32, ObstacleStateError> {
    let mut b = [0; 4];
    r.read_exact(&mut b)?;
    Ok(u32::from_le_bytes(b))
}
fn get_u64(r: &mut impl Read) -> Result<u64, ObstacleStateError> {
    let mut b = [0; 8];
    r.read_exact(&mut b)?;
    Ok(u64::from_le_bytes(b))
}
fn get_f64(r: &mut impl Read) -> Result<f64, ObstacleStateError> {
    Ok(f64::from_bits(get_u64(r)?))
}
fn put_ref(w: &mut impl Write, e: ObstacleEvidenceRef) -> Result<(), ObstacleStateError> {
    put_u64(w, e.id)?;
    put_u64(w, e.version)?;
    Ok(w.write_all(&e.sha256)?)
}
fn get_ref(r: &mut impl Read) -> Result<ObstacleEvidenceRef, ObstacleStateError> {
    let id = get_u64(r)?;
    let version = get_u64(r)?;
    let mut digest = [0; 32];
    r.read_exact(&mut digest)?;
    ObstacleEvidenceRef::new(id, version, digest)
}
fn expect_u64(r: &mut impl Read, v: u64) -> Result<(), ObstacleStateError> {
    if get_u64(r)? != v {
        Err(ObstacleStateError::GeometryMismatch)
    } else {
        Ok(())
    }
}
fn expect_f64(r: &mut impl Read, v: f64) -> Result<(), ObstacleStateError> {
    expect_u64(r, v.to_bits())
}
fn wire_label(v: usize) -> u64 {
    if v == NO_FLUID_COMPONENT {
        u64::MAX
    } else {
        v as u64
    }
}
fn write_geometry(
    w: &mut impl Write,
    g: &StaticObstacleGeometry,
) -> Result<(), ObstacleStateError> {
    for n in g.grid().counts() {
        put_u64(w, n as u64)?;
    }
    for values in [
        g.grid().origin(),
        g.grid().spacing(),
        g.box_bounds().0,
        g.box_bounds().1,
    ] {
        for v in values {
            put_f64(w, v)?;
        }
    }
    put_u64(w, g.stamp().id)?;
    put_u64(w, g.stamp().version)?;
    put_f64(w, g.surface().relative_tolerance())?;
    put_u64(w, g.surface().vertices().len() as u64)?;
    for v in g.surface().vertices() {
        for &a in v {
            put_f64(w, a)?;
        }
    }
    put_u64(w, g.surface().triangles().len() as u64)?;
    for v in g.surface().triangles() {
        for &a in v {
            put_u64(w, a as u64)?;
        }
    }
    put_u64(w, g.component_count() as u64)?;
    for values in [
        g.fluid_volumes(),
        g.open_areas(Axis::X),
        g.open_areas(Axis::Y),
        g.open_areas(Axis::Z),
    ] {
        put_u64(w, values.len() as u64)?;
        for &v in values {
            put_f64(w, v)?;
        }
    }
    put_u64(w, g.component_labels().len() as u64)?;
    for &v in g.component_labels() {
        put_u64(w, wire_label(v))?;
    }
    Ok(())
}
fn compare_geometry(
    r: &mut impl Read,
    g: &StaticObstacleGeometry,
) -> Result<(), ObstacleStateError> {
    for n in g.grid().counts() {
        expect_u64(r, n as u64)?;
    }
    for values in [
        g.grid().origin(),
        g.grid().spacing(),
        g.box_bounds().0,
        g.box_bounds().1,
    ] {
        for v in values {
            expect_f64(r, v)?;
        }
    }
    expect_u64(r, g.stamp().id)?;
    expect_u64(r, g.stamp().version)?;
    expect_f64(r, g.surface().relative_tolerance())?;
    expect_u64(r, g.surface().vertices().len() as u64)?;
    for v in g.surface().vertices() {
        for &a in v {
            expect_f64(r, a)?;
        }
    }
    expect_u64(r, g.surface().triangles().len() as u64)?;
    for v in g.surface().triangles() {
        for &a in v {
            expect_u64(r, a as u64)?;
        }
    }
    expect_u64(r, g.component_count() as u64)?;
    for values in [
        g.fluid_volumes(),
        g.open_areas(Axis::X),
        g.open_areas(Axis::Y),
        g.open_areas(Axis::Z),
    ] {
        expect_u64(r, values.len() as u64)?;
        for &v in values {
            expect_f64(r, v)?;
        }
    }
    expect_u64(r, g.component_labels().len() as u64)?;
    for &v in g.component_labels() {
        expect_u64(r, wire_label(v))?;
    }
    Ok(())
}
fn write_metadata(
    w: &mut impl Write,
    i: ObstaclePhysicalInputs,
    f: ObstacleStateFrame,
) -> Result<(), ObstacleStateError> {
    put_u8(w, 1)?;
    put_u8(
        w,
        match i.model {
            ObstacleStateModel::TransientNavierStokes => 1,
            ObstacleStateModel::TransientStokes => 2,
        },
    )?;
    put_u8(w, 1)?;
    put_u8(w, 0)?;
    put_u8(w, 0)?; // SI/boundary fixed, forced unqualified/no pressure
    for v in [
        i.material.density,
        i.material.dynamic_viscosity,
        i.initial_time,
        f.time,
    ] {
        put_f64(w, v)?;
    }
    put_u64(w, f.generation)?;
    put_ref(w, i.problem)?;
    put_ref(w, i.boundary_evidence)?;
    let (tag, e) = match i.initial {
        ObstacleInitialData::KnownRest(e) => (1, e),
        ObstacleInitialData::Supplied(e) => (2, e),
    };
    put_u8(w, tag)?;
    put_ref(w, e)?;
    put_u8(
        w,
        match i.forcing.kind {
            ObstacleForcingKind::ExplicitNoForcing => 1,
            ObstacleForcingKind::DeclaredExternalForcing => 2,
        },
    )?;
    put_ref(w, i.forcing.evidence)?;
    put_f64(w, i.forcing.start_time)?;
    put_f64(w, i.forcing.end_time)?;
    match f.origin {
        ObstacleStateOrigin::InitialData => put_u8(w, 1)?,
        ObstacleStateOrigin::ExternalSnapshot(e) => {
            put_u8(w, 2)?;
            put_ref(w, e)?;
        }
    }
    put_u8(w, 9)?;
    for e in f.errors.values() {
        match e {
            ObstacleVelocityError::Unknown => put_u8(w, 0)?,
            ObstacleVelocityError::CallerDeclaredUpperBound { value, evidence } => {
                put_u8(w, 1)?;
                put_f64(w, value)?;
                put_ref(w, evidence)?;
            }
        }
    }
    Ok(())
}
fn read_metadata(
    r: &mut impl Read,
) -> Result<(ObstaclePhysicalInputs, ObstacleStateFrame), ObstacleStateError> {
    if get_u8(r)? != 1 {
        return Err(ObstacleStateError::InvalidCheckpoint);
    }
    let model = match get_u8(r)? {
        1 => ObstacleStateModel::TransientNavierStokes,
        2 => ObstacleStateModel::TransientStokes,
        _ => return Err(ObstacleStateError::InvalidCheckpoint),
    };
    if get_u8(r)? != 1 || get_u8(r)? != 0 || get_u8(r)? != 0 {
        return Err(ObstacleStateError::InvalidCheckpoint);
    }
    let material = ObstacleStateMaterial {
        density: get_f64(r)?,
        dynamic_viscosity: get_f64(r)?,
    };
    let initial_time = get_f64(r)?;
    let time = get_f64(r)?;
    let generation = get_u64(r)?;
    let problem = get_ref(r)?;
    let boundary_evidence = get_ref(r)?;
    let tag = get_u8(r)?;
    let evidence = get_ref(r)?;
    let initial = match tag {
        1 => ObstacleInitialData::KnownRest(evidence),
        2 => ObstacleInitialData::Supplied(evidence),
        _ => return Err(ObstacleStateError::InvalidCheckpoint),
    };
    let kind = match get_u8(r)? {
        1 => ObstacleForcingKind::ExplicitNoForcing,
        2 => ObstacleForcingKind::DeclaredExternalForcing,
        _ => return Err(ObstacleStateError::InvalidCheckpoint),
    };
    let forcing = ObstacleForcingHistory {
        kind,
        evidence: get_ref(r)?,
        start_time: get_f64(r)?,
        end_time: get_f64(r)?,
    };
    let origin = match get_u8(r)? {
        1 => ObstacleStateOrigin::InitialData,
        2 => ObstacleStateOrigin::ExternalSnapshot(get_ref(r)?),
        _ => return Err(ObstacleStateError::InvalidCheckpoint),
    };
    if get_u8(r)? != 9 {
        return Err(ObstacleStateError::InvalidCheckpoint);
    }
    let mut errors = [ObstacleVelocityError::Unknown; 9];
    for error in &mut errors {
        *error = match get_u8(r)? {
            0 => ObstacleVelocityError::Unknown,
            1 => ObstacleVelocityError::CallerDeclaredUpperBound {
                value: get_f64(r)?,
                evidence: get_ref(r)?,
            },
            _ => return Err(ObstacleStateError::InvalidCheckpoint),
        };
    }
    Ok((
        ObstaclePhysicalInputs {
            units: ObstacleStateUnits::Si,
            model,
            material,
            problem,
            boundary_evidence,
            boundary: ObstacleStateBoundary::StationaryNoSlipSolidSealedFreeSlipOuter,
            initial,
            initial_time,
            forcing,
        },
        ObstacleStateFrame {
            time,
            generation,
            origin,
            errors: ObstacleVelocityErrors::from_values(errors),
        },
    ))
}
