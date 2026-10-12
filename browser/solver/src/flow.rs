//! Bounded, genuine three-dimensional filled-box flow using the native step.
//!
//! The scalar is appearance tracer, not a liquid surface or conserved volume.
//! No internal obstacle is coupled to this timestep. The first 30 accepted steps
//! inject tracer; the fixed oblique force continues for at most 80 steps.
use crate::{Bits, Grid, Report};
use rheon::{
    Axis, BodyForce, ForceRegion, ForceUnits, GridGeometry, PressureImplementation,
    PressureSettings, Simulation, SimulationConfig, SimulationError, SmokeSource, StepStage,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

pub const FIXTURE: &str = "force-plume-24-v1";
pub const N: usize = 24;
pub const REQUESTED_DT: f64 = 0.02;
pub const MAX_STEPS: usize = 80;
pub const SOURCE_STEPS: usize = 30;
pub const DENSITY: f64 = 1.2;
pub const SOURCE_LOWER: [f64; 3] = [0.30, 0.08, 0.38];
pub const SOURCE_UPPER: [f64; 3] = [0.50, 0.28, 0.58];

#[derive(Debug, Serialize, PartialEq)]
pub struct FlowParameters {
    pub density: f64,
    pub source_lower: [f64; 3],
    pub source_upper: [f64; 3],
    pub tracer_rate: f64,
    pub vertical_acceleration: f64,
    pub force_density: [f64; 3],
    pub force_lower: [f64; 3],
    pub force_upper: [f64; 3],
    pub memory_limit: usize,
    pub relative_residual: f64,
    pub absolute_residual: f64,
    pub predicted_divergence_limit: f64,
    pub pressure_max_iterations: usize,
    pub actual_divergence_limit: f64,
    pub max_courant: f64,
}

fn parameters() -> FlowParameters {
    FlowParameters {
        density: DENSITY,
        source_lower: SOURCE_LOWER,
        source_upper: SOURCE_UPPER,
        tracer_rate: 2.0,
        vertical_acceleration: 0.0,
        force_density: [0.6, 1.2, -0.3],
        force_lower: [0.25, 0.10, 0.30],
        force_upper: [0.60, 0.45, 0.65],
        memory_limit: 64 * 1024 * 1024,
        relative_residual: 1e-9,
        absolute_residual: 1e-12,
        predicted_divergence_limit: 1e-7,
        pressure_max_iterations: 2000,
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FlowSource {
    pub lower: [f64; 3],
    pub upper: [f64; 3],
    pub tracer_rate: f64,
    pub vertical_acceleration: f64,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FlowMetrics {
    pub max_u_v_w: [f64; 3],
    pub component_kinetic_energy: [f64; 3],
    pub actual_divergence_max: f64,
    pub courant: f64,
    pub tracer_min: f64,
    pub tracer_max: f64,
    pub tracer_integral: f64,
    pub tracer_centroid: Option<[f64; 3]>,
    pub tracer_axis_stddev: Option<[f64; 3]>,
    pub tracer_outside_source_fraction: Option<f64>,
    pub tracer_integral_change_since_cutoff: Option<f64>,
    pub tracer_adjacent_slice_rms: [f64; 3],
}

#[derive(Debug, Serialize, PartialEq)]
pub struct FlowSnapshot {
    pub schema: u32,
    pub fixture: &'static str,
    pub method: &'static str,
    pub grid: Grid,
    pub parameters: FlowParameters,
    pub source: FlowSource,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub tracer: Vec<f64>,
    pub bits: Bits,
    pub time: f64,
    pub time_bits: String,
    pub generation: u64,
    pub generation_u64: String,
    pub accepted_steps: usize,
    pub max_steps: usize,
    pub source_steps: usize,
    pub next_source_active: bool,
    pub requested_dt: f64,
    pub last_accepted_dt: Option<f64>,
    pub tracer_integral_at_source_cutoff: Option<f64>,
    pub managed_bytes: usize,
    pub metrics: FlowMetrics,
}

#[derive(Debug, Serialize)]
pub struct FlowOutcome {
    pub status: &'static str,
    pub code: &'static str,
    pub detail: Option<String>,
    pub report: Option<Report>,
    pub state: FlowSnapshot,
}

#[wasm_bindgen]
pub struct FlowSimulation {
    simulation: Simulation,
    accepted_steps: usize,
    last_dt: Option<f64>,
    cutoff_integral: Option<f64>,
}

#[wasm_bindgen]
impl FlowSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(method_id: &str) -> Result<FlowSimulation, String> {
        let method = PressureImplementation::from_id(method_id)
            .ok_or_else(|| format!("unsupported pressure implementation: {method_id}"))?;
        let p = parameters();
        let grid = GridGeometry::new([N as u64; 3], [1.0 / N as f64; 3], [0.0; 3])
            .map_err(|e| format!("fixed flow grid: {e:?}"))?;
        let config = SimulationConfig {
            density: p.density,
            memory_limit: p.memory_limit,
            pressure: PressureSettings {
                relative_residual: p.relative_residual,
                absolute_residual: p.absolute_residual,
                divergence_limit: p.predicted_divergence_limit,
                max_iterations: p.pressure_max_iterations,
            },
            actual_divergence_limit: p.actual_divergence_limit,
            max_courant: p.max_courant,
        };
        Ok(Self {
            simulation: Simulation::with_implementation(grid, config, method)
                .map_err(|e| e.to_string())?,
            accepted_steps: 0,
            last_dt: None,
            cutoff_integral: None,
        })
    }

    pub fn snapshot(&self) -> String {
        crate::json(&self.state())
    }

    /// Exactly one native attempt. Cancellation is a predetermined BeforeCommit
    /// callback, not interruption of a synchronous worker by another message.
    pub fn step(&mut self, cancel_before_commit: bool) -> String {
        crate::json(&self.advance(cancel_before_commit))
    }

    pub fn reset(&mut self) -> Result<String, String> {
        self.reset_state().map(|state| crate::json(&state))
    }
}

impl FlowSimulation {
    pub fn state(&self) -> FlowSnapshot {
        let s = self.simulation.state();
        let values = |v: &[f32]| v.iter().map(|&x| f64::from(x)).collect();
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect();
        FlowSnapshot {
            schema: 1,
            fixture: FIXTURE,
            method: self.simulation.implementation().id(),
            grid: Grid {
                counts: [N; 3],
                spacing: [1.0 / N as f64; 3],
                origin: [0.0; 3],
            },
            parameters: parameters(),
            source: FlowSource {
                lower: SOURCE_LOWER,
                upper: SOURCE_UPPER,
                tracer_rate: 2.0,
                vertical_acceleration: 0.0,
            },
            x: values(s.x),
            y: values(s.y),
            z: values(s.z),
            tracer: values(s.tracer),
            bits: Bits {
                x: bits(s.x),
                y: bits(s.y),
                z: bits(s.z),
                tracer: bits(s.tracer),
            },
            time: s.time,
            time_bits: format!("{:016x}", s.time.to_bits()),
            generation: s.generation,
            generation_u64: s.generation.to_string(),
            accepted_steps: self.accepted_steps,
            max_steps: MAX_STEPS,
            source_steps: SOURCE_STEPS,
            next_source_active: self.accepted_steps < SOURCE_STEPS,
            requested_dt: REQUESTED_DT,
            last_accepted_dt: self.last_dt,
            tracer_integral_at_source_cutoff: self.cutoff_integral,
            managed_bytes: self.simulation.allocated_bytes(),
            metrics: self.measure(),
        }
    }

    fn rejected(&self, code: &'static str, detail: String) -> FlowOutcome {
        FlowOutcome {
            status: "rejected",
            code,
            detail: Some(detail),
            report: None,
            state: self.state(),
        }
    }

    pub fn advance(&mut self, cancel_before_commit: bool) -> FlowOutcome {
        if self.accepted_steps >= MAX_STEPS {
            return self.rejected(
                "step_limit",
                "the bounded 80-step run is complete; reset to start another run".into(),
            );
        }
        let p = parameters();
        let source = (self.accepted_steps < SOURCE_STEPS).then_some(SmokeSource {
            lower: p.source_lower,
            upper: p.source_upper,
            tracer_rate: p.tracer_rate,
            vertical_acceleration: p.vertical_acceleration,
        });
        let force = BodyForce {
            value: p.force_density,
            units: ForceUnits::ForceDensity,
            region: Some(ForceRegion {
                lower: p.force_lower,
                upper: p.force_upper,
            }),
        };
        match self
            .simulation
            .step_with_forces(REQUESTED_DT, source, &[force], |stage| {
                cancel_before_commit && stage == StepStage::BeforeCommit
            }) {
            Ok(report) => {
                self.accepted_steps += 1;
                self.last_dt = Some(report.step.dt);
                if self.accepted_steps == SOURCE_STEPS {
                    self.cutoff_integral = Some(report.step.tracer_integral);
                }
                FlowOutcome {
                    status: "accepted",
                    code: "accepted",
                    detail: None,
                    report: Some(report.into()),
                    state: self.state(),
                }
            }
            Err(error) => {
                let code = if matches!(
                    error,
                    SimulationError::Cancelled {
                        stage: StepStage::BeforeCommit
                    }
                ) {
                    "cancelled_before_commit"
                } else {
                    "solver_error"
                };
                self.rejected(code, error.to_string())
            }
        }
    }

    pub fn reset_state(&mut self) -> Result<FlowSnapshot, String> {
        self.simulation.reset().map_err(|e| e.to_string())?;
        self.accepted_steps = 0;
        self.last_dt = None;
        self.cutoff_integral = None;
        Ok(self.state())
    }

    /// Read-only measurements of accepted arrays. These are useful UI values;
    /// qualification recomputes the equations outside this implementation.
    fn measure(&self) -> FlowMetrics {
        let s = self.simulation.state();
        let g = self.simulation.grid();
        let fields = [s.x, s.y, s.z];
        let max_u_v_w = fields.map(|v| v.iter().fold(0.0_f64, |a, &b| a.max(f64::from(b).abs())));
        let component_kinetic_energy = fields.map(|v| {
            0.5 * DENSITY * g.cell_volume() * v.iter().map(|&x| f64::from(x).powi(2)).sum::<f64>()
        });
        let mut div = 0.0_f64;
        let mut weight = 0.0;
        let mut first = [0.0; 3];
        let mut second = [0.0; 3];
        let mut outside = 0.0;
        let mut adjacent = [0.0; 3];
        for k in 0..N {
            for j in 0..N {
                for i in 0..N {
                    let c = [i, j, k];
                    let mut divergence = 0.0;
                    for (d, axis) in [Axis::X, Axis::Y, Axis::Z].into_iter().enumerate() {
                        let mut hi = c;
                        hi[d] += 1;
                        divergence +=
                            (f64::from(fields[d][g.face_index(axis, hi).expect("bounded face")])
                                - f64::from(
                                    fields[d][g.face_index(axis, c).expect("bounded face")],
                                ))
                                / g.spacing()[d];
                    }
                    div = div.max(divergence.abs());
                    let q = f64::from(s.tracer[g.cell_index(c).expect("bounded cell")]);
                    let position = g.cell_position(c).expect("bounded cell");
                    weight += q;
                    if !(0..3)
                        .all(|d| position[d] >= SOURCE_LOWER[d] && position[d] < SOURCE_UPPER[d])
                    {
                        outside += q;
                    }
                    for d in 0..3 {
                        first[d] += q * position[d];
                        second[d] += q * position[d] * position[d];
                        if c[d] + 1 < N {
                            let mut hi = c;
                            hi[d] += 1;
                            let other =
                                f64::from(s.tracer[g.cell_index(hi).expect("bounded cell")]);
                            adjacent[d] += (other - q).powi(2);
                        }
                    }
                }
            }
        }
        let integral = weight * g.cell_volume();
        let centroid = (weight > 0.0).then(|| first.map(|v| v / weight));
        let deviation = centroid.map(|mean| {
            std::array::from_fn(|d| (second[d] / weight - mean[d].powi(2)).max(0.0).sqrt())
        });
        FlowMetrics {
            max_u_v_w,
            component_kinetic_energy,
            actual_divergence_max: div,
            courant: self.last_dt.unwrap_or(0.0) * max_u_v_w.iter().sum::<f64>() * N as f64,
            tracer_min: s
                .tracer
                .iter()
                .fold(f64::INFINITY, |a, &b| a.min(f64::from(b))),
            tracer_max: s
                .tracer
                .iter()
                .fold(f64::NEG_INFINITY, |a, &b| a.max(f64::from(b))),
            tracer_integral: integral,
            tracer_centroid: centroid,
            tracer_axis_stddev: deviation,
            tracer_outside_source_fraction: (weight > 0.0).then(|| outside / weight),
            tracer_integral_change_since_cutoff: self.cutoff_integral.map(|old| integral - old),
            tracer_adjacent_slice_rms: adjacent.map(|v| (v / ((N - 1) * N * N) as f64).sqrt()),
        }
    }
}
