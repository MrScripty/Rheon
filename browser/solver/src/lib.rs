//! A bounded, fixed-input adapter around the unmodified native timestep.
//!
//! This is the four-edge force fixture in `tests/forces_contract.rs`. It is
//! deliberately limited to one accepted step per reset: it does not claim a
//! validated long-time trajectory or a general browser simulation API.

use rheon::{
    BodyForce, ForceRegion, ForceUnits, ForcedStepReport, GridGeometry, PressureImplementation,
    PressureSettings, Simulation, SimulationConfig, SimulationError, StepStage,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

pub const FIXTURE: &str = "force-cycle-2x2x1-v1";
pub const DT: f64 = 0.25;

#[derive(Debug, Serialize, PartialEq)]
pub struct Grid {
    pub counts: [usize; 3],
    pub spacing: [f64; 3],
    pub origin: [f64; 3],
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Bits {
    pub x: Vec<u32>,
    pub y: Vec<u32>,
    pub z: Vec<u32>,
    pub tracer: Vec<u32>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Snapshot {
    pub schema: u32,
    pub fixture: &'static str,
    pub method: &'static str,
    pub grid: Grid,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub z: Vec<f64>,
    pub tracer: Vec<f64>,
    pub bits: Bits,
    pub time: f64,
    pub time_bits: String,
    pub generation: u64,
    /// Exact even if a consumer stores JSON numbers in a double.
    pub generation_u64: String,
    pub accepted_since_reset: bool,
}

#[derive(Debug, Serialize)]
pub struct Pressure {
    pub iterations: usize,
    pub true_residual_l2: f64,
    pub true_residual_max: f64,
    pub predicted_divergence_max: f64,
}

#[derive(Debug, Serialize)]
pub struct Forces {
    pub kinetic_energy_before: f64,
    pub kinetic_energy_after: f64,
    pub applied_work: f64,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub dt: f64,
    pub time: f64,
    pub generation: u64,
    pub pressure: Pressure,
    pub actual_divergence_max: f64,
    pub courant: f64,
    pub tracer_integral: f64,
    pub kinetic_energy: f64,
    pub forces: Option<Forces>,
}

impl From<ForcedStepReport> for Report {
    fn from(r: ForcedStepReport) -> Self {
        let s = r.step;
        Self {
            dt: s.dt,
            time: s.time,
            generation: s.generation,
            pressure: Pressure {
                iterations: s.pressure.iterations,
                true_residual_l2: s.pressure.true_residual_l2,
                true_residual_max: s.pressure.true_residual_max,
                predicted_divergence_max: s.pressure.predicted_divergence_max,
            },
            actual_divergence_max: s.actual_divergence_max,
            courant: s.courant,
            tracer_integral: s.tracer_integral,
            kinetic_energy: s.kinetic_energy,
            forces: r.forces.map(|f| Forces {
                kinetic_energy_before: f.kinetic_energy_before,
                kinetic_energy_after: f.kinetic_energy_after,
                applied_work: f.applied_work,
            }),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Outcome {
    pub status: &'static str,
    pub code: &'static str,
    pub detail: Option<String>,
    pub report: Option<Report>,
    pub state: Snapshot,
}

/// Owns native accepted and scratch buffers; JavaScript never receives mutable
/// native memory or a pressure-settings mutation interface.
#[wasm_bindgen]
pub struct CycleSimulation {
    simulation: Simulation,
    accepted_since_reset: bool,
}

#[wasm_bindgen]
impl CycleSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(method_id: &str) -> Result<CycleSimulation, String> {
        let method = PressureImplementation::from_id(method_id)
            .ok_or_else(|| format!("unsupported pressure implementation: {method_id}"))?;
        let grid = GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3])
            .map_err(|e| format!("fixed fixture grid: {e:?}"))?;
        let config = SimulationConfig {
            density: 1.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-6,
            max_courant: 1.0,
        };
        Ok(Self {
            simulation: Simulation::with_implementation(grid, config, method)
                .map_err(|e| e.to_string())?,
            accepted_since_reset: false,
        })
    }

    pub fn snapshot(&self) -> String {
        json(&self.state())
    }

    /// A predetermined native callback rejects at BeforeCommit. This is not an
    /// asynchronous interruption mechanism for a running worker.
    pub fn step(&mut self, dt: f64, cancel_before_commit: bool) -> String {
        json(&self.advance(dt, cancel_before_commit))
    }

    pub fn reset(&mut self) -> Result<String, String> {
        self.reset_state().map(|s| json(&s))
    }
}

impl CycleSimulation {
    pub fn state(&self) -> Snapshot {
        let s = self.simulation.state();
        let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect();
        // JSON numbers are read as doubles by the browser. Promote the stored
        // f32 exactly, rather than exporting an f32 shortest decimal that may
        // parse to a neighboring double instead of the stored binary value.
        let values = |v: &[f32]| v.iter().map(|&x| f64::from(x)).collect();
        Snapshot {
            schema: 1,
            fixture: FIXTURE,
            method: self.simulation.implementation().id(),
            grid: Grid {
                counts: [2, 2, 1],
                spacing: [1.0; 3],
                origin: [0.0; 3],
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
            accepted_since_reset: self.accepted_since_reset,
        }
    }

    fn rejected(&self, code: &'static str, detail: String) -> Outcome {
        Outcome {
            status: "rejected",
            code,
            detail: Some(detail),
            report: None,
            state: self.state(),
        }
    }

    pub fn advance(&mut self, dt: f64, cancel_before_commit: bool) -> Outcome {
        if dt != DT {
            return self.rejected(
                "invalid_time_step",
                "this fixed fixture accepts only dt = 0.25".into(),
            );
        }
        if self.accepted_since_reset && !cancel_before_commit {
            return self.rejected(
                "reset_required",
                "reset before another accepted timestep".into(),
            );
        }
        let force = BodyForce {
            value: [2.0, 0.0, 0.0],
            units: ForceUnits::Acceleration,
            region: Some(ForceRegion {
                lower: [0.0; 3],
                upper: [2.0, 1.0, 1.0],
            }),
        };
        match self
            .simulation
            .step_with_forces(dt, None, &[force], |stage| {
                cancel_before_commit && stage == StepStage::BeforeCommit
            }) {
            Ok(report) => {
                self.accepted_since_reset = true;
                Outcome {
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

    pub fn reset_state(&mut self) -> Result<Snapshot, String> {
        self.simulation.reset().map_err(|e| e.to_string())?;
        self.accepted_since_reset = false;
        Ok(self.state())
    }
}

fn json(value: &impl Serialize) -> String {
    serde_json::to_string(value).expect("fixed solver state and reports contain finite values")
}

#[derive(Serialize)]
pub struct ReferenceRun {
    pub method: &'static str,
    pub baseline: Snapshot,
    pub invalid_dt: Outcome,
    pub cancelled_rest: Outcome,
    pub accepted: Outcome,
    pub cancelled_nonzero: Outcome,
    pub second_accept: Outcome,
    pub reset: Snapshot,
    pub after_reset: Outcome,
}

#[derive(Serialize)]
pub struct Reference {
    pub schema: u32,
    pub fixture: &'static str,
    pub dt: f64,
    pub runs: Vec<ReferenceRun>,
}

/// The host exporter executes exactly the same wrapper as the WASM facade.
pub fn reference() -> Result<Reference, String> {
    let mut runs = Vec::new();
    for method in PressureImplementation::ALL {
        let mut sim = CycleSimulation::new(method.id())?;
        runs.push(ReferenceRun {
            method: method.id(),
            baseline: sim.state(),
            invalid_dt: sim.advance(0.0, false),
            cancelled_rest: sim.advance(DT, true),
            accepted: sim.advance(DT, false),
            cancelled_nonzero: sim.advance(DT, true),
            second_accept: sim.advance(DT, false),
            reset: sim.reset_state()?,
            after_reset: sim.advance(DT, false),
        });
    }
    Ok(Reference {
        schema: 1,
        fixture: FIXTURE,
        dt: DT,
        runs,
    })
}
