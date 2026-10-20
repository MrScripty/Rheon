//! Bounded observations of the existing sealed MAC composition.
//! This describes the native method; it is not an executable graph or a claim
//! of continuum accuracy. Receipts are emitted by the numerical owners.
use crate::solver_provider::{ProviderReport, configured::ContextProviderIdentity};
use crate::{GridGeometry, PressureImplementation, PressureReport, PressureSettings};
use sha2::{Digest, Sha256};
use std::{cell::RefCell, sync::Arc};

pub const DESCRIPTOR_VERSION: u32 = 1;
pub const MAX_NODES: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NodeKind {
    Geometry,
    AcceptedState,
    Interval,
    Advection,
    Forces,
    Viscosity,
    PressureRequest,
    PressureCandidate,
    PressureQualification,
    NativeFallback,
    Projection,
    TracerTransport,
    VolumeTransport,
    Publication,
}
impl NodeKind {
    pub const ALL: [Self; 14] = [
        Self::Geometry,
        Self::AcceptedState,
        Self::Interval,
        Self::Advection,
        Self::Forces,
        Self::Viscosity,
        Self::PressureRequest,
        Self::PressureCandidate,
        Self::PressureQualification,
        Self::NativeFallback,
        Self::Projection,
        Self::TracerTransport,
        Self::VolumeTransport,
        Self::Publication,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Geometry => "Sealed Cartesian geometry",
            Self::AcceptedState => "Accepted native MAC state",
            Self::Interval => "Admit actual interval",
            Self::Advection => "Advect face velocity",
            Self::Forces => "Apply prescribed body forces",
            Self::Viscosity => "Newtonian viscosity",
            Self::PressureRequest => "Assemble actual pressure request",
            Self::PressureCandidate => "Configured pressure candidate",
            Self::PressureQualification => "Host pressure qualification",
            Self::NativeFallback => "Native pressure / fallback",
            Self::Projection => "Project and qualify corrected velocity",
            Self::TracerTransport => "Transport appearance tracer",
            Self::VolumeTransport => "Transport represented liquid volume",
            Self::Publication => "Atomic Session publication",
        }
    }
    pub fn lane(self) -> usize {
        match self {
            Self::Geometry => 0,
            Self::AcceptedState => 1,
            Self::PressureQualification | Self::Projection => 3,
            Self::Publication => 4,
            _ => 2,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortKind {
    Geometry,
    NativeState,
    Interval,
    FaceVelocity,
    CellPressure,
    CellFraction,
    CellTracer,
    IntegratedRhs,
    Qualification,
}
impl PortKind {
    pub fn units(self) -> &'static str {
        match self {
            Self::Geometry => "m",
            Self::NativeState => "native MAC fields",
            Self::Interval => "s",
            Self::FaceVelocity => "m/s",
            Self::CellPressure => "Pa",
            Self::CellFraction | Self::CellTracer => "1",
            Self::IntegratedRhs => "m3/s2",
            Self::Qualification => "host gate",
        }
    }
    pub fn association(self) -> &'static str {
        match self {
            Self::FaceVelocity => "staggered Cartesian faces; f32; x-fast",
            Self::CellPressure => "cell centre; f64; x-fast; cell0 gauge=0",
            Self::CellFraction => "cell centre; f64; x-fast",
            Self::CellTracer => "cell centre; f32; x-fast; appearance only",
            Self::IntegratedRhs => "cell centre; f64; x-fast; integrated full operator",
            _ => "composition metadata",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeKind {
    Data,
    Fallback,
    TimeLoop,
    IterationLoop,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkflowEdge {
    pub from: NodeKind,
    pub to: NodeKind,
    pub port: PortKind,
    pub kind: EdgeKind,
}

/// Only the already-supported sealed, constant-density method is representable.
#[derive(Debug, Clone)]
pub struct WorkflowDescriptor {
    pub version: u32,
    pub grid: GridGeometry,
    pub density: f64,
    /// None means disabled; Some(0) is the actually applied zero stage.
    pub viscosity: Option<f64>,
    pub implementation: PressureImplementation,
    pub pressure: PressureSettings,
    pub actual_divergence_limit: f64,
    pub max_courant: f64,
}
impl WorkflowDescriptor {
    pub fn family(&self) -> &'static str {
        "sealed-constant-density-mac-v1"
    }
    pub fn operator(&self) -> &'static str {
        "integrated-sealed-mac-pressure-v1; cell0 gauge=0"
    }
    pub fn nodes(&self) -> &'static [NodeKind] {
        &NodeKind::ALL
    }
    pub fn edges(&self) -> Vec<WorkflowEdge> {
        use EdgeKind::*;
        use NodeKind::*;
        use PortKind::{
            CellFraction, CellPressure, CellTracer, FaceVelocity, IntegratedRhs, NativeState,
            Qualification,
        };
        let velocity = if self.viscosity.is_some() {
            Viscosity
        } else {
            Forces
        };
        let mut edges = vec![
            WorkflowEdge {
                from: AcceptedState,
                to: Interval,
                port: NativeState,
                kind: Data,
            },
            WorkflowEdge {
                from: AcceptedState,
                to: Advection,
                port: FaceVelocity,
                kind: Data,
            },
            WorkflowEdge {
                from: Advection,
                to: Forces,
                port: FaceVelocity,
                kind: Data,
            },
            WorkflowEdge {
                from: velocity,
                to: PressureRequest,
                port: FaceVelocity,
                kind: Data,
            },
            WorkflowEdge {
                from: PressureRequest,
                to: PressureCandidate,
                port: IntegratedRhs,
                kind: Data,
            },
            WorkflowEdge {
                from: PressureRequest,
                to: PressureQualification,
                port: IntegratedRhs,
                kind: Data,
            },
            WorkflowEdge {
                from: PressureCandidate,
                to: PressureQualification,
                port: CellPressure,
                kind: Data,
            },
            WorkflowEdge {
                from: PressureQualification,
                to: Projection,
                port: CellPressure,
                kind: Data,
            },
            WorkflowEdge {
                from: PressureQualification,
                to: NativeFallback,
                port: Qualification,
                kind: Fallback,
            },
            WorkflowEdge {
                from: PressureRequest,
                to: NativeFallback,
                port: IntegratedRhs,
                kind: Data,
            },
            WorkflowEdge {
                from: NativeFallback,
                to: Projection,
                port: CellPressure,
                kind: Data,
            },
            WorkflowEdge {
                from: velocity,
                to: Projection,
                port: FaceVelocity,
                kind: Data,
            },
            WorkflowEdge {
                from: Projection,
                to: TracerTransport,
                port: FaceVelocity,
                kind: Data,
            },
            WorkflowEdge {
                from: Projection,
                to: VolumeTransport,
                port: FaceVelocity,
                kind: Data,
            },
            WorkflowEdge {
                from: AcceptedState,
                to: TracerTransport,
                port: CellTracer,
                kind: Data,
            },
            WorkflowEdge {
                from: AcceptedState,
                to: VolumeTransport,
                port: CellFraction,
                kind: Data,
            },
            WorkflowEdge {
                from: Projection,
                to: Publication,
                port: NativeState,
                kind: Data,
            },
            WorkflowEdge {
                from: TracerTransport,
                to: Publication,
                port: CellTracer,
                kind: Data,
            },
            WorkflowEdge {
                from: VolumeTransport,
                to: Publication,
                port: CellFraction,
                kind: Data,
            },
            WorkflowEdge {
                from: NativeFallback,
                to: NativeFallback,
                port: CellPressure,
                kind: IterationLoop,
            },
            WorkflowEdge {
                from: Publication,
                to: AcceptedState,
                port: NativeState,
                kind: TimeLoop,
            },
        ];
        if self.viscosity.is_some() {
            edges.push(WorkflowEdge {
                from: Forces,
                to: Viscosity,
                port: FaceVelocity,
                kind: Data,
            });
        }
        for to in [
            Advection,
            Forces,
            Viscosity,
            PressureRequest,
            PressureCandidate,
            PressureQualification,
            NativeFallback,
            Projection,
            TracerTransport,
            VolumeTransport,
        ] {
            edges.push(WorkflowEdge {
                from: NodeKind::Geometry,
                to,
                port: PortKind::Geometry,
                kind: Data,
            });
            edges.push(WorkflowEdge {
                from: Interval,
                to,
                port: PortKind::Interval,
                kind: Data,
            });
        }
        edges
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.version != DESCRIPTOR_VERSION {
            return Err("unsupported workflow descriptor version");
        }
        if !self.density.is_finite()
            || self.density <= 0.0
            || self
                .viscosity
                .is_some_and(|v| !v.is_finite() || v < 0.0 || (v != 0.0 && !v.is_normal()))
            || [
                self.pressure.relative_residual,
                self.pressure.absolute_residual,
                self.pressure.divergence_limit,
                self.actual_divergence_limit,
                self.max_courant,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
            || self.max_courant <= 0.0
            || self.max_courant > 1.0
        {
            return Err("invalid workflow descriptor parameters");
        }
        Ok(())
    }
    /// Canonical fixed ordering, exact IEEE parameter bits, no labels/UI layout.
    pub fn digest(&self) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(b"rheon-sealed-mac-workflow-v1\0");
        h.update(self.version.to_le_bytes());
        for n in self.grid.counts() {
            h.update((n as u64).to_le_bytes());
        }
        for v in self
            .grid
            .spacing()
            .into_iter()
            .chain(self.grid.origin())
            .chain([
                self.density,
                self.viscosity.unwrap_or(0.0),
                self.pressure.relative_residual,
                self.pressure.absolute_residual,
                self.pressure.divergence_limit,
                self.actual_divergence_limit,
                self.max_courant,
            ])
        {
            h.update(v.to_bits().to_le_bytes());
        }
        h.update([u8::from(self.viscosity.is_some())]);
        h.update(self.implementation.id().as_bytes());
        h.update((self.pressure.max_iterations as u64).to_le_bytes());
        for node in self.nodes() {
            h.update([*node as u8]);
        }
        for edge in self.edges() {
            h.update([
                edge.from as u8,
                edge.to as u8,
                edge.port as u8,
                edge.kind as u8,
            ]);
        }
        h.finalize().into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeStatus {
    Started,
    Completed,
    CandidateReturned,
    Rejected,
    Selected,
    Accepted,
}
#[derive(Debug, Clone)]
pub struct NodeReceipt {
    pub node: NodeKind,
    pub status: NodeStatus,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttemptOutcome {
    Accepted,
    Cancelled,
    Failed,
    Refused,
}
#[derive(Debug, Clone)]
pub struct PressureFacts {
    pub selected: String,
    pub binding: Option<ContextProviderIdentity>,
    pub prediction_called: bool,
    pub support: Option<bool>,
    pub rejection: Option<String>,
    pub rhs_sha256: Option<[u8; 32]>,
    pub candidate: Option<PressureReport>,
    pub accepted: Option<PressureReport>,
    /// Pressure-call scope only; never presented as whole workflow time.
    pub pressure_call_ns: Option<u64>,
    /// Actual max(absolute, relative * ||rhs||₂), m³/s², from the host dispatcher.
    pub residual_l2_limit: Option<f64>,
}
#[derive(Debug, Clone)]
pub struct WorkflowAttempt {
    pub descriptor: WorkflowDescriptor,
    pub descriptor_sha256: [u8; 32],
    /// Exact accepted native state consumed by this attempt, not a model-input claim.
    pub input_sha256: [u8; 32],
    /// Actual control values, including complete source rates and force recipe.
    pub request_sha256: [u8; 32],
    pub force_count: usize,
    pub smoke_source_present: bool,
    pub volume_source_present: bool,
    pub run_id: u64,
    pub attempt_id: u64,
    pub accepted_generation: u64,
    pub attempted_generation: u64,
    pub interval_start: f64,
    pub requested_dt: f64,
    pub actual_dt: Option<f64>,
    pub publication: Option<u64>,
    pub outcome: AttemptOutcome,
    pub nodes: Vec<NodeReceipt>,
    pub pressure: Option<PressureFacts>,
    pub actual_divergence_max: Option<f64>,
    pub diagnostic: String,
}
impl WorkflowAttempt {
    /// Compare retained actual-call identity with a reconstructed declared recipe.
    /// A match binds controls; it does not qualify their physical accuracy.
    pub fn matches_request(&self, inputs: crate::LiquidStepInputs<'_>) -> bool {
        self.request_sha256 == request_digest(inputs)
            && self.force_count == inputs.forces.len()
            && self.smoke_source_present == inputs.smoke_source.is_some()
            && self.volume_source_present == inputs.source.is_some()
    }
    pub fn validate(&self) -> Result<(), &'static str> {
        self.descriptor.validate()?;
        if self.descriptor_sha256 != self.descriptor.digest() {
            return Err("workflow descriptor digest");
        }
        if self.attempt_id == 0
            || self.accepted_generation.checked_add(1) != Some(self.attempted_generation)
            || !self.interval_start.is_finite()
            || self.interval_start < 0.0
            || !self.requested_dt.is_finite()
            || self.requested_dt <= 0.0
            || self.actual_dt.is_some_and(|dt| {
                !dt.is_finite()
                    || dt <= 0.0
                    || dt > self.requested_dt
                    || !(self.interval_start + dt).is_finite()
                    || self.interval_start + dt <= self.interval_start
            })
            || self.nodes.len() > MAX_NODES
            || self.diagnostic.len() > 1000
            || self
                .actual_divergence_max
                .is_some_and(|v| !v.is_finite() || v < 0.0)
        {
            return Err("workflow attempt association");
        }
        if (self.outcome == AttemptOutcome::Accepted) != self.publication.is_some()
            || self
                .publication
                .is_some_and(|g| g != self.attempted_generation || self.actual_dt.is_none())
            || (self.publication.is_some()
                && self.nodes.last().is_none_or(|n| {
                    n.node != NodeKind::Publication || n.status != NodeStatus::Accepted
                }))
        {
            return Err("workflow publication association");
        }
        if let Some(p) = &self.pressure
            && (p.selected.len() > 1000
                || p.rejection.as_ref().is_some_and(|s| s.len() > 1000)
                || p.binding.as_ref().is_some_and(|b| b.validate().is_err())
                || p.residual_l2_limit
                    .is_some_and(|v| !v.is_finite() || v < 0.0)
                || (p.prediction_called && p.support != Some(true))
                || (p.accepted.is_some() && (p.selected.is_empty() || self.actual_dt.is_none()))
                || p.accepted.is_some_and(|r| {
                    [
                        r.true_residual_l2,
                        r.true_residual_max,
                        r.predicted_divergence_max,
                    ]
                    .iter()
                    .any(|v| !v.is_finite() || *v < 0.0)
                }))
        {
            return Err("workflow pressure facts");
        }
        self.validate_nodes()
    }
    fn validate_nodes(&self) -> Result<(), &'static str> {
        use NodeKind::*;
        use NodeStatus::*;
        let mut expected = vec![
            (Geometry, Completed),
            (AcceptedState, Completed),
            (Interval, Completed),
            (Advection, Completed),
            (Forces, Completed),
        ];
        if self.descriptor.viscosity.is_some() {
            expected.push((Viscosity, Completed));
        }
        expected.push((PressureRequest, Completed));
        let base = expected.len();
        if self.pressure.is_some() && self.nodes.len() < base {
            return Err("workflow pressure stage association");
        }
        let mut fallback = true;
        if let Some(p) = &self.pressure {
            if let Some(supported) = p.support {
                let candidate = self.nodes.get(base).map_or(CandidateReturned, |n| n.status);
                if !matches!(candidate, CandidateReturned | Rejected)
                    || (!supported && candidate != Rejected)
                {
                    return Err("workflow candidate stage status");
                }
                expected.push((PressureCandidate, candidate));
                fallback = p.rejection.is_some();
                expected.push((
                    PressureQualification,
                    if fallback { Rejected } else { Accepted },
                ));
            } else if p.binding.is_some() || p.prediction_called || p.rejection.is_some() {
                return Err("workflow native pressure facts");
            }
        }
        if fallback {
            expected.extend([(NativeFallback, Started), (NativeFallback, Selected)]);
        }
        expected.extend([
            (Projection, Completed),
            (TracerTransport, Completed),
            (VolumeTransport, Completed),
            (Publication, Accepted),
        ]);
        if self.nodes.len() > expected.len()
            || self
                .nodes
                .iter()
                .zip(&expected)
                .any(|(n, (node, status))| n.node != *node || n.status != *status)
            || (self.outcome == AttemptOutcome::Accepted && self.nodes.len() != expected.len())
            || (self.outcome != AttemptOutcome::Accepted
                && self.nodes.iter().any(|n| n.node == Publication))
            || (self.nodes.iter().any(|n| n.node == Interval) != self.actual_dt.is_some())
            || (self.nodes.iter().any(|n| n.node == Projection)
                != self.actual_divergence_max.is_some())
        {
            return Err("workflow execution ordering");
        }
        if self.nodes.iter().any(|n| {
            matches!(
                n.node,
                PressureRequest
                    | PressureCandidate
                    | PressureQualification
                    | NativeFallback
                    | Projection
            )
        }) && self
            .pressure
            .as_ref()
            .is_none_or(|p| p.rhs_sha256.is_none())
        {
            return Err("workflow pressure request association");
        }
        if let Some(p) = &self.pressure {
            let qualification = self
                .nodes
                .iter()
                .find(|n| n.node == PressureQualification)
                .map(|n| n.status);
            let native_selected = self
                .nodes
                .iter()
                .any(|n| n.node == NativeFallback && n.status == Selected);
            let within_limits = |r: PressureReport| {
                [
                    r.true_residual_l2,
                    r.true_residual_max,
                    r.predicted_divergence_max,
                ]
                .iter()
                .all(|v| v.is_finite() && *v >= 0.0)
                    && r.predicted_divergence_max <= self.descriptor.pressure.divergence_limit
                    && p.residual_l2_limit
                        .is_some_and(|limit| r.true_residual_l2 <= limit)
            };
            if p.accepted.is_some_and(|r| !within_limits(r))
                || (qualification == Some(Accepted)
                    && p.candidate.is_none_or(|r| !within_limits(r)))
                || self
                    .actual_divergence_max
                    .is_some_and(|v| v > self.descriptor.actual_divergence_limit)
            {
                return Err("workflow observed qualification limits");
            }
            if (p.candidate.is_some() && qualification.is_none())
                || (qualification == Some(Accepted) && p.candidate.is_none())
                || (p.accepted.is_some() && qualification != Some(Accepted) && !native_selected)
                || (p.pressure_call_ns.is_some() && p.accepted.is_none())
                || (native_selected && p.accepted.is_none())
            {
                return Err("workflow pressure diagnostic stage association");
            }
        }
        if let Some(p) = &self.pressure
            && ((p.support == Some(true) && (p.binding.is_none() || !p.prediction_called))
                || (p.support == Some(false) && (p.prediction_called || p.rejection.is_none()))
                || (self
                    .nodes
                    .iter()
                    .any(|n| n.node == PressureCandidate && n.status == CandidateReturned)
                    && !p.prediction_called)
                || (self.nodes.iter().any(|n| n.node == NativeFallback)
                    && p.selected != self.descriptor.implementation.id()))
        {
            return Err("workflow provider branch association");
        }
        if self.outcome == AttemptOutcome::Accepted {
            let p = self
                .pressure
                .as_ref()
                .ok_or("workflow accepted pressure absent")?;
            let selected = if fallback {
                self.descriptor.implementation.id()
            } else {
                "configured-pressure-context-v1"
            };
            if p.accepted.is_none()
                || p.selected != selected
                || p.rhs_sha256.is_none()
                || p.residual_l2_limit.is_none()
                || self.actual_divergence_max.is_none()
            {
                return Err("workflow accepted pressure association");
            }
            if !fallback {
                let bits = |r: PressureReport| {
                    [
                        r.iterations as u64,
                        r.true_residual_l2.to_bits(),
                        r.true_residual_max.to_bits(),
                        r.predicted_divergence_max.to_bits(),
                    ]
                };
                if p.candidate.map(bits) != p.accepted.map(bits) {
                    return Err("workflow candidate acceptance association");
                }
            }
        }
        Ok(())
    }
}

/// Shared only within one synchronous numerical attempt. No callback creates
/// receipt facts; callbacks can stop the work whose owners emit those facts.
pub(crate) type Capture = RefCell<WorkflowAttempt>;
pub(crate) fn complete(capture: Option<&Capture>, node: NodeKind, status: NodeStatus) {
    if let Some(c) = capture {
        let mut c = c.borrow_mut();
        // The finite composition emits fewer than MAX_NODES summaries; no loop
        // iterations or per-cell data are appended here.
        assert!(c.nodes.len() < MAX_NODES);
        c.nodes.push(NodeReceipt { node, status });
    }
}
pub(crate) fn pressure_completed(capture: Option<&Capture>, report: &ProviderReport) {
    if let Some(c) = capture {
        let mut c = c.borrow_mut();
        let p = c.pressure.get_or_insert_with(|| PressureFacts {
            selected: String::new(),
            binding: None,
            prediction_called: false,
            support: None,
            rejection: None,
            rhs_sha256: None,
            candidate: None,
            accepted: None,
            pressure_call_ns: None,
            residual_l2_limit: None,
        });
        p.selected = report.selected.into();
        p.rejection = report.rejection.as_ref().map(|r| format!("{r:?}"));
        p.candidate = report.candidate;
        p.accepted = Some(report.accepted);
        p.pressure_call_ns = u64::try_from(report.total.as_nanos()).ok();
        drop(c);
    }
}

pub(crate) fn accepted(capture: &Capture) -> Arc<WorkflowAttempt> {
    let mut c = capture.borrow_mut();
    c.outcome = AttemptOutcome::Accepted;
    c.publication = Some(c.attempted_generation);
    c.nodes.push(NodeReceipt {
        node: NodeKind::Publication,
        status: NodeStatus::Accepted,
    });
    Arc::new(c.clone())
}

/// Hash only the actual accepted native fields and identity. Provider request
/// RHS custody is separately observed at the pressure seam. No reconstruction.
pub(crate) fn input_digest(state: crate::LiquidTransportView<'_>) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"rheon-workflow-native-input-v1\0");
    h.update(state.carrier.generation.to_le_bytes());
    h.update(state.carrier.time.to_bits().to_le_bytes());
    for stamp in [state.carrier_stamp, state.liquid.stamp] {
        h.update(stamp.id.to_le_bytes());
        h.update(stamp.version.to_le_bytes());
    }
    h.update(state.liquid.time.to_bits().to_le_bytes());
    h.update(state.liquid.density.to_bits().to_le_bytes());
    for values in [
        state.carrier.x,
        state.carrier.y,
        state.carrier.z,
        state.carrier.tracer,
    ] {
        h.update((values.len() as u64).to_le_bytes());
        for value in values {
            h.update(value.to_bits().to_le_bytes());
        }
    }
    for values in [state.pressure, state.liquid.fraction] {
        h.update((values.len() as u64).to_le_bytes());
        for value in values {
            h.update(value.to_bits().to_le_bytes());
        }
    }
    h.finalize().into()
}

#[cfg(test)]
mod tests;

/// Canonical actual call controls, not a frozen-intent inference.
pub(crate) fn request_digest(inputs: crate::LiquidStepInputs<'_>) -> [u8; 32] {
    let mut h = Sha256::new();
    h.update(b"rheon-workflow-call-input-v1\0");
    h.update(inputs.requested_dt.to_bits().to_le_bytes());
    h.update((inputs.forces.len() as u64).to_le_bytes());
    for force in inputs.forces {
        h.update([match force.units {
            crate::ForceUnits::Acceleration => 0,
            crate::ForceUnits::ForceDensity => 1,
        }]);
        for v in force.value {
            h.update(v.to_bits().to_le_bytes());
        }
        h.update([u8::from(force.region.is_some())]);
        if let Some(r) = force.region {
            for v in r.lower.into_iter().chain(r.upper) {
                h.update(v.to_bits().to_le_bytes());
            }
        }
    }
    h.update([u8::from(inputs.smoke_source.is_some())]);
    if let Some(s) = inputs.smoke_source {
        for v in s
            .lower
            .into_iter()
            .chain(s.upper)
            .chain([s.tracer_rate, s.vertical_acceleration])
        {
            h.update(v.to_bits().to_le_bytes());
        }
    }
    let inlet = inputs.inlet;
    h.update(inlet.stamp().id.to_le_bytes());
    h.update(inlet.stamp().version.to_le_bytes());
    for v in inlet.fractions().into_iter().flatten().chain([
        inputs.volume.max_outward_courant,
        inputs.volume.actual_divergence_limit,
    ]) {
        h.update(v.to_bits().to_le_bytes());
    }
    h.update([u8::from(inputs.source.is_some())]);
    if let Some(source) = inputs.source {
        let (stamp, rates) = source.workflow_parts();
        h.update(stamp.id.to_le_bytes());
        h.update(stamp.version.to_le_bytes());
        h.update((rates.len() as u64).to_le_bytes());
        for v in rates {
            h.update(v.to_bits().to_le_bytes());
        }
    }
    h.finalize().into()
}
pub(crate) fn pressure_requested(capture: Option<&Capture>, rhs: &[f64]) {
    if let Some(trace) = capture {
        let mut h = Sha256::new();
        for v in rhs {
            h.update(v.to_bits().to_le_bytes());
        }
        trace.borrow_mut().pressure = Some(PressureFacts {
            selected: String::new(),
            binding: None,
            prediction_called: false,
            support: None,
            rejection: None,
            rhs_sha256: Some(h.finalize().into()),
            candidate: None,
            accepted: None,
            pressure_call_ns: None,
            residual_l2_limit: None,
        });
        complete(
            Some(trace),
            NodeKind::PressureRequest,
            NodeStatus::Completed,
        );
    }
}
