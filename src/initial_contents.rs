//! Binary initial-domain authoring with one atomic, memory-bounded history.
//! Numerical interfaces remain mixed fractions; this editor never rebinarizes a run.
use crate::{GridGeometry, SceneObjectId, voxel_material::MaterialPair};
use std::{collections::VecDeque, fmt, mem::size_of};

pub const HISTORY_LIMIT: usize = 32 * 1024 * 1024;
pub const CANDIDATE_LIMIT: usize = 8 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum ContentsMaterial {
    Vacuum = 0,
    Water = 1,
    Air = 2,
}
impl ContentsMaterial {
    pub fn from_id(id: u16) -> Result<Self, ContentsError> {
        match id {
            0 => Ok(Self::Vacuum),
            1 => Ok(Self::Water),
            2 => Ok(Self::Air),
            _ => Err(ContentsError::UnknownMaterial(id)),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Vacuum => "Vacuum",
            Self::Water => "Water",
            Self::Air => "Air",
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum ContentsError {
    InvalidIdentity,
    UnsupportedGeometry,
    UnsupportedPalette,
    LengthMismatch,
    UnknownMaterial(u16),
    InvalidBrush,
    InvalidPoint,
    Busy,
    NoStroke,
    Stale,
    RevisionExhausted,
    Allocation,
    Limit { required: usize, limit: usize },
    Vacuum { count: usize, first: [usize; 3] },
}
impl fmt::Display for ContentsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vacuum { count, first } => write!(
                f,
                "{count} Vacuum cells; first at {first:?}. This voxel model needs fluid in every cell. Fill vacuum with Air or Water; evolving vacuum is unsupported."
            ),
            other => write!(f, "Initial contents: {other:?}"),
        }
    }
}
impl std::error::Error for ContentsError {}
fn reserve<T>(n: usize) -> Result<Vec<T>, ContentsError> {
    let mut v = Vec::new();
    v.try_reserve_exact(n)
        .map_err(|_| ContentsError::Allocation)?;
    Ok(v)
}
fn copied<T: Copy>(x: &[T]) -> Result<Vec<T>, ContentsError> {
    let mut v = reserve(x.len())?;
    v.extend_from_slice(x);
    Ok(v)
}
fn bytes<T>(v: &Vec<T>) -> usize {
    v.capacity() * size_of::<T>()
}
fn limit(required: usize, cap: usize) -> Result<(), ContentsError> {
    if required > cap {
        Err(ContentsError::Limit {
            required,
            limit: cap,
        })
    } else {
        Ok(())
    }
}
fn coordinate(g: &GridGeometry, i: usize) -> [usize; 3] {
    let n = g.counts();
    [i % n[0], (i / n[0]) % n[1], i / (n[0] * n[1])]
}

/// Fixed geometry and palette, read-only IDs and monotonic content revision.
#[derive(Debug)]
pub struct InitialContents {
    id: SceneObjectId,
    revision: u64,
    grid: GridGeometry,
    materials: MaterialPair,
    cells: Vec<u16>,
}
impl InitialContents {
    pub fn new(
        id: SceneObjectId,
        grid: GridGeometry,
        fill: ContentsMaterial,
    ) -> Result<Self, ContentsError> {
        if id.0 == 0 {
            return Err(ContentsError::InvalidIdentity);
        }
        if grid.counts().iter().any(|n| !(4..=64).contains(n)) {
            return Err(ContentsError::UnsupportedGeometry);
        }
        let mut cells = reserve(grid.cell_len())?;
        cells.resize(grid.cell_len(), fill as u16);
        Self::restore(id, 0, grid, MaterialPair::water_air(), &cells)
    }
    /// Restore a version-one saved document without replaying commands or
    /// consulting mutable presets. Only the qualified captured Water/Air pair
    /// is admitted; a later reference-property change needs a new profile.
    pub fn restore(
        id: SceneObjectId,
        revision: u64,
        grid: GridGeometry,
        materials: MaterialPair,
        cells: &[u16],
    ) -> Result<Self, ContentsError> {
        if id.0 == 0 {
            return Err(ContentsError::InvalidIdentity);
        }
        if grid.counts().iter().any(|n| !(4..=64).contains(n)) {
            return Err(ContentsError::UnsupportedGeometry);
        }
        if cells.len() != grid.cell_len() {
            return Err(ContentsError::LengthMismatch);
        }
        let actual = [
            materials.liquid().density(),
            materials.liquid().viscosity(),
            materials.gas().density(),
            materials.gas().viscosity(),
        ]
        .map(f64::to_bits);
        // Exact reference values captured at profile admission, independently
        // of the current preset lookup used only for a newly authored document.
        if actual
            != [
                0x408f31999999999a,
                0x3f506ab14ec204f3,
                0x3ff3441c9a98fd61,
                0x3ef315867a02249d,
            ]
        {
            return Err(ContentsError::UnsupportedPalette);
        }
        for &id in cells {
            ContentsMaterial::from_id(id)?;
        }
        let cells = copied(cells)?;
        limit(bytes(&cells), 2 * 64usize.pow(3))?;
        Ok(Self {
            id,
            revision,
            grid,
            materials,
            cells,
        })
    }
    pub fn id(&self) -> SceneObjectId {
        self.id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn geometry(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn materials(&self) -> MaterialPair {
        self.materials
    }
    pub fn cells(&self) -> &[u16] {
        &self.cells
    }
    pub fn counts(&self) -> [usize; 3] {
        let mut out = [0; 3];
        for &id in &self.cells {
            out[id as usize] += 1;
        }
        out
    }
    pub fn validate(&self) -> Result<(), ContentsError> {
        let count = self.cells.iter().filter(|&&x| x == 0).count();
        if count > 0 {
            return Err(ContentsError::Vacuum {
                count,
                first: coordinate(&self.grid, self.cells.iter().position(|&x| x == 0).unwrap()),
            });
        }
        Ok(())
    }
    fn freeze(&self) -> Result<FrozenContents, ContentsError> {
        self.validate()?;
        let mut fractions = reserve(self.cells.len())?;
        fractions.extend(self.cells.iter().map(|&x| {
            if x == ContentsMaterial::Water as u16 {
                1.0
            } else {
                0.0
            }
        }));
        Ok(FrozenContents {
            id: self.id,
            revision: self.revision,
            grid: self.grid.clone(),
            materials: self.materials,
            cells: copied(&self.cells)?,
            fractions,
        })
    }
}
/// Owned exact start input. Editing a draft cannot mutate these bytes.
#[derive(Debug)]
pub struct FrozenContents {
    id: SceneObjectId,
    revision: u64,
    grid: GridGeometry,
    materials: MaterialPair,
    cells: Vec<u16>,
    fractions: Vec<f64>,
}
impl FrozenContents {
    pub fn id(&self) -> SceneObjectId {
        self.id
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn geometry(&self) -> &GridGeometry {
        &self.grid
    }
    pub fn materials(&self) -> MaterialPair {
        self.materials
    }
    pub fn cells(&self) -> &[u16] {
        &self.cells
    }
    pub fn liquid_fraction(&self) -> &[f64] {
        &self.fractions
    }
    pub fn allocated_bytes(&self) -> usize {
        bytes(&self.cells) + bytes(&self.fractions)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushOperation {
    Add,
    Remove,
    Replace,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrushDepth {
    Layer,
    Volume,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ContentsBrush {
    pub operation: BrushOperation,
    pub material: ContentsMaterial,
    pub radius: f64,
    pub axis: usize,
    pub layer: usize,
    pub depth: BrushDepth,
}
impl ContentsBrush {
    pub fn validate(self, grid: &GridGeometry) -> Result<(), ContentsError> {
        let extent = std::array::from_fn::<_, 3, _>(|d| grid.upper()[d] - grid.origin()[d]);
        let maximum = extent.into_iter().fold(0.0_f64, f64::hypot);
        if self.axis >= 3
            || self.layer >= grid.counts()[self.axis]
            || !self.radius.is_normal()
            || self.radius <= 0.0
            || !maximum.is_normal()
            || self.radius > maximum
            || self.material == ContentsMaterial::Vacuum
        {
            return Err(ContentsError::InvalidBrush);
        }
        Ok(())
    }
    pub fn slice_position(self, grid: &GridGeometry) -> Result<f64, ContentsError> {
        self.validate(grid)?;
        Ok(grid.origin()[self.axis] + (self.layer as f64 + 0.5) * grid.spacing()[self.axis])
    }
    fn target(self, old: u16) -> u16 {
        match self.operation {
            BrushOperation::Add if old == 0 => self.material as u16,
            BrushOperation::Remove => 0,
            BrushOperation::Replace if old != 0 => self.material as u16,
            _ => old,
        }
    }
    fn valid_point(self, grid: &GridGeometry, p: [f64; 3]) -> bool {
        p.iter()
            .enumerate()
            .all(|(d, &v)| v.is_finite() && v >= grid.origin()[d] && v < grid.upper()[d])
            && Some(p[self.axis]) == self.slice_position(grid).ok()
    }
    /// Ray/slice intersection in metres, with a half-open domain. Direction is
    /// normalized internally; near is distance along the ray in metres. Zero,
    /// nonfinite or unit near-parallel directions and behind hits refuse.
    pub fn pick(
        self,
        grid: &GridGeometry,
        origin: [f64; 3],
        direction: [f64; 3],
        near: f64,
    ) -> Result<[f64; 3], ContentsError> {
        self.validate(grid)?;
        if !origin.iter().chain(direction.iter()).all(|x| x.is_finite())
            || !near.is_finite()
            || near < 0.0
        {
            return Err(ContentsError::InvalidPoint);
        }
        let scale = direction.into_iter().map(f64::abs).fold(0.0_f64, f64::max);
        if scale == 0.0 {
            return Err(ContentsError::InvalidPoint);
        }
        let scaled = direction.map(|v| v / scale);
        let length = scaled.into_iter().fold(0.0_f64, f64::hypot);
        let direction = scaled.map(|v| v / length);
        if direction[self.axis].abs() <= f64::EPSILON {
            return Err(ContentsError::InvalidPoint);
        }
        let plane = self.slice_position(grid)?;
        let t = (plane - origin[self.axis]) / direction[self.axis];
        if !t.is_finite() || t < near {
            return Err(ContentsError::InvalidPoint);
        }
        let mut p = std::array::from_fn(|d| origin[d] + t * direction[d]);
        p[self.axis] = plane;
        if self.valid_point(grid, p) {
            Ok(p)
        } else {
            Err(ContentsError::InvalidPoint)
        }
    }
}
struct Candidate {
    brush: ContentsBrush,
    cells: Vec<u16>,
    base: u64,
}
#[derive(Clone, Copy)]
struct Change {
    index: usize,
    before: u16,
    after: u16,
}
enum Delta {
    Sparse(Vec<Change>),
    Dense { before: Vec<u16>, after: Vec<u16> },
}
struct Command {
    id: SceneObjectId,
    base: u64,
    result: u64,
    label: &'static str,
    delta: Delta,
}
impl Command {
    fn payload(&self) -> usize {
        match &self.delta {
            Delta::Sparse(v) => bytes(v),
            Delta::Dense { before, after } => bytes(before) + bytes(after),
        }
    }
    fn verify(&self, doc: &InitialContents, redo: bool) -> bool {
        if self.id != doc.id || self.result <= self.base {
            return false;
        }
        match &self.delta {
            Delta::Sparse(v) => v
                .iter()
                .all(|c| doc.cells[c.index] == if redo { c.before } else { c.after }),
            Delta::Dense { before, after } => doc.cells == *if redo { before } else { after },
        }
    }
    fn apply(&self, doc: &mut InitialContents, redo: bool) {
        match &self.delta {
            Delta::Sparse(v) => {
                for c in v {
                    doc.cells[c.index] = if redo { c.after } else { c.before };
                }
            }
            Delta::Dense { before, after } => {
                doc.cells.copy_from_slice(if redo { after } else { before })
            }
        }
    }
}
/// Sole mutable authoring owner. A rejected/cancelled stroke never changes the document.
pub struct ContentsEditor {
    document: InitialContents,
    candidate: Option<Candidate>,
    commands: VecDeque<Command>,
    cursor: usize,
    history_limit: usize,
    candidate_limit: usize,
    evicted: u64,
}
impl ContentsEditor {
    pub fn new(document: InitialContents) -> Self {
        Self::with_limits(document, HISTORY_LIMIT, CANDIDATE_LIMIT)
    }
    pub fn with_limits(
        document: InitialContents,
        history_limit: usize,
        candidate_limit: usize,
    ) -> Self {
        Self {
            document,
            candidate: None,
            commands: VecDeque::new(),
            cursor: 0,
            history_limit,
            candidate_limit,
            evicted: 0,
        }
    }
    pub fn document(&self) -> &InitialContents {
        &self.document
    }
    pub fn active(&self) -> bool {
        self.candidate.is_some()
    }
    pub fn preview_cells(&self) -> &[u16] {
        self.candidate
            .as_ref()
            .map_or(&self.document.cells, |c| &c.cells)
    }
    pub fn history_bytes(&self) -> usize {
        self.commands.capacity() * size_of::<Command>()
            + self.commands.iter().map(Command::payload).sum::<usize>()
    }
    pub fn evicted_commands(&self) -> u64 {
        self.evicted
    }
    pub fn undo_label(&self) -> Option<&'static str> {
        self.cursor.checked_sub(1).map(|i| self.commands[i].label)
    }
    pub fn redo_label(&self) -> Option<&'static str> {
        self.commands.get(self.cursor).map(|c| c.label)
    }
    pub fn freeze(&self) -> Result<FrozenContents, ContentsError> {
        if self.active() {
            Err(ContentsError::Busy)
        } else {
            self.document.freeze()
        }
    }
    pub fn begin(&mut self, brush: ContentsBrush) -> Result<(), ContentsError> {
        if self.active() {
            return Err(ContentsError::Busy);
        }
        brush.validate(&self.document.grid)?;
        limit(self.document.cells.len() * 2, self.candidate_limit)?;
        let cells = copied(&self.document.cells)?;
        limit(bytes(&cells), self.candidate_limit)?;
        self.candidate = Some(Candidate {
            brush,
            cells,
            base: self.document.revision,
        });
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.candidate = None;
    }
    /// Exact swept hard support: partitioning a straight pointer segment does not
    /// introduce a spacing parameter or frame-rate-dependent unpainted gaps.
    pub fn segment(&mut self, from: [f64; 3], to: [f64; 3]) -> Result<(), ContentsError> {
        let Some(c) = &mut self.candidate else {
            return Err(ContentsError::NoStroke);
        };
        let g = &self.document.grid;
        if !c.brush.valid_point(g, from)
            || !c.brush.valid_point(g, to)
            || c.base != self.document.revision
        {
            self.cancel();
            return Err(ContentsError::InvalidPoint);
        }
        let vector: [f64; 3] = std::array::from_fn(|d| to[d] - from[d]);
        let length = vector.into_iter().fold(0.0_f64, f64::hypot);
        let unit = if length > 0.0 {
            vector.map(|x| x / length)
        } else {
            [0.0; 3]
        };
        for (i, id) in c.cells.iter_mut().enumerate() {
            let p = coordinate(g, i);
            if c.brush.depth == BrushDepth::Layer && p[c.brush.axis] != c.brush.layer {
                continue;
            }
            let center = g.cell_position(p).unwrap();
            let offset: [f64; 3] = std::array::from_fn(|d| center[d] - from[d]);
            let along = offset
                .iter()
                .zip(unit)
                .map(|(x, y)| x * y)
                .sum::<f64>()
                .clamp(0.0, length);
            let distance = (0..3)
                .map(|d| offset[d] - along * unit[d])
                .fold(0.0_f64, f64::hypot);
            if distance <= c.brush.radius {
                *id = c.brush.target(*id);
            }
        }
        Ok(())
    }
    pub fn commit(&mut self) -> Result<bool, ContentsError> {
        let candidate = self.candidate.take().ok_or(ContentsError::NoStroke)?;
        if candidate.base != self.document.revision {
            return Err(ContentsError::Stale);
        }
        self.publish(
            candidate.cells,
            match candidate.brush.operation {
                BrushOperation::Add => "Add contents",
                BrushOperation::Remove => "Remove contents",
                BrushOperation::Replace => "Replace contents",
            },
        )
    }
    pub fn fill_air(&mut self) -> Result<bool, ContentsError> {
        if self.active() {
            return Err(ContentsError::Busy);
        }
        limit(self.document.cells.len() * 2, self.candidate_limit)?;
        let mut cells = reserve(self.document.cells.len())?;
        cells.resize(self.document.cells.len(), ContentsMaterial::Air as u16);
        self.publish(cells, "Fill domain with Air")
    }
    fn publish(&mut self, cells: Vec<u16>, label: &'static str) -> Result<bool, ContentsError> {
        let changed = cells
            .iter()
            .zip(&self.document.cells)
            .filter(|(a, b)| a != b)
            .count();
        if changed == 0 {
            return Ok(false);
        }
        let revision = self
            .document
            .revision
            .checked_add(1)
            .ok_or(ContentsError::RevisionExhausted)?;
        let sparse_cost = changed * size_of::<Change>();
        let dense_cost = 4 * self.document.cells.len();
        limit(
            bytes(&cells) + sparse_cost.min(dense_cost),
            self.candidate_limit,
        )?;
        let delta = if sparse_cost < dense_cost {
            let mut records = reserve(changed)?;
            for (index, (&before, &after)) in self.document.cells.iter().zip(&cells).enumerate() {
                if before != after {
                    records.push(Change {
                        index,
                        before,
                        after,
                    });
                }
            }
            Delta::Sparse(records)
        } else {
            Delta::Dense {
                before: copied(&self.document.cells)?,
                after: copied(&cells)?,
            }
        };
        let command = Command {
            id: self.document.id,
            base: self.document.revision,
            result: revision,
            label,
            delta,
        };
        // Allocate replacement metadata before invalidating redo or evicting.
        // Existing payloads move, never clone. Temporary metadata is a candidate
        // charge; permanent metadata and retained deltas are a history charge.
        let mut start = 0;
        let mut retained = self
            .commands
            .iter()
            .take(self.cursor)
            .map(Command::payload)
            .sum::<usize>();
        let mut next = loop {
            // Select the retained suffix AND its metadata before allocating.
            // A fitting dense command can replace many small old commands.
            let estimate = (self.cursor - start + 1) * size_of::<Command>();
            let mut history_required = estimate + command.payload() + retained;
            let mut candidate_required = bytes(&cells) + command.payload() + estimate;
            if history_required <= self.history_limit && candidate_required <= self.candidate_limit
            {
                let mut next = VecDeque::new();
                next.try_reserve_exact(self.cursor - start + 1)
                    .map_err(|_| ContentsError::Allocation)?;
                let actual = next.capacity() * size_of::<Command>();
                history_required = actual + command.payload() + retained;
                candidate_required = bytes(&cells) + command.payload() + actual;
                if history_required <= self.history_limit
                    && candidate_required <= self.candidate_limit
                {
                    break next;
                }
            }
            if start == self.cursor {
                limit(history_required, self.history_limit)?;
                limit(candidate_required, self.candidate_limit)?;
                unreachable!();
            }
            retained -= self.commands[start].payload();
            start += 1;
        };
        let evicted = self
            .evicted
            .checked_add(start as u64)
            .ok_or(ContentsError::RevisionExhausted)?;
        for (i, c) in self.commands.drain(..).enumerate() {
            if i >= start && i < self.cursor {
                next.push_back(c);
            }
        }
        next.push_back(command);
        self.commands = next;
        self.cursor = self.commands.len();
        self.evicted = evicted;
        self.document.cells = cells;
        self.document.revision = revision;
        Ok(true)
    }
    pub fn undo(&mut self) -> Result<bool, ContentsError> {
        if self.active() {
            self.cancel();
            return Ok(false);
        }
        if self.cursor == 0 {
            return Ok(false);
        }
        let c = &self.commands[self.cursor - 1];
        if !c.verify(&self.document, false) {
            return Err(ContentsError::Stale);
        }
        let revision = self
            .document
            .revision
            .checked_add(1)
            .ok_or(ContentsError::RevisionExhausted)?;
        c.apply(&mut self.document, false);
        self.cursor -= 1;
        self.document.revision = revision;
        Ok(true)
    }
    pub fn redo(&mut self) -> Result<bool, ContentsError> {
        if self.active() {
            return Err(ContentsError::Busy);
        }
        let Some(c) = self.commands.get(self.cursor) else {
            return Ok(false);
        };
        if !c.verify(&self.document, true) {
            return Err(ContentsError::Stale);
        }
        let revision = self
            .document
            .revision
            .checked_add(1)
            .ok_or(ContentsError::RevisionExhausted)?;
        c.apply(&mut self.document, true);
        self.cursor += 1;
        self.document.revision = revision;
        Ok(true)
    }
}
