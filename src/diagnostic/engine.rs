//! Independent raw-AI diagnostic and strictly qualified native reference tracks.
//! These experimental frames are never published as accepted Simulation snapshots.
use super::PredictionFailure;
use super::scene::{MU, RHO, Scene};
use crate::recording::diagnostic::{
    BindingIdentity, DiagnosticAttempt, DiagnosticMode, DiagnosticRequest, DiagnosticRun,
    HistoryTransition, NativeReference, NoCandidateReason, ObservedHistory, PredictionOutcome,
    PressureSample, QualificationObservation, QualificationPolicy, SeedOrigin,
};
use crate::solver_provider::context::{
    PressureContext, PressureContextInput, PressureContextMode, PressureHistory,
    PressureHistoryStatus,
};
use crate::{
    Axis, GridGeometry, PressureImplementation, PressureOperator, PressureSettings,
    PressureWorkspace, ViscosityWorkspace, advect_tracer, advect_velocity,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicU64;
static NEXT_ENGINE: AtomicU64 = AtomicU64::new(1);
use std::sync::atomic::{AtomicBool, Ordering};

const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
pub const PRESSURE_ITERATION_LIMIT: usize = 3000;
type Velocity = [Vec<f32>; 3];
fn refs(v: &Velocity) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn muts(v: &mut Velocity) -> [&mut [f32]; 3] {
    let [a, b, c] = v;
    [a, b, c]
}
fn error(e: impl std::fmt::Display) -> String {
    e.to_string()
}
#[derive(Clone, Debug, Serialize)]
pub struct Quality {
    pub relative_residual: Option<f64>,
    pub residual_l2: f64,
    pub predicted_divergence: f64,
    pub actual_divergence: f64,
    pub courant: f64,
    pub net_flux: f64,
    pub qualified: bool,
    pub hard_stop: bool,
    #[serde(skip_serializing)]
    pub flux_limit: f64,
    #[serde(skip_serializing)]
    pub walls_closed: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionPolicy {
    #[default]
    LegacyRawContinuation,
    TeacherStage,
    IndependentStopOnReject,
}
impl ExecutionPolicy {
    /// Upper bound before early termination; independent AI requests also solve
    /// a same-request native reference after the shared warm-up.
    #[allow(
        dead_code,
        reason = "Standalone CLI preflight; module also used by desktop"
    )]
    pub fn native_pressure_solve_budget(self, scene: &Scene) -> Result<usize, String> {
        scene.validate()?;
        let additional = match self {
            Self::IndependentStopOnReject => scene.steps - scene.warmup_steps,
            Self::LegacyRawContinuation | Self::TeacherStage => 0,
        };
        scene
            .steps
            .checked_add(additional)
            .ok_or_else(|| "Native pressure solve budget overflow".into())
    }
    pub fn diagnostic_mode(self) -> Option<DiagnosticMode> {
        match self {
            Self::LegacyRawContinuation => None,
            Self::TeacherStage => Some(DiagnosticMode::TeacherStage),
            Self::IndependentStopOnReject => Some(DiagnosticMode::IndependentStopOnReject),
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::LegacyRawContinuation => "Legacy raw continuation",
            Self::TeacherStage => "Teacher stage: native history",
            Self::IndependentStopOnReject => "Independent: stop on first rejection",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdvanceStop {
    Rejected,
    Cancelled,
    ProviderFailure(NoCandidateReason),
}
pub struct AdvanceResult {
    pub frame: Option<Frame>,
    pub attempt: Option<DiagnosticAttempt>,
    pub stop: Option<AdvanceStop>,
}
impl Quality {
    fn observation(&self, relative_limit: f64) -> Option<QualificationObservation> {
        if [
            self.residual_l2,
            self.predicted_divergence,
            self.actual_divergence,
            self.courant,
            self.net_flux,
            self.flux_limit,
        ]
        .iter()
        .any(|v| !v.is_finite())
            || self.relative_residual.is_some_and(|v| !v.is_finite())
        {
            return None;
        }
        Some(QualificationObservation {
            policy: QualificationPolicy::HostPressureGateV1,
            residual_l2: self.residual_l2,
            relative_residual: self.relative_residual,
            predicted_divergence: self.predicted_divergence,
            actual_divergence: self.actual_divergence,
            courant: self.courant,
            net_flux: self.net_flux,
            qualified: self.qualified,
            hard_stop: self.hard_stop,
            relative_limit,
            absolute_limit: 1e-12,
            predicted_limit: 1e-7,
            actual_limit: 1e-5,
            courant_limit: 0.5,
            flux_limit: self.flux_limit,
            walls_closed: self.walls_closed,
        })
    }
}
fn invalid_candidate(_pressure: &[f64], detail: &str) -> PredictionOutcome {
    PredictionOutcome::NoCandidate {
        reason: NoCandidateReason::InvalidOutput,
        detail: detail
            .chars()
            .filter(|c| !c.is_control())
            .take(1000)
            .collect(),
        // The engine receives canonical values, not the original protocol line.
        invalid_payload_sha256: None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FramePhase {
    Initial,
    NativeWarmup,
    AiPrediction,
}
#[derive(Clone, Serialize)]
pub struct Frame {
    pub phase: FramePhase,
    pub step: usize,
    pub time_s: f64,
    pub reference_pressure: Vec<f64>,
    pub ai_pressure: Vec<f64>,
    pub reference_tracer: Vec<f32>,
    pub ai_tracer: Vec<f32>,
    pub reference_quality: Option<Quality>,
    pub ai_quality: Option<Quality>,
    pub reference_seconds: f64,
    pub ai_seconds: f64,
}
#[derive(Clone)]
struct Track {
    velocity: Velocity,
    tracer: Vec<f32>,
    pressure: Vec<f64>,
}
struct Prepared {
    velocity: Velocity,
    tracer: Vec<f32>,
    rhs: Vec<f64>,
}
pub struct DiagnosticSession {
    scene: Scene,
    grid: GridGeometry,
    reference: Track,
    ai: Track,
    solver: PressureWorkspace,
    viscosity: ViscosityWorkspace,
    step: usize,
    halted: bool,
    run_id: String,
    previous_ai_qualified: bool,
    policy: ExecutionPolicy,
    clock: f64,
    attempts: usize,
}
pub struct Input<'a> {
    pub context: PressureContext<'a>,
    pub initial_guess: &'a [f64],
    pub residual: &'a [f64],
    pub viscosity: f64,
}
impl DiagnosticSession {
    /// Validated immutable scene used by this session.
    pub fn scene(&self) -> &Scene {
        &self.scene
    }
    pub fn new(scene: Scene) -> Result<Self, String> {
        Self::with_policy(
            scene,
            ExecutionPolicy::LegacyRawContinuation,
            format!(
                "engine:{}:{}",
                std::process::id(),
                NEXT_ENGINE.fetch_add(1, Ordering::Relaxed)
            ),
        )
    }
    pub fn with_policy(
        scene: Scene,
        policy: ExecutionPolicy,
        run_id: String,
    ) -> Result<Self, String> {
        scene.validate()?;
        if run_id.is_empty() || run_id.len() > 4096 || run_id.chars().any(char::is_control) {
            return Err("Invalid diagnostic run identity".into());
        }
        let grid = GridGeometry::new(scene.counts.map(|v| v as u64), [scene.h(); 3], [0.0; 3])
            .map_err(error)?;
        let mut tracer = vec![0.0; grid.cell_len()];
        let extents = scene.extents();
        let width = 0.08 * extents.into_iter().fold(f64::INFINITY, f64::min);
        for k in 0..scene.counts[2] {
            for j in 0..scene.counts[1] {
                for i in 0..scene.counts[0] {
                    let pos = grid.cell_position([i, j, k]).ok_or("Cell position")?;
                    let r2 = (0..3)
                        .map(|d| (pos[d] - 0.5 * extents[d]).powi(2))
                        .sum::<f64>();
                    tracer[i + scene.counts[0] * (j + scene.counts[1] * k)] =
                        (0.25 * (-r2 / (2.0 * width * width)).exp()) as f32;
                }
            }
        }
        let create = || Track {
            velocity: std::array::from_fn(|d| vec![0.0; grid.face_len(AXES[d])]),
            tracer: tracer.clone(),
            pressure: vec![0.0; grid.cell_len()],
        };
        let reference = create();
        let ai = create();
        let method = if scene.sgs {
            PressureImplementation::SymmetricGaussSeidelPcgV1
        } else {
            PressureImplementation::JacobiPcgV1
        };
        let solver = PressureWorkspace::with_implementation(&grid, 2 * 1024 * 1024 * 1024, method)
            .map_err(error)?;
        let viscosity = ViscosityWorkspace::new(grid.clone(), 2 * 1024 * 1024 * 1024).map_err(error)?;
        Ok(Self {
            scene,
            grid,
            reference,
            ai,
            solver,
            viscosity,
            step: 0,
            halted: false,
            run_id,
            previous_ai_qualified: true,
            policy,
            clock: 0.0,
            attempts: 0,
        })
    }
    /// Observations of committed independent tracks; no simulation mutation.
    pub fn trajectory_drift(&self) -> (f64, f64, f64, f64) {
        fn measure<'a>(pairs: impl Iterator<Item = (&'a f32, &'a f32)>) -> (f64, f64) {
            let mut error = 0.0_f64;
            let mut signal = 0.0_f64;
            let mut maximum = 0.0_f64;
            for (candidate, reference) in pairs {
                let reference = f64::from(*reference);
                let difference = f64::from(*candidate) - reference;
                error += difference * difference;
                signal += reference * reference;
                maximum = maximum.max(difference.abs());
            }
            (error.sqrt() / signal.sqrt().max(1e-30), maximum)
        }
        let velocity = measure(self.ai.velocity.iter().flatten().zip(self.reference.velocity.iter().flatten()));
        let tracer = measure(self.ai.tracer.iter().zip(&self.reference.tracer));
        (velocity.0, tracer.0, velocity.1, tracer.1)
    }
    pub fn initial(&self) -> Frame {
        self.frame(None, None, 0.0, 0.0)
    }
    fn frame(
        &self,
        reference_quality: Option<Quality>,
        ai_quality: Option<Quality>,
        reference_seconds: f64,
        ai_seconds: f64,
    ) -> Frame {
        Frame {
            phase: if self.step == 0 {
                FramePhase::Initial
            } else if self.step <= self.scene.warmup_steps {
                FramePhase::NativeWarmup
            } else {
                FramePhase::AiPrediction
            },
            step: self.step,
            time_s: self.clock,
            reference_pressure: self.reference.pressure.clone(),
            ai_pressure: self.ai.pressure.clone(),
            reference_tracer: self.reference.tracer.clone(),
            ai_tracer: self.ai.tracer.clone(),
            reference_quality,
            ai_quality,
            reference_seconds,
            ai_seconds,
        }
    }
    pub fn diagnostic_run(
        &self,
        binding: BindingIdentity,
        source_revision: String,
    ) -> Result<DiagnosticRun, String> {
        let mode = self
            .policy
            .diagnostic_mode()
            .ok_or("Legacy raw frames have no typed diagnostic archive")?;
        let teacher = self.policy == ExecutionPolicy::TeacherStage;
        let track = if teacher { &self.reference } else { &self.ai };
        let seed = (self.step > 0).then(|| ObservedHistory {
            generation: self.step as u64,
            interval_end: self.clock,
            status: PressureHistoryStatus::Diagnostic {
                qualified: teacher || self.previous_ai_qualified,
            },
            pressure: track.pressure.clone(),
        });
        let seed_origin = seed.as_ref().map(|_| {
            if teacher || self.step <= self.scene.warmup_steps {
                SeedOrigin::QualifiedNativeDiagnostic
            } else {
                SeedOrigin::QualifiedDiagnostic
            }
        });
        Ok(DiagnosticRun {
            run_id: self.run_id.clone(), source_revision, grid: self.grid.clone(), density: RHO,
            viscosity: MU, dt: super::scheduled_forcing::dt(), mode, native_prefix_steps: self.step.min(self.scene.warmup_steps),
            binding, provenance: "Existing sealed-water diagnostic DiagnosticSession; host-observed native/AI histories, not accepted Session publications".into(),
            seed, seed_origin,
        })
    }
    pub fn advance(
        &mut self,
        cancel: &AtomicBool,
        mut predict: impl FnMut(Input<'_>) -> Result<Vec<f64>, String>,
    ) -> Result<Frame, String> {
        if self.policy != ExecutionPolicy::LegacyRawContinuation {
            return Err("Typed policies require attempt-preserving advance_typed".into());
        }
        let result = self.advance_typed(cancel, |input| {
            predict(input).map_err(|detail| PredictionFailure {
                reason: NoCandidateReason::RuntimeFailure,
                detail,
                dispatched: true,
                invalid_payload_sha256: None,
            })
        })?;
        result
            .frame
            .ok_or("Legacy advance produced no frame".into())
    }
    pub fn advance_typed(
        &mut self,
        cancel: &AtomicBool,
        mut predict: impl FnMut(Input<'_>) -> Result<Vec<f64>, PredictionFailure>,
    ) -> Result<AdvanceResult, String> {
        if self.halted || self.step >= self.scene.steps {
            return Err("Paired run already stopped".into());
        }
        let legacy = self.policy == ExecutionPolicy::LegacyRawContinuation;
        let teacher = self.policy == ExecutionPolicy::TeacherStage;
        let next_time = (self.step + 1) as f64 * super::scheduled_forcing::dt();
        let force_time = self.step as f64 * super::scheduled_forcing::dt();
        let start = std::time::Instant::now();
        let reference_prepared = prepare(
            &self.scene,
            &self.grid,
            &self.reference,
            &mut self.viscosity,
            force_time,
            cancel,
        )?;
        let (reference, qref) =
            native_solve(&self.grid, &reference_prepared, &mut self.solver, cancel)?;
        if !qref.qualified {
            return Err(format!("Native reference rejected: {qref:?}"));
        }
        let reference_seconds = start.elapsed().as_secs_f64();
        if self.step < self.scene.warmup_steps {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            self.ai = reference.clone();
            self.reference = reference;
            self.step += 1;
            self.clock = next_time;
            return Ok(AdvanceResult {
                frame: Some(self.frame(Some(qref), None, reference_seconds, 0.0)),
                attempt: None,
                stop: None,
            });
        }
        let start = std::time::Instant::now();
        let prepared = if teacher {
            reference_prepared
        } else {
            prepare(
                &self.scene,
                &self.grid,
                &self.ai,
                &mut self.viscosity,
                force_time,
                cancel,
            )?
        };
        let prior_track = if teacher { &self.reference } else { &self.ai };
        let context = PressureContext::with_interval_end(
            PressureContextInput {
                grid: &self.grid,
                density: RHO,
                dt: super::scheduled_forcing::dt(),
                interval_start: self.clock,
                rhs: &prepared.rhs,
                prior: (self.step > 0).then_some(PressureHistory {
                    values: &prior_track.pressure,
                    grid: &self.grid,
                    density: RHO,
                    run_id: &self.run_id,
                    generation: self.step as u64,
                    interval_end: self.clock,
                    status: PressureHistoryStatus::Diagnostic {
                        qualified: teacher || self.previous_ai_qualified,
                    },
                }),
                run_id: &self.run_id,
                generation: (self.step + 1) as u64,
                mode: PressureContextMode::DiagnosticCandidate,
            },
            next_time,
        )
        .map_err(error)?;
        let mut residual = vec![0.0; self.grid.cell_len()];
        context.initial_residual(&mut residual).map_err(error)?;
        let request = if legacy {
            None
        } else {
            Some(DiagnosticRequest::capture(&context, MU, 16 * 1024 * 1024).map_err(error)?)
        };
        // This reference solves exactly the request being captured. The paired
        // independent native frame is deliberately not substituted for it.
        let native_reference = if let Some(request) = &request {
            let solved = if teacher {
                Some((reference.clone(), qref.clone()))
            } else {
                native_solve(&self.grid, &prepared, &mut self.solver, cancel).ok()
            };
            solved.and_then(|(track, quality)| {
                quality
                    .observation(1e-8)
                    .map(|observation| NativeReference {
                        request_digest: request.digest(),
                        sample: PressureSample {
                            pressure: track.pressure,
                            observation,
                        },
                        history_status: PressureHistoryStatus::Diagnostic {
                            qualified: quality.qualified,
                        },
                    })
            })
        } else {
            None
        };
        let prediction = predict(Input {
            context,
            initial_guess: &prior_track.pressure,
            residual: &residual,
            viscosity: MU,
        });
        let (candidate, outcome, dispatched) = match prediction {
            Ok(pressure) => match finish(&self.grid, &prepared, pressure.clone(), 1e-6) {
                Ok((track, quality)) if legacy => {
                    if cancel.load(Ordering::Relaxed) {
                        return Err("Cancelled".into());
                    }
                    self.halted = quality.hard_stop;
                    self.previous_ai_qualified = quality.qualified;
                    self.reference = reference;
                    self.ai = track;
                    self.step += 1;
                    self.clock = next_time;
                    return Ok(AdvanceResult {
                        frame: Some(self.frame(
                            Some(qref),
                            Some(quality),
                            reference_seconds,
                            start.elapsed().as_secs_f64(),
                        )),
                        attempt: None,
                        stop: None,
                    });
                }
                Ok((track, quality)) => {
                    if let Some(observation) = quality.observation(1e-6) {
                        (
                            Some((track, quality)),
                            PredictionOutcome::Candidate(PressureSample {
                                pressure,
                                observation,
                            }),
                            true,
                        )
                    } else {
                        (
                            None,
                            invalid_candidate(&pressure, "Nonfinite host qualification arithmetic"),
                            true,
                        )
                    }
                }
                Err(detail) if legacy => return Err(detail),
                Err(detail) => (None, invalid_candidate(&pressure, &detail), true),
            },
            Err(failure) if legacy => return Err(failure.detail),
            Err(failure) => (
                None,
                PredictionOutcome::NoCandidate {
                    reason: failure.reason,
                    detail: failure.detail,
                    invalid_payload_sha256: failure.invalid_payload_sha256,
                },
                failure.dispatched,
            ),
        };
        let mut transition = HistoryTransition::NotAdvanced;
        let mut frame = None;
        let stop = if cancel.load(Ordering::Relaxed) {
            if matches!(outcome, PredictionOutcome::Candidate(_)) {
                transition = HistoryTransition::CancelledBeforeAdvance;
            }
            Some(AdvanceStop::Cancelled)
        } else if let Some((track, quality)) = candidate {
            if teacher {
                transition = HistoryTransition::NativeTeacherAdvanced;
                self.reference = reference;
                self.ai = self.reference.clone();
                self.step += 1;
                self.clock = next_time;
                let mut observed = self.frame(
                    Some(qref),
                    Some(quality),
                    reference_seconds,
                    start.elapsed().as_secs_f64(),
                );
                observed.ai_pressure = track.pressure;
                observed.ai_tracer = track.tracer;
                frame = Some(observed);
                None
            } else if quality.qualified {
                transition = HistoryTransition::DiagnosticAdvanced;
                self.previous_ai_qualified = true;
                self.reference = reference;
                self.ai = track;
                self.step += 1;
                self.clock = next_time;
                frame = Some(self.frame(
                    Some(qref),
                    Some(quality),
                    reference_seconds,
                    start.elapsed().as_secs_f64(),
                ));
                None
            } else {
                Some(AdvanceStop::Rejected)
            }
        } else {
            Some(AdvanceStop::ProviderFailure(match &outcome {
                PredictionOutcome::NoCandidate { reason, .. } => *reason,
                PredictionOutcome::Candidate(_) => unreachable!("candidate has a host observation"),
            }))
        };
        let attempt = DiagnosticAttempt {
            attempt_index: self.attempts,
            request: request.ok_or("Typed request capture missing")?,
            dispatched,
            outcome,
            native_reference,
            transition,
        };
        self.attempts += 1;
        self.halted = stop.is_some();
        Ok(AdvanceResult {
            frame,
            attempt: Some(attempt),
            stop,
        })
    }
}
fn native_solve(
    grid: &GridGeometry,
    prepared: &Prepared,
    solver: &mut PressureWorkspace,
    cancel: &AtomicBool,
) -> Result<(Track, Quality), String> {
    let operator = PressureOperator::new(grid, RHO).map_err(error)?;
    solver
        .solve_rhs(
            &operator,
            &prepared.rhs,
            super::scheduled_forcing::dt(),
            PressureSettings {
                relative_residual: 1e-8,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: PRESSURE_ITERATION_LIMIT,
            },
            || cancel.load(Ordering::Relaxed),
        )
        .map_err(error)?;
    finish(grid, prepared, solver.pressure().to_vec(), 1e-8)
}

/// Seconds spent in the last successful preparation on this thread.
#[derive(Clone, Copy, Default, serde::Serialize)]
pub struct PreparationProfile {
    pub allocation: f64,
    pub advect_velocity: f64,
    pub advect_tracer: f64,
    pub forcing: f64,
    pub viscosity: f64,
    pub rhs: f64,
}
thread_local! {
    static LAST_PREPARATION: std::cell::Cell<PreparationProfile> = std::cell::Cell::new(PreparationProfile::default());
}
pub fn last_preparation_profile() -> PreparationProfile {
    LAST_PREPARATION.with(std::cell::Cell::get)
}
fn prepare(
    scene: &Scene,
    grid: &GridGeometry,
    track: &Track,
    viscosity: &mut ViscosityWorkspace,
    time: f64,
    cancel: &AtomicBool,
) -> Result<Prepared, String> {
    let tick = std::time::Instant::now();
    let mut pre = std::array::from_fn(|d| vec![0.0; grid.face_len(AXES[d])]);
    let mut velocity = pre.clone();
    let mut tracer = vec![0.0; grid.cell_len()];
    let allocation = tick.elapsed().as_secs_f64();
    let tick = std::time::Instant::now();
    advect_velocity(grid, refs(&track.velocity), super::scheduled_forcing::dt(), muts(&mut pre), || {
        cancel.load(Ordering::Relaxed)
    })
    .map_err(error)?;
    let advect_velocity = tick.elapsed().as_secs_f64();
    let tick = std::time::Instant::now();
    advect_tracer(
        grid,
        &track.tracer,
        refs(&track.velocity),
        super::scheduled_forcing::dt(),
        &mut tracer,
        || cancel.load(Ordering::Relaxed),
    )
    .map_err(error)?;
    let advect_tracer = tick.elapsed().as_secs_f64();
    let tick = std::time::Instant::now();
    force(scene, grid, &mut pre, time, cancel)?;
    let forcing = tick.elapsed().as_secs_f64();
    let tick = std::time::Instant::now();
    viscosity
        .update(refs(&pre), RHO, MU, super::scheduled_forcing::dt(), muts(&mut velocity), |_| {
            cancel.load(Ordering::Relaxed)
        })
        .map_err(error)?;
    let viscosity = tick.elapsed().as_secs_f64();
    let tick = std::time::Instant::now();
    let operator = PressureOperator::new(grid, RHO).map_err(error)?;
    let mut rhs = vec![0.0; grid.cell_len()];
    operator
        .build_rhs(refs(&velocity), super::scheduled_forcing::dt(), &mut rhs)
        .map_err(error)?;
    let rhs_seconds = tick.elapsed().as_secs_f64();
    LAST_PREPARATION.with(|profile| profile.set(PreparationProfile { allocation, advect_velocity, advect_tracer, forcing, viscosity, rhs: rhs_seconds }));
    Ok(Prepared {
        velocity,
        tracer,
        rhs,
    })
}
#[cfg(test)]
fn prepare_unprofiled(
    scene: &Scene,
    grid: &GridGeometry,
    track: &Track,
    viscosity: &mut ViscosityWorkspace,
    time: f64,
    cancel: &AtomicBool,
) -> Result<Prepared, String> {
    let mut pre = std::array::from_fn(|d| vec![0.0; grid.face_len(AXES[d])]);
    let mut velocity = pre.clone();
    let mut tracer = vec![0.0; grid.cell_len()];
    advect_velocity(grid, refs(&track.velocity), super::scheduled_forcing::dt(), muts(&mut pre), || {
        cancel.load(Ordering::Relaxed)
    })
    .map_err(error)?;
    advect_tracer(
        grid,
        &track.tracer,
        refs(&track.velocity),
        super::scheduled_forcing::dt(),
        &mut tracer,
        || cancel.load(Ordering::Relaxed),
    )
    .map_err(error)?;
    force(scene, grid, &mut pre, time, cancel)?;
    viscosity
        .update(refs(&pre), RHO, MU, super::scheduled_forcing::dt(), muts(&mut velocity), |_| {
            cancel.load(Ordering::Relaxed)
        })
        .map_err(error)?;
    let operator = PressureOperator::new(grid, RHO).map_err(error)?;
    let mut rhs = vec![0.0; grid.cell_len()];
    operator
        .build_rhs(refs(&velocity), super::scheduled_forcing::dt(), &mut rhs)
        .map_err(error)?;
    Ok(Prepared {
        velocity,
        tracer,
        rhs,
    })
}
fn finish(
    grid: &GridGeometry,
    pre: &Prepared,
    pressure: Vec<f64>,
    tolerance: f64,
) -> Result<(Track, Quality), String> {
    if pressure.len() != grid.cell_len()
        || pressure[0] != 0.0
        || pressure.iter().any(|p| !p.is_finite())
    {
        return Err("AI pressure shape, finiteness or gauge rejected".into());
    }
    let operator = PressureOperator::new(grid, RHO).map_err(error)?;
    let mut action = vec![0.0; grid.cell_len()];
    operator.apply_full(&pressure, &mut action).map_err(error)?;
    let mut residual = 0.0;
    let mut b2 = 0.0;
    let mut maximum = 0.0_f64;
    for (a, b) in action.iter().zip(&pre.rhs) {
        residual += (b - a).powi(2);
        b2 += b * b;
        maximum = maximum.max((b - a).abs());
    }
    let residual_l2 = residual.sqrt();
    let predicted = super::scheduled_forcing::dt() * maximum / grid.cell_volume();
    let mut velocity: Velocity = std::array::from_fn(|d| vec![0.0; grid.face_len(AXES[d])]);
    operator
        .correct_velocity(refs(&pre.velocity), &pressure, super::scheduled_forcing::dt(), muts(&mut velocity))
        .map_err(error)?;
    operator
        .divergence(refs(&velocity), &mut action)
        .map_err(error)?;
    let actual = action.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
    let net = action.iter().sum::<f64>() * grid.cell_volume();
    let flux_limit =
        1e-12_f64.max(1e-8 * action.iter().map(|v| v.abs()).sum::<f64>() * grid.cell_volume());
    let courant = (0..3)
        .map(|d| {
            super::scheduled_forcing::dt() * velocity[d]
                .iter()
                .fold(0.0_f64, |a, b| a.max(f64::from(*b).abs()))
                / grid.spacing()[d]
        })
        .sum::<f64>();
    let mut walls = true;
    for d in 0..3 {
        let shape = grid.face_counts(AXES[d]);
        for k in 0..shape[2] {
            for j in 0..shape[1] {
                for i in 0..shape[0] {
                    let p = [i, j, k];
                    if p[d] == 0 || p[d] + 1 == shape[d] {
                        walls &=
                            velocity[d][grid.face_index(AXES[d], p).ok_or("Wall index")?] == 0.0;
                    }
                }
            }
        }
    }
    let hard_stop = !courant.is_finite() || courant > 0.5 || !walls || net.abs() > flux_limit;
    let qualified = !hard_stop
        && residual_l2 <= 1e-12_f64.max(tolerance * b2.sqrt())
        && predicted <= 1e-7
        && actual <= 1e-5;
    let quality = Quality {
        relative_residual: if b2 > 0.0 {
            Some(residual_l2 / b2.sqrt())
        } else {
            None
        },
        residual_l2,
        predicted_divergence: predicted,
        actual_divergence: actual,
        courant,
        net_flux: net,
        qualified,
        hard_stop,
        flux_limit,
        walls_closed: walls,
    };
    Ok((
        Track {
            velocity,
            tracer: pre.tracer.clone(),
            pressure,
        },
        quality,
    ))
}
fn force(
    scene: &Scene,
    grid: &GridGeometry,
    velocity: &mut Velocity,
    time: f64,
    cancel: &AtomicBool,
) -> Result<(), String> {
    let extents = scene.extents();
    let radius = scene.radius * extents.into_iter().fold(f64::INFINITY, f64::min);
    let (direction, phase) = scene.direction_phase();
    let omega = std::f64::consts::TAU * scene.frequency;
    let control = super::scheduled_forcing::sample(time)?;
    let center = match control {
        Some(frame) => std::array::from_fn::<_, 3, _>(|d| extents[d] * frame.center[d]),
        None => std::array::from_fn::<_, 3, _>(|d| {
            extents[d] * (scene.center[d] + 0.05 * (omega * time + phase + d as f64).sin())
        }),
    };
    let pulse = match control {
        Some(frame) => frame.pulse,
        None => 0.65 + 0.35 * (omega * time + phase).sin(),
    };
    for d in 0..3 {
        let shape = grid.face_counts(AXES[d]);
        for k in 0..shape[2] {
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            for j in 0..shape[1] {
                for i in 0..shape[0] {
                    let p = [i, j, k];
                    if p[d] == 0 || p[d] + 1 == shape[d] {
                        continue;
                    }
                    let pos = grid.face_position(AXES[d], p).ok_or("Force position")?;
                    let offset = std::array::from_fn::<_, 3, _>(|a| (pos[a] - center[a]) / radius);
                    let r2 = offset.iter().map(|v| v * v).sum::<f64>();
                    let envelope = if r2 < 16.0 {
                        (-0.5 * r2).exp() * (1.0 - r2 / 16.0).powi(2)
                    } else {
                        0.0
                    };
                    let a = (d + 1) % 3;
                    let b = (d + 2) % 3;
                    let rotation = direction[a] * offset[b] - direction[b] * offset[a];
                    let wave = (0..3)
                        .map(|a| {
                            (std::f64::consts::TAU * pos[a] / extents[a]
                                + omega * time * (a + 1) as f64
                                + phase
                                + d as f64)
                                .sin()
                        })
                        .sum::<f64>()
                        / 3.0;
                    let acceleration = scene.acceleration
                        * pulse
                        * envelope
                        * (direction[d] + scene.swirl * rotation + scene.fluctuation * wave);
                    let index = grid.face_index(AXES[d], p).ok_or("Force index")?;
                    velocity[d][index] = (f64::from(velocity[d][index]) + super::scheduled_forcing::dt() * acceleration) as f32;
                }
            }
        }
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_prefix_supplies_previous_solved_pressure_before_first_prediction() {
        let mut engine = DiagnosticSession::new(Scene {
            counts: [8, 6, 5],
            steps: 4,
            warmup_steps: 2,
            ..Default::default()
        })
        .unwrap();
        let cancel = AtomicBool::new(false);
        for step in 1..=2 {
            let frame = engine
                .advance(&cancel, |_| panic!("Inference during native prefix"))
                .unwrap();
            assert_eq!(frame.step, step);
            assert_eq!(frame.phase, FramePhase::NativeWarmup);
            assert!(frame.reference_quality.unwrap().qualified);
            assert!(frame.ai_quality.is_none());
            assert_eq!(frame.ai_pressure, frame.reference_pressure);
        }
        let previous = engine.reference.pressure.clone();
        assert!(previous.iter().any(|p| p.abs() > 0.0));
        let frame = engine
            .advance(&cancel, |input| {
                assert_eq!(input.context.generation(), 3);
                assert_eq!(input.initial_guess, previous);
                Ok(vec![0.0; input.initial_guess.len()])
            })
            .unwrap();
        assert_eq!(frame.phase, FramePhase::AiPrediction);
        assert!(frame.ai_quality.is_some());
        assert_ne!(frame.ai_pressure, frame.reference_pressure);
    }
    #[test]
    fn bad_ai_is_observed_without_contaminating_reference_or_fallback() {
        let scene = Scene {
            counts: [8, 6, 5],
            steps: 3,
            warmup_steps: 0,
            ..Default::default()
        };
        let mut a = DiagnosticSession::new(scene.clone()).unwrap();
        let mut b = DiagnosticSession::new(scene).unwrap();
        let cancel = AtomicBool::new(false);
        for _ in 0..2 {
            let one = a
                .advance(&cancel, |r| Ok(vec![0.0; r.initial_guess.len()]))
                .unwrap();
            let two = b
                .advance(&cancel, |r| {
                    let mut p = r.residual.to_vec();
                    p[0] = 0.0;
                    Ok(p)
                })
                .unwrap();
            assert!(one.reference_quality.unwrap().qualified);
            assert!(!one.ai_quality.unwrap().qualified);
            assert_eq!(one.reference_pressure, two.reference_pressure);
            assert_ne!(one.ai_pressure, two.ai_pressure);
            assert_eq!(one.time_s, two.time_s);
            assert_ne!(one.reference_pressure, one.ai_pressure);
        }
    }
    #[test]
    fn zero_forcing_keeps_both_tracks_exact_and_cancellation_stops() {
        let mut engine = DiagnosticSession::new(Scene {
            counts: [5; 3],
            acceleration: 0.0,
            steps: 2,
            warmup_steps: 0,
            ..Default::default()
        })
        .unwrap();
        let cancel = AtomicBool::new(false);
        let frame = engine
            .advance(&cancel, |r| Ok(r.initial_guess.to_vec()))
            .unwrap();
        assert!(frame.ai_quality.unwrap().qualified);
        assert_eq!(frame.reference_pressure, frame.ai_pressure);
        cancel.store(true, Ordering::Relaxed);
        assert!(
            engine
                .advance(&cancel, |r| Ok(r.initial_guess.to_vec()))
                .is_err()
        );
    }
}

#[cfg(test)]
#[path = "engine_typed_tests.rs"]
mod typed_tests;

#[cfg(test)]
mod trajectory_drift_observation_tests {
    use super::*;
    #[test]
    fn equal_tracks_have_zero_drift_and_observation_is_read_only() {
        let engine = DiagnosticSession::new(Scene::default()).unwrap();
        let before = engine.ai.tracer.clone();
        assert_eq!(engine.trajectory_drift(), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(engine.trajectory_drift(), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(engine.ai.tracer, before);
    }
    #[test]
    fn observed_velocity_and_tracer_norms_match_known_perturbations() {
        let mut engine = DiagnosticSession::new(Scene::default()).unwrap();
        engine.reference.velocity[0][1] = 1.0;
        engine.ai.velocity[0][1] = 2.0;
        engine.reference.tracer.fill(0.0);
        engine.ai.tracer.fill(0.0);
        engine.reference.tracer[0] = 1.0;
        engine.ai.tracer[0] = 0.5;
        assert_eq!(engine.trajectory_drift(), (1.0, 0.5, 1.0, 0.5));
    }
}

// Included inside diagnostic::engine in the isolated experimental Rheon copy.
// No reference track, same-request oracle solve, frame history or fallback.
pub struct SingleTrackSession {
    scene: Scene,
    grid: GridGeometry,
    track: Track,
    solver: PressureWorkspace,
    viscosity: ViscosityWorkspace,
    step: usize,
    halted: bool,
}
#[derive(Serialize)]
pub struct SingleStep {
    pub step: usize,
    pub preparation_seconds: f64,
    pub pressure_seconds: f64,
    pub projection_seconds: f64,
    pub frame_seconds: f64,
    pub quality: Quality,
    pub advanced: bool,
}
impl SingleTrackSession {
    pub fn new(scene: Scene) -> Result<Self, String> {
        // Reuse the qualified initializer, discarding its unused reference before
        // stepping. This temporary duplicate initialization is included in setup.
        if !(1..=38400).contains(&scene.steps) || scene.warmup_steps >= scene.steps {
            return Err("Choose 1–38400 streamed steps and a shorter warmup".into());
        }
        // Validate and allocate only the initializer's actual one-frame needs.
        // The original scene still owns all forcing, step and warmup semantics.
        let setup = Scene { steps: 1, warmup_steps: 0, ..scene.clone() };
        let session = DiagnosticSession::new(setup)?;
        Ok(Self { scene, grid: session.grid, track: session.ai,
            solver: session.solver, viscosity: session.viscosity, step: 0, halted: false })
    }
    pub fn state_fields(&self) -> (&[f64], [&[f32]; 3], &[f32]) {
        (&self.track.pressure, refs(&self.track.velocity), &self.track.tracer)
    }
    pub fn advance(
        &mut self,
        native: bool,
        native_tolerance: f64,
        mut predict: impl FnMut(&[f64], &[f64]) -> Result<Vec<f64>, String>,
    ) -> Result<SingleStep, String> {
        if self.halted || self.step >= self.scene.steps {
            return Err("Single track already stopped".into());
        }
        let start = std::time::Instant::now();
        let cancel = AtomicBool::new(false);
        let prepared = prepare(&self.scene, &self.grid, &self.track,
            &mut self.viscosity, self.step as f64 * super::scheduled_forcing::dt(), &cancel)?;
        let preparation_seconds = start.elapsed().as_secs_f64();
        let pressure_start = std::time::Instant::now();
        let pressure = if native || self.step < self.scene.warmup_steps {
            let operator = PressureOperator::new(&self.grid, RHO).map_err(error)?;
            self.solver.solve_rhs(&operator, &prepared.rhs, super::scheduled_forcing::dt(),
                PressureSettings { relative_residual: native_tolerance,
                    absolute_residual: 1e-12, divergence_limit: 1e-7,
                    max_iterations: PRESSURE_ITERATION_LIMIT }, || false).map_err(error)?;
            self.solver.pressure().to_vec()
        } else {
            let operator = PressureOperator::new(&self.grid, RHO).map_err(error)?;
            let mut residual = vec![0.0; self.grid.cell_len()];
            operator.apply_full(&self.track.pressure, &mut residual).map_err(error)?;
            for (r, b) in residual.iter_mut().zip(&prepared.rhs) { *r = b - *r; }
            predict(&self.track.pressure, &residual)?
        };
        let pressure_seconds = pressure_start.elapsed().as_secs_f64();
        let projection_start = std::time::Instant::now();
        let (track, quality) = finish(&self.grid, &prepared, pressure, 1e-6)?;
        let projection_seconds = projection_start.elapsed().as_secs_f64();
        let advanced = quality.qualified;
        if advanced { self.track = track; self.step += 1; } else { self.halted = true; }
        Ok(SingleStep { step: self.step, preparation_seconds, pressure_seconds,
            projection_seconds, frame_seconds: start.elapsed().as_secs_f64(), quality, advanced })
    }
}
#[cfg(test)]
mod single_track_tests {
    use super::*;
    #[test]
    fn native_single_track_matches_existing_reference_fields_exactly() {
        let scene = Scene { counts: [8, 7, 5], steps: 8, warmup_steps: 2,
            acceleration: 0.125, ..Default::default() };
        let mut paired = DiagnosticSession::new(scene.clone()).unwrap();
        let mut single = SingleTrackSession::new(scene).unwrap();
        for _ in 0..8 {
            paired.advance(&AtomicBool::new(false), |r| Ok(r.initial_guess.to_vec())).unwrap();
            let step = single.advance(true, 1e-8, |_, _| panic!("native callback")).unwrap();
            assert!(step.advanced);
            assert_eq!(single.track.pressure, paired.reference.pressure);
            assert_eq!(single.track.velocity, paired.reference.velocity);
            assert_eq!(single.track.tracer, paired.reference.tracer);
        }
    }
    #[test]
    fn rejected_candidate_never_advances_and_no_oracle_or_fallback_is_called() {
        let mut single = SingleTrackSession::new(Scene { counts: [8, 7, 5],
            steps: 2, warmup_steps: 0, ..Default::default() }).unwrap();
        let prior = single.track.clone();
        let mut calls = 0;
        let step = single.advance(false, 1e-8, |base, _| { calls += 1; Ok(vec![0.0; base.len()]) }).unwrap();
        assert!(!step.advanced);
        assert_eq!(calls, 1);
        assert_eq!(single.step, 0);
        assert_eq!(single.track.pressure, prior.pressure);
        assert_eq!(single.track.velocity, prior.velocity);
        assert_eq!(single.track.tracer, prior.tracer);
        assert!(single.advance(true, 1e-8, |_, _| unreachable!()).is_err());
    }
}

#[cfg(test)]
mod profile_parity_tests {
    use super::*;
    #[test]
    fn timed_preparation_preserves_all_fields_bitwise() {
        for counts in [[8, 7, 5], [16, 16, 16]] {
            let scene = Scene { counts, steps: 2, warmup_steps: 0, ..Default::default() };
            let session = SingleTrackSession::new(scene.clone()).unwrap();
            let mut v1 = ViscosityWorkspace::new(session.grid.clone(), 128*1024*1024).unwrap();
            let mut v2 = ViscosityWorkspace::new(session.grid.clone(), 128*1024*1024).unwrap();
            for time in [0., 0.5] {
                let a = prepare(&scene, &session.grid, &session.track, &mut v1, time, &AtomicBool::new(false)).unwrap();
                let b = prepare_unprofiled(&scene, &session.grid, &session.track, &mut v2, time, &AtomicBool::new(false)).unwrap();
                assert_eq!(a.velocity, b.velocity);
                assert_eq!(a.tracer, b.tracer);
                assert_eq!(a.rhs, b.rhs);
            }
        }
    }
}

#[cfg(test)]
mod streamed_state_tests {
    use super::*;
    #[test]
    fn streaming_does_not_require_historical_frame_storage() {
        let scene = Scene { counts: [32; 3], steps: 2000, warmup_steps: 0, ..Default::default() };
        assert!(scene.validate().is_err()); // Paired history would exceed its guard.
        let single = SingleTrackSession::new(scene).unwrap();
        let (pressure, velocity, tracer) = single.state_fields();
        assert_eq!(pressure.len(), 32usize.pow(3));
        assert_eq!(tracer.len(), pressure.len());
        assert!(velocity.iter().all(|v| v.len() == 33*32*32));
        assert_eq!(single.scene.steps, 2000);
    }
    #[test]
    fn streaming_preserves_geometry_and_step_bounds() {
        for scene in [
            Scene { steps: 0, warmup_steps: 0, ..Default::default() },
            Scene { steps: 38401, ..Default::default() },
            Scene { counts: [257; 3], ..Default::default() },
            Scene { steps: 2, warmup_steps: 2, ..Default::default() },
        ] { assert!(SingleTrackSession::new(scene).is_err()); }
    }
}

// Isolated research restart API; the existing advance/finish pressure gates are unchanged.
impl SingleTrackSession {
    pub fn state_step(&self) -> usize { self.step }
    pub fn restore_state_fields(&mut self, step: usize, pressure: Vec<f64>,
        velocity: [Vec<f32>;3], tracer: Vec<f32>) -> Result<(),String> {
        if step>self.scene.steps || pressure.len()!=self.grid.cell_len() || tracer.len()!=self.grid.cell_len()
            || pressure.first()!=Some(&0.) || pressure.iter().any(|v|!v.is_finite())
            || tracer.iter().any(|v|!v.is_finite())
            || (0..3).any(|d|velocity[d].len()!=self.grid.face_len(AXES[d]) || velocity[d].iter().any(|v|!v.is_finite())) {
            return Err("Invalid complete research restart state".into());
        }
        let solver=PressureWorkspace::new(&self.grid,2*1024*1024*1024).map_err(error)?;
        let viscosity=ViscosityWorkspace::new(self.grid.clone(),2*1024*1024*1024).map_err(error)?;
        self.track=Track {pressure,velocity,tracer};self.step=step;self.halted=false;
        self.solver=solver;self.viscosity=viscosity;
        Ok(())
    }
}
#[cfg(test)]
mod temporal_restart_tests {
    use super::*;
    #[test]
    fn eight_future_native_steps_are_bitwise_equal_after_complete_restart() {
        for counts in [[8,7,5],[16,16,16]] {
            let scene=Scene {counts,steps:16,warmup_steps:0,..Default::default()};
            let mut original=SingleTrackSession::new(scene.clone()).unwrap();
            for _ in 0..4 { assert!(original.advance(true,1e-8,|_,_|unreachable!()).unwrap().advanced); }
            let mut restarted=SingleTrackSession::new(scene).unwrap();
            restarted.restore_state_fields(original.step,original.track.pressure.clone(),
                original.track.velocity.clone(),original.track.tracer.clone()).unwrap();
            for _ in 0..8 {
                assert!(original.advance(true,1e-8,|_,_|unreachable!()).unwrap().advanced);
                assert!(restarted.advance(true,1e-8,|_,_|unreachable!()).unwrap().advanced);
                assert_eq!(original.step,restarted.step);
                assert_eq!(original.track.pressure,restarted.track.pressure);
                assert_eq!(original.track.velocity,restarted.track.velocity);
                assert_eq!(original.track.tracer,restarted.track.tracer);
            }
        }
    }
    #[test]
    fn invalid_restart_preserves_the_prior_state() {
        let mut session=SingleTrackSession::new(Scene {steps:16,warmup_steps:0,..Default::default()}).unwrap();
        let before=session.track.clone();
        let mut bad=before.pressure.clone();bad[0]=f64::NAN;
        assert!(session.restore_state_fields(0,bad,before.velocity.clone(),before.tracer.clone()).is_err());
        assert!(session.restore_state_fields(17,before.pressure.clone(),before.velocity.clone(),before.tracer.clone()).is_err());
        assert_eq!(session.track.pressure,before.pressure);assert_eq!(session.step,0);
    }
}

// Included only in the isolated research engine for independent forcing tests.
impl SingleTrackSession {
    pub fn preview_prescribed_force(&self, step: usize, mut velocity: [Vec<f32>; 3])
        -> Result<[Vec<f32>; 3], String>
    {
        if step >= self.scene.steps || velocity.iter().enumerate().any(|(d, values)| {
            values.len() != self.grid.face_counts(AXES[d]).iter().product::<usize>()
                || values.iter().any(|v| !v.is_finite())
        }) {
            return Err("Bounded finite prescribed-force preview required".into());
        }
        force(&self.scene, &self.grid, &mut velocity, step as f64 * super::scheduled_forcing::dt(), &AtomicBool::new(false))?;
        Ok(velocity)
    }
}
