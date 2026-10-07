//! Bounded instantaneous operators on one periodic, fitted 2.5D height strip.
//! This workspace has no accepted flow state, integrator or simulation clock.
use std::fmt;
const NONE: usize = usize::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FittedHeightStage {
    BeforeAssembly,
    GeometryNode,
    Triangle,
    PressureImage,
    BeforeOutput,
}
#[derive(Debug, Clone, PartialEq)]
pub enum FittedHeightError {
    InvalidGeometry,
    InvalidCoefficient,
    ShapeMismatch,
    ArithmeticFailure,
    IntegerOverflow,
    AllocationFailed,
    BufferLimit { required: usize, limit: usize },
    ColumnLimit,
    PressureTermLimit,
    PressureRank { expected: usize, actual: usize },
    KinematicMismatch,
    IdentityFailure,
    ProvenanceMismatch,
    Cancelled { stage: FittedHeightStage },
}
impl fmt::Display for FittedHeightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fitted-height diagnostic rejected: {self:?}")
    }
}
impl std::error::Error for FittedHeightError {}
fn checked(x: f64) -> Result<f64, FittedHeightError> {
    if x == 0.0 || x.is_normal() {
        Ok(x)
    } else {
        Err(FittedHeightError::ArithmeticFailure)
    }
}
fn add(a: f64, b: f64) -> Result<f64, FittedHeightError> {
    checked(a + b)
}
fn mul(a: f64, b: f64) -> Result<f64, FittedHeightError> {
    let x = checked(a * b)?;
    if a != 0.0 && b != 0.0 && x == 0.0 {
        Err(FittedHeightError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn div(a: f64, b: f64) -> Result<f64, FittedHeightError> {
    if !b.is_normal() {
        return Err(FittedHeightError::ArithmeticFailure);
    }
    let x = checked(a / b)?;
    if a != 0.0 && x == 0.0 {
        Err(FittedHeightError::ArithmeticFailure)
    } else {
        Ok(x)
    }
}
fn checkpoint(
    cancel: &mut impl FnMut(FittedHeightStage) -> bool,
    stage: FittedHeightStage,
) -> Result<(), FittedHeightError> {
    if cancel(stage) {
        Err(FittedHeightError::Cancelled { stage })
    } else {
        Ok(())
    }
}
#[derive(Clone, Copy, Default)]
struct Dual {
    v: f64,
    d: f64,
}
impl Dual {
    fn add(self, b: Self) -> Result<Self, FittedHeightError> {
        Ok(Self {
            v: add(self.v, b.v)?,
            d: add(self.d, b.d)?,
        })
    }
    fn sub(self, b: Self) -> Result<Self, FittedHeightError> {
        self.add(Self { v: -b.v, d: -b.d })
    }
    fn mul(self, b: Self) -> Result<Self, FittedHeightError> {
        Ok(Self {
            v: mul(self.v, b.v)?,
            d: add(mul(self.d, b.v)?, mul(self.v, b.d)?)?,
        })
    }
    fn div(self, b: Self) -> Result<Self, FittedHeightError> {
        Ok(Self {
            v: div(self.v, b.v)?,
            d: div(add(mul(self.d, b.v)?, -mul(self.v, b.d)?)?, mul(b.v, b.v)?)?,
        })
    }
    fn scale(self, s: f64) -> Result<Self, FittedHeightError> {
        self.mul(Self { v: s, d: 0.0 })
    }
}
fn cross(a: [Dual; 2], b: [Dual; 2]) -> Result<Dual, FittedHeightError> {
    a[0].mul(b[1])?.sub(a[1].mul(b[0])?)
}
fn difference(a: [Dual; 2], b: [Dual; 2]) -> Result<[Dual; 2], FittedHeightError> {
    Ok([a[0].sub(b[0])?, a[1].sub(b[1])?])
}

/// One periodic polygonal strip; bottom abscissae are fixed during diagnostics.
/// Cap and bottom share a period, but need not be vertically aligned.
#[derive(Clone, Copy)]
pub struct FittedHeightGeometry<'a> {
    pub cap: &'a [[f64; 2]],
    pub bottom_x: &'a [f64],
    pub extrusion_width: f64,
    pub density: f64,
    pub dynamic_viscosity: f64,
}
/// Constructor limits are rejection thresholds, never weight floors.
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightSettings {
    pub max_columns: usize,
    pub max_pressure_terms: usize,
    pub memory_limit: usize,
    pub minimum_height: f64,
    /// Twice triangle area divided by the sum of squared edge lengths.
    pub minimum_shape_quality: f64,
    /// Global relative elimination threshold; rank is also checked against topology.
    pub rank_relative_tolerance: f64,
    /// Absolute plus relative error allowance for diagnostic work identities.
    pub identity_tolerance: f64,
}
impl Default for FittedHeightSettings {
    fn default() -> Self {
        Self {
            max_columns: 64,
            max_pressure_terms: 9216,
            memory_limit: 64 << 20,
            minimum_height: 1e-10,
            minimum_shape_quality: 1e-6,
            rank_relative_tolerance: 1e-10,
            identity_tolerance: 1e-11,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FittedHeightPlan {
    pub columns: usize,
    pub geometric_nodes: usize,
    pub periodic_nodes: usize,
    pub triangles: usize,
    pub reduced_velocity_unknowns: usize,
    pub pressure_modes: usize,
    pub pressure_term_capacity: usize,
    pub rank_scratch_entries: usize,
    /// Preallocation payload including persistent constructor rank scratch.
    pub nominal_bytes: usize,
}
impl FittedHeightPlan {
    pub fn new(columns: usize) -> Result<Self, FittedHeightError> {
        if columns < 2 {
            return Err(FittedHeightError::InvalidGeometry);
        }
        let times = |n: usize| {
            columns
                .checked_mul(n)
                .ok_or(FittedHeightError::IntegerOverflow)
        };
        let raw = times(8)?
            .checked_add(3)
            .ok_or(FittedHeightError::IntegerOverflow)?;
        let nodes = times(8)?;
        let tris = times(12)?;
        let xy = times(11)?;
        let rank = nodes;
        let rank_entries = tris
            .checked_mul(xy)
            .ok_or(FittedHeightError::IntegerOverflow)?;
        let mut bytes = 0usize;
        let mut include = |count: usize, size: usize| -> Result<(), FittedHeightError> {
            bytes = bytes
                .checked_add(
                    count
                        .checked_mul(size)
                        .ok_or(FittedHeightError::IntegerOverflow)?,
                )
                .ok_or(FittedHeightError::IntegerOverflow)?;
            Ok(())
        };
        include(raw, std::mem::size_of::<FittedHeightNode>())?;
        include(times(2)?, std::mem::size_of::<[usize; 3]>())?;
        include(
            times(4)?
                .checked_add(1)
                .ok_or(FittedHeightError::IntegerOverflow)?,
            std::mem::size_of::<MacroEdge>(),
        )?;
        include(tris, std::mem::size_of::<FittedHeightTriangle>())?;
        include(nodes, std::mem::size_of::<Embedding>())?;
        include(times(36)?, std::mem::size_of::<FacePiece>())?;
        include(times(36)?, std::mem::size_of::<FittedHeightDualFace>())?;
        include(times(144)?, std::mem::size_of::<FittedHeightPressureTerm>())?;
        include(xy, std::mem::size_of::<usize>())?;
        include(rank_entries, std::mem::size_of::<f64>())?;
        include(nodes, std::mem::size_of::<f64>())?;
        include(raw, std::mem::size_of::<[Dual; 2]>())?;
        include(nodes, std::mem::size_of::<FittedHeightNodeDiagnostic>())?;
        include(tris, std::mem::size_of::<[f64; 2]>())?;
        Ok(Self {
            columns,
            geometric_nodes: raw,
            periodic_nodes: nodes,
            triangles: tris,
            reduced_velocity_unknowns: times(17)?,
            pressure_modes: rank,
            pressure_term_capacity: times(144)?,
            rank_scratch_entries: rank_entries,
            nominal_bytes: bytes,
        })
    }
}
struct Budget {
    limit: usize,
    used: usize,
}
impl Budget {
    fn vector<T>(&mut self, n: usize) -> Result<Vec<T>, FittedHeightError> {
        let requested = n
            .checked_mul(std::mem::size_of::<T>())
            .and_then(|v| v.checked_add(self.used))
            .ok_or(FittedHeightError::IntegerOverflow)?;
        if requested > self.limit {
            return Err(FittedHeightError::BufferLimit {
                required: requested,
                limit: self.limit,
            });
        }
        let mut v = Vec::new();
        v.try_reserve_exact(n)
            .map_err(|_| FittedHeightError::AllocationFailed)?;
        self.used = self
            .used
            .checked_add(
                v.capacity()
                    .checked_mul(std::mem::size_of::<T>())
                    .ok_or(FittedHeightError::IntegerOverflow)?,
            )
            .ok_or(FittedHeightError::IntegerOverflow)?;
        if self.used > self.limit {
            return Err(FittedHeightError::BufferLimit {
                required: self.used,
                limit: self.limit,
            });
        }
        Ok(v)
    }
}
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightNode {
    pub position: [f64; 2],
    pub periodic_index: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightTriangle {
    pub nodes: [usize; 3],
    pub area: f64,
    pub gradients: [[f64; 2]; 3],
}
#[derive(Clone, Copy)]
struct MacroEdge {
    ends: [usize; 2],
    owners: [usize; 2],
    node: usize,
}
#[derive(Clone, Copy)]
struct Embedding {
    columns: [[usize; 2]; 3],
    weights: [f64; 2],
}
#[derive(Clone, Copy)]
struct FacePiece {
    nodes: [usize; 3],
    normal: [f64; 2],
    pair: usize,
    sign: f64,
}
/// Shared oriented face, from the smaller periodic node ID to the larger ID.
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightDualFace {
    pub nodes: [usize; 2],
    pub flux: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightPressureTerm {
    pub mode: usize,
    pub triangle: usize,
    pub value: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightVelocityRow {
    pub columns: [Option<usize>; 2],
    pub weights: [f64; 2],
}
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FittedHeightNodeDiagnostic {
    pub mass_rate: f64,
    pub mass_flux_sum: f64,
    pub divergence_mass: f64,
    pub convection: [f64; 3],
    pub strain_force: [f64; 3],
    pub pressure_force: [f64; 3],
}
#[derive(Clone, Copy)]
pub struct FittedHeightInputs<'a> {
    pub velocity: &'a [[f64; 3]],
    pub pressure_coefficients: &'a [f64],
}
#[derive(Debug, Clone, Copy)]
pub struct FittedHeightReport {
    pub liquid_volume: f64,
    pub total_mass: f64,
    pub divergence_max: f64,
    /// m' + sum(f); zero only for a divergence-free input.
    pub continuity_defect_max: f64,
    /// Reynolds identity including the explicitly integrated divergence defect.
    pub geometric_identity_error: f64,
    pub strain_power: f64,
    pub advection_dissipation: f64,
    pub pressure_work: f64,
    pub pressure_adjoint_error: f64,
    pub strain_work_error: f64,
    pub convection_work_error: f64,
    pub total_mass_rate: f64,
    pub momentum_flux_sum: [f64; 3],
    pub strain_force_sum: [f64; 3],
    pub pressure_force_sum: [f64; 3],
}
/// One immutable geometry and bounded working arrays. No flow-state ownership,
/// geometry replacement, time advancement, pressure solve or dense transfer map.
#[cfg_attr(test, derive(Clone))]
pub struct FittedHeightWorkspace {
    plan: FittedHeightPlan,
    settings: FittedHeightSettings,
    width: f64,
    density: f64,
    viscosity: f64,
    nodes: Vec<FittedHeightNode>,
    macro_triangles: Vec<[usize; 3]>,
    edges: Vec<MacroEdge>,
    triangles: Vec<FittedHeightTriangle>,
    embedding: Vec<Embedding>,
    pieces: Vec<FacePiece>,
    faces: Vec<FittedHeightDualFace>,
    pressure_terms: Vec<FittedHeightPressureTerm>,
    pressure_columns: Vec<usize>,
    rank_scratch: Vec<f64>,
    mass: Vec<f64>,
    motion: Vec<[Dual; 2]>,
    candidate: Vec<FittedHeightNodeDiagnostic>,
    triangle_scratch: Vec<[f64; 2]>,
    allocated_bytes: usize,
    pivot_ratio: f64,
}
impl FittedHeightWorkspace {
    /// Copy one validated fixed-topology geometry. No accepted flow state.
    pub fn new(
        geometry: FittedHeightGeometry<'_>,
        settings: FittedHeightSettings,
        mut cancel: impl FnMut(FittedHeightStage) -> bool,
    ) -> Result<Self, FittedHeightError> {
        let FittedHeightGeometry {
            cap,
            bottom_x,
            extrusion_width: width,
            density,
            dynamic_viscosity: viscosity,
        } = geometry;
        if bottom_x.len() != cap.len() {
            return Err(FittedHeightError::ShapeMismatch);
        }
        let c = cap
            .len()
            .checked_sub(1)
            .ok_or(FittedHeightError::InvalidGeometry)?;
        let plan = FittedHeightPlan::new(c)?;
        if c > settings.max_columns {
            return Err(FittedHeightError::ColumnLimit);
        }
        if plan.pressure_term_capacity > settings.max_pressure_terms {
            return Err(FittedHeightError::PressureTermLimit);
        }
        for x in [
            width,
            density,
            settings.minimum_height,
            settings.minimum_shape_quality,
            settings.rank_relative_tolerance,
            settings.identity_tolerance,
        ] {
            if !x.is_normal() || x <= 0.0 {
                return Err(FittedHeightError::InvalidCoefficient);
            }
        }
        if viscosity < 0.0
            || checked(viscosity).is_err()
            || settings.rank_relative_tolerance >= 1.0
            || settings.minimum_shape_quality >= 0.5
        {
            return Err(FittedHeightError::InvalidCoefficient);
        }
        for p in cap {
            checked(p[0])?;
            checked(p[1])?;
            if p[1] < settings.minimum_height {
                return Err(FittedHeightError::InvalidGeometry);
            }
        }
        if cap[0][1].to_bits() != cap[c][1].to_bits() || cap.windows(2).any(|p| p[1][0] <= p[0][0])
        {
            return Err(FittedHeightError::InvalidGeometry);
        }
        for &x in bottom_x {
            checked(x)?;
        }
        if bottom_x.windows(2).any(|p| p[1] <= p[0]) {
            return Err(FittedHeightError::InvalidGeometry);
        }
        let period = checked(bottom_x[c] - bottom_x[0])?;
        if cap[c][0].to_bits() != add(cap[0][0], period)?.to_bits() {
            return Err(FittedHeightError::InvalidGeometry);
        }
        if plan.nominal_bytes > settings.memory_limit {
            return Err(FittedHeightError::BufferLimit {
                required: plan.nominal_bytes,
                limit: settings.memory_limit,
            });
        }
        checkpoint(&mut cancel, FittedHeightStage::BeforeAssembly)?;
        let mut budget = Budget {
            limit: settings.memory_limit,
            used: 0,
        };
        let nodes = budget.vector(plan.geometric_nodes)?;
        let macro_triangles = budget.vector(2 * c)?;
        let edges = budget.vector(4 * c + 1)?;
        let triangles = budget.vector(plan.triangles)?;
        let embedding = budget.vector(plan.periodic_nodes)?;
        let pieces = budget.vector(3 * plan.triangles)?;
        let faces = budget.vector(3 * plan.triangles)?;
        let pressure_terms = budget.vector(plan.pressure_term_capacity)?;
        let pressure_columns = budget.vector(11 * c)?;
        let mut rank_scratch = budget.vector(plan.rank_scratch_entries)?;
        rank_scratch.resize(plan.rank_scratch_entries, 0.0);
        let mut mass = budget.vector(plan.periodic_nodes)?;
        mass.resize(plan.periodic_nodes, 0.0);
        let mut motion = budget.vector(plan.geometric_nodes)?;
        motion.resize(plan.geometric_nodes, [Dual::default(); 2]);
        let mut candidate = budget.vector(plan.periodic_nodes)?;
        candidate.resize(plan.periodic_nodes, FittedHeightNodeDiagnostic::default());
        let mut triangle_scratch = budget.vector(plan.triangles)?;
        triangle_scratch.resize(plan.triangles, [0.0; 2]);
        let mut w = Self {
            plan,
            settings,
            width,
            density,
            viscosity,
            nodes,
            macro_triangles,
            edges,
            triangles,
            embedding,
            pieces,
            faces,
            pressure_terms,
            pressure_columns,
            rank_scratch,
            mass,
            motion,
            candidate,
            triangle_scratch,
            allocated_bytes: budget.used,
            pivot_ratio: 0.0,
        };
        w.build(cap, bottom_x, &mut cancel)?;
        Ok(w)
    }
    /// Candidate-only fixed-topology reconstruction using existing capacities.
    /// Rejection may leave this workspace incomplete; no accepted flow owns it
    /// while this method runs. Coefficients and limits retain their constructor
    /// values, and the caller must publish only after all composed gates pass.
    pub(crate) fn reassemble(
        &mut self,
        cap: &[[f64; 2]],
        bottom_x: &[f64],
    ) -> Result<(), FittedHeightError> {
        let c = self.plan.columns;
        if cap.len() != c + 1 || bottom_x.len() != c + 1 {
            return Err(FittedHeightError::ShapeMismatch);
        }
        for p in cap {
            checked(p[0])?;
            checked(p[1])?;
            if p[1] < self.settings.minimum_height {
                return Err(FittedHeightError::InvalidGeometry);
            }
        }
        for &x in bottom_x {
            checked(x)?;
        }
        if cap[0][1].to_bits() != cap[c][1].to_bits()
            || cap.windows(2).any(|p| p[1][0] <= p[0][0])
            || bottom_x.windows(2).any(|p| p[1] <= p[0])
            || cap[c][0].to_bits() != add(cap[0][0], checked(bottom_x[c] - bottom_x[0])?)?.to_bits()
        {
            return Err(FittedHeightError::InvalidGeometry);
        }
        self.nodes.clear();
        self.macro_triangles.clear();
        self.edges.clear();
        self.triangles.clear();
        self.embedding.clear();
        self.pieces.clear();
        self.faces.clear();
        self.pressure_terms.clear();
        self.pressure_columns.clear();
        self.mass.fill(0.0);
        self.motion.fill([Dual::default(); 2]);
        self.candidate.fill(FittedHeightNodeDiagnostic::default());
        self.triangle_scratch.fill([0.0; 2]);
        self.build(cap, bottom_x, &mut |_| false)
    }
    pub fn plan(&self) -> FittedHeightPlan {
        self.plan
    }
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes
    }
    pub fn nodes(&self) -> &[FittedHeightNode] {
        &self.nodes
    }
    pub fn triangles(&self) -> &[FittedHeightTriangle] {
        &self.triangles
    }
    pub fn nodal_mass(&self) -> &[f64] {
        &self.mass
    }
    pub fn pressure_basis(&self) -> &[FittedHeightPressureTerm] {
        &self.pressure_terms
    }
    pub fn pressure_columns(&self) -> &[usize] {
        &self.pressure_columns
    }
    /// Elimination pivot ratio, not an inf-sup constant or stability certificate.
    pub fn velocity_embedding(
        &self,
        node: usize,
        component: usize,
    ) -> Option<FittedHeightVelocityRow> {
        let row = self.embedding.get(node)?;
        if component >= 3 {
            return None;
        }
        Some(FittedHeightVelocityRow {
            columns: row.columns[component].map(|i| if i == NONE { None } else { Some(i) }),
            weights: row.weights,
        })
    }
    pub fn pivot_ratio(&self) -> f64 {
        self.pivot_ratio
    }
    /// Scratch only; may be incomplete after cancellation/rejection.
    pub fn shared_flux_scratch(&self) -> &[FittedHeightDualFace] {
        &self.faces
    }
    /// [divergence, reconstructed pressure] per triangle; working scratch only.
    /// Actual differentiated mesh motion, working scratch only.
    pub fn node_motion_scratch(&self, index: usize) -> Option<[f64; 2]> {
        Some(self.motion.get(index)?.map(|v| v.d))
    }
    pub fn triangle_scratch(&self) -> &[[f64; 2]] {
        &self.triangle_scratch
    }
    pub fn embed_velocity(
        &self,
        reduced: &[f64],
        out: &mut [[f64; 3]],
    ) -> Result<(), FittedHeightError> {
        if reduced.len() != self.plan.reduced_velocity_unknowns
            || out.len() != self.plan.periodic_nodes
        {
            return Err(FittedHeightError::ShapeMismatch);
        }
        for &x in reduced {
            checked(x)?;
        }
        // Precheck all arithmetic before writing any caller output.
        for e in &self.embedding {
            for d in 0..3 {
                self.embedded(e, d, reduced)?;
            }
        }
        for (v, e) in out.iter_mut().zip(&self.embedding) {
            for (d, x) in v.iter_mut().enumerate() {
                *x = self.embedded(e, d, reduced)?;
            }
        }
        Ok(())
    }
    fn embedded(&self, e: &Embedding, d: usize, v: &[f64]) -> Result<f64, FittedHeightError> {
        let mut x = 0.0;
        for j in 0..2 {
            if e.columns[d][j] != NONE {
                x = add(x, mul(e.weights[j], v[e.columns[d][j]])?)?;
            }
        }
        Ok(x)
    }
    fn push_node(&mut self, p: [f64; 2]) -> usize {
        let i = self.nodes.len();
        self.nodes.push(FittedHeightNode {
            position: p,
            periodic_index: i,
        });
        i
    }
    fn build(
        &mut self,
        cap: &[[f64; 2]],
        bottom_x: &[f64],
        cancel: &mut impl FnMut(FittedHeightStage) -> bool,
    ) -> Result<(), FittedHeightError> {
        let c = self.plan.columns;
        for &x in bottom_x {
            self.push_node([x, 0.0]);
        }
        for &p in cap {
            self.push_node(p);
        }
        for i in 0..c {
            self.macro_triangles.push([i, i + 1, i + c + 2]);
            self.macro_triangles.push([i, i + c + 2, i + c + 1]);
        }
        for ti in 0..self.macro_triangles.len() {
            checkpoint(cancel, FittedHeightStage::GeometryNode)?;
            let tri = self.macro_triangles[ti];
            let mut p = [0.0; 2];
            for (d, x) in p.iter_mut().enumerate() {
                *x = div(
                    add(
                        add(
                            self.nodes[tri[0]].position[d],
                            self.nodes[tri[1]].position[d],
                        )?,
                        self.nodes[tri[2]].position[d],
                    )?,
                    3.0,
                )?;
            }
            self.push_node(p);
            for j in 0..3 {
                let a = tri[j].min(tri[(j + 1) % 3]);
                let b = tri[j].max(tri[(j + 1) % 3]);
                if let Some(e) = self.edges.iter_mut().find(|e| e.ends == [a, b]) {
                    e.owners[1] = ti;
                } else {
                    self.edges.push(MacroEdge {
                        ends: [a, b],
                        owners: [ti, NONE],
                        node: NONE,
                    });
                }
            }
        }
        self.edges.sort_unstable_by_key(|e| e.ends);
        // A seam has a real neighboring macrotriangle across the period;
        // it is not a physical boundary edge with a midpoint split.
        let left = self
            .edges
            .iter()
            .position(|e| e.ends == [0, c + 1])
            .unwrap();
        let right = self
            .edges
            .iter()
            .position(|e| e.ends == [c, 2 * c + 1])
            .unwrap();
        self.edges[left].owners[1] = self.edges[right].owners[0];
        self.edges[right].owners[1] = self.edges[left].owners[0];
        for i in 0..self.nodes.len() {
            self.motion[i] = self.nodes[i].position.map(|v| Dual { v, d: 0.0 });
        }
        for j in 0..self.edges.len() {
            let p = self.split_position(self.edges[j])?;
            let i = self.push_node(p.map(|v| v.v));
            self.motion[i] = p;
            self.edges[j].node = i;
        }
        let side_left = self
            .edges
            .iter()
            .find(|e| e.ends == [0, c + 1])
            .unwrap()
            .node;
        let side_right = self
            .edges
            .iter()
            .find(|e| e.ends == [c, 2 * c + 1])
            .unwrap()
            .node;
        let mut next = 0;
        for i in 0..self.nodes.len() {
            let rep = if i == c {
                0
            } else if i == 2 * c + 1 {
                c + 1
            } else if i == side_right {
                side_left
            } else {
                i
            };
            let id = if rep == i {
                let n = next;
                next += 1;
                n
            } else {
                self.nodes[rep].periodic_index
            };
            self.nodes[i].periodic_index = id;
        }
        if next != self.plan.periodic_nodes || self.nodes.len() != self.plan.geometric_nodes {
            return Err(FittedHeightError::InvalidGeometry);
        }
        self.embedding.resize(
            next,
            Embedding {
                columns: [[NONE; 2]; 3],
                weights: [1.0, 0.0],
            },
        );
        let mut column = 0;
        for d in 0..3 {
            for i in 0..next {
                let midpoint = self.edges.iter().any(|e| {
                    e.owners[1] == NONE
                        && (e.ends[1] <= c || e.ends[0] > c)
                        && self.nodes[e.node].periodic_index == i
                });
                let bottom = self
                    .nodes
                    .iter()
                    .any(|n| n.periodic_index == i && n.position[1] == 0.0);
                if !(midpoint || d == 1 && bottom) {
                    self.embedding[i].columns[d][0] = column;
                    column += 1;
                }
            }
            for e in &self.edges {
                if e.owners[1] == NONE && (e.ends[1] <= c || e.ends[0] > c) {
                    let i = self.nodes[e.node].periodic_index;
                    let a = self.nodes[e.ends[0]].periodic_index;
                    let b = self.nodes[e.ends[1]].periodic_index;
                    self.embedding[i].columns[d] = [
                        self.embedding[a].columns[d][0],
                        self.embedding[b].columns[d][0],
                    ];
                    self.embedding[i].weights = [0.5, 0.5];
                }
            }
        }
        if column != self.plan.reduced_velocity_unknowns {
            return Err(FittedHeightError::InvalidGeometry);
        }
        for ti in 0..self.macro_triangles.len() {
            let tri = self.macro_triangles[ti];
            let center = 2 * (c + 1) + ti;
            for j in 0..3 {
                let a = tri[j];
                let b = tri[(j + 1) % 3];
                let edge = self
                    .edges
                    .iter()
                    .find(|e| e.ends == [a.min(b), a.max(b)])
                    .unwrap()
                    .node;
                self.push_triangle([a, edge, center])?;
                self.push_triangle([edge, b, center])?;
            }
        }
        self.build_pressure(cancel)?;
        self.build_faces()?;
        Ok(())
    }
    fn split_position(&self, e: MacroEdge) -> Result<[Dual; 2], FittedHeightError> {
        let [a, b] = e.ends.map(|i| self.motion[i]);
        if e.owners[1] == NONE {
            return Ok([a[0].add(b[0])?.scale(0.5)?, a[1].add(b[1])?.scale(0.5)?]);
        }
        let c = self.plan.columns;
        let period = checked(self.nodes[c].position[0] - self.nodes[0].position[0])?;
        if e.ends == [c, 2 * c + 1] {
            let left = *self
                .edges
                .iter()
                .find(|edge| edge.ends == [0, c + 1])
                .unwrap();
            let mut point = self.split_position(left)?;
            point[0] = point[0].add(Dual { v: period, d: 0.0 })?;
            return Ok(point);
        }
        let [p, mut q] = e.owners.map(|i| self.motion[2 * (c + 1) + i]);
        if e.ends == [0, c + 1] {
            q[0] = q[0].sub(Dual { v: period, d: 0.0 })?;
        }
        let ab = difference(b, a)?;
        let pq = difference(q, p)?;
        let s = cross(difference(p, a)?, pq)?.div(cross(ab, pq)?)?;
        if s.v <= 0.0 || s.v >= 1.0 {
            return Err(FittedHeightError::InvalidGeometry);
        }
        Ok([a[0].add(ab[0].mul(s)?)?, a[1].add(ab[1].mul(s)?)?])
    }
    fn push_triangle(&mut self, ids: [usize; 3]) -> Result<(), FittedHeightError> {
        let p = ids.map(|i| self.nodes[i].position);
        // Area/gradients use the same oriented geometry as subsequent motion.
        let ab = [checked(p[1][0] - p[0][0])?, checked(p[1][1] - p[0][1])?];
        let ac = [checked(p[2][0] - p[0][0])?, checked(p[2][1] - p[0][1])?];
        let area2 = add(mul(ab[0], ac[1])?, -mul(ab[1], ac[0])?)?;
        if area2 <= 0.0 {
            return Err(FittedHeightError::InvalidGeometry);
        }
        let mut lengths = 0.0;
        for j in 0..3 {
            for (&a, &b) in p[j].iter().zip(&p[(j + 1) % 3]) {
                let e = checked(a - b)?;
                lengths = add(lengths, mul(e, e)?)?;
            }
        }
        if div(area2, lengths)? < self.settings.minimum_shape_quality {
            return Err(FittedHeightError::InvalidGeometry);
        }
        let mut gradients = [[0.0; 2]; 3];
        for (i, g) in gradients.iter_mut().enumerate() {
            let j = (i + 1) % 3;
            let k = (i + 2) % 3;
            *g = [
                div(checked(p[j][1] - p[k][1])?, area2)?,
                div(checked(p[k][0] - p[j][0])?, area2)?,
            ];
        }
        let area = mul(area2, 0.5)?;
        let m = div(mul(mul(self.density, self.width)?, area)?, 3.0)?;
        for id in ids {
            let i = self.nodes[id].periodic_index;
            self.mass[i] = add(self.mass[i], m)?;
        }
        self.triangles.push(FittedHeightTriangle {
            nodes: ids,
            area,
            gradients,
        });
        Ok(())
    }
    fn fill_divergence_matrix(&mut self) -> Result<(), FittedHeightError> {
        let cols = 11 * self.plan.columns;
        self.rank_scratch.fill(0.0);
        for (t, tri) in self.triangles.iter().enumerate() {
            for j in 0..3 {
                let e = self.embedding[self.nodes[tri.nodes[j]].periodic_index];
                for d in 0..2 {
                    for k in 0..2 {
                        let col = e.columns[d][k];
                        if col != NONE {
                            let idx = t * cols + col;
                            self.rank_scratch[idx] = add(
                                self.rank_scratch[idx],
                                mul(tri.gradients[j][d], e.weights[k])?,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    fn build_pressure(
        &mut self,
        cancel: &mut impl FnMut(FittedHeightStage) -> bool,
    ) -> Result<(), FittedHeightError> {
        self.fill_divergence_matrix()?;
        let cols = 11 * self.plan.columns;
        let rows = self.plan.triangles;
        let scale = self
            .rank_scratch
            .iter()
            .copied()
            .map(f64::abs)
            .fold(0.0, f64::max);
        let threshold = mul(scale, self.settings.rank_relative_tolerance)?;
        let mut rank = 0;
        let mut min_pivot = f64::INFINITY;
        let mut max_pivot = 0.0_f64;
        for j in 0..cols {
            checkpoint(cancel, FittedHeightStage::PressureImage)?;
            if rank == rows {
                break;
            }
            let p = (rank..rows)
                .max_by(|&a, &b| {
                    self.rank_scratch[a * cols + j]
                        .abs()
                        .total_cmp(&self.rank_scratch[b * cols + j].abs())
                })
                .unwrap();
            let pivot = self.rank_scratch[p * cols + j];
            if pivot.abs() <= threshold {
                continue;
            }
            min_pivot = min_pivot.min(pivot.abs());
            max_pivot = max_pivot.max(pivot.abs());
            for k in 0..cols {
                self.rank_scratch.swap(rank * cols + k, p * cols + k);
            }
            for k in j..cols {
                self.rank_scratch[rank * cols + k] =
                    div(self.rank_scratch[rank * cols + k], pivot)?;
            }
            for i in rank + 1..rows {
                let factor = self.rank_scratch[i * cols + j];
                self.rank_scratch[i * cols + j] = 0.0;
                for k in j + 1..cols {
                    self.rank_scratch[i * cols + k] = add(
                        self.rank_scratch[i * cols + k],
                        -mul(factor, self.rank_scratch[rank * cols + k])?,
                    )?;
                }
            }
            self.pressure_columns.push(j);
            rank += 1;
        }
        if rank != self.plan.pressure_modes {
            return Err(FittedHeightError::PressureRank {
                expected: self.plan.pressure_modes,
                actual: rank,
            });
        }
        self.pivot_ratio = div(min_pivot, max_pivot)?;
        self.fill_divergence_matrix()?;
        for (mode, &col) in self.pressure_columns.iter().enumerate() {
            for triangle in 0..rows {
                let value = self.rank_scratch[triangle * cols + col];
                if value != 0.0 {
                    if self.pressure_terms.len() == self.plan.pressure_term_capacity {
                        return Err(FittedHeightError::PressureTermLimit);
                    }
                    self.pressure_terms.push(FittedHeightPressureTerm {
                        mode,
                        triangle,
                        value,
                    });
                }
            }
        }
        Ok(())
    }
    fn build_faces(&mut self) -> Result<(), FittedHeightError> {
        for t in 0..self.triangles.len() {
            let tri = self.triangles[t];
            let p = tri.nodes.map(|i| self.nodes[i].position);
            let center = [
                div(add(add(p[0][0], p[1][0])?, p[2][0])?, 3.0)?,
                div(add(add(p[0][1], p[1][1])?, p[2][1])?, 3.0)?,
            ];
            for j in 0..3 {
                let k = (j + 1) % 3;
                let l = (j + 2) % 3;
                let i = self.nodes[tri.nodes[j]].periodic_index;
                let h = self.nodes[tri.nodes[k]].periodic_index;
                if i == h {
                    continue;
                }
                let ends = [i.min(h), i.max(h)];
                let pair = if let Some(idx) = self.faces.iter().position(|f| f.nodes == ends) {
                    idx
                } else {
                    self.faces.push(FittedHeightDualFace {
                        nodes: ends,
                        flux: 0.0,
                    });
                    self.faces.len() - 1
                };
                let segment = [
                    checked(center[0] - mul(add(p[j][0], p[k][0])?, 0.5)?)?,
                    checked(center[1] - mul(add(p[j][1], p[k][1])?, 0.5)?)?,
                ];
                self.pieces.push(FacePiece {
                    nodes: [tri.nodes[j], tri.nodes[k], tri.nodes[l]],
                    normal: [segment[1], -segment[0]],
                    pair,
                    sign: if i < h { 1.0 } else { -1.0 },
                });
            }
        }
        Ok(())
    }
    /// Full 9×9 element stiffness on raw triangle nodes, including out-of-plane
    /// shears. A fixed-size stack result; it bypasses global trace restrictions.
    pub fn triangle_stiffness(&self, index: usize) -> Result<[[f64; 9]; 9], FittedHeightError> {
        let tri = self
            .triangles
            .get(index)
            .ok_or(FittedHeightError::ShapeMismatch)?;
        let mut e = [[0.0; 9]; 5];
        for (j, [gx, gy]) in tri.gradients.into_iter().enumerate() {
            e[0][3 * j] = gx;
            e[1][3 * j + 1] = gy;
            e[2][3 * j] = gy;
            e[2][3 * j + 1] = gx;
            e[3][3 * j + 2] = gx;
            e[4][3 * j + 2] = gy;
        }
        let weight = mul(mul(self.viscosity, self.width)?, tri.area)?;
        let mut k = [[0.0; 9]; 9];
        for (a, row) in k.iter_mut().enumerate() {
            for (b, x) in row.iter_mut().enumerate() {
                for (s, w) in [2.0, 2.0, 1.0, 1.0, 1.0].into_iter().enumerate() {
                    *x = add(*x, mul(weight, mul(w, mul(e[s][a], e[s][b])?)?)?)?;
                }
            }
        }
        Ok(k)
    }
    /// Compute instantaneous diagnostics. All output rows retain their exact
    /// previous bits on any failure/cancellation; scratch has no such promise.
    pub fn inspect(
        &mut self,
        input: FittedHeightInputs<'_>,
        output: &mut [FittedHeightNodeDiagnostic],
        mut cancel: impl FnMut(FittedHeightStage) -> bool,
    ) -> Result<FittedHeightReport, FittedHeightError> {
        if output.len() != self.plan.periodic_nodes {
            return Err(FittedHeightError::ShapeMismatch);
        }
        let report = self.compute(input, &mut cancel)?;
        checkpoint(&mut cancel, FittedHeightStage::BeforeOutput)?;
        output.copy_from_slice(&self.candidate);
        Ok(report)
    }
    /// Recompute from the same physical velocity/mesh motion. Correct marginals
    /// alone do not pass: every oriented pair and flux bit must match.
    pub fn audit_shared_flux(
        &mut self,
        input: FittedHeightInputs<'_>,
        stored: &[FittedHeightDualFace],
        mut cancel: impl FnMut(FittedHeightStage) -> bool,
    ) -> Result<(), FittedHeightError> {
        if stored.len() != self.faces.len() {
            return Err(FittedHeightError::ShapeMismatch);
        }
        self.compute(input, &mut cancel)?;
        for (given, actual) in stored.iter().zip(&self.faces) {
            if given.nodes != actual.nodes || given.flux.to_bits() != actual.flux.to_bits() {
                return Err(FittedHeightError::ProvenanceMismatch);
            }
        }
        checkpoint(&mut cancel, FittedHeightStage::BeforeOutput)
    }
    fn compute(
        &mut self,
        input: FittedHeightInputs<'_>,
        cancel: &mut impl FnMut(FittedHeightStage) -> bool,
    ) -> Result<FittedHeightReport, FittedHeightError> {
        let c = self.plan.columns;
        let u = input.velocity;
        if u.len() != self.plan.periodic_nodes
            || input.pressure_coefficients.len() != self.plan.pressure_modes
        {
            return Err(FittedHeightError::ShapeMismatch);
        }
        for v in u {
            for &x in v {
                checked(x)?;
            }
        }
        for &p in input.pressure_coefficients {
            checked(p)?;
        }
        for n in &self.nodes {
            if n.position[1] == 0.0 && u[n.periodic_index][1] != 0.0 {
                return Err(FittedHeightError::KinematicMismatch);
            }
        }
        for edge in &self.edges {
            if edge.owners[1] == NONE && (edge.ends[1] <= c || edge.ends[0] > c) {
                let ids = edge.ends.map(|i| self.nodes[i].periodic_index);
                let mid = self.nodes[edge.node].periodic_index;
                for ((&value, &a), &b) in u[mid].iter().zip(&u[ids[0]]).zip(&u[ids[1]]) {
                    let expected = mul(add(a, b)?, 0.5)?;
                    if value.to_bits() != expected.to_bits() && !(value == 0.0 && expected == 0.0) {
                        return Err(FittedHeightError::KinematicMismatch);
                    }
                }
            }
        }
        checkpoint(cancel, FittedHeightStage::BeforeAssembly)?;
        self.candidate.fill(FittedHeightNodeDiagnostic::default());
        self.triangle_scratch.fill([0.0; 2]);
        for f in &mut self.faces {
            f.flux = 0.0;
        }
        for (i, n) in self.nodes.iter().enumerate() {
            self.motion[i] = n.position.map(|v| Dual { v, d: 0.0 });
        }
        for i in c + 1..2 * (c + 1) {
            for (m, &v) in self.motion[i]
                .iter_mut()
                .zip(&u[self.nodes[i].periodic_index])
            {
                m.d = v;
            }
        }
        for t in 0..self.macro_triangles.len() {
            checkpoint(cancel, FittedHeightStage::GeometryNode)?;
            let tri = self.macro_triangles[t];
            let id = 2 * (c + 1) + t;
            for d in 0..2 {
                self.motion[id][d].d = div(
                    add(
                        add(self.motion[tri[0]][d].d, self.motion[tri[1]][d].d)?,
                        self.motion[tri[2]][d].d,
                    )?,
                    3.0,
                )?;
            }
        }
        for edge in &self.edges {
            let p = self.split_position(*edge)?;
            if p.map(|v| v.v.to_bits()) != self.nodes[edge.node].position.map(f64::to_bits) {
                return Err(FittedHeightError::InvalidGeometry);
            }
            self.motion[edge.node] = p;
        }
        checkpoint(cancel, FittedHeightStage::PressureImage)?;
        for term in &self.pressure_terms {
            let x = &mut self.triangle_scratch[term.triangle][1];
            *x = add(*x, mul(term.value, input.pressure_coefficients[term.mode])?)?;
        }
        let rho_width = mul(self.density, self.width)?;
        let mut strain_power = 0.0;
        let mut pressure_integral = 0.0;
        let mut divmax = 0.0_f64;
        for (t, tri) in self.triangles.iter().enumerate() {
            checkpoint(cancel, FittedHeightStage::Triangle)?;
            let [p0, p1, p2] = tri.nodes.map(|i| self.motion[i]);
            let area = cross(difference(p1, p0)?, difference(p2, p0)?)?.scale(0.5)?;
            let rate = div(mul(rho_width, area.d)?, 3.0)?;
            let ids = tri.nodes.map(|i| self.nodes[i].periodic_index);
            let mut g = [[0.0; 2]; 3];
            for j in 0..3 {
                for (d, row) in g.iter_mut().enumerate() {
                    for (e, x) in row.iter_mut().enumerate() {
                        *x = add(*x, mul(u[ids[j]][d], tri.gradients[j][e])?)?;
                    }
                }
            }
            let divergence = add(g[0][0], g[1][1])?;
            self.triangle_scratch[t][0] = divergence;
            divmax = divmax.max(divergence.abs());
            let defect = div(mul(mul(rho_width, tri.area)?, divergence)?, 3.0)?;
            let strain = [g[0][0], g[1][1], add(g[0][1], g[1][0])?, g[2][0], g[2][1]];
            let weight = mul(mul(self.viscosity, self.width)?, tri.area)?;
            for (s, w) in [2.0, 2.0, 1.0, 1.0, 1.0].into_iter().enumerate() {
                strain_power = add(
                    strain_power,
                    mul(weight, mul(w, mul(strain[s], strain[s])?)?)?,
                )?;
            }
            let q = self.triangle_scratch[t][1];
            let qw = mul(mul(self.width, tri.area)?, q)?;
            pressure_integral = add(pressure_integral, mul(qw, divergence)?)?;
            for (j, &id) in ids.iter().enumerate() {
                let [gx, gy] = tri.gradients[j];
                let out = &mut self.candidate[id];
                out.mass_rate = add(out.mass_rate, rate)?;
                out.divergence_mass = add(out.divergence_mass, defect)?;
                let force = [
                    mul(
                        weight,
                        add(mul(2.0, mul(gx, strain[0])?)?, mul(gy, strain[2])?)?,
                    )?,
                    mul(
                        weight,
                        add(mul(2.0, mul(gy, strain[1])?)?, mul(gx, strain[2])?)?,
                    )?,
                    mul(weight, add(mul(gx, strain[3])?, mul(gy, strain[4])?)?)?,
                ];
                for (d, &f) in force.iter().enumerate() {
                    out.strain_force[d] = add(out.strain_force[d], f)?;
                }
                out.pressure_force[0] = add(out.pressure_force[0], -mul(qw, gx)?)?;
                out.pressure_force[1] = add(out.pressure_force[1], -mul(qw, gy)?)?;
            }
        }
        for piece in &self.pieces {
            let ids = piece.nodes.map(|i| self.nodes[i].periodic_index);
            let mut flux = 0.0;
            for (d, &normal) in piece.normal.iter().enumerate() {
                let mut relative = 0.0;
                for (j, w) in [5.0 / 12.0, 5.0 / 12.0, 1.0 / 6.0].into_iter().enumerate() {
                    relative = add(
                        relative,
                        mul(w, checked(u[ids[j]][d] - self.motion[piece.nodes[j]][d].d)?)?,
                    )?;
                }
                flux = add(flux, mul(relative, normal)?)?;
            }
            let f = &mut self.faces[piece.pair].flux;
            *f = add(*f, mul(piece.sign, mul(rho_width, flux)?)?)?;
        }
        let mut diss = 0.0;
        for face in &self.faces {
            let [i, j] = face.nodes;
            let f = face.flux;
            self.candidate[i].mass_flux_sum = add(self.candidate[i].mass_flux_sum, f)?;
            self.candidate[j].mass_flux_sum = add(self.candidate[j].mass_flux_sum, -f)?;
            for (d, (&ui, &uj)) in u[i].iter().zip(&u[j]).enumerate() {
                let delta = checked(ui - uj)?;
                let momentum = mul(add(mul(f, add(ui, uj)?)?, mul(f.abs(), delta)?)?, 0.5)?;
                self.candidate[i].convection[d] = add(self.candidate[i].convection[d], momentum)?;
                self.candidate[j].convection[d] = add(self.candidate[j].convection[d], -momentum)?;
                diss = add(diss, mul(0.5, mul(f.abs(), mul(delta, delta)?)?)?)?;
            }
        }
        let mut strain_work = 0.0;
        let mut pressure_work = 0.0;
        let mut convection_work = 0.0;
        let mut mass_work = 0.0;
        let mut continuity = 0.0_f64;
        let mut geometric_error = 0.0_f64;
        let mut mass_rate = 0.0;
        let mut sums = [[0.0; 3]; 3];
        let mut total_mass = 0.0;
        for (i, out) in self.candidate.iter().enumerate() {
            let balance = add(out.mass_rate, out.mass_flux_sum)?;
            continuity = continuity.max(balance.abs());
            geometric_error = geometric_error.max(checked(balance - out.divergence_mass)?.abs());
            mass_rate = add(mass_rate, out.mass_rate)?;
            total_mass = add(total_mass, self.mass[i])?;
            for d in 0..3 {
                strain_work = add(strain_work, mul(u[i][d], out.strain_force[d])?)?;
                pressure_work = add(pressure_work, mul(u[i][d], out.pressure_force[d])?)?;
                convection_work = add(convection_work, mul(u[i][d], out.convection[d])?)?;
                mass_work = add(
                    mass_work,
                    mul(0.5, mul(mul(u[i][d], u[i][d])?, out.mass_flux_sum)?)?,
                )?;
                for (k, f) in [out.convection, out.strain_force, out.pressure_force]
                    .into_iter()
                    .enumerate()
                {
                    sums[k][d] = add(sums[k][d], f[d])?;
                }
            }
        }
        let strain_error = checked(strain_work - strain_power)?;
        let pressure_error = add(pressure_work, pressure_integral)?;
        let conv_error = checked(checked(convection_work - mass_work)? - diss)?;
        for (err, scale) in [
            (geometric_error, total_mass.abs() + mass_rate.abs()),
            (strain_error, strain_power.abs() + strain_work.abs()),
            (
                pressure_error,
                pressure_work.abs() + pressure_integral.abs(),
            ),
            (conv_error, convection_work.abs() + mass_work.abs() + diss),
        ] {
            if err.abs() > mul(self.settings.identity_tolerance, add(1.0, checked(scale)?)?)? {
                return Err(FittedHeightError::IdentityFailure);
            }
        }
        Ok(FittedHeightReport {
            liquid_volume: div(total_mass, self.density)?,
            total_mass,
            divergence_max: divmax,
            continuity_defect_max: continuity,
            geometric_identity_error: geometric_error,
            strain_power,
            advection_dissipation: diss,
            pressure_work,
            pressure_adjoint_error: pressure_error,
            strain_work_error: strain_error,
            convection_work_error: conv_error,
            total_mass_rate: mass_rate,
            momentum_flux_sum: sums[0],
            strain_force_sum: sums[1],
            pressure_force_sum: sums[2],
        })
    }
}

#[cfg(test)]
impl FittedHeightWorkspace {
    pub(crate) fn research_paired_geometry_layout() -> [(usize, usize); 14] {
        // Exactly the constructor's 14 Budget::vector arguments at columns=2.
        // No geometry is constructed and no equation is evaluated here.
        [
            (19, std::mem::size_of::<FittedHeightNode>()),
            (4, std::mem::size_of::<[usize; 3]>()),
            (9, std::mem::size_of::<MacroEdge>()),
            (24, std::mem::size_of::<FittedHeightTriangle>()),
            (16, std::mem::size_of::<Embedding>()),
            (72, std::mem::size_of::<FacePiece>()),
            (72, std::mem::size_of::<FittedHeightDualFace>()),
            (288, std::mem::size_of::<FittedHeightPressureTerm>()),
            (22, std::mem::size_of::<usize>()),
            (528, std::mem::size_of::<f64>()),
            (16, std::mem::size_of::<f64>()),
            (19, std::mem::size_of::<[Dual; 2]>()),
            (16, std::mem::size_of::<FittedHeightNodeDiagnostic>()),
            (24, std::mem::size_of::<[f64; 2]>()),
        ]
    }
}
