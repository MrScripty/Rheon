//! One joinable owner. Terminal state follows model kill/wait and reader joins.
use super::{
    diagnostic_runs::{FrozenDiagnosticSpec, RunStatistics},
    engine::{Engine, ExecutionPolicy, Frame},
    model_process::{ChildSlot, ModelProcess, ProviderIdentity, cancel},
    record::Record,
    scene::Scene,
};
use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
pub(super) struct RunOptions {
    pub policy: ExecutionPolicy,
    pub saved: Option<(FrozenDiagnosticSpec, [u8; 32])>,
}
pub struct Shared {
    pub frames: Vec<Arc<Frame>>,
    pub message: String,
    pub state: String,
    pub directory: Option<PathBuf>,
    pub provider: Option<ProviderIdentity>,
    pub pressure_range: f64,
    pub difference_range: f64,
    pub policy: ExecutionPolicy,
    pub last_attempt: Option<Arc<rheon::recording::diagnostic::DiagnosticAttempt>>,
    pub diagnostic_manifest: Option<PathBuf>,
    pub statistics: Option<RunStatistics>,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            message: "Preparing CPU checkpoint".into(),
            state: "running".into(),
            directory: None,
            provider: None,
            pressure_range: 0.0,
            difference_range: 0.0,
            policy: ExecutionPolicy::LegacyRawContinuation,
            last_attempt: None,
            diagnostic_manifest: None,
            statistics: None,
        }
    }
}
pub struct Worker {
    pub shared: Arc<Mutex<Shared>>,
    pub flag: Arc<AtomicBool>,
    slot: ChildSlot,
    handle: Option<JoinHandle<()>>,
}
impl Worker {
    pub fn start(scene: Scene, deployment: PathBuf, root: PathBuf) -> Result<Self, String> {
        Self::start_with_policy(
            scene,
            deployment,
            root,
            ExecutionPolicy::LegacyRawContinuation,
        )
    }
    pub fn start_with_policy(
        scene: Scene,
        deployment: PathBuf,
        root: PathBuf,
        policy: ExecutionPolicy,
    ) -> Result<Self, String> {
        Self::start_configured(scene, deployment, root, policy, None)
    }
    pub fn start_saved(
        spec: FrozenDiagnosticSpec,
        parent: [u8; 32],
        root: PathBuf,
    ) -> Result<Self, String> {
        spec.validate()?;
        Self::start_configured(
            spec.scene.clone(),
            PathBuf::from(&spec.deployment),
            root,
            spec.policy,
            Some((spec, parent)),
        )
    }
    fn start_configured(
        scene: Scene,
        deployment: PathBuf,
        root: PathBuf,
        policy: ExecutionPolicy,
        saved: Option<(FrozenDiagnosticSpec, [u8; 32])>,
    ) -> Result<Self, String> {
        scene.validate()?;
        let shared = Arc::new(Mutex::new(Shared::default()));
        let flag = Arc::new(AtomicBool::new(false));
        let slot = Arc::new(Mutex::new(None));
        let view = Arc::clone(&shared);
        let cancelled = Arc::clone(&flag);
        let child = Arc::clone(&slot);
        let handle = thread::Builder::new()
            .name("water-ai-comparison".into())
            .spawn(move || {
                if let Err(error) = run_configured(
                    scene,
                    deployment,
                    root,
                    &view,
                    &cancelled,
                    child,
                    RunOptions { policy, saved },
                ) && let Ok(mut data) = view.lock()
                {
                    data.state = if policy == ExecutionPolicy::LegacyRawContinuation
                        && cancelled.load(Ordering::Relaxed)
                    {
                        "cancelled"
                    } else {
                        "failed"
                    }
                    .into();
                    data.message = error;
                }
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            shared,
            flag,
            slot,
            handle: Some(handle),
        })
    }
    pub fn cancel(&self) {
        cancel(&self.slot, &self.flag);
    }
    pub fn finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }
    pub fn join_finished(&mut self) {
        if self.finished()
            && let Some(handle) = self.handle.take()
            && handle.join().is_err()
        {
            self.cancel();
            if let Ok(mut data) = self.shared.lock() {
                data.state = "failed".into();
                data.message =
                    "Comparison worker panicked; terminal recording is unconfirmed".into();
            }
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.cancel();
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        let child = self.slot.lock().ok().and_then(|mut guard| guard.take());
        if let Some(mut child) = child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
fn update(shared: &Mutex<Shared>, message: &str) -> Result<(), String> {
    shared
        .lock()
        .map_err(|_| "Comparison history lock")?
        .message = message.into();
    Ok(())
}
pub fn run(
    scene: Scene,
    deployment: PathBuf,
    root: PathBuf,
    shared: &Mutex<Shared>,
    flag: &AtomicBool,
    slot: ChildSlot,
) -> Result<(), String> {
    run_configured(
        scene,
        deployment,
        root,
        shared,
        flag,
        slot,
        RunOptions {
            policy: ExecutionPolicy::LegacyRawContinuation,
            saved: None,
        },
    )
}
pub fn run_with_policy(
    scene: Scene,
    deployment: PathBuf,
    root: PathBuf,
    shared: &Mutex<Shared>,
    flag: &AtomicBool,
    slot: ChildSlot,
    policy: ExecutionPolicy,
) -> Result<(), String> {
    run_configured(
        scene,
        deployment,
        root,
        shared,
        flag,
        slot,
        RunOptions {
            policy,
            saved: None,
        },
    )
}
fn run_configured(
    scene: Scene,
    deployment: PathBuf,
    root: PathBuf,
    shared: &Mutex<Shared>,
    flag: &AtomicBool,
    slot: ChildSlot,
    options: RunOptions,
) -> Result<(), String> {
    scene.validate()?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .map_err(|e| e.to_string())?;
    pool.install(|| {
        if options.policy == ExecutionPolicy::LegacyRawContinuation {
            run_native(scene, deployment, root, shared, flag, slot)
        } else {
            super::diagnostic_worker::run_typed(
                scene, deployment, root, shared, flag, slot, options,
            )
        }
    })
}
fn run_native(
    scene: Scene,
    deployment: PathBuf,
    root: PathBuf,
    shared: &Mutex<Shared>,
    flag: &AtomicBool,
    slot: ChildSlot,
) -> Result<(), String> {
    let mut record = Record::begin(&root, &scene)?;
    {
        shared.lock().map_err(|_| "Comparison lock")?.directory = Some(record.directory.clone());
    }
    record.activity("running", "Validating CPU deployment", 0)?;
    let result = (|| -> Result<String, String> {
        ModelProcess::preflight(&deployment, &scene)?;
        let mut model = ModelProcess::start(&deployment, slot, flag, |phase| {
            update(shared, phase)?;
            record.activity("running", phase, 0)
        })?;
        record.model(&model.identity, &model.deployment_path)?;
        shared.lock().map_err(|_| "Comparison lock")?.provider = Some(model.identity.clone());
        if scene.warmup_steps < model.minimum_native_steps {
            return Err(format!(
                "Unsupported testing setup: checkpoint requires at least {} native warm-up steps; configured {}",
                model.minimum_native_steps, scene.warmup_steps
            ));
        }
        let mut engine = Engine::new(scene.clone())?;
        let initial = Arc::new(engine.initial());
        record.frame(&initial)?;
        shared
            .lock()
            .map_err(|_| "Comparison lock")?
            .frames
            .push(initial);
        let deadline = Instant::now() + Duration::from_secs(600);
        let mut qualified = 0;
        let mut attempted = 0;
        for _ in 0..scene.steps {
            if flag.load(Ordering::Relaxed) {
                return Err("Cancelled; retained paired frames preserved".into());
            }
            if Instant::now() >= deadline {
                return Err("Paired-run 600 s deadline reached".into());
            }
            let frame = Arc::new(engine.advance(flag, |input| model.predict(input, flag))?);
            let hard_stop = frame.ai_quality.as_ref().is_some_and(|q| q.hard_stop);
            let message = if let Some(quality) = &frame.ai_quality {
                attempted += 1;
                qualified += usize::from(quality.qualified);
                format!(
                    "Step {}/{}; AI pressure qualified {qualified}/{attempted}; raw diagnostic track",
                    frame.step, scene.steps
                )
            } else {
                format!(
                    "Native warm-up {}/{}; AI not evaluated",
                    frame.step, scene.warmup_steps
                )
            };
            record.frame(&frame)?;
            record.activity("running", &message, frame.step)?;
            let ranges = super::view::ranges(&frame);
            {
                let mut view = shared.lock().map_err(|_| "Comparison lock")?;
                view.message = message;
                view.pressure_range = view.pressure_range.max(ranges.0);
                view.difference_range = view.difference_range.max(ranges.1);
                view.frames.push(frame);
            }
            if hard_stop {
                return Err(
                    "AI diagnostic reached its Courant/flux bound; last raw paired frame retained"
                        .into(),
                );
            }
            let next = Instant::now() + Duration::from_millis(100);
            while Instant::now() < next && !flag.load(Ordering::Relaxed) {
                thread::sleep(Duration::from_millis(10));
            }
        }
        drop(model);
        Ok(format!(
            "{} paired steps complete ({} native warm-up); AI pressure qualified {qualified}/{attempted}; CPU model drained",
            scene.steps, scene.warmup_steps
        ))
    })();
    let steps = shared
        .lock()
        .map_err(|_| "Comparison lock")?
        .frames
        .last()
        .map_or(0, |f| f.step);
    let (state, message) = match &result {
        Ok(v) => ("completed", v.as_str()),
        Err(v) if flag.load(Ordering::Relaxed) => ("cancelled", v.as_str()),
        Err(v) => ("failed", v.as_str()),
    };
    record.finish(state, message, steps)?;
    {
        let mut view = shared.lock().map_err(|_| "Comparison lock")?;
        view.state = state.into();
        view.message = message.into();
    }
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "linux")]
    use super::super::cpu_contract_tests::assert_child_liveness;
    use super::*;

    fn wait_for(worker: &Worker, mut ready: impl FnMut(&Shared) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            if ready(&worker.shared.lock().unwrap()) {
                return;
            }
            assert!(
                !worker.finished(),
                "worker ended before requested observation"
            );
            assert!(Instant::now() < deadline, "comparison observation deadline");
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn configurable_cpu_adapter_cancels_after_predictions_and_restarts() {
        let package = super::super::cpu_contract_tests::Fixture::new(false);
        let root = package.root.join("observations");
        let scene = Scene {
            counts: [8, 6, 5],
            steps: 64,
            ..Default::default()
        };
        let mut first =
            Worker::start(scene.clone(), package.manifest.clone(), root.clone()).unwrap();
        wait_for(&first, |data| {
            data.frames.last().is_some_and(|f| f.step >= 17)
        });
        let first_directory = first.shared.lock().unwrap().directory.clone();
        let first_pid = first.slot.lock().unwrap().as_ref().unwrap().id();
        #[cfg(target_os = "linux")]
        assert_child_liveness(first_pid, true);
        first.cancel();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !first.finished() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        first.join_finished();
        assert!(first.slot.lock().unwrap().is_none());
        assert_eq!(first.shared.lock().unwrap().state, "cancelled");
        assert!(first.shared.lock().unwrap().frames.len() >= 18);
        let terminal: serde_json::Value = serde_json::from_slice(
            &std::fs::read(first_directory.as_ref().unwrap().join("completion.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(terminal["state"], "cancelled");
        #[cfg(target_os = "linux")]
        assert_child_liveness(first_pid, false);
        drop(first);
        let mut second =
            Worker::start(Scene { steps: 16, ..scene }, package.manifest.clone(), root).unwrap();
        wait_for(&second, |data| !data.frames.is_empty());
        let second_pid = second.slot.lock().unwrap().as_ref().unwrap().id();
        #[cfg(target_os = "linux")]
        assert_child_liveness(second_pid, true);
        let deadline = Instant::now() + Duration::from_secs(20);
        while !second.finished() {
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        second.join_finished();
        assert!(second.slot.lock().unwrap().is_none());
        let data = second.shared.lock().unwrap();
        assert_eq!(data.state, "completed", "{}", data.message);
        assert_eq!(data.frames.len(), 17);
        assert_eq!(
            data.frames
                .iter()
                .filter(|f| f.ai_quality.is_some())
                .count(),
            1
        );
        assert_ne!(data.directory, first_directory);
        #[cfg(target_os = "linux")]
        assert_child_liveness(second_pid, false);
    }

    #[test]
    #[ignore = "Requires an exported measured CPU deployment; see docs/ai-pressure-comparison.md"]
    fn real_cpu_deployment_cancellation_drains_child_and_allows_restart() {
        let deployment =
            PathBuf::from(std::env::var_os("RHEON_AI_TEST_DEPLOYMENT").expect("deployment path"));
        let root =
            PathBuf::from(std::env::var_os("RHEON_AI_TEST_OUTPUT").expect("fresh output path"));
        assert!(
            !root.exists(),
            "lifecycle evidence needs a fresh output directory"
        );
        let scene = Scene {
            counts: [8, 6, 5],
            steps: 64,
            ..Default::default()
        };
        let mut first = Worker::start(scene.clone(), deployment.clone(), root.clone()).unwrap();
        wait_for(&first, |data| {
            data.frames.last().is_some_and(|f| f.step >= 17)
        });
        let child_id = first
            .slot
            .lock()
            .unwrap()
            .as_ref()
            .expect("resident CPU child")
            .id();
        #[cfg(target_os = "linux")]
        assert_child_liveness(child_id, true);
        let first_directory = first.shared.lock().unwrap().directory.clone().unwrap();
        first.cancel();
        let deadline = Instant::now() + Duration::from_secs(30);
        while !first.finished() {
            assert!(Instant::now() < deadline, "cancel did not drain owned work");
            thread::sleep(Duration::from_millis(10));
        }
        first.join_finished();
        assert!(first.slot.lock().unwrap().is_none());
        #[cfg(target_os = "linux")]
        assert_child_liveness(child_id, false);
        assert_eq!(first.shared.lock().unwrap().state, "cancelled");
        assert!(first.shared.lock().unwrap().frames.len() >= 18);
        let completion: serde_json::Value = serde_json::from_slice(
            &std::fs::read(first_directory.join("completion.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(completion["state"], "cancelled");
        drop(first);
        let mut second =
            Worker::start(Scene { steps: 16, ..scene }, deployment, root.clone()).unwrap();
        wait_for(&second, |data| !data.frames.is_empty());
        let restart_child_id = second
            .slot
            .lock()
            .unwrap()
            .as_ref()
            .expect("resident restart CPU child")
            .id();
        #[cfg(target_os = "linux")]
        assert_child_liveness(restart_child_id, true);
        let deadline = Instant::now() + Duration::from_secs(120);
        while !second.finished() {
            assert!(Instant::now() < deadline, "restart deadline");
            thread::sleep(Duration::from_millis(10));
        }
        second.join_finished();
        assert!(second.slot.lock().unwrap().is_none());
        #[cfg(target_os = "linux")]
        assert_child_liveness(restart_child_id, false);
        let view = second.shared.lock().unwrap();
        assert_eq!(view.state, "completed", "{}", view.message);
        assert_eq!(view.frames.len(), 17);
        assert_ne!(view.directory.as_ref(), Some(&first_directory));
        let restart_ai_predictions = view
            .frames
            .iter()
            .filter(|frame| frame.ai_quality.is_some())
            .count();
        assert_eq!(
            restart_ai_predictions, 1,
            "16 steps must include AI inference after 15 native steps"
        );
        assert_eq!(
            view.frames.last().unwrap().phase,
            super::super::engine::FramePhase::AiPrediction
        );
        assert_eq!(view.frames.last().unwrap().step, 16);
        assert!(
            view.frames
                .iter()
                .take(16)
                .all(|frame| frame.ai_quality.is_none())
        );
        println!(
            "Cancelled resident CPU child {child_id}; terminal record and retained frames verified; restart child {restart_child_id} completed 16 steps with {restart_ai_predictions} AI prediction; child slots empty; Linux OS liveness positive-before-negative checked: {}",
            cfg!(target_os = "linux")
        );
    }
}
