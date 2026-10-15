//! Native live 3D laboratory. Numerical state stays owned by the worker; the
//! viewer consumes bounded immutable accepted previews and optional diagnostics.
#[path = "desktop/ai_comparison/mod.rs"]
mod ai_comparison;
#[path = "desktop/camera.rs"]
mod camera;
#[path = "desktop/emitter_controls.rs"]
mod emitter_controls;
#[path = "desktop/engineering.rs"]
mod engineering;
#[path = "desktop/export_ui.rs"]
mod export_ui;

#[path = "desktop/archive_view.rs"]
mod archive_view;
#[path = "desktop/collision_ui.rs"]
mod collision_ui;
#[path = "desktop/history.rs"]
mod history;
#[path = "desktop/identity.rs"]
mod identity;
#[path = "desktop/inspector.rs"]
mod inspector;
#[path = "desktop/particle_view.rs"]
mod particle_view;
#[path = "desktop/settings.rs"]
mod settings;
#[path = "desktop/transport_view.rs"]
mod transport_view;
#[path = "desktop/viewport.rs"]
mod viewport;
use eframe::egui;
use history::{LiveHistory, PreviewFrame};
use rheon::{
    CalculationObserver, GridGeometry, PressureImplementation, PressureSettings,
    RotatingCylinderEmitter, Simulation, SimulationConfig, SmokeSource, guidance_pixels,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Request {
    collision_mesh: Option<Arc<rheon::CollisionMesh>>,
    size: u64,
    steps: usize,
    tight: bool,
    threads: usize,
    methods: Vec<PressureImplementation>,
    emitter: Option<RotatingCylinderEmitter>,
    observe: bool,
    history_budget_mib: usize,
    domain_side: f64,
    requested_dt: f64,
    material: Option<rheon::ParticleFluidConfig>,
    material_color: [u8; 3],
    recording_limits: rheon::recording::RecordingLimits,
    recording_directory: String,
    #[cfg(test)]
    progress_gate: Option<Arc<ProgressGate>>,
}
#[cfg(test)]
struct ProgressGate {
    reached: std::sync::mpsc::SyncSender<()>,
    release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}
struct ResultRow {
    method: PressureImplementation,
    seconds: f64,
    iterations: usize,
    divergence: f64,
    bytes: usize,
    time: f64,
    pixels: Vec<u8>,
    particle_report: Option<rheon::ParticleStepReport>,
}
struct Worker {
    cancel: Arc<AtomicBool>,
    progress: Arc<AtomicUsize>,
    total: usize,
    scheduler: Arc<rheon::CpuScheduler>,
    history: Arc<LiveHistory>,
    observer: Option<Arc<CalculationObserver>>,
    handle: JoinHandle<Result<Vec<ResultRow>, String>>,
}
impl Worker {
    fn start(request: Request) -> std::io::Result<Self> {
        validate_request(&request).map_err(std::io::Error::other)?;
        let cancel = Arc::new(AtomicBool::new(false));
        let progress = Arc::new(AtomicUsize::new(0));
        let history = LiveHistory::with_budget(
            request.size as usize,
            request.steps,
            request.methods.len(),
            request.history_budget_mib * 1024 * 1024,
        )
        .map_err(std::io::Error::other)?;
        let h = request.domain_side / request.size as f64;
        let grid = GridGeometry::new([request.size; 3], [h; 3], [0.0; 3])
            .map_err(std::io::Error::other)?;
        let observer = if request.observe {
            Some(CalculationObserver::new(&grid, request.threads).map_err(std::io::Error::other)?)
        } else {
            None
        };
        let c = Arc::clone(&cancel);
        let p = Arc::clone(&progress);
        let frames = Arc::clone(&history);
        let scheduler = rheon::CpuScheduler::new();
        let scheduling = Arc::clone(&scheduler);
        let observe = observer.clone();
        let total = request.methods.len() * request.steps;
        let handle = std::thread::Builder::new()
            .name("rheon-live-simulation".into())
            .spawn(move || run_owned(request, &c, &p, &frames, observe, scheduling))?;
        Ok(Self {
            cancel,
            progress,
            total,
            scheduler,
            history,
            observer,
            handle,
        })
    }
}
fn available_threads() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}
fn validate_request(request: &Request) -> Result<(), String> {
    {
        request
            .recording_limits
            .validate()
            .map_err(|e| e.to_string())?;
        if request.recording_directory.is_empty() || request.recording_directory.len() > 4096 {
            return Err("Invalid recording directory".into());
        }
    }
    if let Some(mesh) = &request.collision_mesh {
        if request.material.is_none() {
            return Err("Collision mesh requires material simulation".into());
        }
        collision_ui::validate_bounds(mesh, request.domain_side)?;
    }
    let maximum = if request.material.is_some() {
        history::MAX_PREVIEW_EDGE as u64
    } else {
        128
    };
    if !(1..=maximum).contains(&request.size)
        || !(1..=10000).contains(&request.steps)
        || request.methods.is_empty()
        || request.methods.len() > 2
        || (request.methods.len() == 2 && request.methods[0] == request.methods[1])
    {
        return Err("Invalid simulation settings".into());
    }
    if !(16..=4096).contains(&request.history_budget_mib)
        || request
            .history_budget_mib
            .checked_mul(1024 * 1024)
            .is_none()
    {
        return Err("Invalid timeline memory budget".into());
    }
    if !request.domain_side.is_finite()
        || !(0.001..=100.0).contains(&request.domain_side)
        || !request.requested_dt.is_finite()
        || request.requested_dt <= 0.0
    {
        return Err("Invalid physical domain size or step duration".into());
    }
    if request.threads == 0 || request.threads > available_threads() {
        return Err("CPU threads must be between 1 and the available CPU count".into());
    }
    Ok(())
}
#[cfg(test)]
fn run(
    request: Request,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
) -> Result<Vec<ResultRow>, String> {
    validate_request(&request)?;
    let history = LiveHistory::with_budget(
        request.size as usize,
        request.steps,
        request.methods.len(),
        request.history_budget_mib * 1024 * 1024,
    )?;
    run_owned(
        request,
        cancel,
        progress,
        &history,
        None,
        rheon::CpuScheduler::new(),
    )
}
fn run_owned(
    request: Request,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
    history: &LiveHistory,
    observer: Option<Arc<CalculationObserver>>,
    scheduler: Arc<rheon::CpuScheduler>,
) -> Result<Vec<ResultRow>, String> {
    validate_request(&request)?;
    rayon::ThreadPoolBuilder::new()
        .num_threads(request.threads)
        .thread_name(|i| format!("rheon-cpu-{i}"))
        .build_scoped(
            |thread| {
                scheduler.scope(|| {
                    if let Some(observer) = &observer {
                        observer.observe(|| thread.run());
                    } else {
                        thread.run();
                    }
                })
            },
            |pool| pool.install(|| run_simulations(request, cancel, progress, history)),
        )
        .map_err(|e| format!("Cannot start CPU pool: {e}"))?
}
fn run_simulations(
    request: Request,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
    history: &LiveHistory,
) -> Result<Vec<ResultRow>, String> {
    if let Some(config) = request.material {
        return run_material(request, config, cancel, progress, history);
    }
    let mut rows = Vec::new();
    for (track, method) in request.methods.iter().copied().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled; accepted preview frames retained".into());
        }
        let h = request.domain_side / request.size as f64;
        let grid =
            GridGeometry::new([request.size; 3], [h; 3], [0.0; 3]).map_err(|e| e.to_string())?;
        let config = SimulationConfig {
            density: 1.0,
            memory_limit: 192 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: if request.tight { 1e-11 } else { 1e-9 },
                absolute_residual: 1e-12,
                divergence_limit: if request.tight { 1e-9 } else { 1e-7 },
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        };
        let mut sim =
            Simulation::with_implementation(grid, config, method).map_err(|e| e.to_string())?;
        let path = new_recording_path(&request.recording_directory, method.id())?;
        let inspector = &history.carrier_inspectors[track];
        let mut recorder = inspector
            .begin_carrier(&sim, &path, request.recording_limits)
            .map_err(|e| format!("Carrier recording start at {} failed: {e}", path.display()))?;
        let result: Result<(), String> = (|| {
            history.publish(
                track,
                PreviewFrame::capture(&sim, history.preview_edge, 0, 0.0, None)?,
            )?;
            let source = SmokeSource {
                lower: [0.35, 0.08, 0.35].map(|v| v * request.domain_side),
                upper: [0.65, 0.30, 0.65].map(|v| v * request.domain_side),
                tracer_rate: 2.0,
                vertical_acceleration: 1.0,
            };
            let mut iterations = 0;
            let mut divergence = 0.0_f64;
            let mut seconds = 0.0;
            for step in 0..request.steps {
                history.begin_step(track, step + 1)?;
                let start = Instant::now();
                let result = if let Some(emitter) = request.emitter {
                    sim.step_with_emitter(request.requested_dt, emitter, |_| {
                        cancel.load(Ordering::Relaxed)
                    })
                } else {
                    sim.step(
                        request.requested_dt,
                        (step < request.steps / 2).then_some(source),
                        |_| cancel.load(Ordering::Relaxed),
                    )
                };
                let step_seconds = start.elapsed().as_secs_f64();
                history.end_step()?;
                let r = result.map_err(|e| e.to_string())?;
                inspector.record_carrier(&mut recorder, &sim)?;
                iterations += r.pressure.iterations;
                divergence = divergence.max(r.actual_divergence_max);
                seconds += step_seconds;
                history.publish(
                    track,
                    PreviewFrame::capture(
                        &sim,
                        history.preview_edge,
                        step + 1,
                        step_seconds,
                        Some(r),
                    )?,
                )?;
                progress.fetch_add(1, Ordering::Relaxed);
                #[cfg(test)]
                if progress.load(Ordering::Relaxed) == 1
                    && let Some(gate) = &request.progress_gate
                {
                    gate.reached.send(()).map_err(|e| e.to_string())?;
                    gate.release
                        .lock()
                        .map_err(|e| e.to_string())?
                        .recv_timeout(Duration::from_secs(10))
                        .map_err(|e| e.to_string())?;
                }
            }
            if cancel.load(Ordering::Relaxed) {
                return Err("Cancelled; accepted recording prefix retained".into());
            }
            let pixels = guidance_pixels(&sim, 128 * 128).map_err(|e| e.to_string())?;
            rows.push(ResultRow {
                method,
                seconds,
                iterations,
                divergence,
                bytes: sim.allocated_bytes(),
                time: sim.state().time,
                pixels,
                particle_report: None,
            });
            Ok(())
        })();
        let outcome = if result.is_ok() {
            "completed"
        } else if cancel.load(Ordering::Relaxed) {
            "cancelled"
        } else {
            "failed"
        };
        let detail = match &result {
            Ok(()) => "All accepted carrier steps retained; no restartable checkpoint claim".into(),
            Err(e) => format!(
                "{e}; accepted generation {}; acknowledged publications {}; accepted tail may be unrecorded",
                sim.state().generation,
                recorder.recorded_count()
            ),
        };
        recorder
            .finish(outcome, &detail, Some(sim.state().generation))
            .map_err(|e| {
                format!(
                    "{detail}; terminal status not confirmed at {}: {e}",
                    path.display()
                )
            })?;
        result?;
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled; accepted preview frames retained".into());
    }
    Ok(rows)
}
fn new_recording_path(directory: &str, label: &str) -> Result<std::path::PathBuf, String> {
    let directory = std::path::Path::new(directory);
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_nanos();
    Ok(directory.join(format!(
        "{label}-{timestamp}-{}-{}.sqlite",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )))
}
fn run_material(
    request: Request,
    config: rheon::ParticleFluidConfig,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
    history: &LiveHistory,
) -> Result<Vec<ResultRow>, String> {
    if request.methods.len() != 1 {
        return Err("Compare pressure solvers in the original carrier mode".into());
    }
    let method = request.methods[0];
    let h = request.domain_side / request.size as f64;
    let grid = GridGeometry::new([request.size; 3], [h; 3], [0.0; 3]).map_err(|e| e.to_string())?;
    let mut sim = rheon::ParticleFluidSimulation::new(grid, config).map_err(|e| e.to_string())?;
    sim.set_collision_mesh(request.collision_mesh.clone())
        .map_err(|e| e.to_string())?;
    let path = new_recording_path(&request.recording_directory, "material")?;
    let mut recorder = history
        .inspector
        .begin(&sim, &path, request.recording_limits)
        .map_err(|e| format!("Recording start at {} failed: {e}", path.display()))?;
    let result = (|| {
        history.publish(
            0,
            PreviewFrame::capture_particles(
                &sim,
                method,
                history.preview_edge,
                0,
                0.0,
                None,
                request.material_color,
            )?,
        )?;
        let mut seconds = 0.0;
        let mut last_report = None;
        for step in 0..request.steps {
            history.begin_step(0, step + 1)?;
            let start = Instant::now();
            let result = sim.step(request.requested_dt, request.emitter, || {
                cancel.load(Ordering::Relaxed)
            });
            let elapsed = start.elapsed().as_secs_f64();
            history.end_step()?;
            let report = result.map_err(|e| e.to_string())?;
            seconds += elapsed;
            last_report = Some(report);
            history.inspector.record(
                &mut recorder,
                &sim,
                report,
                rheon::MaterialStepControls {
                    requested_dt: request.requested_dt,
                    emitter: request
                        .emitter
                        .map(|emitter| rheon::MaterialEmitterControl {
                            source_id: 1,
                            source_version: 1,
                            emitter,
                        }),
                },
            )?;
            history.publish(
                0,
                PreviewFrame::capture_particles(
                    &sim,
                    method,
                    history.preview_edge,
                    step + 1,
                    elapsed,
                    Some(report),
                    request.material_color,
                )?,
            )?;
            progress.fetch_add(1, Ordering::Relaxed);
            #[cfg(test)]
            if progress.load(Ordering::Relaxed) == 1
                && let Some(gate) = &request.progress_gate
            {
                gate.reached.send(()).map_err(|e| e.to_string())?;
                gate.release
                    .lock()
                    .map_err(|e| e.to_string())?
                    .recv_timeout(Duration::from_secs(10))
                    .map_err(|e| e.to_string())?;
            }
        }
        if cancel.load(Ordering::Relaxed) {
            return Err("Cancelled; accepted preview frames retained".into());
        }
        let n = request.size as usize;
        let mut pixels = vec![0_u8; n * n];
        for p in sim.particles().iter().filter(|p| p.material == 0) {
            let x = ((p.position[0] / request.domain_side * n as f64).floor() as usize).min(n - 1);
            let z = ((p.position[2] / request.domain_side * n as f64).floor() as usize).min(n - 1);
            pixels[x + n * z] = pixels[x + n * z].saturating_add(32);
        }
        Ok(vec![ResultRow {
            method,
            seconds,
            iterations: 0,
            divergence: 0.0,
            bytes: sim.allocated_bytes(),
            time: sim.time(),
            pixels,
            particle_report: last_report,
        }])
    })();
    let outcome = if result.is_ok() {
        "completed"
    } else if cancel.load(Ordering::Relaxed) {
        "cancelled"
    } else {
        "failed"
    };
    let detail = match &result {
        Ok(_) => "All accepted steps retained; no restartable checkpoint claim".to_owned(),
        Err(e) => format!(
            "{e}; accepted generation {}; acknowledged publications {}; an accepted tail may be unrecorded",
            sim.generation(),
            recorder.recorded_count()
        ),
    };
    if let Err(e) = recorder.finish(outcome, &detail, Some(sim.generation())) {
        return Err(format!(
            "{detail}; recording terminal status not confirmed at {}: {e}",
            path.display()
        ));
    }
    result
}
struct App {
    ai_comparison: Option<ai_comparison::ComparisonApp>,
    archive: archive_view::ArchiveView,
    collision: collision_ui::CollisionControls,
    selected: PressureImplementation,
    domain_side: f64,
    requested_dt: f64,
    material_mode: bool,
    history_budget_mib: usize,
    recording_database_mib: usize,
    recording_record_mib: usize,
    recording_directory: String,
    materials: settings::MaterialControls,
    size: u64,
    steps: usize,
    tight: bool,
    threads: usize,
    thread_limit: usize,
    automatic_threads: bool,
    emitter: RotatingCylinderEmitter,
    emitter_controls: emitter_controls::EmitterControls,
    cylinder: bool,
    show_workers: bool,
    worker: Option<Worker>,
    scheduler: Option<Arc<rheon::CpuScheduler>>,
    rows: Vec<ResultRow>,
    textures: Vec<egui::TextureHandle>,
    status: String,
    running_request: Option<Request>,
    closing: bool,
    history: Option<Arc<LiveHistory>>,
    viewport: viewport::Viewport,
    inspector_open: bool,
    timeline_step: usize,
    track: usize,
    follow_live: bool,
    playing: bool,
    play_time: f64,
    playback_speed: f64,
}
impl Default for App {
    fn default() -> Self {
        let thread_limit = available_threads();
        Self {
            ai_comparison: None,
            archive: archive_view::ArchiveView::default(),
            collision: collision_ui::CollisionControls::default(),
            selected: PressureImplementation::default(),
            domain_side: 1.0,
            requested_dt: 0.02,
            material_mode: true,
            history_budget_mib: history::HISTORY_BUDGET / (1024 * 1024),
            recording_database_mib: 1024,
            recording_record_mib: 64,
            recording_directory: ".generated/recordings".into(),
            materials: settings::MaterialControls::default(),
            size: 32,
            steps: 1024,
            tight: false,
            threads: thread_limit,
            thread_limit,
            automatic_threads: true,
            emitter_controls: emitter_controls::EmitterControls::default(),
            emitter: RotatingCylinderEmitter {
                pressure_heads: [1000.0, 800.0],
                ..RotatingCylinderEmitter::default()
            },
            cylinder: true,
            show_workers: true,
            worker: None,
            scheduler: None,
            rows: Vec::new(),
            textures: Vec::new(),
            status: "Ready — rotating two-ended pressure emitter.".into(),
            running_request: None,
            closing: false,
            history: None,
            viewport: viewport::Viewport::default(),
            inspector_open: false,
            timeline_step: 0,
            track: 0,
            follow_live: true,
            playing: false,
            play_time: 0.0,
            playback_speed: 1.0,
        }
    }
}
impl App {
    fn sidebar(&mut self, ctx: &egui::Context) -> egui::Rect {
        egui::SidePanel::left("settings")
            .exact_width(280.0)
            .resizable(false)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.inspector_open, false, "Settings");
                    ui.selectable_value(&mut self.inspector_open, true, "Inspector");
                });
                self.archive.controls(
                    ui,
                    self.worker.is_none(),
                    rheon::recording::RecordingLimits {
                        max_database_bytes: (self.recording_database_mib as u64)
                            .saturating_mul(1024 * 1024),
                        max_record_bytes: self.recording_record_mib.saturating_mul(1024 * 1024),
                        max_records: 10001,
                    },
                );
                if self.archive.active {
                    return;
                }
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt("sidebar-content")
                    .show(ui, |ui| {
                        if self.inspector_open {
                            ui.heading("Numerical inspector");
                            if let Some(history) = &self.history {
                                if self
                                    .running_request
                                    .as_ref()
                                    .is_some_and(|r| r.material.is_none())
                                {
                                    if let Some(inspector) =
                                        history.carrier_inspectors.get(self.track)
                                    {
                                        inspector.ui(ui);
                                    }
                                } else {
                                    history.inspector.ui(ui);
                                }
                            } else {
                                ui.label("Start a simulation or open a numerical archive");
                            }
                        } else {
                            self.controls(ui);
                        }
                    });
            })
            .response
            .rect
    }
    fn reset_timeline(&mut self) {
        self.history = None;
        self.running_request = None;
        self.rows.clear();
        self.textures.clear();
        self.scheduler = None;
        self.playing = false;
        self.timeline_step = 0;
        self.track = 0;
        self.follow_live = true;
    }
    fn refresh_threads(&mut self, available: usize) {
        self.thread_limit = available.max(1);
        self.threads = if self.automatic_threads {
            self.thread_limit
        } else {
            self.threads.clamp(1, self.thread_limit)
        };
    }
    fn start(&mut self, all: bool) {
        if self.worker.is_some() {
            return;
        }
        self.refresh_threads(available_threads());
        let (Some(database_bytes), Some(record_bytes), Some(max_records)) = (
            u64::try_from(self.recording_database_mib)
                .ok()
                .and_then(|n| n.checked_mul(1024 * 1024)),
            self.recording_record_mib.checked_mul(1024 * 1024),
            self.steps.checked_add(1),
        ) else {
            self.status = "Recording limit overflow".into();
            return;
        };
        let recording_limits = rheon::recording::RecordingLimits {
            max_database_bytes: database_bytes,
            max_record_bytes: record_bytes,
            max_records,
        };
        if let Err(e) = recording_limits.validate() {
            self.status = e.to_string();
            return;
        }
        let request = Request {
            collision_mesh: self.collision.mesh.clone(),
            size: self.size,
            steps: self.steps,
            tight: self.tight,
            threads: self.threads,
            methods: if all {
                PressureImplementation::ALL.to_vec()
            } else {
                vec![self.selected]
            },
            emitter: self.cylinder.then_some(self.emitter),
            observe: self.show_workers,
            history_budget_mib: self.history_budget_mib,
            domain_side: self.domain_side,
            requested_dt: self.requested_dt,
            material: self.material_mode.then_some(self.materials.config),
            material_color: self.materials.color(),
            recording_limits,
            recording_directory: self.recording_directory.clone(),
            #[cfg(test)]
            progress_gate: None,
        };
        match Worker::start(request.clone()) {
            Ok(worker) => {
                self.scheduler = Some(Arc::clone(&worker.scheduler));
                self.history = Some(Arc::clone(&worker.history));
                self.worker = Some(worker);
                self.running_request = Some(request);
                self.rows.clear();
                self.textures.clear();
                self.timeline_step = 0;
                self.track = 0;
                self.follow_live = true;
                self.playing = false;
                self.status = "Simulating — accepted steps appear live.".into();
            }
            Err(e) => self.status = format!("Cannot start worker: {e}"),
        }
    }
    fn cancel(&self) {
        if let Some(w) = &self.worker {
            w.cancel.store(true, Ordering::Relaxed);
        }
    }
    fn poll(&mut self, ctx: &egui::Context) {
        if self.worker.as_ref().is_some_and(|w| w.handle.is_finished()) {
            let worker = self.worker.take().expect("finished worker");
            let cancelled = worker.cancel.load(Ordering::Relaxed);
            match worker.handle.join() {
                Ok(Ok(_)) if cancelled => {
                    self.status = "Cancelled; accepted preview frames retained".into()
                }
                Ok(Ok(rows)) => {
                    let size = self.running_request.as_ref().expect("worker request").size as usize;
                    self.textures = rows
                        .iter()
                        .map(|r| {
                            ctx.load_texture(
                                r.method.id(),
                                egui::ColorImage::from_gray([size, size], &r.pixels),
                                egui::TextureOptions::NEAREST,
                            )
                        })
                        .collect();
                    self.rows = rows;
                    self.status = "Complete — scrub or play the recorded simulation.".into();
                    if self.rows.windows(2).any(|w| w[0].time != w[1].time) {
                        self.status
                            .push_str(" Solver tracks reached different simulated times.");
                    }
                }
                Ok(Err(e)) => {
                    self.status =
                        format!("Simulation stopped: {e}. Accepted preview frames retained.")
                }
                Err(_) => self.status = "Worker failed; accepted preview frames retained.".into(),
            }
        }
    }
    fn selected_frame(&mut self) -> (usize, Option<Arc<PreviewFrame>>) {
        let Some(history) = &self.history else {
            return (0, None);
        };
        if self.follow_live {
            self.track = history.active_track.load(Ordering::Relaxed);
        }
        if self.playing {
            loop {
                let Ok((count, Some(next))) =
                    history.view(self.track, self.timeline_step + 1, false)
                else {
                    break;
                };
                if self.timeline_step + 1 >= count || next.time > self.play_time {
                    break;
                }
                self.timeline_step += 1;
            }
        }
        match history.view(self.track, self.timeline_step, self.follow_live) {
            Ok((count, frame)) => {
                if let Some(frame) = &frame {
                    self.timeline_step = frame.step;
                    {
                        let inspector = if frame.material_color.is_some() {
                            &history.inspector
                        } else {
                            &history.carrier_inspectors[self.track]
                        };
                        if let Err(e) = inspector.ensure_timestep(frame.step) {
                            self.status =
                                format!("Selected numerical step {} unavailable: {e}", frame.step);
                        }
                    }
                }
                if self.playing && self.timeline_step + 1 >= count && self.worker.is_none() {
                    self.playing = false;
                }
                (count, frame)
            }
            Err(e) => {
                self.status = e;
                (0, None)
            }
        }
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Rheon");
        ui.label(
            egui::RichText::new("LIVE FLUID LABORATORY")
                .small()
                .color(egui::Color32::from_rgb(90, 165, 200)),
        );
        ui.add_space(10.0);
        ui.add_enabled_ui(self.worker.is_none() && !self.closing, |ui| {
            egui::CollapsingHeader::new("Domain & environment").default_open(false).show(ui, |ui| {
                let previous_side=self.domain_side;
                if self.material_mode {self.materials.domain(ui, &mut self.domain_side);} else {
                    settings::quantity(ui,"Domain side",&mut self.domain_side,"m",0.01,0.001..=100.0);
                    ui.small("Original demo: filled carrier fluid throughout the box.");
                }
                if self.domain_side!=previous_side {
                    let scale=self.domain_side/previous_side;
                    self.emitter.center=self.emitter.center.map(|v|v*scale);
                    self.emitter.half_length*=scale;self.emitter.radius*=scale;self.emitter.nozzle_depth*=scale;
                }
            });
            if self.collision.show(ui, self.domain_side, self.material_mode) { self.reset_timeline(); }
            egui::CollapsingHeader::new("Fluid material").default_open(true).show(ui, |ui| {
                ui.add_enabled_ui(self.material_mode,|ui|self.materials.fluid(ui));
            });
            egui::CollapsingHeader::new("Emitter").default_open(false).show(ui, |ui| {
                ui.checkbox(&mut self.cylinder,"Rotating cylinder / two end jets");
                if self.cylinder {
                    self.emitter_controls.show(ui,&mut self.emitter,self.domain_side,if self.material_mode {self.materials.config.material.density}else{1.25});
                } else if self.material_mode {ui.small("Emission off; only the initial domain contents evolve.");}
                else {ui.small("Original smoke source; turns off halfway.");}
            });
            egui::CollapsingHeader::new("Simulation").default_open(false).show(ui, |ui| {
                if ui.checkbox(&mut self.material_mode,"Liquid / gas material simulation").changed() {
                    self.emitter.pressure_heads=if self.material_mode {[1000.0,800.0]} else {[0.12,0.08]};
                    if !self.material_mode {self.size=self.size.min(128);}
                }
                ui.horizontal(|ui| {ui.label("Preview grid");ui.add(egui::DragValue::new(&mut self.size).range(4..=if self.material_mode {512}else{128}).suffix("³"));});
                ui.horizontal(|ui| {ui.label("Steps");ui.add(egui::DragValue::new(&mut self.steps).range(1..=10000));});
                {
                    ui.collapsing("Numerical recording (every accepted step)", |ui| {
                        ui.label("Local recording directory");
                        ui.text_edit_singleline(&mut self.recording_directory);
                        ui.add(egui::DragValue::new(&mut self.recording_database_mib).range(1..=16384).suffix(" MiB database cap"));
                        ui.add(egui::DragValue::new(&mut self.recording_record_mib).range(1..=64).suffix(" MiB per publication"));
                        ui.small("Retention: keep all; stop at limits. Caps apply per track/database; comparisons create separate archives. Record quota = requested steps + constructor. Files persist after reset; delete explicitly outside Rheon.");
                        ui.small("Rollback journal uses additional temporary disk, at most approximately one database plus sector/page overhead. SQLite/allocator memory is outside the payload limit.");
                    });
                }
                ui.small("Carrier inventory: native face velocity, cell appearance tracer, accepted interval pressure. Material inventory: particle attributes and preintegration diagnostics. Other Rheon session/lab backends are not recorded here.");
                ui.horizontal(|ui| {ui.label("Timeline budget");ui.add(egui::DragValue::new(&mut self.history_budget_mib).range(16..=4096).suffix(" MiB"));});
                if let Ok(plan)=LiveHistory::plan(self.size as usize,self.steps,1,self.history_budget_mib.saturating_mul(1024*1024)) {ui.small(format!("Stored preview {}³ · {:.1} MiB for every step",plan.edge,plan.bytes as f64/1048576.0));}
                ui.small("Particle samples control physical detail; preview voxels control the image.");
                settings::quantity(ui,"Maximum step",&mut self.requested_dt,"s",0.001,0.000001..=0.1);
                ui.small("Accepted time steps adapt to physical stability limits.");
                if self.material_mode {
                    ui.horizontal(|ui| {ui.label("Particle samples / side");ui.add(egui::DragValue::new(&mut self.materials.config.samples_per_axis).range(4..=64));});
                    ui.small(format!("Particle spacing {:.4} m",self.domain_side/self.materials.config.samples_per_axis as f64));
                    ui.horizontal(|ui| {ui.label("Particle limit");ui.add(egui::DragValue::new(&mut self.materials.config.max_particles).range(32768..=262144));});
                    settings::quantity(ui,"Liquid preview stiffness",&mut self.materials.config.liquid_sound_speed,"m/s",1.0,1.0..=100.0).on_hover_text("Reduced acoustic speed for a weakly compressible preview; higher stiffness needs shorter steps. Gas pressure uses the ideal-gas equation.");
                } else {
                    egui::ComboBox::from_id_salt("implementation").selected_text(self.selected.label()).width(240.0).show_ui(ui,|ui| {
                        for method in PressureImplementation::ALL {ui.selectable_value(&mut self.selected,method,method.label());}
                    });
                    ui.checkbox(&mut self.tight,"Tight pressure tolerance");
                }
            });
            egui::CollapsingHeader::new("CPU workers").default_open(false).show(ui, |ui| {
                if ui.checkbox(&mut self.automatic_threads,"Automatic hardware detection").changed() {self.refresh_threads(available_threads());}
                ui.horizontal(|ui| {ui.add_enabled(!self.automatic_threads,egui::DragValue::new(&mut self.threads).range(1..=self.thread_limit));ui.label(format!("/ {} available",self.thread_limit));});
                ui.checkbox(&mut self.show_workers,"Orange worker voxel boxes").on_hover_text("Sampled actual worker positions; diagnostics add calculation overhead.");
            });
            ui.add_space(8.0);
            if ui.add_sized([ui.available_width(),32.0],egui::Button::new("▶ Simulate")).clicked() {self.start(false);}
            if ui.button("Reset timeline (keep mesh)").clicked() { self.reset_timeline(); self.status = "Ready to repeat simulation with accepted mesh".into(); }
            if !self.material_mode && ui.button("Compare both pressure solvers").clicked() {self.start(true);}
            if ui.add_enabled(self.worker.is_none(), egui::Button::new("Compare Rheon / trained AI")).clicked() {
                let mut comparison = ai_comparison::ComparisonApp::new(String::new(),std::env::current_dir().unwrap_or_default().join(".generated/ai-comparison"));
                comparison.set_steps(self.steps.clamp(1,512));
                self.ai_comparison = Some(comparison);
            }

        });
        if let Some(w) = &self.worker {
            ui.add_space(8.0);
            let n = w.progress.load(Ordering::Relaxed);
            ui.add(
                egui::ProgressBar::new(n as f32 / w.total as f32)
                    .text(format!("{n}/{} steps", w.total)),
            );
            if ui.button("■ Cancel simulation").clicked() {
                self.cancel();
            }
            if let Some((_, step, elapsed)) = w.history.timing() {
                ui.label(format!("Calculating step {step} · {elapsed:.3} s"));
            }
        }
        ui.label(&self.status);
        egui::CollapsingHeader::new("Viewport & navigation").default_open(false).show(ui, |ui| {
            ui.checkbox(&mut self.viewport.touchpad_mode,"Touchpad scroll / two-finger orbit").on_hover_text("Enable when your touchpad reports wheel events. Shift pans; Ctrl zooms. Native pinch is supported.");
            ui.checkbox(&mut self.viewport.camera.orthographic,"Orthographic");
            if ui.button("Frame scene [Home]").clicked() {self.viewport.camera=viewport::OrbitCamera::default();}
            ui.horizontal(|ui| {ui.label("Surface threshold");ui.add(egui::Slider::new(&mut self.viewport.threshold,1..=80).show_value(false));});
            ui.label("Mouse: middle drag orbit, Shift+middle pan, wheel zoom.");
            ui.label("Touchpad: left drag orbit, Shift+left pan, Ctrl+left zoom. Two-finger orbit, Shift-scroll pan, pinch/Ctrl-scroll zoom.");
            ui.label("1 front · 3 right · 7 top · 5 perspective/ortho · Home frame");
        });
        egui::CollapsingHeader::new("Calculation diagnostics")
            .default_open(false)
            .show(ui, |ui| {
                if let Some(scheduler) = &self.scheduler {
                    let samples = scheduler.snapshot();
                    let timed_ns = samples
                        .iter()
                        .map(|s| s.total_wall_nanoseconds as f64)
                        .sum::<f64>();
                    for batch in samples {
                        ui.label(format!(
                            "{:?}: {} cells/job · {} jobs",
                            batch.stage, batch.batch_cells, batch.tasks
                        ));
                        if timed_ns > 0.0 {
                            ui.small(format!(
                                "{:.1}% of timed kernels · {:.3} s",
                                100.0 * batch.total_wall_nanoseconds as f64 / timed_ns,
                                batch.total_wall_nanoseconds as f64 * 1e-9
                            ));
                        }
                    }
                    ui.small("Whole-pass batch tuning; timing shares exclude unrecorded work.");
                }
                if let Some(history) = &self.history {
                    if let Ok((_, Some(frame))) =
                        history.view(self.track, self.timeline_step, self.follow_live)
                    {
                        if frame.material_color.is_some() {
                            ui.small(format!(
                                "Occupied preview cells: {}",
                                frame.nonzero_tracer_cells
                            ));
                        } else {
                            let edge = self.running_request.as_ref().map_or(self.size, |r| r.size)
                                as usize;
                            ui.small(format!(
                                "Nonzero tracer: {} / {} cells",
                                frame.nonzero_tracer_cells,
                                edge.pow(3)
                            ));
                            ui.small("Pressure includes clear carrier fluid throughout the box.");
                        }
                    }
                    ui.small(format!(
                        "Preview {}³ · history ≤ {:.1} MiB",
                        history.preview_edge,
                        history.planned_bytes as f64 / 1048576.0
                    ));
                }
            });
        if !self.rows.is_empty() {
            egui::CollapsingHeader::new("Simulation results")
                .default_open(false)
                .show(ui, |ui| {
                    for row in &self.rows {
                        ui.label(format!(
                            "Compute {:.3} s · simulated {:.4} s",
                            row.seconds, row.time
                        ));
                        if let Some(r) = row.particle_report {
                            ui.label(format!(
                                "{} particles · mass {:.6} kg",
                                r.particles, r.total_mass
                            ));
                            ui.label(format!("Accepted max speed {:.3} m/s", r.max_speed));
                            ui.label(format!(
                                "Max pressure used for forces {:.1} Pa",
                                r.max_pressure
                            ))
                            .on_hover_text(
                                "Evaluated before the position, velocity and wall-temperature update; \
                                 not pressure recomputed on the displayed accepted particles.",
                            );
                        } else {
                            ui.label(row.method.label());
                            ui.label(format!(
                                "{} iterations · divergence {:.2e}",
                                row.iterations, row.divergence
                            ));
                        }
                        ui.small(format!("Arrays {:.2} MiB", row.bytes as f64 / 1048576.0));
                    }
                });
        }
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(comparison) = self.ai_comparison.as_mut() {
            if !comparison.show(ctx) {
                self.ai_comparison = None;
            }
            return;
        }
        self.poll(ctx);
        if ctx.input(|i| i.viewport().close_requested()) && self.worker.is_some() {
            self.closing = true;
            self.cancel();
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing && self.worker.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if self.archive.active {
            self.archive.show(ctx);
            return;
        }
        if self.playing {
            self.play_time += self.playback_speed * ctx.input(|i| i.stable_dt.min(0.1)) as f64;
        }
        let (count, frame) = self.selected_frame();
        self.sidebar(ctx);
        if self.archive.active {
            ctx.request_repaint();
            return;
        }
        egui::TopBottomPanel::bottom("timeline")
            .min_height(106.0)
            .show(ctx, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(if self.playing {
                            "❚❚ Pause"
                        } else {
                            "▶ Play"
                        })
                        .clicked()
                    {
                        self.follow_live = false;
                        self.playing = !self.playing;
                        if self.timeline_step + 1 >= count {
                            self.timeline_step = 0;
                            self.play_time = 0.0;
                        } else {
                            self.play_time = frame.as_ref().map_or(0.0, |f| f.time);
                        }
                    }
                    if ui.button("◀ Step").clicked() {
                        self.follow_live = false;
                        self.playing = false;
                        self.timeline_step = self.timeline_step.saturating_sub(1);
                    }
                    if ui.button("Step ▶").clicked() {
                        self.follow_live = false;
                        self.playing = false;
                        self.timeline_step = (self.timeline_step + 1).min(count.saturating_sub(1));
                    }
                    ui.checkbox(&mut self.follow_live, "Follow live");
                    if self.follow_live {
                        self.playing = false;
                    }
                    ui.add(
                        egui::DragValue::new(&mut self.playback_speed)
                            .speed(0.1)
                            .range(0.1..=4.0)
                            .suffix("× playback"),
                    );
                    if let Some(request) = &self.running_request
                        && request.material.is_none()
                    {
                        egui::ComboBox::from_id_salt("timeline-track")
                            .selected_text(request.methods[self.track].id())
                            .show_ui(ui, |ui| {
                                for (track, method) in request.methods.iter().enumerate() {
                                    if ui
                                        .selectable_value(&mut self.track, track, method.id())
                                        .clicked()
                                    {
                                        self.follow_live = false;
                                        self.playing = false;
                                        self.timeline_step = 0;
                                    }
                                }
                            });
                    }
                });
                ui.style_mut().spacing.slider_width = (ui.available_width() - 155.0).max(150.0);
                if ui
                    .add_enabled(
                        count > 0,
                        egui::Slider::new(&mut self.timeline_step, 0..=count.saturating_sub(1))
                            .text("Step"),
                    )
                    .changed()
                {
                    self.follow_live = false;
                    self.playing = false;
                }
                if let Some(frame) = &frame {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!(
                            "{} · Step {} / {}",
                            if frame.material_color.is_some() {
                                "Material fluid"
                            } else {
                                frame.method.id()
                            },
                            frame.step,
                            count.saturating_sub(1)
                        ));
                        ui.separator();
                        ui.label(format!("Simulated {}", settings::duration(frame.time)));
                        ui.separator();
                        ui.label(if frame.step == 0 {
                            "Initial state".into()
                        } else {
                            format!("Step calculation {}", settings::duration(frame.seconds))
                        });
                        if let Some(r) = frame.particle_report {
                            ui.separator();
                            ui.label(format!(
                                "{} particles · {:.6} kg · {:.1} °C",
                                r.particles,
                                r.total_mass,
                                r.mean_temperature_kelvin - 273.15
                            ));
                        }
                        if let Some(report) = frame.report {
                            ui.separator();
                            ui.label(format!(
                                "{} pressure iterations · divergence {:.2e}",
                                report.pressure.iterations, report.actual_divergence_max
                            ));
                        }
                    });
                }
            });
        let workers = if self.follow_live && self.show_workers {
            self.worker
                .as_ref()
                .and_then(|w| w.observer.as_ref())
                .map_or_else(Vec::new, |o| o.snapshot())
        } else {
            Vec::new()
        };
        let domain_side = self
            .running_request
            .as_ref()
            .map_or(self.domain_side, |r| r.domain_side);
        self.viewport.domain_side = domain_side;
        let emitter = self
            .running_request
            .as_ref()
            .map_or(self.cylinder.then_some(self.emitter), |r| r.emitter)
            .map(|mut e| {
                e.center = e.center.map(|v| v / domain_side);
                e.half_length /= domain_side;
                e.radius /= domain_side;
                e.nozzle_depth /= domain_side;
                e.motion_velocity = e.motion_velocity.map(|v| v / domain_side);
                e.emitted_velocity = e.emitted_velocity.map(|v| v.map(|x| x / domain_side));
                e
            });
        self.viewport.collision_mesh = self
            .running_request
            .as_ref()
            .map_or_else(|| self.collision.mesh.clone(), |r| r.collision_mesh.clone());
        let grid = self.running_request.as_ref().map_or(self.size, |r| r.size) as usize;
        let (_, frame) = self.selected_frame();
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                self.viewport.show(
                    ui,
                    frame,
                    emitter,
                    &workers,
                    grid,
                    self.history.as_ref().map(|h| &h.inspector),
                )
            });
        if self.worker.is_some() || self.playing {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}
impl Drop for App {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.cancel.store(true, Ordering::Relaxed);
            let _ = worker.handle.join();
        }
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Rheon / Live 3D",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1250.0, 820.0])
                .with_min_inner_size([900.0, 650.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals::dark();
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(22, 98, 138);
            style.spacing.item_spacing = egui::vec2(8.0, 7.0);
            cc.egui_ctx.set_style(style);
            let mut app = App::default();
            let mut args = std::env::args().skip(1);
            let mut run = false;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--collision-mesh" => {
                        app.collision.path = args
                            .next()
                            .filter(|p| !p.is_empty() && !p.starts_with("--"))
                            .ok_or("--collision-mesh requires a local path")?;
                    }
                    "--collision-policy" => match args.next().as_deref() {
                        Some("solid") => app.collision.solid = true,
                        Some("surface") => app.collision.solid = false,
                        _ => return Err("--collision-policy requires solid or surface".into()),
                    },
                    "--collision-run" => {
                        run = true;
                        app.steps = 180;
                    }
                    _ => return Err(format!("Unknown argument: {arg}").into()),
                }
            }
            if !app.collision.path.is_empty() {
                app.collision
                    .apply(app.domain_side)
                    .map_err(std::io::Error::other)?;
            }
            if run {
                app.start(false);
            }
            Ok(Box::new(app))
        }),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn collision_run_retains_geometry_and_reset_repeats_real_worker() {
        let mesh = Arc::new(
            rheon::CollisionMesh::new(
                vec![[0.1, 0.3, 0.1], [0.9, 0.3, 0.1], [0.1, 0.3, 0.9]],
                vec![[0, 1, 2]],
                rheon::MeshPolicy::TwoSidedSurface,
            )
            .unwrap(),
        );
        let mut app = App::default();
        app.steps = 2;
        app.size = 8;
        app.collision.mesh = Some(mesh.clone());
        app.start(false);
        assert!(Arc::ptr_eq(
            app.running_request
                .as_ref()
                .unwrap()
                .collision_mesh
                .as_ref()
                .unwrap(),
            &mesh
        ));
        let worker = app.worker.take().unwrap();
        wait_until(|| worker.handle.is_finished());
        assert_eq!(worker.handle.join().unwrap().unwrap().len(), 1);
        assert_eq!(app.history.as_ref().unwrap().view(0, 0, true).unwrap().0, 3);
        app.collision.mesh = None;
        assert!(
            app.running_request
                .as_ref()
                .unwrap()
                .collision_mesh
                .is_some()
        );
        app.reset_timeline();
        assert!(app.history.is_none());
        assert!(app.running_request.is_none());
        app.collision.mesh = Some(mesh);
        app.start(false);
        let worker = app.worker.take().unwrap();
        wait_until(|| worker.handle.is_finished());
        assert!(worker.handle.join().unwrap().is_ok());
        app.material_mode = false;
        app.start(false);
        assert!(app.worker.is_none());
        assert!(app.status.contains("Collision mesh requires material"));
    }
    #[test]
    fn material_worker_accepts_and_captures_a_genuine_256_cube_preview() {
        let request = Request {
            collision_mesh: None,
            size: 256,
            steps: 1,
            tight: false,
            threads: available_threads().min(2),
            methods: vec![PressureImplementation::default()],
            emitter: Some(RotatingCylinderEmitter {
                radius: 0.2,
                pressure_heads: [100000.0; 2],
                ..RotatingCylinderEmitter::default()
            }),
            observe: true,
            history_budget_mib: 96,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: Some(rheon::ParticleFluidConfig::default()),
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            progress_gate: None,
        };
        let worker = Worker::start(request).unwrap();
        wait_until(|| worker.handle.is_finished());
        let history = Arc::clone(&worker.history);
        assert_eq!(worker.handle.join().unwrap().unwrap().len(), 1);
        let (count, frame) = history.view(0, 0, true).unwrap();
        let frame = frame.unwrap();
        assert_eq!(count, 2);
        assert_eq!(frame.edge, 256);
        assert_eq!(frame.density.len(), 256_usize.pow(3));
        assert!(frame.density.iter().any(|v| *v > 0));
    }
    #[test]
    fn archive_opening_frame_suppresses_the_previous_live_inspector() {
        let ctx = egui::Context::default();
        let mut app = App::default();
        app.inspector_open = true;
        app.history = Some(LiveHistory::with_budget(4, 1, 1, 16 * 1024 * 1024).unwrap());
        app.archive.path = std::env::temp_dir()
            .join(format!(
                "rheon-absent-transition-{}.sqlite",
                std::process::id()
            ))
            .to_str()
            .unwrap()
            .into();
        let render = |app: &mut App, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1400.0, 1600.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    app.sidebar(ctx);
                },
            )
        };
        let click = |app: &mut App, label: &str| {
            let _ = render(app, vec![]);
            let output = render(app, vec![]);
            let pos = output
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == label => {
                        Some(t.pos + t.galley.size() * 0.5)
                    }
                    _ => None,
                })
                .unwrap_or_else(|| panic!("missing sidebar control {label}"));
            let _ = render(
                app,
                vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                ],
            );
            render(
                app,
                vec![egui::Event::PointerButton {
                    pos,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: Default::default(),
                }],
            )
        };
        let _ = click(&mut app, "Open numerical archive");
        let output = render(&mut app, vec![]);
        assert!(output.shapes.iter().any(
            |s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Numerical inspector")
        ));
        let output = click(&mut app, "Open recorded run");
        assert!(app.archive.active);
        assert!(!output.shapes.iter().any(
            |s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Numerical inspector")
        ));
        assert!(app.archive.inspector.selected_identity().is_none());
    }
    #[test]
    fn docked_inspector_cannot_overlap_viewport_at_supported_window_sizes() {
        for size in [egui::vec2(900.0, 650.0), egui::vec2(1250.0, 820.0)] {
            let ctx = egui::Context::default();
            let mut app = App::default();
            app.inspector_open = true;
            let mut dock = egui::Rect::NOTHING;
            let mut center = egui::Rect::NOTHING;
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                    ..Default::default()
                },
                |ctx| {
                    dock = app.sidebar(ctx);
                    center = egui::CentralPanel::default()
                        .show(ctx, |ui| {
                            ui.label("CAD viewport");
                        })
                        .response
                        .rect;
                },
            );
            assert!(dock.right() <= center.left());
            assert!(center.width() >= 600.0);
            assert!(dock.bottom() <= size.y);
            assert!(output.shapes.iter().any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Numerical inspector")));
            assert!(!output.shapes.iter().any(|s|s.clip_rect.left()<center.left() && s.clip_rect.right()>center.left()+1.0 && matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Numerical inspector")));
        }
    }
    #[test]
    fn sidebar_sections_collapse_and_preserve_their_state_through_real_egui_input() {
        let ctx = egui::Context::default();
        ctx.style_mut(|s| s.animation_time = 0.0);
        let mut app = App::default();
        let render = |app: &mut App, events| {
            ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 1500.0),
                    )),
                    events,
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default().show(ctx, |ui| app.controls(ui));
                },
            )
        };
        let text = |output: &egui::FullOutput| {
            output
                .shapes
                .iter()
                .filter_map(|s| match &s.shape {
                    egui::Shape::Text(t) => {
                        Some((t.galley.job.text.clone(), t.pos + t.galley.size() * 0.5))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>()
        };
        render(&mut app, vec![]);
        let visible = text(&render(&mut app, vec![]));
        assert!(visible.iter().any(|(t, _)| t == "Viscosity"));
        assert!(!visible.iter().any(|(t, _)| t == "Domain side"));
        for (section, child, open) in [
            ("Fluid material", "Viscosity", false),
            ("Domain & environment", "Domain side", true),
        ] {
            let pos = text(&render(&mut app, vec![]))
                .into_iter()
                .find(|(t, _)| t == section)
                .unwrap()
                .1;
            for pressed in [true, false] {
                render(
                    &mut app,
                    vec![
                        egui::Event::PointerMoved(pos),
                        egui::Event::PointerButton {
                            pos,
                            button: egui::PointerButton::Primary,
                            pressed,
                            modifiers: egui::Modifiers::default(),
                        },
                    ],
                );
            }
            for _ in 0..2 {
                assert_eq!(
                    text(&render(&mut app, vec![]))
                        .iter()
                        .any(|(t, _)| t == child),
                    open
                );
            }
        }
    }
    #[test]
    fn material_worker_publishes_live_frames_and_retains_immutable_physical_scale() {
        let (reached, first_step) = std::sync::mpsc::sync_channel(1);
        let (release, resume) = std::sync::mpsc::sync_channel(1);
        let request = Request {
            collision_mesh: None,
            size: 32,
            steps: 64,
            tight: false,
            threads: available_threads().min(4),
            methods: vec![PressureImplementation::default()],
            emitter: Some(RotatingCylinderEmitter {
                radius: 0.2,
                pressure_heads: [100000.0; 2],
                ..RotatingCylinderEmitter::default()
            }),
            observe: true,
            history_budget_mib: 96,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: Some(rheon::ParticleFluidConfig::default()),
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            progress_gate: Some(Arc::new(ProgressGate {
                reached,
                release: std::sync::Mutex::new(resume),
            })),
        };
        let worker = Worker::start(request.clone()).unwrap();
        let mut app = App::default();
        app.running_request = Some(request);
        app.history = Some(Arc::clone(&worker.history));
        app.worker = Some(worker);
        first_step.recv_timeout(Duration::from_secs(10)).unwrap();
        let (_, frame) = app.selected_frame();
        let frame = frame.unwrap();
        assert!(frame.particle_report.is_some());
        assert!(frame.report.is_none());
        assert!(!app.worker.as_ref().unwrap().handle.is_finished());
        app.follow_live = false;
        app.timeline_step = 0;
        release.send(()).unwrap();
        wait_until(|| app.worker.as_ref().unwrap().handle.is_finished());
        app.poll(&egui::Context::default());
        let (count, frame) = app.selected_frame();
        assert_eq!(count, 65);
        assert_eq!(frame.unwrap().time, 0.0);
        assert_eq!(app.rows.len(), 1);
        assert!(app.rows[0].particle_report.unwrap().total_mass > 0.0);
        let (_, latest) = app.history.as_ref().unwrap().view(0, 0, true).unwrap();
        assert!(latest.unwrap().density.iter().any(|value| *value > 0));
        app.domain_side = 0.1;
        assert_eq!(app.running_request.as_ref().unwrap().domain_side, 1.0);
    }
    #[test]
    fn live_emitter_frames_arrive_before_join_and_scrubbing_survives_background_work() {
        let (reached, first_step) = std::sync::mpsc::sync_channel(1);
        let (release, resume) = std::sync::mpsc::sync_channel(1);
        let request = Request {
            collision_mesh: None,
            size: 32,
            steps: 3,
            tight: false,
            threads: available_threads().min(2),
            methods: vec![PressureImplementation::SymmetricGaussSeidelPcgV1],
            emitter: Some(RotatingCylinderEmitter::default()),
            observe: true,
            history_budget_mib: 96,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: None,
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            progress_gate: Some(Arc::new(ProgressGate {
                reached,
                release: std::sync::Mutex::new(resume),
            })),
        };
        let worker = Worker::start(request.clone()).unwrap();
        let mut app = App::default();
        app.history = Some(Arc::clone(&worker.history));
        app.worker = Some(worker);
        app.running_request = Some(request);
        first_step.recv_timeout(Duration::from_secs(10)).unwrap();
        assert!(!app.worker.as_ref().unwrap().handle.is_finished());
        let (count, frame) = app.selected_frame();
        assert_eq!(count, 2);
        let frame = frame.unwrap();
        assert_eq!(frame.step, 1);
        assert!(frame.seconds > 0.0);
        assert!(frame.density.iter().any(|v| *v > 0));
        assert!(frame.report.is_some());
        app.follow_live = false;
        app.timeline_step = 0;
        release.send(()).unwrap();
        wait_until(|| app.worker.as_ref().unwrap().handle.is_finished());
        app.poll(&egui::Context::default());
        let (count, held) = app.selected_frame();
        assert_eq!(count, 4);
        assert_eq!(held.unwrap().step, 0);
        app.follow_live = true;
        assert_eq!(app.selected_frame().1.unwrap().step, 3);
        assert_eq!(app.rows.len(), 1);
    }
    #[test]
    fn automatic_threads_follow_hardware_and_manual_counts_stay_within_the_limit() {
        let mut app = App::default();
        assert!(app.automatic_threads);
        assert_eq!(app.threads, available_threads());
        app.refresh_threads(24);
        assert_eq!((app.threads, app.thread_limit), (24, 24));
        app.automatic_threads = false;
        app.threads = 12;
        app.refresh_threads(32);
        assert_eq!((app.threads, app.thread_limit), (12, 32));
        app.refresh_threads(4);
        assert_eq!((app.threads, app.thread_limit), (4, 4));
        app.automatic_threads = true;
        app.refresh_threads(16);
        assert_eq!((app.threads, app.thread_limit), (16, 16));
        app.refresh_threads(0);
        assert_eq!((app.threads, app.thread_limit), (1, 1));
    }
    fn wait_until(mut condition: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(10);
        while !condition() {
            assert!(Instant::now() < deadline, "worker lifecycle timed out");
            std::thread::yield_now();
        }
    }
    #[test]
    fn cancel_after_progress_joins_without_rows_and_allows_restart() {
        let mut app = App::default();
        app.size = 8;
        app.steps = 4;
        app.cylinder = false;
        let (reached, first_step) = std::sync::mpsc::sync_channel(1);
        let (release, resume) = std::sync::mpsc::sync_channel(1);
        let request = Request {
            collision_mesh: None,
            size: app.size,
            steps: app.steps,
            tight: app.tight,
            threads: app.threads,
            methods: PressureImplementation::ALL.to_vec(),
            emitter: None,
            observe: false,
            history_budget_mib: 96,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: None,
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            progress_gate: Some(Arc::new(ProgressGate {
                reached,
                release: std::sync::Mutex::new(resume),
            })),
        };
        app.running_request = Some(request.clone());
        app.worker = Some(Worker::start(request).unwrap());
        let progress = Arc::clone(&app.worker.as_ref().unwrap().progress);
        first_step
            .recv_timeout(Duration::from_secs(10))
            .expect("worker must reach its first accepted step");
        assert_eq!(progress.load(Ordering::Relaxed), 1);
        assert!(!app.worker.as_ref().unwrap().handle.is_finished());
        assert!(progress.load(Ordering::Relaxed) < app.worker.as_ref().unwrap().total);
        app.cancel();
        release.send(()).unwrap();
        wait_until(|| app.worker.as_ref().unwrap().handle.is_finished());
        assert!(progress.load(Ordering::Relaxed) < app.worker.as_ref().unwrap().total);
        let ctx = egui::Context::default();
        app.poll(&ctx);
        assert!(app.worker.is_none());
        assert!(app.rows.is_empty());
        assert!(app.textures.is_empty());
        assert!(app.status.contains("Cancelled { stage:"));
        app.size = 4;
        app.steps = 2;
        app.cylinder = false;
        app.selected = PressureImplementation::SymmetricGaussSeidelPcgV1;
        app.start(false);
        wait_until(|| app.worker.as_ref().unwrap().handle.is_finished());
        app.poll(&ctx);
        assert!(app.worker.is_none());
        assert_eq!(app.rows.len(), 1);
        assert_eq!(app.textures.len(), 1);
        assert_eq!(app.rows[0].method, app.selected);
        assert_eq!(app.rows[0].pixels.len(), 16);
    }
    #[test]
    fn dropping_app_cancels_and_joins_owned_worker() {
        let cancel = Arc::new(AtomicBool::new(false));
        let exited = Arc::new(AtomicBool::new(false));
        let worker_cancel = Arc::clone(&cancel);
        let worker_exited = Arc::clone(&exited);
        let handle = std::thread::spawn(move || {
            wait_until(|| worker_cancel.load(Ordering::Relaxed));
            worker_exited.store(true, Ordering::Relaxed);
            Ok(Vec::new())
        });
        let mut app = App::default();
        app.worker = Some(Worker {
            scheduler: rheon::CpuScheduler::new(),
            cancel: Arc::clone(&cancel),
            progress: Arc::new(AtomicUsize::new(0)),
            total: 1,
            history: LiveHistory::new(4, 1, 1).unwrap(),
            observer: None,
            handle,
        });
        drop(app);
        assert!(cancel.load(Ordering::Relaxed));
        assert!(exited.load(Ordering::Relaxed));
    }
    #[test]
    fn worker_runs_selected_and_equal_input_comparison() {
        let request = Request {
            collision_mesh: None,
            size: 8,
            steps: 4,
            tight: false,
            threads: available_threads().min(4),
            methods: PressureImplementation::ALL.to_vec(),
            emitter: None,
            observe: false,
            history_budget_mib: 96,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: None,
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            progress_gate: None,
        };
        let progress = AtomicUsize::new(0);
        let history = LiveHistory::with_budget(8, 4, 2, 96 * 1024 * 1024).unwrap();
        let rows = run_owned(
            request.clone(),
            &AtomicBool::new(false),
            &progress,
            &history,
            None,
            rheon::CpuScheduler::new(),
        )
        .unwrap();
        let mut ids = Vec::new();
        for inspector in &history.carrier_inspectors {
            assert_eq!(inspector.recorded_count().unwrap(), 5);
            inspector.verify().unwrap();
            inspector.select_timestep(1).unwrap();
            ids.push(inspector.selected_identity().unwrap());
        }
        assert_ne!(ids[0].0, ids[1].0);
        history.carrier_inspectors[1].select_timestep(4).unwrap();
        assert_eq!(
            history.carrier_inspectors[0].selected_identity(),
            Some(ids[0])
        );
        let mut cold = archive_view::ArchiveView::default();
        cold.path = history.carrier_inspectors[0]
            .archive_path()
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        cold.open(rheon::recording::RecordingLimits::default())
            .unwrap();
        cold.inspector.select_timestep(1).unwrap();
        assert_eq!(cold.inspector.selected_identity(), Some(ids[0]));
        let mut limited = request.clone();
        limited.recording_limits.max_records = 2;
        let partial = LiveHistory::with_budget(8, 4, 2, 96 * 1024 * 1024).unwrap();
        assert!(
            run_owned(
                limited,
                &AtomicBool::new(false),
                &AtomicUsize::new(0),
                &partial,
                None,
                rheon::CpuScheduler::new()
            )
            .err()
            .expect("record quota must stop worker")
            .contains("not confirmed recorded")
        );
        let reader = rheon::recording::RecordingReader::open(
            &partial.carrier_inspectors[0].archive_path().unwrap(),
            rheon::recording::RecordingLimits::default(),
        )
        .unwrap();
        reader.verify().unwrap();
        assert_eq!(reader.count().unwrap(), 2);
        assert_eq!(reader.status().unwrap().outcome, "failed");
        assert_eq!(reader.status().unwrap().accepted_generation, Some(2));
        assert_eq!(partial.view(0, 0, true).unwrap().0, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(progress.load(Ordering::Relaxed), 8);
        assert_eq!(rows[0].time, rows[1].time);
        assert_eq!(rows[0].bytes, rows[1].bytes);
        assert_eq!(rows[0].pixels.len(), 64);
        assert_eq!(rows[0].pixels, rows[1].pixels);
    }
    #[test]
    fn cancelled_worker_never_publishes_rows() {
        assert!(
            run(
                Request {
                    collision_mesh: None,
                    size: 8,
                    steps: 4,
                    tight: false,
                    threads: available_threads().min(4),
                    methods: PressureImplementation::ALL.to_vec(),
                    emitter: None,
                    observe: false,
                    history_budget_mib: 96,
                    domain_side: 1.0,
                    requested_dt: 0.02,
                    material: None,
                    material_color: [85, 190, 245],
                    recording_limits: rheon::recording::RecordingLimits::default(),
                    recording_directory: ".generated/recordings".into(),
                    progress_gate: None
                },
                &AtomicBool::new(true),
                &AtomicUsize::new(0)
            )
            .is_err()
        );
    }
    #[test]
    fn repeated_start_cannot_replace_active_work() {
        let mut app = App::default();
        app.size = 8;
        app.steps = 4;
        app.cylinder = false;
        app.start(false);
        let token = Arc::clone(&app.worker.as_ref().unwrap().cancel);
        app.start(true);
        assert!(Arc::ptr_eq(&token, &app.worker.as_ref().unwrap().cancel));
        assert_eq!(app.running_request.as_ref().unwrap().methods.len(), 1);
    }
    #[test]
    fn cancellation_before_publication_suppresses_already_finished_result() {
        let mut app = App::default();
        app.size = 4;
        app.steps = 2;
        app.cylinder = false;
        app.start(false);
        let deadline = Instant::now() + Duration::from_secs(10);
        while !app.worker.as_ref().unwrap().handle.is_finished() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(1));
        }
        app.cancel();
        app.poll(&egui::Context::default());
        assert!(app.worker.is_none());
        assert!(app.rows.is_empty());
        assert!(app.status.starts_with("Cancelled"));
    }
}
