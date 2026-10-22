//! Native live 3D laboratory. Numerical state stays owned by the worker; the
//! viewer consumes bounded immutable accepted previews and optional diagnostics.
#[path = "desktop/accepted_session.rs"]
mod accepted_session;
#[path = "desktop/ai_comparison/mod.rs"]
mod ai_comparison;
#[path = "desktop/build_identity.rs"]
mod build_identity;
#[path = "desktop/camera.rs"]
mod camera;
#[path = "desktop/carrier_execution.rs"]
mod carrier_execution;
#[cfg(test)]
#[path = "desktop/configured_comparison_tests.rs"]
mod configured_comparison_tests;
#[path = "desktop/configured_controls.rs"]
mod configured_controls;
#[cfg(test)]
#[path = "desktop/configured_producer_exit_tests.rs"]
mod configured_producer_exit_tests;
#[cfg(test)]
#[path = "desktop/configured_worker_tests.rs"]
mod configured_worker_tests;
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
#[path = "desktop/comparison_archive.rs"]
pub mod comparison_archive;
#[path = "desktop/comparison_field_view.rs"]
pub mod comparison_field_view;
#[path = "desktop/history.rs"]
mod history;
#[path = "desktop/identity.rs"]
mod identity;
#[path = "desktop/inspector.rs"]
mod inspector;
#[path = "desktop/particle_view.rs"]
mod particle_view;
#[path = "desktop/run_set_view.rs"]
mod run_set_view;
#[path = "desktop/run_sets.rs"]
pub mod run_sets;
#[path = "desktop/comparison.rs"]
pub mod saved_comparison;
use rheon::recording::comparison as native_comparison;
use saved_comparison as comparison;
#[path = "desktop/authoring_history.rs"]
mod authoring_history;
#[path = "desktop/force_run_policy.rs"]
mod force_run_policy;
#[cfg(test)]
#[path = "desktop/force_run_policy_tests.rs"]
mod force_run_policy_tests;
#[cfg(test)]
#[path = "desktop/force_runtime_integration_tests.rs"]
mod force_runtime_integration_tests;
#[cfg(test)]
#[path = "desktop/force_runtime_ui_tests.rs"]
mod force_runtime_ui_tests;
#[path = "desktop/initial_contents_ui.rs"]
mod initial_contents_ui;
#[cfg(test)]
#[path = "desktop/native_presentation_harness.rs"]
mod native_presentation_harness;
#[path = "desktop/scene_handles.rs"]
mod scene_handles;
#[path = "desktop/scene_ui.rs"]
mod scene_ui;
#[path = "desktop/scene_view.rs"]
mod scene_view;
#[path = "desktop/settings.rs"]
mod settings;
#[cfg(test)]
#[path = "desktop/setup_ui_tests.rs"]
mod setup_ui_tests;
#[cfg(test)]
#[path = "desktop/timeline_ui_tests.rs"]
mod timeline_ui_tests;
#[path = "desktop/transport_view.rs"]
mod transport_view;
#[path = "desktop/viewport.rs"]
mod viewport;
#[path = "desktop/voxel_material_authoring.rs"]
mod voxel_material_authoring;
#[path = "desktop/voxel_material_execution.rs"]
mod voxel_material_execution;
#[cfg(test)]
#[path = "desktop/voxel_material_integration_tests.rs"]
mod voxel_material_integration_tests;
#[path = "desktop/voxel_material_launch.rs"]
mod voxel_material_launch;
#[path = "desktop/voxel_material_setup.rs"]
mod voxel_material_setup;
use eframe::egui;
use history::{LiveHistory, PreviewFrame};
use rheon::{
    CalculationObserver, GridGeometry, PressureImplementation, PressureSettings,
    RotatingCylinderEmitter, Simulation, SimulationConfig, SmokeSource,
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Request {
    /// Empty means legacy on every track; otherwise one immutable option per track.
    force_fields: Vec<Option<rheon::ForceSnapshot>>,
    collision_mesh: Option<Arc<rheon::CollisionMesh>>,
    size: u64,
    steps: usize,
    tight: bool,
    threads: usize,
    methods: Vec<PressureImplementation>,
    emitter: Option<RotatingCylinderEmitter>,
    observe: bool,
    history_budget_mib: usize,
    preview_resolution: history::PreviewResolution,
    domain_side: f64,
    requested_dt: f64,
    material: Option<rheon::ParticleFluidConfig>,
    material_color: [u8; 3],
    recording_limits: rheon::recording::RecordingLimits,
    recording_directory: String,
    saved_set: Option<run_sets::RunSetRevision>,
    configured_set: Option<ConfiguredRequest>,
    /// Direct source run IDs to avoid even after producer-process restart.
    prior_run_ids: Vec<u64>,
    voxel: Option<voxel_material_execution::VoxelRequest>,
    manifest_receipt: Option<Arc<std::sync::Mutex<std::path::PathBuf>>>,
    #[cfg(test)]
    progress_gate: Option<Arc<ProgressGate>>,
    #[cfg(test)]
    failure_gate: Option<Arc<ProgressGate>>,
}
#[derive(Clone)]
struct ConfiguredRequest {
    revision: run_sets::configured::ConfiguredRunSetRevision,
    project_root: std::path::PathBuf,
    /// Filled once by the existing Worker with its own cancellation/child handles.
    runtime: Option<(Arc<AtomicBool>, accepted_session::model_process::ChildSlot)>,
}
impl Request {
    fn force_target(&self) -> force_run_policy::Target {
        self.setup_workflow().force_target(self.material.is_some())
    }
    fn fields_for_track(&self, track: usize) -> Option<&rheon::ForceSnapshot> {
        self.force_fields.get(track).and_then(Option::as_ref)
    }

    fn setup_workflow(&self) -> SetupWorkflow {
        if self.configured_set.is_some() {
            SetupWorkflow::AcceptedSession
        } else {
            SetupWorkflow::Preview
        }
    }
    fn preview_edge(&self) -> Result<usize, String> {
        self.preview_resolution
            .edge(self.domain_side, self.size as usize)
    }
    fn preview_history(&self) -> Result<Arc<LiveHistory>, String> {
        if self.voxel.is_some() {
            return LiveHistory::with_voxel_resolution(
                self.preview_edge()?,
                self.steps,
                self.history_budget_mib
                    .checked_mul(1024 * 1024)
                    .ok_or("Voxel history budget overflow")?,
                self.preview_resolution.exact(),
            );
        }
        LiveHistory::with_resolution(
            self.preview_edge()?,
            self.steps,
            self.methods.len(),
            self.history_budget_mib
                .checked_mul(1024 * 1024)
                .ok_or("Preview budget overflow")?,
            self.preview_resolution.exact(),
        )
    }
    fn track_label(&self, index: usize) -> String {
        if self.voxel.is_some() {
            return if index == 0 {
                voxel_material_execution::METHOD
            } else {
                "Unavailable voxel track"
            }
            .into();
        }
        if let Some(configured) = &self.configured_set {
            let Some(member) = configured.revision.members.get(index) else {
                return "Unavailable configured member".into();
            };
            match &member.spec.future.pressure {
                run_sets::configured::PressureIntent::NativeOnly => format!(
                    "{} · native {}",
                    member.member_id,
                    member
                        .spec
                        .constructor
                        .pressure_method
                        .implementation()
                        .id()
                ),
                run_sets::configured::PressureIntent::NativePrefixThenConfigured {
                    provider,
                    ..
                } => format!("{} · configured {}", member.member_id, provider.id),
            }
        } else {
            self.methods
                .get(index)
                .map_or("Unavailable track", |method| method.id())
                .into()
        }
    }
}
#[cfg(test)]
struct ProgressGate {
    reached: std::sync::mpsc::SyncSender<()>,
    release: std::sync::Mutex<std::sync::mpsc::Receiver<()>>,
}
struct ResultRow {
    method: PressureImplementation,
    label: String,
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
    child: Option<accepted_session::model_process::ChildSlot>,
    handle: JoinHandle<Result<Vec<ResultRow>, String>>,
}
impl Worker {
    fn cancel(&self) {
        if let Some(slot) = &self.child {
            accepted_session::model_process::cancel(slot, &self.cancel);
        } else {
            self.cancel.store(true, Ordering::Relaxed);
        }
    }
    fn start(mut request: Request) -> std::io::Result<Self> {
        validate_request(&request).map_err(std::io::Error::other)?;
        let cancel = Arc::new(AtomicBool::new(false));
        let child = request.configured_set.as_mut().map(|configured| {
            let slot = Arc::new(std::sync::Mutex::new(None));
            configured.runtime = Some((Arc::clone(&cancel), Arc::clone(&slot)));
            slot
        });
        let progress = Arc::new(AtomicUsize::new(0));
        let history = request.preview_history().map_err(std::io::Error::other)?;
        let observer = if request.observe {
            let h = request.domain_side / request.size as f64;
            let grid = GridGeometry::new([request.size; 3], [h; 3], [0.0; 3])
                .map_err(std::io::Error::other)?;
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
            child,
            handle,
        })
    }
}
fn available_threads() -> usize {
    std::thread::available_parallelism().map_or(1, usize::from)
}
fn configured_executor() -> run_sets::configured::ExecutorDeclaration {
    run_sets::configured::ExecutorDeclaration {
        id: "rheon-desktop-configured-session-v1".into(),
        source_revision: build_identity::executor_revision("desktop-configured-session"),
        build_provenance: build_identity::material_sha256().map(str::to_owned),
    }
}
fn validate_request(request: &Request) -> Result<(), String> {
    if request.voxel.is_some() {
        return voxel_material_execution::validate(request);
    }
    if !request.force_fields.is_empty() && request.force_fields.len() != request.methods.len() {
        return Err("Force snapshots must match the exact track count".into());
    }
    for fields in request.force_fields.iter().flatten() {
        rheon::recording::encode_force_metadata(fields).map_err(|e| e.to_string())?;
    }
    let authored = request.force_fields.iter().any(Option::is_some);
    let target = request.force_target();
    if authored
        && target == force_run_policy::Target::ParticleMaterial
        && request.methods.len() != 1
    {
        return Err("Authored particle material requires exactly one material track".into());
    }
    if let Some(set) = &request.saved_set {
        set.validate().map_err(|e| e.to_string())?;
        if set.members.len() != request.methods.len() {
            return Err("Saved member/track mismatch".into());
        }
        for (track, member) in set.members.iter().enumerate() {
            let intended = member.spec.force_snapshot().map_err(|e| e.to_string())?;
            let bytes = |f: Option<&rheon::ForceSnapshot>| {
                f.map(rheon::recording::encode_force_metadata)
                    .transpose()
                    .map_err(|e| e.to_string())
            };
            if bytes(intended.as_ref())? != bytes(request.fields_for_track(track))? {
                return Err("Saved member force snapshot mismatch".into());
            }
        }
    }
    let qualification = force_run_policy::PRODUCTION_QUALIFICATION;
    force_run_policy::check(target, authored, qualification).map_err(str::to_owned)?;

    if let Some(configured) = &request.configured_set {
        configured.revision.validate().map_err(|e| e.to_string())?;
        if request.saved_set.is_some()
            || request.material.is_some()
            || request.emitter.is_some()
            || request.collision_mesh.is_some()
            || configured.revision.members.len() != request.methods.len()
        {
            return Err(
                "Configured desktop execution requires its own closed accepted Session set".into(),
            );
        }
        for (member, method) in configured.revision.members.iter().zip(&request.methods) {
            let spec = &member.spec;
            spec.validate_execution(&configured_executor())
                .map_err(|e| e.to_string())?;
            if spec.constructor.counts_xyz != [request.size; 3]
                || spec.constructor.spacing_m != [request.domain_side / request.size as f64; 3]
                || spec.constructor.origin_m != [0.0; 3]
                || spec.constructor.pressure_method.implementation() != *method
                || spec.future.requested_steps != request.steps
                || spec.future.requested_dt_s.to_bits() != request.requested_dt.to_bits()
                || spec.future.recording.limits() != request.recording_limits
                || !spec.future.capture_constructor
            {
                return Err("Configured desktop supports up to two cubic zero-origin members with common requested horizon/budgets and constructor capture; cold inspection remains available".into());
            }
        }
    }
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
        512
    } else {
        rheon::MAX_GRID_EDGE
    };
    if !(1..=maximum).contains(&request.size)
        || !(1..=10000).contains(&request.steps)
        || request.methods.is_empty()
        || request.methods.len() > 2
        || (request.configured_set.is_none()
            && request.methods.len() == 2
            && request.methods[0] == request.methods[1])
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
    // Frozen native/configured intent retains its historical spacing; all new
    // requests, including private non-UI callers, obey the physical edit cap.
    if request.saved_set.is_none() && request.configured_set.is_none() {
        let samples = request
            .material
            .map_or(request.size as usize, |m| m.samples_per_axis);
        let actual = samples as f64 / request.domain_side;
        if actual > f64::from(rheon::MAX_CELLS_PER_METRE).next_up() {
            return Err(format!(
                "Requested physical sampling {actual:.6}/m exceeds 1000/m; adjust extent or counts"
            ));
        }
    }
    // Preflight before a worker, recorder or numerical field is allocated.
    let edge = request.preview_edge()?;
    LiveHistory::plan_resolution(
        edge,
        request.steps,
        request.methods.len(),
        request.history_budget_mib * 1024 * 1024,
        request.preview_resolution.exact(),
    )?;
    if request.material.is_none() {
        let grid = GridGeometry::new(
            [request.size; 3],
            [request.domain_side / request.size as f64; 3],
            [0.0; 3],
        )
        .map_err(|e| e.to_string())?;
        rheon::BufferPlan::for_grid(&grid, 192 * 1024 * 1024).map_err(|e| e.to_string())?;
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
    let history = request.preview_history()?;
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
    mut request: Request,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
    history: &LiveHistory,
) -> Result<Vec<ResultRow>, String> {
    if request.voxel.is_some() {
        return voxel_material_execution::run(request, cancel, progress, history);
    }
    if let Some(config) = request.material {
        return run_material(request, config, cancel, progress, history);
    }
    let mut saved_set = request.saved_set.take();
    let mut configured_set = request.configured_set.as_ref().map(|c| c.revision.clone());
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
        let frozen = saved_set
            .as_ref()
            .map(|set| set.members[track].spec.clone());
        let configured_frozen = configured_set
            .as_ref()
            .map(|set| set.members[track].spec.clone());
        let mut sim = if let Some(spec) = &configured_frozen {
            let runtime = request
                .configured_set
                .as_ref()
                .and_then(|c| c.runtime.as_ref())
                .ok_or("Configured execution requires the existing Worker's cancellation owner")?;
            let mut identity = identity::new_identity("configured-session");
            while request.prior_run_ids.contains(&identity.run_id) {
                identity = identity::new_identity("configured-session");
            }
            carrier_execution::CarrierExecution::Configured(Box::new(
                accepted_session::configured_track::ConfiguredTrack::prepare(
                    spec.clone(),
                    identity,
                    &configured_executor(),
                    Arc::clone(&runtime.0),
                )?,
            ))
        } else if let Some(spec) = &frozen {
            carrier_execution::CarrierExecution::Native(Box::new(
                spec.spec.create_simulation().map_err(|e| e.to_string())?,
            ))
        } else {
            carrier_execution::CarrierExecution::Native(Box::new(
                Simulation::with_implementation(grid, config, method).map_err(|e| e.to_string())?,
            ))
        };
        let path = new_recording_path(&request.recording_directory, method.id())?;
        let inspector = &history.carrier_inspectors[track];
        let mut recorder = if configured_frozen.is_some() {
            let configured = sim.configured().ok_or("Configured owner required")?;
            let snapshot = configured.snapshot();
            let run =
                run_sets::configured::archive::bound_configured_run(configured.frozen(), &snapshot)
                    .map_err(|e| e.to_string())?;
            let set = configured_set
                .as_mut()
                .ok_or("Configured set owner required")?;
            let parent = set.digest().map_err(|e| e.to_string())?;
            set.members[track].attempt = Some(run_sets::configured::ConfiguredRunAttempt {
                attempt_id: format!("{}-run-{}", set.set_id, run.identity.run_id),
                archive_relative_path: path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .ok_or("Configured archive filename")?
                    .into(),
                expected_run_id: run.identity.run_id,
                expected_source_revision: run.identity.source_revision.clone(),
                finalized: None,
            });
            set.parent_sha256 = Some(parent);
            publish_configured_revision(&request, set, &format!("member-{track}-started.json"))?;
            rheon::recording::StepRecorder::create(&path, run, request.recording_limits).map_err(
                |e| {
                    format!(
                        "Configured recording start at {} failed: {e}",
                        path.display()
                    )
                },
            )?
        } else if let Some(spec) = &frozen {
            let mut identity = identity::new_identity("saved-carrier");
            while request.prior_run_ids.contains(&identity.run_id) {
                identity = identity::new_identity("saved-carrier");
            }
            let run = run_sets::bound_native_run(
                spec,
                sim.native()?,
                identity.clone(),
                &build_identity::executor_revision("desktop-saved-native-v1"),
            )
            .map_err(|e| e.to_string())?;
            let set = saved_set.as_mut().expect("saved spec owner");
            let parent = set.sha256().map_err(|e| e.to_string())?;
            set.members[track].attempt = Some(run_sets::RunAttempt {
                attempt_id: format!("{}-run-{}", set.set_id, identity.run_id),
                archive_relative_path: path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or("archive name")?
                    .into(),
                expected_run_id: identity.run_id,
                expected_source_revision: identity.source_revision,
                finalized: None,
            });
            set.parent_sha256 = Some(parent);
            publish_saved_revision(&request, set, &format!("member-{track}-started.json"))?;
            match request.fields_for_track(track) {
                Some(fields) => inspector.begin_bound_carrier_with_force_fields(
                    sim.native()?,
                    &path,
                    request.recording_limits,
                    run,
                    Some(fields),
                ),
                None => inspector.begin_bound_carrier(
                    sim.native()?,
                    &path,
                    request.recording_limits,
                    run,
                ),
            }?
        } else {
            match request.fields_for_track(track) {
                Some(fields) => inspector.begin_carrier_with_force_fields(
                    sim.native()?,
                    &path,
                    request.recording_limits,
                    Some(fields),
                ),
                None => inspector.begin_carrier(sim.native()?, &path, request.recording_limits),
            }
            .map_err(|e| format!("Carrier recording start at {} failed: {e}", path.display()))?
        };
        use rheon::recording::provider::runtime::{RuntimeTermination, RuntimeTerminationCause};
        let mut termination_cause = RuntimeTerminationCause::StorageFailure;
        let result: Result<(), String> = (|| {
            if let Some(configured) = sim.configured() {
                inspector.record_configured(&mut recorder, &configured.snapshot(), None)?;
                inspector.reload(&path, request.recording_limits)?;
                inspector.select_timestep(0)?;
            }
            history.publish(track, sim.preview(history.preview_edge, 0, 0.0, None)?)?;
            if let Some(configured) = sim.configured_mut() {
                termination_cause = RuntimeTerminationCause::StartupFailure;
                let context = request
                    .configured_set
                    .as_ref()
                    .ok_or("Configured execution metadata")?;
                let (_, slot) = context.runtime.as_ref().ok_or("Configured runtime owner")?;
                configured
                    .start_provider(&context.project_root, slot, |_| Ok(()))
                    .map_err(|error| {
                        if matches!(
                            error,
                            accepted_session::model_process::StartupFailure::Cancelled
                        ) {
                            termination_cause = RuntimeTerminationCause::Cancelled;
                        }
                        error.to_string()
                    })?;
            }
            termination_cause = RuntimeTerminationCause::NumericalFailure;
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
                let result = sim.advance(
                    &request,
                    frozen.as_ref().map_or_else(
                        || (step < request.steps / 2).then_some(source),
                        |spec| spec.spec.source_at(step),
                    ),
                    cancel,
                    request.fields_for_track(track),
                );
                let step_seconds = start.elapsed().as_secs_f64();
                history.end_step()?;
                if let Err(error) = &result {
                    termination_cause = error.termination_cause();
                }
                let r = result.map_err(|e| e.to_string())?;
                termination_cause = RuntimeTerminationCause::StorageFailure;
                if let Some(configured) = sim.configured() {
                    inspector.record_configured(
                        &mut recorder,
                        &configured.snapshot(),
                        configured.runtime_observation(),
                    )?;
                } else {
                    match request.fields_for_track(track) {
                        Some(fields) => inspector.record_carrier_with_force_fields(
                            &mut recorder,
                            sim.native()?,
                            Some(fields),
                        ),
                        None => inspector.record_carrier(&mut recorder, sim.native()?),
                    }?;
                }
                termination_cause = RuntimeTerminationCause::NumericalFailure;
                if frozen.is_some() && r.dt.to_bits() != request.requested_dt.to_bits() {
                    return Err("Native step shortened dt; fixed-dt saved specification unsupported. Exact accepted archive retained for direct inspection".into());
                }
                iterations += r.pressure.iterations;
                divergence = divergence.max(r.actual_divergence_max);
                seconds += step_seconds;
                history.publish(
                    track,
                    sim.preview(history.preview_edge, step + 1, step_seconds, Some(r))?,
                )?;
                progress.fetch_add(1, Ordering::Relaxed);
                if let Some(failure) = sim.configured().and_then(|c| c.current_failure())
                    && matches!(
                        failure.reason,
                        rheon::recording::diagnostic::NoCandidateReason::RuntimeFailure
                            | rheon::recording::diagnostic::NoCandidateReason::InvalidOutput
                    )
                {
                    termination_cause = RuntimeTerminationCause::RuntimeFailure;
                    return Err(format!(
                        "Configured runtime retired after accepted native fallback: {:?}: {}; dispatch {}",
                        failure.reason, failure.detail, failure.dispatched
                    ));
                }
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
                termination_cause = RuntimeTerminationCause::Cancelled;
                return Err("Cancelled; accepted recording prefix retained".into());
            }
            let pixels = sim.guidance()?;
            rows.push(ResultRow {
                method,
                label: request.track_label(track),
                seconds,
                iterations,
                divergence,
                bytes: sim.bytes(),
                time: sim.state().time,
                pixels,
                particle_report: None,
            });
            termination_cause = RuntimeTerminationCause::Completed;
            Ok(())
        })();
        #[cfg(test)]
        if result.is_err()
            && let Some(gate) = &request.failure_gate
        {
            gate.reached.send(()).map_err(|e| e.to_string())?;
            gate.release
                .lock()
                .map_err(|e| e.to_string())?
                .recv_timeout(Duration::from_secs(10))
                .map_err(|e| e.to_string())?;
        }
        if let Some(configured) = sim.configured_mut() {
            configured.retire_runtime();
        }
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
        let terminal = if let Some(configured) = sim.configured() {
            RuntimeTermination::capture(
                &configured.snapshot(),
                termination_cause,
                &accepted_session::recording_detail(&detail),
                configured.runtime_observation(),
            )
            .and_then(|termination| termination.finish(&mut recorder))
        } else {
            recorder.finish(outcome, &detail, Some(sim.state().generation))
        };
        terminal.map_err(|e| {
            format!(
                "{detail}; terminal status not confirmed at {}: {e}",
                path.display()
            )
        })?;
        drop(recorder);
        if let Some(set) = saved_set.as_mut() {
            let parent = set.sha256().map_err(|e| e.to_string())?;
            let member = &mut set.members[track];
            let pin = member
                .attempt
                .as_mut()
                .expect("started attempt")
                .pin_closed(
                    std::path::Path::new(&request.recording_directory),
                    &member.spec,
                );
            set.parent_sha256 = Some(parent);
            publish_saved_revision(&request, set, &format!("member-{track}-finished.json"))?;
            pin.map_err(|e| {
                format!(
                    "Saved member unavailable: {e}; preserve archive and started/finished manifests"
                )
            })?;
        }
        if let Some(set) = configured_set.as_mut() {
            let parent = set.digest().map_err(|e| e.to_string())?;
            let member = &mut set.members[track];
            let pin = member
                .attempt
                .as_mut()
                .ok_or("Configured started attempt")?
                .pin_closed(
                    std::path::Path::new(&request.recording_directory),
                    &member.spec,
                );
            set.parent_sha256 = Some(parent);
            publish_configured_revision(&request, set, &format!("member-{track}-finished.json"))?;
            pin.map_err(|e|format!("Configured member unavailable: {e}; preserve archive and immutable started/finished manifests"))?;
        }
        result?;
    }
    if let Some(set) = &saved_set {
        publish_saved_revision(&request, set, "completed.json")?;
    }
    if let Some(set) = &configured_set {
        publish_configured_revision(&request, set, "completed.json")?;
    }
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled; accepted preview frames retained".into());
    }
    Ok(rows)
}
/// Publish immutable data first, then hand the exact acknowledged path to the UI.
fn publish_saved_revision(
    request: &Request,
    set: &run_sets::RunSetRevision,
    name: &str,
) -> Result<(), String> {
    publish_set_revision(request, name, |path| set.save_new(path))
}
fn publish_configured_revision(
    request: &Request,
    set: &run_sets::configured::ConfiguredRunSetRevision,
    name: &str,
) -> Result<(), String> {
    publish_set_revision(request, name, |path| set.save_new(path))
}
fn publish_set_revision(
    request: &Request,
    name: &str,
    write: impl FnOnce(&std::path::Path) -> Result<(), run_sets::RunSetError>,
) -> Result<(), String> {
    let path = std::path::Path::new(&request.recording_directory).join(name);
    write(&path).map_err(|e| {
        format!(
            "Saved revision {}: {e}; preserve and reconcile this exact path",
            path.display()
        )
    })?;
    if let Some(receipt) = &request.manifest_receipt {
        let mut acknowledged = receipt.lock().map_err(|_| {
            format!(
                "Saved revision {} published; UI receipt unavailable",
                path.display()
            )
        })?;
        *acknowledged = path;
    }
    Ok(())
}
/// Freeze the exact ordinary native controls before the existing worker starts.
fn freeze_native_set(request: &mut Request) -> Result<std::path::PathBuf, String> {
    use run_sets::*;
    validate_request(request)?;
    if request.material.is_some() || request.emitter.is_some() || request.collision_mesh.is_some() {
        return Err(
            "Saved native specifications require carrier mode without emitter or collision mesh"
                .into(),
        );
    }
    let mut members = Vec::new();
    for (track, method) in request.methods.iter().enumerate() {
        let spec = FrozenSpec::new(NativeCarrierSpec {
            counts_xyz: [request.size; 3],
            spacing_m: [request.domain_side / request.size as f64; 3],
            origin_m: [0.0; 3],
            density_kg_m3: 1.0,
            pressure_method: match method {
                PressureImplementation::JacobiPcgV1 => NativePressureMethod::JacobiPcgV1,
                PressureImplementation::SymmetricGaussSeidelPcgV1 => {
                    NativePressureMethod::SymmetricGaussSeidelPcgV1
                }
            },
            tolerances: NativeTolerances {
                relative_residual: if request.tight { 1e-11 } else { 1e-9 },
                absolute_residual_m3_s2: 1e-12,
                predicted_divergence_per_s: if request.tight { 1e-9 } else { 1e-7 },
                actual_divergence_per_s: 1e-5,
                max_courant: 1.0,
                max_pressure_iterations: 2000,
            },
            requested_dt_s: request.requested_dt,
            requested_steps: request.steps,
            source: Some(SourceSpec {
                lower_m: [0.35, 0.08, 0.35].map(|v| v * request.domain_side),
                upper_m: [0.65, 0.30, 0.65].map(|v| v * request.domain_side),
                tracer_rate_per_s: 2.0,
                vertical_acceleration_m_s2: 1.0,
                active_steps: request.steps / 2,
            }),
            simulation_memory_limit_bytes: 192 * 1024 * 1024,
            recording: ArchiveBudget {
                max_database_bytes: request.recording_limits.max_database_bytes,
                max_record_bytes: request.recording_limits.max_record_bytes,
                max_records: request.recording_limits.max_records,
            },
        })
        .map_err(|e| e.to_string())?;
        let spec = match request.fields_for_track(track) {
            Some(fields) => spec.with_force_fields(fields).map_err(|e| e.to_string())?,
            None => spec,
        };
        members.push(RunSetMember {
            member_id: format!("native-{track}"),
            spec,
            differences_from_baseline: if track == 0 {
                vec![]
            } else {
                vec![SpecDifference::PressureMethod]
            },
            attempt: None,
        });
    }
    let root =
        new_recording_path(&request.recording_directory, "run-set")?.with_extension("runset");
    let set = RunSetRevision {
        schema_version: 1,
        set_id: root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("set directory")?
            .into(),
        parent_sha256: None,
        baseline_member: "native-0".into(),
        members,
    };
    set.validate().map_err(|e| e.to_string())?;
    std::fs::create_dir(&root).map_err(|e| e.to_string())?;
    set.save_new(&root.join("intent.json"))
        .map_err(|e| e.to_string())?;
    request.saved_set = Some(set);
    request.manifest_receipt = Some(Arc::new(std::sync::Mutex::new(root.join("intent.json"))));
    request.recording_directory = root.to_string_lossy().into();
    Ok(root.join("intent.json"))
}

fn new_recording_path(directory: &str, label: &str) -> Result<std::path::PathBuf, String> {
    let directory = std::path::Path::new(directory);
    std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
    recording_candidate_path(directory, label)
}
/// Allocate a fresh name without creating output before intent/host validation.
fn recording_candidate_path(
    directory: &std::path::Path,
    label: &str,
) -> Result<std::path::PathBuf, String> {
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
    let fields = request.fields_for_track(0);
    let mut recorder = match fields {
        Some(fields) => history.inspector.begin_with_force_fields(
            &sim,
            &path,
            request.recording_limits,
            Some(fields),
        ),
        None => history
            .inspector
            .begin(&sim, &path, request.recording_limits),
    }
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
            let result = match fields {
                Some(fields) => sim.step_with_force_fields(
                    request.requested_dt,
                    request.emitter,
                    fields,
                    || cancel.load(Ordering::Relaxed),
                ),
                None => sim.step(request.requested_dt, request.emitter, || {
                    cancel.load(Ordering::Relaxed)
                }),
            };
            let elapsed = start.elapsed().as_secs_f64();
            history.end_step()?;
            let report = result.map_err(|e| e.to_string())?;
            seconds += elapsed;
            last_report = Some(report);
            let controls = rheon::MaterialStepControls {
                requested_dt: request.requested_dt,
                emitter: request
                    .emitter
                    .map(|emitter| rheon::MaterialEmitterControl {
                        source_id: 1,
                        source_version: 1,
                        emitter,
                    }),
            };
            match fields {
                Some(fields) => history.inspector.record_with_force_fields(
                    &mut recorder,
                    &sim,
                    report,
                    controls,
                    Some(fields),
                ),
                None => history
                    .inspector
                    .record(&mut recorder, &sim, report, controls),
            }?;
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
            label: request.track_label(0),
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
#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SetupWorkflow {
    #[default]
    Preview,
    AcceptedSession,
}
impl SetupWorkflow {
    fn force_target(self, material: bool) -> force_run_policy::Target {
        match self {
            Self::AcceptedSession => force_run_policy::Target::Unsupported,
            Self::Preview if material => force_run_policy::Target::ParticleMaterial,
            Self::Preview => force_run_policy::Target::NativeCarrier,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Preview => "Simulation preview",
            Self::AcceptedSession => "Accepted Session",
        }
    }
}
#[derive(Clone, Copy)]
enum SetupAction {
    Run(SetupWorkflow),
    RunContents,
    Cancel,
}
fn running_setup_ui(ui: &mut egui::Ui, request: &Request) {
    ui.heading("Running setup");
    ui.label(request.setup_workflow().label());
    ui.small("Inputs frozen at launch.");
    let samples = request
        .material
        .map_or(request.size as usize, |material| material.samples_per_axis);
    ui.label(format!(
        "{samples} samples/axis · domain {:.3} m",
        request.domain_side
    ));
    ui.label(format!(
        "{} steps · maximum step {}",
        request.steps,
        settings::duration(request.requested_dt)
    ));
    if request.voxel.is_some() {
        ui.small(voxel_material_execution::METHOD);
        ui.small("Filled incompressible Water/Air · starts at rest · closed free-slip walls");
        ui.small("Actual accepted intervals may be shorter than the requested maximum.");
    } else if request.configured_set.is_some() {
        ui.small("Sealed full liquid · starts at rest · no inlet");
        ui.small(
            "Constant forcing only; no appearance, emitter, phase source, or model accuracy claim.",
        );
    } else if request.material.is_some() {
        ui.small("Liquid / gas material preview");
    } else {
        ui.small("Filled carrier-fluid preview");
    }
}
/// Per-paint value projection only; App remains the playback state owner.
#[derive(Clone)]
struct TimelineControls {
    timeline_step: usize,
    track: usize,
    follow_live: bool,
    playing: bool,
    play_time: f64,
    playback_speed: f64,
    playback_speed_text: String,
    playback_speed_invalid: bool,
}
// Prewrap against the whole row, then let the wrapped layout place this caption
// as one widget. Layout against a small row remainder can clip split text.
fn timeline_caption(ui: &mut egui::Ui, text: String) {
    let galley = egui::WidgetText::from(text).into_galley(
        ui,
        Some(egui::TextWrapMode::Wrap),
        ui.max_rect().width(),
        egui::TextStyle::Body,
    );
    ui.label(galley);
}
impl TimelineControls {
    fn contents(
        &mut self,
        ui: &mut egui::Ui,
        count: usize,
        frame: Option<&PreviewFrame>,
        request: Option<&Request>,
        track_caption: &str,
    ) {
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
            // A bounded editor has the same geometry while disabled, focused or
            // holding invalid text. DragValue's growing private edit buffer does not.
            let speed_label = ui.label("Speed");
            let speed = ui
                .add(
                    egui::TextEdit::singleline(&mut self.playback_speed_text)
                        .id_salt("playback-speed")
                        .desired_width(44.0)
                        .clip_text(true),
                )
                .labelled_by(speed_label.id)
                .on_hover_text(
                    "Playback speed: 0.1–4×. Edit the number; Up/Down changes it by 0.1.",
                );
            ui.label("×");
            // Feedback is projected from the start of this paint, so input cannot
            // add an unmeasured row partway through bottom-panel placement.
            let displayed_invalid = self.playback_speed_invalid;
            if displayed_invalid {
                ui.colored_label(egui::Color32::LIGHT_RED, "Invalid speed")
                    .on_hover_text("Enter a finite number. Allowed range: 0.1–4×.");
            }
            if speed.changed() {
                match self.playback_speed_text.parse::<f64>() {
                    Ok(value) if value.is_finite() => {
                        self.playback_speed = value.clamp(0.1, 4.0);
                        self.playback_speed_invalid = false;
                    }
                    _ => self.playback_speed_invalid = true,
                }
            }
            if speed.has_focus() {
                let increment = ui.input(|input| {
                    i32::from(input.key_pressed(egui::Key::ArrowUp))
                        - i32::from(input.key_pressed(egui::Key::ArrowDown))
                });
                if increment != 0 {
                    self.playback_speed =
                        (self.playback_speed + f64::from(increment) * 0.1).clamp(0.1, 4.0);
                    self.playback_speed_text = self.playback_speed.to_string();
                    self.playback_speed_invalid = false;
                }
            } else if !self.playback_speed_invalid {
                self.playback_speed_text = self.playback_speed.to_string();
            }
            if let Some(request) = request
                && request.material.is_none()
            {
                let decoration = 2.0 * ui.spacing().button_padding.x
                    + ui.spacing().icon_spacing
                    + ui.spacing().icon_width;
                let text = egui::WidgetText::from(request.track_label(self.track)).into_galley(
                    ui,
                    Some(egui::TextWrapMode::Wrap),
                    (ui.max_rect().width() - decoration).max(1.0),
                    egui::TextStyle::Button,
                );
                let width = text.size().x + decoration;
                // ComboBox's inner horizontal UI cannot move itself to a new row.
                if ui.available_size_before_wrap().x < width {
                    ui.end_row();
                }
                egui::ComboBox::from_id_salt("timeline-track")
                    .width(width)
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        for track in 0..request.methods.len() {
                            if ui
                                .selectable_value(
                                    &mut self.track,
                                    track,
                                    request.track_label(track),
                                )
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
                timeline_caption(
                    ui,
                    format!(
                        "{} · Step {} / {}",
                        if frame.voxel.is_some() {
                            voxel_material_execution::METHOD.to_owned()
                        } else if frame.material_color.is_some() {
                            "Material fluid".to_owned()
                        } else {
                            track_caption.to_owned()
                        },
                        frame.step,
                        count.saturating_sub(1)
                    ),
                );
                ui.separator();
                timeline_caption(ui, format!("Simulated {}", settings::duration(frame.time)));
                ui.separator();
                timeline_caption(
                    ui,
                    if frame.step == 0 {
                        "Initial state".into()
                    } else {
                        format!("Step calculation {}", settings::duration(frame.seconds))
                    },
                );
                if let Some(r) = frame.particle_report {
                    ui.separator();
                    timeline_caption(
                        ui,
                        format!(
                            "{} particles · {:.6} kg · {:.1} °C",
                            r.particles,
                            r.total_mass,
                            r.mean_temperature_kelvin - 273.15
                        ),
                    );
                }
                if let Some(report) = frame.voxel.as_ref().and_then(|v| v.report) {
                    ui.separator();
                    timeline_caption(
                        ui,
                        format!(
                            "{} pressure iterations · divergence {:.2e} · accepted dt{}s",
                            report.projection.pressure.iterations,
                            report.projection.actual_divergence_max,
                            report.dt
                        ),
                    );
                }
                if let Some(report) = frame.report {
                    ui.separator();
                    timeline_caption(
                        ui,
                        format!(
                            "{} pressure iterations · divergence {:.2e}",
                            report.pressure.iterations, report.actual_divergence_max
                        ),
                    );
                }
            });
        }
    }
}

struct App {
    scene: scene_ui::SceneUi,
    setup_workflow: SetupWorkflow,
    ai_comparison: Option<ai_comparison::ComparisonApp>,
    archive: archive_view::ArchiveView,
    run_sets: run_set_view::RunSetView,
    save_native_set: bool,
    configured_controls: configured_controls::ConfiguredControls,
    saved_comparison: saved_comparison::SavedComparisonPage,
    saved_comparison_active: bool,
    comparison_fields: comparison_field_view::ComparisonFieldView,
    collision: collision_ui::CollisionControls,
    selected: PressureImplementation,
    numerical_cells_per_metre: Option<u32>,
    domain_side: f64,
    requested_dt: f64,
    material_mode: bool,
    history_budget_mib: usize,
    preview_resolution: history::PreviewResolution,
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
    pending_voxel: Option<voxel_material_launch::PendingVoxel>,
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
    playback_speed_text: String,
    playback_speed_invalid: bool,
}
impl Default for App {
    fn default() -> Self {
        let thread_limit = available_threads();
        Self {
            scene: scene_ui::SceneUi::default(),
            setup_workflow: SetupWorkflow::default(),
            ai_comparison: None,
            archive: archive_view::ArchiveView::default(),
            run_sets: run_set_view::RunSetView::default(),
            save_native_set: false,
            configured_controls: Default::default(),
            saved_comparison: saved_comparison::SavedComparisonPage::default(),
            saved_comparison_active: false,
            comparison_fields: Default::default(),
            collision: collision_ui::CollisionControls::default(),
            selected: PressureImplementation::default(),
            numerical_cells_per_metre: None,
            domain_side: 1.0,
            requested_dt: 0.02,
            material_mode: true,
            history_budget_mib: history::HISTORY_BUDGET / (1024 * 1024),
            preview_resolution: history::PreviewResolution::Adaptive,
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
            pending_voxel: None,
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
            playback_speed_text: "1".into(),
            playback_speed_invalid: false,
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
                self.run_sets.controls(ui, self.worker.is_none());
                if ui
                    .add_enabled(
                        self.worker.is_none(),
                        egui::Button::new("Open saved comparisons"),
                    )
                    .clicked()
                {
                    self.saved_comparison_active = true;
                }
                if self.archive.active || self.run_sets.active || self.saved_comparison_active {
                    return;
                }
                ui.separator();
                let displayed_workflow = self
                    .running_request
                    .as_ref()
                    .filter(|_| self.worker.is_some())
                    .map_or(self.setup_workflow, Request::setup_workflow);
                ui.add_enabled_ui(self.worker.is_none() && !self.closing, |ui| {
                    egui::ComboBox::from_label("Setup")
                        .selected_text(displayed_workflow.label())
                        .show_ui(ui, |ui| {
                            for workflow in [SetupWorkflow::Preview, SetupWorkflow::AcceptedSession]
                            {
                                ui.selectable_value(
                                    &mut self.setup_workflow,
                                    workflow,
                                    workflow.label(),
                                );
                            }
                        });
                });
                // Keep actions outside scrolling content. Dispatch only after this
                // frame's form edits and package-readiness invalidation.
                let action = self.run_controls(ui);
                ui.separator();
                egui::ScrollArea::vertical()
                    .id_salt(("sidebar-content", self.setup_workflow as u8))
                    .show(ui, |ui| {
                        if let Some(pending) = &self.pending_voxel {
                            ui.heading("Preparing voxel run");
                            running_setup_ui(ui, &pending.request);
                            ui.label("Current Inspector and playback selection held until verified adoption.");
                        } else if self.inspector_open {
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
                        } else if let Some(request) = self
                            .running_request
                            .as_ref()
                            .filter(|_| self.worker.is_some())
                        {
                            running_setup_ui(ui, request);
                        } else {
                            self.controls(ui);
                        }
                    });
                self.dispatch_setup_action(ctx, action);
            })
            .response
            .rect
    }
    fn dispatch_setup_action(&mut self, ctx: &egui::Context, action: Option<SetupAction>) {
        if let Some(action) = action {
            match action {
                SetupAction::Run(SetupWorkflow::Preview) => self.start(false),
                SetupAction::Run(SetupWorkflow::AcceptedSession) => {
                    if let Err(error) = self.start_configured() {
                        self.status = error;
                    }
                }
                SetupAction::RunContents => {
                    if let Err(error) = self.start_contents() {
                        self.status = error;
                    }
                }
                SetupAction::Cancel => self.cancel(),
            }
            ctx.request_repaint();
        }
    }
    fn scene_run_strip(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("scene-global-run-strip").show(ctx, |ui| {
            let action = self.run_controls(ui);
            self.dispatch_setup_action(ctx, action);
        });
    }
    fn run_controls(&mut self, ui: &mut egui::Ui) -> Option<SetupAction> {
        let mut action = None;
        if let Some(worker) = &self.worker {
            if self.pending_voxel.is_some() {
                ui.small("Preparing voxel Water/Air · current result held");
            }
            let n = worker.progress.load(Ordering::Relaxed);
            ui.add(
                egui::ProgressBar::new(n as f32 / worker.total as f32)
                    .text(format!("{n}/{} steps", worker.total)),
            );
            if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new("Cancel simulation"),
                )
                .clicked()
            {
                action = Some(SetupAction::Cancel);
            }
            if let Some((_, step, elapsed)) = worker.history.timing() {
                ui.small(format!("Calculating step {step} · {elapsed:.3} s"));
            }
        } else if self.scene.contents_selected() {
            let readiness = self.contents_run_readiness();
            let disabled = readiness.as_ref().err().map_or("", String::as_str);
            if ui
                .add_enabled(
                    readiness.is_ok(),
                    egui::Button::new("Simulate contents")
                        .min_size(egui::vec2(ui.available_width(), 32.0)),
                )
                .on_disabled_hover_text(disabled)
                .clicked()
            {
                action = Some(SetupAction::RunContents);
            }
            if let Err(error) = readiness {
                ui.colored_label(egui::Color32::YELLOW, "Initial contents run unavailable")
                    .on_hover_text(error);
            }
        } else {
            let readiness = self
                .scene
                .launch_readiness(self.setup_workflow.force_target(self.material_mode));
            let can_launch = !self.closing && readiness.is_ok();
            let launch = match self.setup_workflow {
                SetupWorkflow::Preview => ui
                    .add_enabled(
                        can_launch,
                        egui::Button::new("Simulate")
                            .min_size(egui::vec2(ui.available_width(), 32.0)),
                    )
                    .clicked(),
                SetupWorkflow::AcceptedSession => {
                    self.configured_controls.launch_button(ui, can_launch)
                }
            };
            if let Err(error) = readiness {
                ui.colored_label(
                    egui::Color32::YELLOW,
                    if self.scene.contents_selected() || self.scene.contents.busy() {
                        "Initial contents run unavailable"
                    } else {
                        "Authored-force runs unavailable"
                    },
                )
                .on_hover_text(error);
            }
            if launch {
                action = Some(SetupAction::Run(self.setup_workflow));
            }
        }
        // Keep long feedback from displacing run controls. The complete original
        // message is available through a named keyboard-operable disclosure.
        ui.add(egui::Label::new(&self.status).truncate());
        ui.collapsing("Status details", |ui| {
            egui::ScrollArea::vertical()
                .id_salt("run-status-details")
                .max_height(96.0)
                .show(ui, |ui| {
                    let mut status = self.status.as_str();
                    ui.add(
                        egui::TextEdit::multiline(&mut status)
                            .id(egui::Id::new("run-status-text"))
                            .font(egui::TextStyle::Body)
                            .desired_width(f32::INFINITY),
                    );
                });
        });
        action
    }
    fn reset_timeline(&mut self) {
        self.scene.clear_run();
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
    fn selected_recording_limits(&self) -> Result<rheon::recording::RecordingLimits, String> {
        let max_database_bytes = u64::try_from(self.recording_database_mib)
            .ok()
            .and_then(|n| n.checked_mul(1024 * 1024))
            .ok_or("Recording database limit overflow")?;
        let max_record_bytes = self
            .recording_record_mib
            .checked_mul(1024 * 1024)
            .ok_or("Recording publication limit overflow")?;
        let max_records = self
            .steps
            .checked_add(1)
            .ok_or("Recording count overflow")?;
        let limits = rheon::recording::RecordingLimits {
            max_database_bytes,
            max_record_bytes,
            max_records,
        };
        limits.validate().map_err(|e| e.to_string())?;
        Ok(limits)
    }
    fn start_configured(&mut self) -> Result<(), String> {
        self.scene
            .launch_readiness(force_run_policy::Target::Unsupported)?;
        if self.worker.is_some() {
            return Err("Finish the active worker first".into());
        }
        self.refresh_threads(available_threads());
        let limits = self.selected_recording_limits()?;
        let root = recording_candidate_path(
            std::path::Path::new(&self.recording_directory),
            "configured-set",
        )?
        .with_extension("runset");
        let set_id = root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Configured set directory")?
            .to_owned();
        let size = self.selected_carrier_count()?;
        let grid = GridGeometry::new([size; 3], [self.domain_side / size as f64; 3], [0.0; 3])
            .map_err(|e| e.to_string())?;
        let revision =
            self.configured_controls
                .freeze(configured_controls::ConfiguredLaunchInputs {
                    set_id,
                    grid,
                    pressure_implementation: self.selected,
                    pressure: PressureSettings {
                        relative_residual: if self.tight { 1e-11 } else { 1e-9 },
                        absolute_residual: 1e-12,
                        divergence_limit: if self.tight { 1e-9 } else { 1e-7 },
                        max_iterations: 2000,
                    },
                    actual_divergence_limit: 1e-5,
                    max_courant: 1.0,
                    simulation_memory_limit_bytes: 192 * 1024 * 1024,
                    boundary_workspace_limit_bytes: 64 * 1024 * 1024,
                    volume_stamp: rheon::VolumeStamp { id: 1, version: 0 },
                    carrier_id: 2,
                    requested_dt_s: self.requested_dt,
                    requested_steps: self.steps,
                    volume: run_sets::configured::VolumeControls {
                        max_outward_courant: 0.5,
                        actual_divergence_per_s: 1e-5,
                    },
                    recording: run_sets::ArchiveBudget {
                        max_database_bytes: limits.max_database_bytes,
                        max_record_bytes: limits.max_record_bytes,
                        max_records: limits.max_records,
                    },
                    executor: configured_executor(),
                })?;
        self.publish_and_launch_configured(revision, &root, Vec::new())
    }
    fn publish_and_launch_configured(
        &mut self,
        revision: run_sets::configured::ConfiguredRunSetRevision,
        root: &std::path::Path,
        prior_run_ids: Vec<u64>,
    ) -> Result<(), String> {
        revision.validate().map_err(|e| e.to_string())?;
        let baseline = &revision.members[0].spec;
        let intent_path = root.join("intent.json");
        let request = Request {
            force_fields: Vec::new(),
            collision_mesh: None,
            size: baseline.constructor.counts_xyz[0],
            steps: baseline.future.requested_steps,
            tight: false,
            threads: self.threads.min(available_threads()).max(1),
            methods: revision
                .members
                .iter()
                .map(|m| m.spec.constructor.pressure_method.implementation())
                .collect(),
            emitter: None,
            observe: self.show_workers,
            history_budget_mib: self.history_budget_mib,
            preview_resolution: self.preview_resolution,
            domain_side: baseline.constructor.spacing_m[0]
                * baseline.constructor.counts_xyz[0] as f64,
            requested_dt: baseline.future.requested_dt_s,
            material: None,
            material_color: self.materials.color(),
            recording_limits: baseline.future.recording.limits(),
            recording_directory: root.to_string_lossy().into(),
            saved_set: None,
            configured_set: Some(ConfiguredRequest {
                revision,
                project_root: std::env::current_dir().map_err(|e| e.to_string())?,
                runtime: None,
            }),
            prior_run_ids,
            voxel: None,
            manifest_receipt: Some(Arc::new(std::sync::Mutex::new(intent_path.clone()))),
            #[cfg(test)]
            progress_gate: None,
            #[cfg(test)]
            failure_gate: None,
        };
        // Exact frozen host/profile checks precede every output and code launch.
        validate_request(&request)?;
        let parent = root.parent().ok_or("Configured output parent")?;
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        std::fs::create_dir(root).map_err(|e| e.to_string())?;
        request
            .configured_set
            .as_ref()
            .ok_or("Configured intent owner")?
            .revision
            .save_new(&intent_path)
            .map_err(|e| e.to_string())?;
        self.run_sets.path = intent_path.to_string_lossy().into();
        self.launch_request(request)
    }
    fn rerun_configured(
        &mut self,
        original: run_sets::configured::ConfiguredRunSetRevision,
    ) -> Result<(), String> {
        if self.worker.is_some() {
            return Err("Finish the active worker first".into());
        }
        original.validate().map_err(|e| e.to_string())?;
        for member in &original.members {
            member
                .spec
                .validate_execution(&configured_executor())
                .map_err(|e| e.to_string())?;
        }
        let root = recording_candidate_path(
            std::path::Path::new(&self.recording_directory),
            "configured-rerun",
        )?
        .with_extension("runset");
        let mut revision = original.clone();
        revision.parent_sha256 = Some(original.digest().map_err(|e| e.to_string())?);
        revision.set_id = root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("Configured rerun directory")?
            .into();
        for member in &mut revision.members {
            member.attempt = None;
        }
        let prior_run_ids = original
            .members
            .iter()
            .filter_map(|m| m.attempt.as_ref().map(|a| a.expected_run_id))
            .collect();
        self.publish_and_launch_configured(revision, &root, prior_run_ids)
    }
    /// New edits obey the physical density cap. Historical frozen geometry is
    /// decoded/rerun through its existing exact-spacing path without this edit.
    fn check_sampling_density(&self, count: usize) -> Result<(), String> {
        let actual = count as f64 / self.domain_side;
        if !actual.is_finite() || actual > f64::from(rheon::MAX_CELLS_PER_METRE).next_up() {
            return Err(format!(
                "{count} samples across {:.9} m resolves to {actual:.6} samples/m, above the 1000/m physical sampling limit. Adjust extent or counts; covering rounding never silently lowers a density request",
                self.domain_side
            ));
        }
        Ok(())
    }
    fn selected_carrier_count(&self) -> Result<u64, String> {
        let edge = self
            .numerical_cells_per_metre
            .map_or(Ok(self.size as usize), |d| {
                history::edge_at_density(self.domain_side, d)
            })?;
        self.check_sampling_density(edge)?;
        Ok(edge as u64)
    }
    /// Density preferences survive domain edits; counts freeze only at launch.
    fn selected_sampling(&self) -> Result<(u64, Option<rheon::ParticleFluidConfig>), String> {
        if !self.material_mode {
            return Ok((self.selected_carrier_count()?, None));
        }
        let mut config = self.materials.config;
        if let Some(density) = self.numerical_cells_per_metre {
            let edge = history::edge_at_density(self.domain_side, density)?;
            if !(4..=64).contains(&edge) {
                return Err("Requested particle density requires 4–64 samples per side; reduce domain extent or density".into());
            }
            config.samples_per_axis = edge;
        }
        self.check_sampling_density(config.samples_per_axis)?;
        Ok((self.size, Some(config)))
    }
    fn start(&mut self, all: bool) {
        if let Err(error) = self
            .scene
            .launch_readiness(SetupWorkflow::Preview.force_target(self.material_mode))
        {
            self.status = error;
            return;
        }
        if self.worker.is_some() {
            return;
        }
        self.refresh_threads(available_threads());
        let recording_limits = match self.selected_recording_limits() {
            Ok(limits) => limits,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let (size, material) = match self.selected_sampling() {
            Ok(value) => value,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let fields = match self.scene.freeze_for_run() {
            Ok(fields) => fields,
            Err(error) => {
                self.status = error;
                return;
            }
        };
        let mut request = Request {
            force_fields: fields.map_or_else(Vec::new, |fields| {
                vec![Some(fields); if all { 2 } else { 1 }]
            }),
            collision_mesh: self.collision.mesh.clone(),
            size,
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
            preview_resolution: self.preview_resolution,
            domain_side: self.domain_side,
            requested_dt: self.requested_dt,
            material,
            material_color: self.materials.color(),
            recording_limits,
            recording_directory: self.recording_directory.clone(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: Vec::new(),
            voxel: None,
            manifest_receipt: None,
            #[cfg(test)]
            progress_gate: None,
            #[cfg(test)]
            failure_gate: None,
        };
        if self.save_native_set {
            match freeze_native_set(&mut request) {
                Ok(path) => self.run_sets.path = path.to_string_lossy().into(),
                Err(e) => {
                    self.status = e;
                    return;
                }
            }
        }
        let _ = self.launch_request(request);
    }
    /// Re-execute captured native intent from constructor zero; never resume fields.
    fn rerun_saved(&mut self, original: run_sets::RunSetRevision) -> Result<(), String> {
        if self.worker.is_some() {
            return Err("Finish the active worker first".into());
        }
        original.validate().map_err(|e| e.to_string())?;
        let baseline = &original.members[0].spec.spec;
        if original.members.len() > 2
            || baseline.counts_xyz != [baseline.counts_xyz[0]; 3]
            || baseline.spacing_m != [baseline.spacing_m[0]; 3]
            || baseline.origin_m != [0.0; 3]
            || original.members.iter().any(|m| {
                let s = &m.spec.spec;
                s.counts_xyz != baseline.counts_xyz
                    || s.spacing_m != baseline.spacing_m
                    || s.origin_m != baseline.origin_m
                    || s.requested_steps != baseline.requested_steps
                    || s.requested_dt_s != baseline.requested_dt_s
                    || s.recording != baseline.recording
            })
        {
            return Err("Desktop rerun supports up to two cubic zero-origin members with common geometry, requested horizon and budgets; cold inspection remains available".into());
        }
        let mut request = Request {
            force_fields: Vec::new(),
            collision_mesh: None,
            size: baseline.counts_xyz[0],
            steps: baseline.requested_steps,
            tight: false,
            threads: self.threads.min(available_threads()).max(1),
            methods: original
                .members
                .iter()
                .map(|m| m.spec.spec.pressure_method.implementation())
                .collect(),
            emitter: None,
            observe: self.show_workers,
            history_budget_mib: self.history_budget_mib,
            preview_resolution: self.preview_resolution,
            domain_side: baseline.spacing_m[0] * baseline.counts_xyz[0] as f64,
            requested_dt: baseline.requested_dt_s,
            material: None,
            material_color: self.materials.color(),
            recording_limits: baseline.recording.limits(),
            recording_directory: self.recording_directory.clone(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: original
                .members
                .iter()
                .filter_map(|member| member.attempt.as_ref().map(|a| a.expected_run_id))
                .collect(),
            voxel: None,
            manifest_receipt: None,
            #[cfg(test)]
            progress_gate: None,
            #[cfg(test)]
            failure_gate: None,
        };
        request.force_fields = original
            .members
            .iter()
            .map(|m| m.spec.force_snapshot().map_err(|e| e.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        validate_request(&request)?;
        let root =
            new_recording_path(&request.recording_directory, "rerun")?.with_extension("runset");
        let mut next = original.clone();
        next.parent_sha256 = Some(original.sha256().map_err(|e| e.to_string())?);
        next.set_id = root
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or("set directory")?
            .into();
        for member in &mut next.members {
            member.attempt = None;
        }
        std::fs::create_dir(&root).map_err(|e| e.to_string())?;
        next.save_new(&root.join("intent.json"))
            .map_err(|e| e.to_string())?;
        request.recording_directory = root.to_string_lossy().into();
        request.manifest_receipt = Some(Arc::new(std::sync::Mutex::new(root.join("intent.json"))));
        request.saved_set = Some(next);
        self.run_sets.path = root.join("intent.json").to_string_lossy().into();
        self.launch_request(request)
    }
    /// Sole production queued-authoring boundary. Never execute a stale command
    /// against retained/pending simulation custody; report refusal in its own UI.
    fn service_initial_contents_commands(&mut self, ctx: &egui::Context) {
        if self.worker.is_some() || self.pending_voxel.is_some() {
            let queued = std::mem::take(&mut self.scene.contents.create_requested)
                | self.scene.contents.draft_action.take().is_some()
                | std::mem::take(&mut self.scene.contents.run_requested);
            if queued {
                self.scene.contents.io_feedback = "Initial contents command not executed while a run is preparing/running; issue it again in idle Draft.".into();
            }
            return;
        }
        self.service_draft_file_action(ctx);
        if self.scene.contents.create_requested {
            self.scene.contents.create_requested = false;
            let result = GridGeometry::new(
                [self.size; 3],
                [self.domain_side / self.size as f64; 3],
                [0.0; 3],
            )
            .map_err(|e| e.to_string())
            .and_then(|grid| self.scene.create_contents(grid));
            if let Err(error) = result {
                self.status = error;
            }
        }
        if std::mem::take(&mut self.scene.contents.run_requested)
            && let Err(error) = self.start_contents()
        {
            self.status = error;
        }
    }
    fn launch_request(&mut self, request: Request) -> Result<(), String> {
        if request.voxel.is_some() {
            return self.launch_voxel_request(request);
        }
        match Worker::start(request.clone()) {
            Ok(worker) => {
                self.scheduler = Some(Arc::clone(&worker.scheduler));
                self.history = Some(Arc::clone(&worker.history));
                self.worker = Some(worker);
                self.scene.begin_run(request.collision_mesh.is_some());
                let retained_binding = self
                    .scene
                    .bind_retained_fields(0, request.fields_for_track(0));
                self.running_request = Some(request);
                self.rows.clear();
                self.textures.clear();
                self.timeline_step = 0;
                self.track = 0;
                self.follow_live = true;
                self.playing = false;
                self.status = retained_binding.map_or_else(
                    |error| format!("Running; retained force display unavailable: {error}"),
                    |()| "Simulating — accepted steps appear live.".into(),
                );
                Ok(())
            }
            Err(e) => {
                let message = format!("Cannot start worker: {e}");
                self.status = message.clone();
                Err(message)
            }
        }
    }
    fn cancel(&self) {
        if let Some(w) = &self.worker {
            w.cancel();
        }
    }
    fn poll(&mut self, ctx: &egui::Context) {
        if let Err(error) = self.acknowledge_voxel_constructor() {
            self.status = format!(
                "Voxel constructor acknowledgment unavailable: {error}; previous result held"
            );
        }
        if self.worker.as_ref().is_some_and(|w| w.handle.is_finished()) {
            let worker = self.worker.take().expect("finished worker");
            if self.pending_voxel.is_some() {
                self.pending_voxel_terminal(worker.handle.join());
                return;
            }
            let cancelled = worker.cancel.load(Ordering::Relaxed)
                && self
                    .running_request
                    .as_ref()
                    .is_none_or(|r| r.voxel.is_none());
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
                                r.label.as_str(),
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
            if let Some(receipt) = self
                .running_request
                .as_ref()
                .and_then(|r| r.manifest_receipt.as_ref())
            {
                match receipt.lock() {
                    Ok(path) => {
                        self.run_sets.path = path.to_string_lossy().into();
                        self.status
                            .push_str(&format!(" Saved revision: {}", path.display()));
                    }
                    Err(_) => self.status.push_str(
                        " Saved revision UI receipt unavailable; preserve the run-set directory.",
                    ),
                }
            }
        }
    }
    /// One render-frame snapshot of track, appearance and force metadata. Track
    /// intents painted later apply visually next frame, never to half a frame.
    fn prepare_scene_presentation(&mut self) -> Option<(usize, Option<Arc<PreviewFrame>>)> {
        if let Some(pending) = &self.pending_voxel {
            return (self.scene.context == scene_ui::SceneContext::Retained)
                .then(|| pending.view());
        }
        if self
            .running_request
            .as_ref()
            .is_some_and(|r| r.voxel.is_some())
        {
            self.scene.bind_retained_no_forces(self.track);
            return (self.scene.context == scene_ui::SceneContext::Retained)
                .then(|| self.selected_frame());
        }
        let view =
            (self.scene.context == scene_ui::SceneContext::Retained).then(|| self.selected_frame());
        if let Some(request) = &self.running_request
            && let Err(error) = self
                .scene
                .bind_retained_fields(self.track, request.fields_for_track(self.track))
        {
            self.status = error;
        }
        view
    }
    fn selected_frame(&mut self) -> (usize, Option<Arc<PreviewFrame>>) {
        self.selected_frame_inner(true)
    }
    fn selected_frame_bound_track(&mut self) -> (usize, Option<Arc<PreviewFrame>>) {
        self.selected_frame_inner(false)
    }
    fn selected_frame_inner(
        &mut self,
        resolve_active_track: bool,
    ) -> (usize, Option<Arc<PreviewFrame>>) {
        if let Some(pending) = &self.pending_voxel {
            return pending.view();
        }
        let Some(history) = &self.history else {
            return (0, None);
        };
        if self.follow_live && resolve_active_track {
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
                        let inspector = if frame.material_color.is_some() && frame.voxel.is_none() {
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
    /// Draft authoring never clamps or commits retained playback state.
    fn scene_timeline_ui(
        &mut self,
        ctx: &egui::Context,
        count: usize,
        frame: Option<&PreviewFrame>,
    ) -> Option<egui::Rect> {
        if self.scene.context == scene_ui::SceneContext::Retained {
            Some(self.timeline_ui(ctx, count, frame))
        } else {
            None
        }
    }
    /// Paint playback only; frame/history/worker ownership stays with the caller.
    fn timeline_ui(
        &mut self,
        ctx: &egui::Context,
        count: usize,
        frame: Option<&PreviewFrame>,
    ) -> egui::Rect {
        if self.pending_voxel.is_some() {
            return egui::TopBottomPanel::bottom("timeline")
                .show(ctx, |ui| {
                    ui.label("Playback and numerical selection held while preparing voxel run");
                })
                .response
                .rect;
        }
        let mut controls = TimelineControls {
            timeline_step: self.timeline_step,
            track: self.track,
            follow_live: self.follow_live,
            playing: self.playing,
            play_time: self.play_time,
            playback_speed: self.playback_speed,
            playback_speed_text: self.playback_speed_text.clone(),
            playback_speed_invalid: self.playback_speed_invalid,
        };
        let request = self.running_request.as_ref();
        // Freeze display context across sizing and interactive painting. A changed
        // track is projected on the next frame, without measuring mutable history.
        let track_caption = request.map_or_else(
            || frame.map_or_else(String::new, |frame| frame.method.id().to_owned()),
            |request| request.track_label(controls.track),
        );
        let panel_frame = egui::Frame::side_top_panel(&ctx.style());
        let mut sizing_ui = egui::Ui::new(
            ctx.clone(),
            egui::Id::new("timeline-sizing"),
            egui::UiBuilder::new()
                .max_rect(ctx.available_rect())
                .layout(egui::Layout::top_down(egui::Align::Min))
                .invisible(),
        );
        // Invisible widgets still produce AccessKit nodes. Hide the measurement
        // subtree so assistive technology receives only the real playback controls.
        ctx.accesskit_node_builder(sizing_ui.id(), |node| node.set_hidden());
        // Bottom panels reserve height before painting. Measure the same contents
        // locally so cold frames and changing captions never rely on cached height.
        // This disabled, disjoint-ID pass edits only a disposable scalar copy.
        let mut sizing_controls = controls.clone();
        let measured = panel_frame.show(&mut sizing_ui, |ui| {
            sizing_controls.contents(ui, count, frame, request, &track_caption);
        });
        let response = egui::TopBottomPanel::bottom("timeline")
            .frame(panel_frame)
            .exact_height(measured.response.rect.height().ceil())
            .show(ctx, |ui| {
                controls.contents(ui, count, frame, request, &track_caption);
            });
        // Commit real interaction once. Never rerun App::update to measure layout.
        self.timeline_step = controls.timeline_step;
        self.track = controls.track;
        self.follow_live = controls.follow_live;
        self.playing = controls.playing;
        self.play_time = controls.play_time;
        self.playback_speed = controls.playback_speed;
        self.playback_speed_text = controls.playback_speed_text;
        self.playback_speed_invalid = controls.playback_speed_invalid;
        response.response.rect
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        if self.scene.contents_selected() {
            self.contents_setup_ui(ui);
            return;
        }
        let configured = self.setup_workflow == SetupWorkflow::AcceptedSession;
        let material_preview = self.material_mode && !configured;
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
                if material_preview {self.materials.domain(ui, &mut self.domain_side);} else {
                    settings::quantity(ui,"Domain side",&mut self.domain_side,"m",0.01,0.001..=100.0);
                    ui.small("Filled carrier fluid throughout the box.");
                }
                if self.domain_side!=previous_side {
                    let scale=self.domain_side/previous_side;
                    self.emitter.center=self.emitter.center.map(|v|v*scale);
                    self.emitter.half_length*=scale;self.emitter.radius*=scale;self.emitter.nozzle_depth*=scale;
                }
            });
            if configured {
                self.configured_controls.show(ui, self.worker.is_none());
            } else {
            let previous_mesh = self.collision.mesh.clone();
            if self.collision.show(ui, self.domain_side, self.material_mode) {
                match self.scene.replace_model(self.collision.mesh.is_some()) {
                    Ok(()) => self.reset_timeline(),
                    Err(error) => {
                        self.collision.mesh = previous_mesh;
                        self.collision.status = format!("Import rejected; previous mesh retained: {error}");
                    }
                }
            }
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
            }
            egui::CollapsingHeader::new("Simulation").default_open(false).show(ui, |ui| {
                if !configured && ui.checkbox(&mut self.material_mode,"Liquid / gas material simulation").changed() {
                    self.emitter.pressure_heads=if self.material_mode {[1000.0,800.0]} else {[0.12,0.08]};
                }
                let material_preview = self.material_mode && !configured;
                ui.horizontal_wrapped(|ui| {ui.label(if material_preview {"Adaptive preview edge"} else {"Numerical cells / axis (count mode)"});ui.add_enabled(material_preview || self.numerical_cells_per_metre.is_none(),egui::DragValue::new(&mut self.size).range(4..=if material_preview {512}else{rheon::MAX_GRID_EDGE}).suffix("³"));});
                let mut physical = self.numerical_cells_per_metre.is_some();
                if ui.checkbox(&mut physical,"Select physical sampling density").changed() {
                    self.numerical_cells_per_metre = physical.then_some(32);
                }
                if let Some(density) = &mut self.numerical_cells_per_metre {
                    ui.add(egui::DragValue::new(density).range(1..=rheon::MAX_CELLS_PER_METRE).suffix(if material_preview {" particle samples/m"} else {" cells/m"}));
                    ui.small("Counts = ceil(domain side × requested density); exact physical extent retained.");
                }
                egui::ComboBox::from_id_salt("preview-resolution-mode").selected_text(match self.preview_resolution {
                    history::PreviewResolution::Adaptive => "Adaptive preview (legacy)",
                    history::PreviewResolution::CellsPerMetre(_) => "Exact preview density",
                    history::PreviewResolution::Edge(_) => "Exact preview edge count",
                }).show_ui(ui,|ui| {
                    if ui.selectable_label(self.preview_resolution == history::PreviewResolution::Adaptive,"Adaptive preview (legacy)").clicked() {self.preview_resolution=history::PreviewResolution::Adaptive;}
                    if ui.selectable_label(matches!(self.preview_resolution,history::PreviewResolution::CellsPerMetre(_)),"Exact preview density").clicked() {self.preview_resolution=history::PreviewResolution::CellsPerMetre(32);}
                    if ui.selectable_label(matches!(self.preview_resolution,history::PreviewResolution::Edge(_)),"Exact preview edge count").clicked() {self.preview_resolution=history::PreviewResolution::Edge(32);}
                });
                match &mut self.preview_resolution {
                    history::PreviewResolution::CellsPerMetre(density) => {ui.add(egui::DragValue::new(density).range(1..=rheon::MAX_CELLS_PER_METRE).suffix(" preview cells/m"));}
                    history::PreviewResolution::Edge(edge) => {ui.add(egui::DragValue::new(edge).range(1..=history::MAX_PREVIEW_EDGE).suffix(" preview cells / axis"));}
                    history::PreviewResolution::Adaptive => {}
                }
                ui.small("Preview is display resampling only. Finer voxels replicate appearance; physical sampling stays unchanged.");
                ui.horizontal_wrapped(|ui| {ui.label("Steps");ui.add(egui::DragValue::new(&mut self.steps).range(1..=10000));});
                {
                    let recording_label = egui::WidgetText::from("Numerical recording (every accepted step)")
                        .into_galley(ui, Some(egui::TextWrapMode::Wrap),
                            (ui.available_width() - ui.spacing().indent - ui.spacing().button_padding.x).max(1.0),
                            egui::TextStyle::Button);
                    egui::CollapsingHeader::new(recording_label)
                        .id_salt("Numerical recording (every accepted step)").show(ui, |ui| {
                        ui.label("Local recording directory");
                        ui.add(egui::TextEdit::singleline(&mut self.recording_directory).desired_width(ui.available_width()));
                        ui.add(egui::DragValue::new(&mut self.recording_database_mib).range(1..=16384).suffix(" MiB database cap"));
                        ui.add(egui::DragValue::new(&mut self.recording_record_mib).range(1..=64).suffix(" MiB per publication"));
                        ui.small("Retention: keep all; stop at limits. Caps apply per track/database; comparisons create separate archives. Record quota = requested steps + constructor. Files persist after reset; delete explicitly outside Rheon.");
                        ui.small("Rollback journal uses additional temporary disk, at most approximately one database plus sector/page overhead. SQLite/allocator memory is outside the payload limit.");
                    });
                }
                ui.small("Carrier inventory: native face velocity, cell appearance tracer, accepted interval pressure. Material inventory: particle attributes and preintegration diagnostics. Other Rheon session/lab backends are not recorded here.");
                ui.horizontal_wrapped(|ui| {ui.label("Timeline budget");ui.add(egui::DragValue::new(&mut self.history_budget_mib).range(16..=4096).suffix(" MiB"));});
                let sampling = if configured {
                    self.selected_carrier_count().map(|size| (size, None))
                } else {
                    self.selected_sampling()
                };
                match &sampling {
                    Ok((size, material)) => {
                        let numerical = material.map_or(*size as usize, |m|m.samples_per_axis);
                        ui.small(format!("Physical sampling: {numerical} per axis · {:.3} samples/m · {:.9} m spacing", numerical as f64 / self.domain_side, self.domain_side / numerical as f64));
                        if material.is_none() && let Ok(grid) = GridGeometry::new([*size;3],[self.domain_side / *size as f64;3],[0.0;3]) {
                                match rheon::BufferPlan::for_grid(&grid,192*1024*1024) {
                                    Ok(plan) => {ui.small(format!("{} numerical cells · {:.1} MiB retained native arrays / track; pressure work ≤ {} cell-iterations / track",grid.cell_len(),plan.total_bytes as f64/1048576.0,grid.cell_len() as u128*self.steps as u128*2000));},
                                    Err(e) => {ui.colored_label(egui::Color32::LIGHT_RED,e.to_string());}
                                }
                        }
                        let edge = self.preview_resolution.edge(self.domain_side,*size as usize);
                        match edge.and_then(|n|LiveHistory::plan_resolution(n,self.steps,1,self.history_budget_mib*1024*1024,self.preview_resolution.exact())) {
                            Ok(plan) => {ui.small(format!("Stored preview {}³ · {:.3} cells/m · {:.1} MiB for every step / track{}",plan.edge,plan.edge as f64/self.domain_side,plan.bytes as f64/1048576.0,if !self.preview_resolution.exact() {" (adaptive)"} else {" (exact)"}));
                                ui.small(viewport::resource_estimate(plan.edge));},
                            Err(e) => {ui.colored_label(egui::Color32::LIGHT_RED,e);}
                        }
                        ui.small("Payload/work estimates exclude allocator, recording, UI and model memory. Comparisons multiply tracks; actual launch preflights all tracks.");
                    }
                    Err(e) => {ui.colored_label(egui::Color32::LIGHT_RED,e);}
                }
                ui.small(if material_preview {"Particle samples control physical detail; preview voxels control the image."} else {"Numerical cells control physical detail; preview voxels control the image."});
                settings::quantity(ui,"Maximum step",&mut self.requested_dt,"s",0.001,0.000001..=0.1);
                ui.small("Accepted time steps adapt to physical stability limits.");
                if material_preview {
                    ui.horizontal_wrapped(|ui| {ui.label("Particle samples / side");ui.add_enabled(self.numerical_cells_per_metre.is_none(),egui::DragValue::new(&mut self.materials.config.samples_per_axis).range(4..=64));});

                    ui.horizontal_wrapped(|ui| {ui.label("Particle limit");ui.add(egui::DragValue::new(&mut self.materials.config.max_particles).range(32768..=262144));});
                    settings::quantity(ui,"Liquid preview stiffness",&mut self.materials.config.liquid_sound_speed,"m/s",1.0,1.0..=100.0).on_hover_text("Reduced acoustic speed for a weakly compressible preview; higher stiffness needs shorter steps. Gas pressure uses the ideal-gas equation.");
                } else {
                    let pressure_label = egui::WidgetText::from(self.selected.label()).into_galley(
                        ui, Some(egui::TextWrapMode::Wrap),
                        (ui.available_width() - 2.0 * ui.spacing().button_padding.x - ui.spacing().icon_spacing - ui.spacing().icon_width).max(1.0),
                        egui::TextStyle::Button);
                    egui::ComboBox::from_id_salt("implementation").selected_text(pressure_label).width(ui.available_width().min(240.0)).show_ui(ui,|ui| {
                        for method in PressureImplementation::ALL {ui.selectable_value(&mut self.selected,method,method.label());}
                    });
                    ui.checkbox(&mut self.tight,"Tight pressure tolerance");
                }
            });
            egui::CollapsingHeader::new("CPU workers").default_open(false).show(ui, |ui| {
                if ui.checkbox(&mut self.automatic_threads,"Automatic hardware detection").changed() {self.refresh_threads(available_threads());}
                ui.horizontal_wrapped(|ui| {ui.add_enabled(!self.automatic_threads,egui::DragValue::new(&mut self.threads).range(1..=self.thread_limit));ui.label(format!("/ {} available",self.thread_limit));});
                ui.checkbox(&mut self.show_workers,"Orange worker voxel boxes").on_hover_text("Sampled actual worker positions; diagnostics add calculation overhead.");
            });
            if !configured {
            ui.collapsing("Comparisons & reset", |ui| {
            if ui.button("Clear timeline").clicked() { self.reset_timeline(); self.status = "Ready to repeat simulation with accepted mesh".into(); }
            ui.small("Keeps mesh and saved recordings.");
            if !self.material_mode {
                ui.checkbox(&mut self.save_native_set, "Save immutable native run set");
                ui.small("Fixed actual dt; a shortened step retains its archive and reports unsupported saved intent.");
                if ui.button("Compare both pressure solvers").clicked() {self.start(true);}
            }
            if ui.add_enabled(self.worker.is_none() && self.scene.launch_readiness(force_run_policy::Target::Unsupported).is_ok(), egui::Button::new("Compare Rheon / trained AI")).clicked() {
                let mut comparison = ai_comparison::ComparisonApp::new(String::new(),std::env::current_dir().unwrap_or_default().join(".generated/ai-comparison"));
                comparison.set_steps(self.steps.clamp(1,512));
                self.ai_comparison = Some(comparison);
            }
            });
            }
        });
        egui::CollapsingHeader::new("Viewport & navigation").default_open(false).show(ui, |ui| {
            ui.checkbox(&mut self.viewport.touchpad_mode,"Touchpad scroll / two-finger orbit").on_hover_text("Enable when your touchpad reports wheel events. Shift pans; Ctrl zooms. Native pinch is supported.");
            ui.checkbox(&mut self.viewport.camera.orthographic,"Orthographic");
            if ui.button("Frame scene [Home]").clicked() {self.viewport.camera=viewport::OrbitCamera::default();}
            ui.horizontal_wrapped(|ui| {ui.label("Surface threshold");ui.add(egui::Slider::new(&mut self.viewport.threshold,1..=80).show_value(false));});
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
                        if frame.voxel.is_some() {
                            ui.small(format!("Cells with positive liquid fraction: {}", frame.nonzero_tracer_cells));
                            ui.small("Liquid appearance only; transparent gas remains occupied. Inspect both fractions in Numerical inspector.");
                        } else if frame.material_color.is_some() {
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
                            ui.label(&row.label).on_hover_text(if self.running_request.as_ref().is_some_and(|r| r.voxel.is_some()) {
                                "Filled-box two-material voxel method; Jacobi is its pressure component".to_owned()
                            } else { format!("Native baseline/fallback implementation: {}",row.method.label()) });
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
        if self.ai_comparison.is_some()
            || self.saved_comparison_active
            || self.run_sets.active
            || self.archive.active
        {
            self.scene.contents.hidden(ctx);
        }
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
        if self.saved_comparison_active {
            egui::TopBottomPanel::top("comparison-navigation").show(ctx, |ui| {
                if ui.button("Back to simulation").clicked() {
                    self.saved_comparison_active = false;
                }
            });
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("saved-comparison-and-fields")
                    .show(ui, |ui| {
                        self.saved_comparison.show(ui);
                        self.comparison_fields
                            .show(ui, self.saved_comparison.selected_pair());
                    });
            });
            return;
        }
        if self.run_sets.active {
            let mut compare = false;
            egui::TopBottomPanel::bottom("run-set-comparison-action").show(ctx, |ui| {
                compare = ui.button("Compare exact saved members").clicked();
            });
            if compare {
                let path = std::path::Path::new(&self.run_sets.path);
                if let (Some(parent), Some(name)) = (path.parent(), path.file_name()) {
                    self.saved_comparison
                        .set_location(parent.to_string_lossy(), name.to_string_lossy());
                    self.saved_comparison.load_manifest();
                    self.saved_comparison_active = true;
                    self.run_sets.active = false;
                    ctx.request_repaint();
                }
                return;
            }
            if let Some(revision) = self.run_sets.show(ctx, &mut self.archive) {
                let result = match revision {
                    run_set_view::SavedRunSet::Native(set) => self.rerun_saved(set),
                    run_set_view::SavedRunSet::Configured(set) => self.rerun_configured(set),
                    run_set_view::SavedRunSet::Voxel(set) => self.rerun_voxel(set),
                };
                if let Err(error) = result {
                    self.status = error;
                }
            }
            return;
        }
        if self.archive.active {
            self.archive.show(ctx);
            return;
        }
        if self.pending_voxel.is_none() && self.scene.context == scene_ui::SceneContext::Draft {
            self.playing = false;
        }
        if self.playing && self.pending_voxel.is_none() {
            self.play_time += self.playback_speed * ctx.input(|i| i.stable_dt.min(0.1)) as f64;
        }
        if self.scene.open && ctx.content_rect().width() < 1100.0 {
            self.scene_run_strip(ctx);
        } else {
            self.sidebar(ctx);
        }
        if self.archive.active || self.run_sets.active || self.saved_comparison_active {
            self.scene.contents.hidden(ctx);
            ctx.request_repaint();
            return;
        }
        let retained_view = self.prepare_scene_presentation();
        let model_mesh = if self.scene.context == scene_ui::SceneContext::Retained {
            self.running_request
                .as_ref()
                .and_then(|request| request.collision_mesh.as_ref())
        } else {
            self.collision.mesh.as_ref()
        };
        let model = model_mesh.map(|mesh| scene_ui::ModelSummary {
            triangles: mesh.triangles().len(),
            bounds: mesh.bounds(),
        });
        let previous_context = self.scene.context;
        if self.pending_voxel.is_some() {
            self.scene.show_preparing(ctx);
        } else {
            let contents_readiness = self.contents_run_readiness();
            self.scene.show(
                ctx,
                self.running_request.is_some(),
                self.worker.is_some(),
                model,
                self.setup_workflow.force_target(self.material_mode),
                contents_readiness,
            );
        }
        self.service_initial_contents_commands(ctx);
        if previous_context != self.scene.context {
            self.scene.contents.cancel();
            self.playing = false;
        }
        let retained = self.scene.context == scene_ui::SceneContext::Retained;
        let (count, selected) = if retained {
            retained_view.unwrap_or_else(|| self.selected_frame_bound_track())
        } else {
            (0, None)
        };
        self.scene_timeline_ui(ctx, count, selected.as_deref());
        let workers = if retained && self.follow_live && self.show_workers {
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
            .filter(|_| retained)
            .map_or(self.domain_side, |r| r.domain_side);
        self.viewport.domain_side = domain_side;
        let emitter = self
            .running_request
            .as_ref()
            .filter(|_| retained)
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
            .filter(|_| retained)
            .map_or_else(|| self.collision.mesh.clone(), |r| r.collision_mesh.clone());
        let grid = self
            .running_request
            .as_ref()
            .filter(|_| retained)
            .map_or(self.size, |r| r.size) as usize;
        self.viewport.authored_forces_pending = !self.scene.display_scene().objects.is_empty();
        let frame = selected;
        let editable_scene =
            self.worker.is_none() && self.scene.context == scene_ui::SceneContext::Draft;
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.add_enabled_ui(self.pending_voxel.is_none(), |ui| {
                    self.viewport.show_scene(
                        ui,
                        viewport::ViewInputs {
                            frame,
                            emitter,
                            workers: &workers,
                            grid_edge: grid,
                            inspector: self
                                .history
                                .as_ref()
                                .filter(|_| retained)
                                .map(|h| &h.inspector),
                            scene: Some(&mut self.scene),
                            editable_scene,
                        },
                    );
                });
            });
        if self.worker.is_some() || self.playing {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }
}
impl Drop for App {
    fn drop(&mut self) {
        if let Some(worker) = self.worker.take() {
            worker.cancel();
            let _ = worker.handle.join();
        }
    }
}
#[cfg(not(test))]
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
            let run = app.configure_arguments(std::env::args().skip(1))?;
            if !app.collision.path.is_empty() {
                app.collision
                    .apply(app.domain_side)
                    .map_err(std::io::Error::other)?;
                app.scene
                    .replace_model(true)
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
fn main() -> eframe::Result {
    native_presentation_harness::run_native_presentation()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn configured_carrier_density_freezes_independently_of_particle_mode_and_preview() {
        let root = configured_worker_tests::Output::new();
        for side in [0.01, 0.02] {
            let mut app = App::default();
            app.domain_side = side;
            app.numerical_cells_per_metre = Some(1000);
            app.preview_resolution = history::PreviewResolution::Edge(5);
            app.material_mode = true;
            app.size = 32;
            app.steps = 1;
            app.requested_dt = 0.000001;
            app.threads = 1;
            app.automatic_threads = false;
            app.show_workers = false;
            app.recording_directory = root.0.to_string_lossy().into();
            app.start_configured().unwrap();
            let request = app.running_request.as_ref().unwrap();
            let expected = (side * 1000.0) as u64;
            assert_eq!(request.size, expected);
            assert!(request.material.is_none());
            assert_eq!(request.preview_edge().unwrap(), 5);
            let spec = &request.configured_set.as_ref().unwrap().revision.members[0].spec;
            assert_eq!(spec.constructor.counts_xyz, [expected; 3]);
            assert_eq!(spec.constructor.spacing_m, [0.001; 3]);
            app.cancel();
            drop(app); // Existing App/Worker owns cancel and join before output cleanup.
        }
    }
    #[test]
    fn density_request_survives_domain_edits_and_does_not_follow_preview_count() {
        let mut app = App::default();
        app.material_mode = false;
        app.domain_side = 0.016;
        app.numerical_cells_per_metre = Some(1000);
        app.preview_resolution = history::PreviewResolution::Edge(5);
        assert_eq!(app.selected_sampling().unwrap().0, 16);
        app.domain_side = 0.032;
        assert_eq!(app.selected_sampling().unwrap().0, 32);
        assert_eq!(app.numerical_cells_per_metre, Some(1000));
        app.preview_resolution = history::PreviewResolution::Edge(10000);
        assert_eq!(app.selected_sampling().unwrap().0, 32);
        app.material_mode = true;
        let (_, material) = app.selected_sampling().unwrap();
        assert_eq!(material.unwrap().samples_per_axis, 32);
        app.domain_side = 1.0;
        assert!(app.selected_sampling().is_err());
        app.domain_side = 0.0015;
        assert!(
            app.selected_carrier_count()
                .unwrap_err()
                .contains("1333.333333")
        );
        app.domain_side = 0.002;
        assert_eq!(app.selected_carrier_count().unwrap(), 2);
        for count in 1..=64 {
            app.domain_side = count as f64 / 1000.0;
            assert_eq!(app.selected_carrier_count().unwrap(), count);
        }
        app.numerical_cells_per_metre = None;
        app.size = 32;
        app.domain_side = 0.016;
        assert!(app.selected_carrier_count().is_err());
    }
    #[test]
    fn saved_native_cold_rerun_process() {
        let Ok(root) = std::env::var("RHEON_SAVED_COLD_RERUN") else {
            return;
        };
        let root = std::path::PathBuf::from(root);
        let mut app = App::default();
        app.threads = 1;
        app.automatic_threads = false;
        app.show_workers = false;
        app.recording_directory = root.to_string_lossy().into();
        let pointer = root.join("source-path.txt");
        if std::env::var("RHEON_SAVED_COLD_PHASE").unwrap() == "produce" {
            app.material_mode = false;
            app.cylinder = false;
            app.size = 8;
            app.steps = 2;
            app.requested_dt = 0.003;
            app.save_native_set = true;
            app.start(true);
        } else {
            let path = std::fs::read_to_string(&pointer).unwrap();
            let source = run_sets::RunSetRevision::open(std::path::Path::new(&path)).unwrap();
            app.rerun_saved(source).unwrap();
        }
        wait_until(|| app.worker.as_ref().unwrap().handle.is_finished());
        app.poll(&egui::Context::default());
        assert_eq!(app.rows.len(), 2, "{}", app.status);
        let target = if pointer.exists() {
            root.join("rerun-path.txt")
        } else {
            pointer
        };
        std::fs::write(target, &app.run_sets.path).unwrap();
    }
    #[test]
    fn saved_native_rerun_after_producer_exit_avoids_all_source_ids() {
        let root = std::env::temp_dir().join(format!(
            "rheon-cold-rerun-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let exe = std::env::current_exe().unwrap();
        let mut original_files = Vec::new();
        for phase in ["produce", "rerun"] {
            let status = std::process::Command::new(&exe)
                .args([
                    "--exact",
                    "tests::saved_native_cold_rerun_process",
                    "--nocapture",
                ])
                .env("RHEON_SAVED_COLD_RERUN", &root)
                .env("RHEON_SAVED_COLD_PHASE", phase)
                .status()
                .unwrap();
            assert!(status.success(), "fresh {phase} process failed");
            if phase == "produce" {
                let source = std::path::PathBuf::from(
                    std::fs::read_to_string(root.join("source-path.txt")).unwrap(),
                );
                original_files = std::fs::read_dir(source.parent().unwrap())
                    .unwrap()
                    .map(|entry| {
                        let path = entry.unwrap().path();
                        let bytes = std::fs::read(&path).unwrap();
                        (path, bytes)
                    })
                    .collect();
            }
        }
        let source_path = std::path::PathBuf::from(
            std::fs::read_to_string(root.join("source-path.txt")).unwrap(),
        );
        let rerun_path =
            std::path::PathBuf::from(std::fs::read_to_string(root.join("rerun-path.txt")).unwrap());
        let source = run_sets::RunSetRevision::open(&source_path).unwrap();
        let rerun = run_sets::RunSetRevision::open(&rerun_path).unwrap();
        let source_ids: Vec<_> = source
            .members
            .iter()
            .map(|m| m.attempt.as_ref().unwrap().expected_run_id)
            .collect();
        assert_eq!(source_ids, [1, 2]);
        let intent =
            run_sets::RunSetRevision::open(&rerun_path.parent().unwrap().join("intent.json"))
                .unwrap();
        assert_eq!(intent.parent_sha256, Some(source.sha256().unwrap()));
        for (old, new) in source.members.iter().zip(&rerun.members) {
            let attempt = new.attempt.as_ref().unwrap();
            assert!(!source_ids.contains(&attempt.expected_run_id));
            assert_ne!(old.attempt.as_ref().unwrap().attempt_id, attempt.attempt_id);
            assert_eq!(old.spec, new.spec);
            let run_sets::MemberState::Ready(a) =
                run_sets::open_member(source_path.parent().unwrap(), old)
            else {
                panic!("source member unavailable");
            };
            let run_sets::MemberState::Ready(b) =
                run_sets::open_member(rerun_path.parent().unwrap(), new)
            else {
                panic!("rerun member unavailable");
            };
            for ordinal in 0..3 {
                let a = a.select(ordinal).unwrap();
                let b = b.select(ordinal).unwrap();
                assert_eq!(a.time.to_bits(), b.time.to_bits());
                for (a, b) in a.fields.iter().zip(&b.fields) {
                    assert_eq!(a.data, b.data);
                }
            }
        }
        for (path, before) in original_files {
            assert_eq!(std::fs::read(path).unwrap(), before);
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn saved_native_worker_closes_and_cold_opens_exact_two_member_set() {
        saved_native_worker_case(false);
    }
    #[test]
    fn thousand_per_metre_zero_flow_cold_reruns_with_independent_preview_and_unchanged_archive() {
        saved_native_worker_case(true);
    }
    fn saved_native_worker_case(high_density: bool) {
        let root = std::env::temp_dir().join(format!(
            "rheon-saved-worker-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut app = App::default();
        app.material_mode = false;
        app.cylinder = false;
        app.size = 8;
        if high_density {
            app.domain_side = 0.008;
            app.numerical_cells_per_metre = Some(1000);
            app.preview_resolution = history::PreviewResolution::Edge(5);
        }
        app.steps = if high_density { 1 } else { 2 };
        let last_ordinal = app.steps;
        app.requested_dt = if high_density { 0.000001 } else { 0.003 };
        app.threads = 1;
        app.automatic_threads = false;
        app.show_workers = false;
        app.save_native_set = true;
        app.recording_directory = root.to_string_lossy().into();
        app.start(true);
        assert!(app.worker.is_some(), "{}", app.status);
        wait_until(|| app.worker.as_ref().unwrap().handle.is_finished());
        app.poll(&egui::Context::default());
        assert!(app.worker.is_none());
        assert_eq!(app.rows.len(), 2, "{}", app.status);
        let manifest = std::path::PathBuf::from(&app.run_sets.path);
        assert!(manifest.ends_with("completed.json"));
        let mut retry = app.running_request.as_ref().unwrap().clone();
        if high_density {
            assert_eq!(retry.size, 8);
            assert_eq!(retry.domain_side / retry.size as f64, 0.001);
            assert_eq!(app.history.as_ref().unwrap().preview_edge, 5);
        }
        drop(app);
        let set = run_sets::RunSetRevision::open(&manifest).unwrap();
        assert_eq!(set.members.len(), 2);
        for member in &set.members {
            let run_sets::MemberState::Ready(reader) =
                run_sets::open_member(manifest.parent().unwrap(), member)
            else {
                panic!("cold member unavailable");
            };
            assert_eq!(reader.count().unwrap(), last_ordinal + 1);
            assert_eq!(reader.status().unwrap().outcome, "completed");
            assert_eq!(
                reader.select(last_ordinal).unwrap().generation,
                last_ordinal as u64
            );
        }
        let mut view = run_set_view::RunSetView::default();
        view.path = manifest.to_string_lossy().into();
        view.open().unwrap();
        let context = egui::Context::default();
        let mut archive = archive_view::ArchiveView::default();
        let _ = context.run(egui::RawInput::default(), |ctx| {
            let _ = view.show(ctx, &mut archive);
        });
        // Opening an invalid manifest invalidates the previous member capability.
        view.path = root.join("missing.json").to_string_lossy().into();
        assert!(view.open().is_err());
        assert!(!view.active);
        let original_bytes = std::fs::read(&manifest).unwrap();
        let mut again = App::default();
        again.threads = 1;
        again.show_workers = false;
        if high_density {
            again.preview_resolution = history::PreviewResolution::Edge(7);
        }
        again.recording_directory = root.to_string_lossy().into();
        again.rerun_saved(set.clone()).unwrap();
        wait_until(|| again.worker.as_ref().unwrap().handle.is_finished());
        again.poll(&egui::Context::default());
        assert_eq!(again.rows.len(), 2, "{}", again.status);
        if high_density {
            assert_eq!(again.history.as_ref().unwrap().preview_edge, 7);
        }
        let repeated_path = std::path::PathBuf::from(&again.run_sets.path);
        assert_ne!(repeated_path, manifest);
        let intent =
            run_sets::RunSetRevision::open(&repeated_path.parent().unwrap().join("intent.json"))
                .unwrap();
        assert_eq!(intent.parent_sha256, Some(set.sha256().unwrap()));
        let repeated = run_sets::RunSetRevision::open(&repeated_path).unwrap();
        for (old, new) in set.members.iter().zip(&repeated.members) {
            assert_eq!(old.spec, new.spec);
            assert_ne!(
                old.attempt.as_ref().unwrap().attempt_id,
                new.attempt.as_ref().unwrap().attempt_id
            );
            let run_sets::MemberState::Ready(a) =
                run_sets::open_member(manifest.parent().unwrap(), old)
            else {
                panic!("original member");
            };
            let run_sets::MemberState::Ready(b) =
                run_sets::open_member(repeated_path.parent().unwrap(), new)
            else {
                panic!("rerun member");
            };
            assert_ne!(a.run().identity.run_id, b.run().identity.run_id);
            let a = a.select(last_ordinal).unwrap();
            let b = b.select(last_ordinal).unwrap();
            assert_eq!(a.time.to_bits(), b.time.to_bits());
            for (a, b) in a.fields.iter().zip(&b.fields) {
                assert_eq!(a.data, b.data);
            }
        }
        drop(again);
        assert_eq!(std::fs::read(&manifest).unwrap(), original_bytes);
        if high_density {
            std::fs::remove_dir_all(root).unwrap();
            return;
        }
        retry.recording_directory = root.to_string_lossy().into();
        retry.saved_set = None;
        let retry_initial = freeze_native_set(&mut retry).unwrap();
        let retry_receipt = Arc::clone(retry.manifest_receipt.as_ref().unwrap());
        let retry_completed = retry_initial.parent().unwrap().join("completed.json");
        let (reached, wait) = std::sync::mpsc::sync_channel(1);
        let (release, proceed) = std::sync::mpsc::sync_channel(1);
        retry.progress_gate = Some(Arc::new(ProgressGate {
            reached,
            release: std::sync::Mutex::new(proceed),
        }));
        let mut cancelled_app = App::default();
        cancelled_app.running_request = Some(retry.clone());
        cancelled_app.worker = Some(Worker::start(retry).unwrap());
        wait.recv_timeout(Duration::from_secs(10)).unwrap();
        cancelled_app.cancel();
        release.send(()).unwrap();
        wait_until(|| cancelled_app.worker.as_ref().unwrap().handle.is_finished());
        cancelled_app.poll(&egui::Context::default());
        assert!(cancelled_app.worker.is_none());
        assert!(
            cancelled_app.status.contains("Cancelled")
                || cancelled_app.status.contains("cancelled")
        );
        assert_eq!(
            std::path::Path::new(&cancelled_app.run_sets.path),
            retry_receipt.lock().unwrap().as_path()
        );
        drop(cancelled_app);
        assert!(!retry_completed.exists());
        let terminal_path = retry_receipt.lock().unwrap().clone();
        assert!(terminal_path.ends_with("member-0-finished.json"));
        assert!(terminal_path.exists());
        let cancelled = run_sets::RunSetRevision::open(
            &retry_completed
                .parent()
                .unwrap()
                .join("member-0-finished.json"),
        )
        .unwrap();
        let run_sets::MemberState::Ready(reader) =
            run_sets::open_member(retry_completed.parent().unwrap(), &cancelled.members[0])
        else {
            panic!("cancelled prefix unavailable");
        };
        assert_eq!(reader.status().unwrap().outcome, "cancelled");
        assert_eq!(reader.count().unwrap(), 2);
        assert!(matches!(
            run_sets::open_member(retry_completed.parent().unwrap(), &cancelled.members[1]),
            run_sets::MemberState::NotRun
        ));
        drop(reader);
        std::fs::remove_dir_all(root).unwrap();
    }

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
            force_fields: Vec::new(),
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
            preview_resolution: history::PreviewResolution::Adaptive,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: Some(rheon::ParticleFluidConfig::default()),
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: Vec::new(),
            voxel: None,
            manifest_receipt: None,
            progress_gate: None,
            #[cfg(test)]
            failure_gate: None,
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
            force_fields: Vec::new(),
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
            preview_resolution: history::PreviewResolution::Adaptive,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: Some(rheon::ParticleFluidConfig::default()),
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: Vec::new(),
            voxel: None,
            manifest_receipt: None,
            progress_gate: Some(Arc::new(ProgressGate {
                reached,
                release: std::sync::Mutex::new(resume),
            })),
            failure_gate: None,
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
            force_fields: Vec::new(),
            collision_mesh: None,
            size: 32,
            steps: 3,
            tight: false,
            threads: available_threads().min(2),
            methods: vec![PressureImplementation::SymmetricGaussSeidelPcgV1],
            emitter: Some(RotatingCylinderEmitter::default()),
            observe: true,
            history_budget_mib: 96,
            preview_resolution: history::PreviewResolution::Adaptive,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: None,
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: Vec::new(),
            voxel: None,
            manifest_receipt: None,
            progress_gate: Some(Arc::new(ProgressGate {
                reached,
                release: std::sync::Mutex::new(resume),
            })),
            failure_gate: None,
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
            force_fields: Vec::new(),
            collision_mesh: None,
            size: app.size,
            steps: app.steps,
            tight: app.tight,
            threads: app.threads,
            methods: PressureImplementation::ALL.to_vec(),
            emitter: None,
            observe: false,
            history_budget_mib: 96,
            preview_resolution: history::PreviewResolution::Adaptive,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: None,
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: Vec::new(),
            voxel: None,
            manifest_receipt: None,
            progress_gate: Some(Arc::new(ProgressGate {
                reached,
                release: std::sync::Mutex::new(resume),
            })),
            failure_gate: None,
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
            child: None,
            handle,
        });
        drop(app);
        assert!(cancel.load(Ordering::Relaxed));
        assert!(exited.load(Ordering::Relaxed));
    }
    #[test]
    fn worker_runs_selected_and_equal_input_comparison() {
        let request = Request {
            force_fields: Vec::new(),
            collision_mesh: None,
            size: 8,
            steps: 4,
            tight: false,
            threads: available_threads().min(4),
            methods: PressureImplementation::ALL.to_vec(),
            emitter: None,
            observe: false,
            history_budget_mib: 96,
            preview_resolution: history::PreviewResolution::Adaptive,
            domain_side: 1.0,
            requested_dt: 0.02,
            material: None,
            material_color: [85, 190, 245],
            recording_limits: rheon::recording::RecordingLimits::default(),
            recording_directory: ".generated/recordings".into(),
            saved_set: None,
            configured_set: None,
            prior_run_ids: Vec::new(),
            voxel: None,
            manifest_receipt: None,
            progress_gate: None,
            #[cfg(test)]
            failure_gate: None,
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
                    force_fields: Vec::new(),
                    collision_mesh: None,
                    size: 8,
                    steps: 4,
                    tight: false,
                    threads: available_threads().min(4),
                    methods: PressureImplementation::ALL.to_vec(),
                    emitter: None,
                    observe: false,
                    history_budget_mib: 96,
                    preview_resolution: history::PreviewResolution::Adaptive,
                    domain_side: 1.0,
                    requested_dt: 0.02,
                    material: None,
                    material_color: [85, 190, 245],
                    recording_limits: rheon::recording::RecordingLimits::default(),
                    recording_directory: ".generated/recordings".into(),
                    saved_set: None,
                    configured_set: None,
                    prior_run_ids: Vec::new(),
                    voxel: None,
                    manifest_receipt: None,
                    progress_gate: None,
                    failure_gate: None
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
