//! Bounded independently evolved trajectories; no teacher states enter learned rollouts.
//! Usage: ai_pressure_trajectory MODEL_EXPORT NEW_OUTPUT_DIRECTORY
//! Scenario source adapted from Apache-2.0 MrScripty/rheon-surrogate-lab,
//! commit e8a90bfef6b2955c502008ccc62443e29dd9f2ec, probes/trajectory/src/main.rs.
use rheon::solver_provider::*;
use rheon::*;
use std::{
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
type Velocity = [Vec<f32>; 3];
const DT: f64 = 1.0 / 60.0;
const FRAMES: usize = 24;
const FAULT_FRAME: usize = 9;
const MODES: [&str; 6] = [
    "jacobi",
    "sgs",
    "learned",
    "injected-fallback",
    "shifted-jacobi",
    "unsupported-fallback",
];
fn refs(v: &Velocity) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn muts(v: &mut Velocity) -> [&mut [f32]; 3] {
    let [x, y, z] = v;
    [x, y, z]
}

// A labeled failure fixture, not a learned prediction or a replacement model.
struct FaultProbe<'a> {
    model: &'a mut LabModalPressure,
    inject: bool,
}
impl PressureProvider for FaultProbe<'_> {
    fn metadata(&self) -> SolverMetadata {
        let mut metadata = self.model.metadata();
        if self.inject {
            metadata.id = "test-injected-error-after-real-inference";
        }
        metadata
    }
    fn supports(
        &self,
        request: &PressureRequest<'_>,
    ) -> std::result::Result<(), ProviderRejection> {
        self.model.supports(request)
    }
    fn predict(
        &mut self,
        request: &PressureRequest<'_>,
        output: &mut [f64],
        cancelled: &mut dyn FnMut() -> bool,
    ) -> std::result::Result<(), ProviderRejection> {
        self.model.predict(request, output, cancelled)?;
        if self.inject {
            output[1] += 1.0;
        }
        Ok(())
    }
}
fn source(grid: &GridGeometry, v: &mut Velocity, tracer: &mut [f32], seed: u64, time: f64) {
    let phase = (seed % 997) as f64 * 0.137;
    let center = [
        0.43 + 0.13 * (phase + time).sin(),
        0.28 + 0.06 * phase.cos(),
        0.45 + 0.13 * (phase - time).cos(),
    ];
    for (d, values) in v.iter_mut().enumerate() {
        let shape = grid.face_counts([Axis::X, Axis::Y, Axis::Z][d]);
        for k in 0..shape[2] {
            for j in 0..shape[1] {
                for i in 0..shape[0] {
                    let p = [i, j, k];
                    if p[d] == 0 || p[d] + 1 == shape[d] {
                        continue;
                    }
                    let pos = grid
                        .face_position([Axis::X, Axis::Y, Axis::Z][d], p)
                        .unwrap();
                    let r2: f64 = (0..3)
                        .map(|a| (pos[a] - grid.origin()[a] - center[a]).powi(2))
                        .sum();
                    let direction = [
                        0.75 * (phase + 2.0 * time).cos(),
                        1.0,
                        0.75 * (phase + 2.0 * time).sin(),
                    ][d];
                    let index = grid.face_index([Axis::X, Axis::Y, Axis::Z][d], p).unwrap();
                    // Small deterministic broadband stirring excites every pressure
                    // mode. It is part of the declared scene on every method/split.
                    let mut bits = (index as u64)
                        .wrapping_add(seed.wrapping_mul(7919))
                        .wrapping_add((time / DT).round() as u64 * 104729)
                        .wrapping_add(d as u64 * 997);
                    bits = (bits ^ (bits >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
                    bits = (bits ^ (bits >> 27)).wrapping_mul(0x94d049bb133111eb);
                    bits ^= bits >> 31;
                    let noise = 2.0 * ((bits >> 40) as f64 / 16_777_216.0) - 1.0;
                    let acceleration = 1.0 * (-r2 / 0.012).exp() * direction + 0.05 * noise;
                    values[index] = (f64::from(values[index]) + DT * acceleration) as f32;
                }
            }
        }
    }
    let [nx, ny, nz] = grid.counts();
    for k in 0..nz {
        for j in 0..ny {
            for i in 0..nx {
                let pos = grid.cell_position([i, j, k]).unwrap();
                let r2: f64 = (0..3)
                    .map(|a| (pos[a] - grid.origin()[a] - center[a]).powi(2))
                    .sum();
                let index = i + nx * (j + ny * k);
                tracer[index] =
                    (f64::from(tracer[index]) + DT * 8.0 * (-r2 / 0.006).exp()).min(1.0) as f32;
            }
        }
    }
}

struct State {
    grid: GridGeometry,
    accepted: Velocity,
    candidate: Velocity,
    corrected: Velocity,
    tracer: Vec<f32>,
    source_tracer: Vec<f32>,
    next_tracer: Vec<f32>,
    rhs: Vec<f64>,
    divergence: Vec<f64>,
    workspace: ProviderWorkspace,
    frame: usize,
}
impl State {
    fn new(mode: &str) -> Result<Self> {
        let shifted = mode == "shifted-jacobi" || mode == "unsupported-fallback";
        let grid = GridGeometry::new(
            [32; 3],
            [1.0 / 32.0; 3],
            if shifted { [1.0; 3] } else { [0.0; 3] },
        )?;
        let accepted: Velocity =
            std::array::from_fn(|d| vec![0.0; grid.face_len([Axis::X, Axis::Y, Axis::Z][d])]);
        let tracer = vec![0.0; grid.cell_len()];
        let rhs = vec![0.0; grid.cell_len()];
        let fallback = if mode == "sgs" {
            PressureImplementation::SymmetricGaussSeidelPcgV1
        } else {
            PressureImplementation::JacobiPcgV1
        };
        let workspace = ProviderWorkspace::new(&grid, 64 * 1024 * 1024, fallback)?;
        Ok(Self {
            grid,
            candidate: accepted.clone(),
            corrected: accepted.clone(),
            accepted,
            source_tracer: tracer.clone(),
            next_tracer: tracer.clone(),
            tracer,
            divergence: rhs.clone(),
            rhs,
            workspace,
            frame: 0,
        })
    }
    fn step(&mut self, mode: &str, seed: u64, model: &mut LabModalPressure) -> Result<FrameReport> {
        let begin = Instant::now();
        advect_velocity(
            &self.grid,
            refs(&self.accepted),
            DT,
            muts(&mut self.candidate),
            || false,
        )?;
        self.source_tracer.copy_from_slice(&self.tracer);
        source(
            &self.grid,
            &mut self.candidate,
            &mut self.source_tracer,
            seed,
            self.frame as f64 * DT,
        );
        let operator = PressureOperator::new(&self.grid, 1.25)?;
        operator.build_rhs(refs(&self.candidate), DT, &mut self.rhs)?;
        let request = PressureRequest {
            grid: &self.grid,
            density: 1.25,
            dt: DT,
            rhs: &self.rhs,
        };
        let settings = PressureSettings {
            relative_residual: 1e-8,
            absolute_residual: 1e-12,
            divergence_limit: 1e-7,
            max_iterations: 3000,
        };
        let inject = mode == "injected-fallback" && self.frame + 1 == FAULT_FRAME;
        let mut probe = FaultProbe { model, inject };
        let preferred: Option<&mut dyn PressureProvider> =
            if ["learned", "injected-fallback", "unsupported-fallback"].contains(&mode) {
                Some(&mut probe)
            } else {
                None
            };
        let provider = self
            .workspace
            .solve(request, settings, preferred, || false)?;
        let rejection = match &provider.rejection {
            None => "none",
            Some(ProviderRejection::QualityGate) => "quality-gate",
            Some(ProviderRejection::Unsupported(_)) => "unsupported",
            other => return Err(format!("unexpected rejection {other:?}").into()),
        };
        let expected = if mode == "unsupported-fallback" {
            ("jacobi-pcg-v1", "unsupported")
        } else if inject {
            ("jacobi-pcg-v1", "quality-gate")
        } else if mode == "learned" || mode == "injected-fallback" {
            ("lab-modal-pressure-v1", "none")
        } else if mode == "sgs" {
            ("sgs-pcg-v1", "none")
        } else {
            ("jacobi-pcg-v1", "none")
        };
        if (provider.selected, rejection) != expected {
            return Err(format!(
                "unexpected selection on {mode} frame {}: {provider:?}",
                self.frame + 1
            )
            .into());
        }
        operator.correct_velocity(
            refs(&self.candidate),
            self.workspace.pressure(),
            DT,
            muts(&mut self.corrected),
        )?;
        operator.divergence(refs(&self.corrected), &mut self.divergence)?;
        let actual = self.divergence.iter().fold(0.0_f64, |a, b| a.max(b.abs()));
        let courant = DT
            * self
                .corrected
                .iter()
                .enumerate()
                .map(|(d, field)| {
                    field
                        .iter()
                        .fold(0.0_f64, |a, b| a.max(f64::from(*b).abs()))
                        / self.grid.spacing()[d]
                })
                .sum::<f64>();
        if !actual.is_finite() || actual > 1e-5 || !courant.is_finite() || courant > 1.0 {
            return Err("actual divergence/Courant gate".into());
        }
        advect_tracer(
            &self.grid,
            &self.source_tracer,
            refs(&self.corrected),
            DT,
            &mut self.next_tracer,
            || false,
        )?;
        if self
            .next_tracer
            .iter()
            .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || self.corrected.iter().flatten().any(|v| !v.is_finite())
        {
            return Err("finite/range gate".into());
        }
        let integral =
            self.next_tracer.iter().map(|v| f64::from(*v)).sum::<f64>() * self.grid.cell_volume();
        // Only commit after every complete-step gate has passed.
        std::mem::swap(&mut self.accepted, &mut self.corrected);
        std::mem::swap(&mut self.tracer, &mut self.next_tracer);
        self.frame += 1;
        Ok(FrameReport {
            provider,
            rejection,
            actual,
            courant,
            integral,
            total_ms: begin.elapsed().as_secs_f64() * 1000.0,
        })
    }
    fn capture(&self, path: &Path) -> Result<()> {
        let mut out = BufWriter::new(fs::File::create(path)?);
        for field in [self.rhs.as_slice(), self.workspace.pressure()] {
            for value in field {
                out.write_all(&value.to_le_bytes())?;
            }
        }
        for field in self
            .accepted
            .iter()
            .map(Vec::as_slice)
            .chain([self.tracer.as_slice()])
        {
            for value in field {
                out.write_all(&value.to_le_bytes())?;
            }
        }
        out.flush()?;
        Ok(())
    }
}
struct FrameReport {
    provider: ProviderReport,
    rejection: &'static str,
    actual: f64,
    courant: f64,
    integral: f64,
    total_ms: f64,
}
fn main() -> Result<()> {
    let process = Instant::now();
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 3 {
        return Err("expected MODEL_EXPORT NEW_OUTPUT_DIRECTORY".into());
    }
    let output = Path::new(&args[2]);
    fs::create_dir(output)?;
    let bytes = fs::read(&args[1])?;
    let mut model = LabModalPressure::from_export(&bytes).map_err(|e| format!("{e:?}"))?;
    let model_load_ms = process.elapsed().as_secs_f64() * 1000.0;
    let mut frames = BufWriter::new(fs::File::create(output.join("frames.csv"))?);
    let mut runs = BufWriter::new(fs::File::create(output.join("runs.csv"))?);
    writeln!(
        frames,
        "phase,repetition,seed,mode,frame,time,selected,rejection,step_ms,provider_ms,inference_ms,qualification_ms,fallback_ms,residual_l2,predicted_divergence,actual_divergence,courant,tracer_integral"
    )?;
    writeln!(
        runs,
        "phase,repetition,seed,mode,frames,initialization_ms,steps_ms,rollout_wall_ms"
    )?;
    // Timing runs do not capture fields. Fourth repetition is a separate replay.
    for repetition in 0..4 {
        let capture = repetition == 3;
        let phase = if capture { "capture" } else { "timed" };
        for seed in [900, 901] {
            for offset in 0..MODES.len() {
                let mode = MODES[(offset + repetition) % MODES.len()];
                let begin = Instant::now();
                let mut state = State::new(mode)?;
                let initialization_ms = begin.elapsed().as_secs_f64() * 1000.0;
                let directory = output.join(format!("{seed}-{mode}"));
                if capture {
                    fs::create_dir(&directory)?;
                }
                let mut rows = Vec::with_capacity(FRAMES);
                let mut steps_ms = 0.0;
                for _ in 0..FRAMES {
                    let row = state.step(mode, seed, &mut model)?;
                    steps_ms += row.total_ms;
                    if capture {
                        state.capture(&directory.join(format!("{:03}.bin", state.frame)))?;
                    }
                    rows.push(row);
                }
                let rollout_wall_ms = begin.elapsed().as_secs_f64() * 1000.0;
                writeln!(
                    runs,
                    "{phase},{repetition},{seed},{mode},{FRAMES},{initialization_ms},{steps_ms},{rollout_wall_ms}"
                )?;
                for (index, row) in rows.into_iter().enumerate() {
                    let p = row.provider;
                    writeln!(
                        frames,
                        "{phase},{repetition},{seed},{mode},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                        index + 1,
                        (index + 1) as f64 * DT,
                        p.selected,
                        row.rejection,
                        row.total_ms,
                        p.total.as_secs_f64() * 1000.0,
                        p.inference.as_secs_f64() * 1000.0,
                        p.qualification.as_secs_f64() * 1000.0,
                        p.fallback.as_secs_f64() * 1000.0,
                        p.accepted.true_residual_l2,
                        p.accepted.predicted_divergence_max,
                        row.actual,
                        row.courant,
                        row.integral
                    )?;
                }
            }
        }
    }
    frames.flush()?;
    runs.flush()?;
    // Complete manifest is written last; a failure never produces one.
    fs::write(
        output.join("complete.json"),
        format!(
            "{{\"schema\":\"rheon-ai-trajectory-v1\",\"frames\":{FRAMES},\"seeds\":[900,901],\"timing_repetitions\":3,\"fault_frame\":{FAULT_FRAME},\"model_load_ms\":{model_load_ms},\"process_ms\":{},\"checkpoint_sha256\":\"25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9\"}}\n",
            process.elapsed().as_secs_f64() * 1000.0
        ),
    )?;
    println!("completed independent trajectories in {}", output.display());
    Ok(())
}
