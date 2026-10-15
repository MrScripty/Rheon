//! One joinable owner. Terminal state follows model kill/wait and reader joins.
use super::{
    engine::{Engine, Frame},
    model_process::{ChildSlot, ModelProcess, cancel},
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
pub struct Shared {
    pub frames: Vec<Arc<Frame>>,
    pub message: String,
    pub state: String,
    pub directory: Option<PathBuf>,
    pub checkpoint: String,
    pub pressure_range: f64,
    pub difference_range: f64,
}
impl Default for Shared {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            message: "Preparing CPU checkpoint".into(),
            state: "running".into(),
            directory: None,
            checkpoint: String::new(),
            pressure_range: 0.0,
            difference_range: 0.0,
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
                if let Err(error) = run(scene, deployment, root, &view, &cancelled, child)
                    && let Ok(mut data) = view.lock()
                {
                    data.state = if cancelled.load(Ordering::Relaxed) {
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
    scene.validate()?;
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .map_err(|e| e.to_string())?;
    pool.install(|| run_native(scene, deployment, root, shared, flag, slot))
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
        let mut model = ModelProcess::start(&deployment, slot, flag, |phase| {
            update(shared, phase)?;
            record.activity("running", phase, 0)
        })?;
        record.model(
            &model.checkpoint,
            &model.training_run,
            &model.deployment_path,
        )?;
        shared.lock().map_err(|_| "Comparison lock")?.checkpoint = model.checkpoint.clone();
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
        let first_directory = first.shared.lock().unwrap().directory.clone().unwrap();
        first.cancel();
        let deadline = Instant::now() + Duration::from_secs(30);
        while !first.finished() {
            assert!(Instant::now() < deadline, "cancel did not drain owned work");
            thread::sleep(Duration::from_millis(10));
        }
        first.join_finished();
        assert!(first.slot.lock().unwrap().is_none());
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
        let deadline = Instant::now() + Duration::from_secs(120);
        while !second.finished() {
            assert!(Instant::now() < deadline, "restart deadline");
            thread::sleep(Duration::from_millis(10));
        }
        second.join_finished();
        assert!(second.slot.lock().unwrap().is_none());
        let view = second.shared.lock().unwrap();
        assert_eq!(view.state, "completed", "{}", view.message);
        assert_eq!(view.frames.len(), 17);
        assert_ne!(view.directory.as_ref(), Some(&first_directory));
        println!(
            "Cancelled resident CPU child {child_id}; terminal record and retained frames verified; restart completed; child slots empty"
        );
    }
}
