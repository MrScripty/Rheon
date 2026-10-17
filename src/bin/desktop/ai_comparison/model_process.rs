//! One bounded CPU child, owned readers, explicit cancellation and teardown.
use super::adapter::{AdapterPackage, InputSemantic, PriorPressure};
use super::engine::Input;
use rheon::recording::diagnostic::NoCandidateReason;
use serde::Serialize;
use serde_json::Value;
#[cfg(test)]
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
const LIMIT: usize = 8 * 1024 * 1024;
#[derive(Debug)]
pub(crate) enum StartupFailure {
    Cancelled,
    Failure(String),
}
impl std::fmt::Display for StartupFailure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => out.write_str("Cancelled"),
            Self::Failure(detail) => out.write_str(detail),
        }
    }
}
impl From<String> for StartupFailure {
    fn from(detail: String) -> Self {
        Self::Failure(detail)
    }
}
impl From<&str> for StartupFailure {
    fn from(detail: &str) -> Self {
        Self::Failure(detail.into())
    }
}
pub use rheon::diagnostic::PredictionFailure;
fn prediction_failure(
    reason: NoCandidateReason,
    detail: impl AsRef<str>,
    dispatched: bool,
    digest: Option<[u8; 32]>,
) -> PredictionFailure {
    PredictionFailure {
        reason,
        detail: detail
            .as_ref()
            .chars()
            .filter(|c| !c.is_control())
            .take(2000)
            .collect(),
        dispatched,
        invalid_payload_sha256: digest,
    }
}
struct ReceivedReply {
    reply: Reply,
    payload_sha256: [u8; 32],
}
pub type ChildSlot = Arc<Mutex<Option<Child>>>;
use rheon::diagnostic::{AdapterErrorCode as ErrorCode, AdapterReply as Reply};
fn bridge_error(code: ErrorCode, message: String) -> String {
    code.detail(message)
}
fn kill(slot: &ChildSlot) {
    if let Ok(mut guard) = slot.lock()
        && let Some(child) = guard.as_mut()
    {
        let _ = child.kill();
    }
}
pub fn cancel(slot: &ChildSlot, flag: &AtomicBool) {
    flag.store(true, Ordering::Relaxed);
    kill(slot);
}
fn deliver(
    sender: &SyncSender<Result<ReceivedReply, PredictionFailure>>,
    mut item: Result<ReceivedReply, PredictionFailure>,
    stop: &AtomicBool,
) {
    while !stop.load(Ordering::Relaxed) {
        match sender.try_send(item) {
            Ok(()) | Err(TrySendError::Disconnected(_)) => return,
            Err(TrySendError::Full(v)) => {
                item = v;
                thread::sleep(Duration::from_millis(1));
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProviderIdentity {
    pub id: String,
    pub version: String,
    pub manifest_sha256: String,
    pub checkpoint_sha256: Option<String>,
    pub training_run_id: Option<String>,
}
struct RuntimeConfig {
    python: PathBuf,
    bridge: PathBuf,
    identity: ProviderIdentity,
    minimum_native_steps: usize,
    adapter: AdapterPackage,
}
fn runtime_config(raw: &[u8], path: &Path) -> Result<RuntimeConfig, String> {
    let value: Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    if value.get("schema").and_then(Value::as_str) != Some(super::adapter::SCHEMA) {
        return Err(
            "Unsupported CPU adapter package schema; convert offline to rheon-pressure-adapter-v1"
                .into(),
        );
    }
    let adapter = AdapterPackage::load(path).map_err(|e| e.to_string())?;
    Ok(RuntimeConfig {
        python: adapter.python().to_owned(),
        bridge: adapter.entrypoint().to_owned(),
        identity: ProviderIdentity {
            id: adapter.id().into(),
            version: adapter.version().into(),
            manifest_sha256: adapter.manifest_sha256().into(),
            checkpoint_sha256: None,
            training_run_id: None,
        },
        minimum_native_steps: adapter.context().minimum_native_steps,
        adapter,
    })
}
pub struct ModelProcess {
    sender: Option<SyncSender<Vec<u8>>>,
    writes: Receiver<Result<(), String>>,
    receiver: Receiver<Result<ReceivedReply, PredictionFailure>>,
    slot: ChildSlot,
    stop: Arc<AtomicBool>,
    readers: Vec<JoinHandle<()>>,
    pub identity: ProviderIdentity,
    adapter: AdapterPackage,
    pub(crate) prediction_timeout: Duration,
    pub deployment_path: PathBuf,
    pub minimum_native_steps: usize,
}
fn read_runtime(path: &Path) -> Result<(PathBuf, RuntimeConfig), String> {
    let path = path.canonicalize().map_err(|e| e.to_string())?;
    if !path.is_file() {
        return Err("Deployment must be a regular file".into());
    }
    let mut raw = Vec::new();
    std::fs::File::open(&path)
        .map_err(|e| e.to_string())?
        .take(1024 * 1024 + 1)
        .read_to_end(&mut raw)
        .map_err(|e| e.to_string())?;
    if raw.len() > 1024 * 1024 {
        return Err("Deployment descriptor exceeds 1 MiB".into());
    }
    let config = runtime_config(&raw, &path)?;
    Ok((path, config))
}
impl ModelProcess {
    /// Inert support check. No adapter/import/interpreter execution occurs.
    pub fn preflight(path: &Path, scene: &super::scene::Scene) -> Result<ProviderIdentity, String> {
        scene.validate()?;
        let (_, runtime) = read_runtime(path)?;
        let grid =
            rheon::GridGeometry::new(scene.counts.map(|n| n as u64), [scene.h(); 3], [0.0; 3])
                .map_err(|e| e.to_string())?;
        runtime
            .adapter
            .validate_support(
                &grid,
                super::scene::RHO,
                super::scene::MU,
                super::scene::DT,
                scene.warmup_steps,
                scene.warmup_steps > 0,
            )
            .map_err(|e| e.to_string())?;
        Ok(runtime.identity)
    }
    pub fn start(
        path: &Path,
        slot: ChildSlot,
        cancelled: &AtomicBool,
        progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        Self::start_with_pin(path, slot, cancelled, None, progress).map_err(|e| e.to_string())
    }
    /// Freeze the complete resolved binding before any model code executes.
    pub(crate) fn start_pinned(
        path: &Path,
        slot: ChildSlot,
        cancelled: &AtomicBool,
        expected: &ProviderIdentity,
        progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        Self::start_pinned_typed(path, slot, cancelled, expected, progress)
            .map_err(|e| e.to_string())
    }
    pub(crate) fn start_pinned_typed(
        path: &Path,
        slot: ChildSlot,
        cancelled: &AtomicBool,
        expected: &ProviderIdentity,
        progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, StartupFailure> {
        Self::start_with_pin(path, slot, cancelled, Some(expected), progress)
    }
    fn start_with_pin(
        path: &Path,
        slot: ChildSlot,
        cancelled: &AtomicBool,
        expected: Option<&ProviderIdentity>,
        mut progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, StartupFailure> {
        let (canonical, runtime) = read_runtime(path)?;
        if expected.is_some_and(|expected| expected != &runtime.identity) {
            return Err("CPU provider binding differs from frozen pin".into());
        }
        let path = canonical.as_path();
        if cancelled.load(Ordering::Relaxed) {
            return Err(StartupFailure::Cancelled);
        }
        let mut child = Command::new(&runtime.python)
            .args(["-I", "-S", "-B"])
            .arg(&runtime.bridge)
            .arg("--deployment")
            .arg(path)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("CUDA_VISIBLE_DEVICES", "")
            .env("TULDOK_TORCH_DEVICE", "cpu")
            .env("OMP_NUM_THREADS", "1")
            .env("MKL_NUM_THREADS", "1")
            .env("OPENBLAS_NUM_THREADS", "1")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut stdin = child.stdin.take().ok_or("Model stdin")?;
        let stdout = child.stdout.take().ok_or("Model stdout")?;
        let stderr = child.stderr.take().ok_or("Model stderr")?;
        {
            let mut guard = slot.lock().map_err(|_| "Model child lock")?;
            if cancelled.load(Ordering::Relaxed) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(StartupFailure::Cancelled);
            }
            *guard = Some(child);
        }
        let stop = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = mpsc::sync_channel(2);
        let (request_sender, request_receiver) = mpsc::sync_channel::<Vec<u8>>(1);
        let (write_sender, writes) = mpsc::sync_channel(1);
        let flag = Arc::clone(&stop);
        let owner = Arc::clone(&slot);
        let mut result = Self {
            sender: Some(request_sender),
            writes,
            receiver,
            slot: Arc::clone(&slot),
            stop: Arc::clone(&stop),
            readers: Vec::new(),
            identity: runtime.identity,
            adapter: runtime.adapter,
            prediction_timeout: Duration::from_secs(30),
            deployment_path: path.to_owned(),
            minimum_native_steps: runtime.minimum_native_steps,
        };
        let writer = thread::Builder::new()
            .name("pressure-model-input".into())
            .spawn(move || {
                while let Ok(bytes) = request_receiver.recv() {
                    let result = stdin.write_all(&bytes).map_err(|e| e.to_string());
                    let failed = result.is_err();
                    if write_sender.send(result).is_err() || failed {
                        break;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        result.readers.push(writer);
        let reader = thread::Builder::new()
            .name("pressure-model-output".into())
            .spawn(move || {
                let mut input = BufReader::new(stdout);
                while !flag.load(Ordering::Relaxed) {
                    let mut bytes = Vec::new();
                    let read =
                        Read::take(&mut input, (LIMIT + 1) as u64).read_until(b'\n', &mut bytes);
                    let item = match read {
                        Ok(0) => break,
                        Ok(_) if bytes.len() <= LIMIT && bytes.ends_with(b"\n") => {
                            let digest = Sha256::digest(&bytes).into();
                            serde_json::from_slice(&bytes)
                                .map(|reply| ReceivedReply {
                                    reply,
                                    payload_sha256: digest,
                                })
                                .map_err(|e| {
                                    prediction_failure(
                                        NoCandidateReason::InvalidOutput,
                                        e.to_string(),
                                        false,
                                        Some(digest),
                                    )
                                })
                        }
                        Ok(_) => Err(prediction_failure(
                            NoCandidateReason::InvalidOutput,
                            "Model output record exceeds bound or is incomplete",
                            false,
                            bytes
                                .ends_with(b"\n")
                                .then(|| Sha256::digest(&bytes).into()),
                        )),
                        Err(e) => Err(prediction_failure(
                            NoCandidateReason::RuntimeFailure,
                            e.to_string(),
                            false,
                            None,
                        )),
                    };
                    let failed = item.is_err();
                    deliver(&sender, item, &flag);
                    if failed {
                        kill(&owner);
                        break;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        result.readers.push(reader);
        let owner = Arc::clone(&slot);
        let err_reader = thread::Builder::new()
            .name("pressure-model-errors".into())
            .spawn(move || {
                let mut source = stderr;
                let mut total = 0;
                let mut buffer = [0_u8; 4096];
                while let Ok(n) = source.read(&mut buffer) {
                    if n == 0 {
                        break;
                    }
                    total += n;
                    if total > 65536 {
                        kill(&owner);
                        break;
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        result.readers.push(err_reader);
        let deadline = Instant::now() + Duration::from_secs(90);
        loop {
            let message = result
                .receive_typed(cancelled, deadline, false)
                .map_err(|failure| {
                    if failure.reason == NoCandidateReason::Cancelled {
                        StartupFailure::Cancelled
                    } else {
                        StartupFailure::Failure(failure.detail)
                    }
                })?
                .reply;
            match message {
                Reply::Loading { phase }
                    if phase.len() <= 2000 && !phase.chars().any(char::is_control) =>
                {
                    progress(&phase)?
                }
                Reply::AdapterReady {
                    id,
                    version,
                    manifest_sha256,
                    device,
                    optimizer_updates,
                } if id == result.identity.id
                    && version == result.identity.version
                    && manifest_sha256 == result.identity.manifest_sha256
                    && device == "cpu"
                    && optimizer_updates == 0 =>
                {
                    return Ok(result);
                }
                Reply::Error {
                    code,
                    id: None,
                    message,
                } => return Err(bridge_error(code, message).into()),
                _ => return Err("Unexpected CPU startup record".into()),
            }
        }
    }
    fn receive_typed(
        &mut self,
        cancelled: &AtomicBool,
        deadline: Instant,
        dispatched: bool,
    ) -> Result<ReceivedReply, PredictionFailure> {
        self.receive_typed_with_poll(cancelled, deadline, dispatched, &mut || false)
    }
    fn cancellation_requested(
        &self,
        cancelled: &AtomicBool,
        poll: &mut dyn FnMut() -> bool,
    ) -> bool {
        if cancelled.load(Ordering::Relaxed) {
            true
        } else if poll() {
            // Reuse the existing immediate-child owner and flag. The outer
            // owner still performs wait/join through this type's Drop.
            cancel(&self.slot, cancelled);
            true
        } else {
            false
        }
    }
    fn receive_typed_with_poll(
        &mut self,
        cancelled: &AtomicBool,
        deadline: Instant,
        dispatched: bool,
        poll: &mut dyn FnMut() -> bool,
    ) -> Result<ReceivedReply, PredictionFailure> {
        loop {
            if self.cancellation_requested(cancelled, poll) {
                return Err(prediction_failure(
                    NoCandidateReason::Cancelled,
                    "Cancelled",
                    dispatched,
                    None,
                ));
            }
            if Instant::now() >= deadline {
                return Err(prediction_failure(
                    NoCandidateReason::RuntimeFailure,
                    "CPU model reply deadline exceeded",
                    dispatched,
                    None,
                ));
            }
            match self.receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(v) => {
                    return v.map_err(|mut e| {
                        e.dispatched = dispatched;
                        if cancelled.load(Ordering::Relaxed)
                            && e.reason == NoCandidateReason::RuntimeFailure
                            && e.invalid_payload_sha256.is_none()
                        {
                            e.reason = NoCandidateReason::Cancelled;
                            e.detail = "Cancelled".into();
                        }
                        e
                    });
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => {
                    return Err(prediction_failure(
                        if cancelled.load(Ordering::Relaxed) {
                            NoCandidateReason::Cancelled
                        } else {
                            NoCandidateReason::RuntimeFailure
                        },
                        if cancelled.load(Ordering::Relaxed) {
                            "Cancelled"
                        } else {
                            "CPU model closed its output"
                        },
                        dispatched,
                        None,
                    ));
                }
            }
        }
    }
    /// Legacy callers retain their string result; typed consumers keep the
    /// structured reason/dispatch facts rather than parsing display text.
    pub fn predict(
        &mut self,
        input: Input<'_>,
        cancelled: &AtomicBool,
    ) -> Result<Vec<f64>, String> {
        self.predict_typed(input, cancelled).map_err(|e| e.detail)
    }
    pub(crate) fn predict_typed(
        &mut self,
        input: Input<'_>,
        cancelled: &AtomicBool,
    ) -> Result<Vec<f64>, PredictionFailure> {
        self.predict_typed_with_poll(input, cancelled, &mut || false)
    }
    /// A synchronous accepted Session can supply its cooperative cancellation /
    /// deadline checkpoint without another thread or a second child owner.
    pub(crate) fn predict_typed_with_poll(
        &mut self,
        input: Input<'_>,
        cancelled: &AtomicBool,
        poll: &mut dyn FnMut() -> bool,
    ) -> Result<Vec<f64>, PredictionFailure> {
        let before = |reason, detail| prediction_failure(reason, detail, false, None);
        if self.cancellation_requested(cancelled, poll) {
            return Err(before(NoCandidateReason::Cancelled, "Cancelled"));
        }
        let request_id = usize::try_from(input.context.generation())
            .map_err(|_| before(NoCandidateReason::UnsupportedContext, "Request id overflow"))?;
        if input.context.generation() >= (1_u64 << 53) {
            return Err(before(
                NoCandidateReason::UnsupportedContext,
                "Request id exceeds protocol range",
            ));
        }
        let n = input.context.grid().cell_len();
        if n > 65_536
            || input.initial_guess.len() != n
            || input.residual.len() != n
            || input.initial_guess.first() != Some(&0.0)
            || input
                .initial_guess
                .iter()
                .chain(input.residual)
                .any(|v| !v.is_finite())
        {
            return Err(before(
                NoCandidateReason::UnsupportedContext,
                "Invalid physical input shape, gauge or finiteness",
            ));
        }
        let request = adapter_request(&self.adapter, &input, request_id)?;
        let mut raw = serde_json::to_vec(&request).map_err(|e| {
            prediction_failure(
                NoCandidateReason::UnsupportedContext,
                e.to_string(),
                false,
                None,
            )
        })?;
        if raw.len() + 1 > LIMIT {
            return Err(before(
                NoCandidateReason::UnsupportedContext,
                "CPU request budget exceeded",
            ));
        }
        raw.push(b'\n');
        if self.cancellation_requested(cancelled, poll) {
            return Err(before(NoCandidateReason::Cancelled, "Cancelled"));
        }
        let deadline = Instant::now() + self.prediction_timeout;
        self.sender
            .as_ref()
            .ok_or_else(|| before(NoCandidateReason::RuntimeFailure, "Model input closed"))?
            .try_send(raw)
            .map_err(|_| {
                before(
                    NoCandidateReason::RuntimeFailure,
                    "Model input unavailable or full",
                )
            })?;
        // Dispatch means the existing writer accepted this request, not proof
        // that the child consumed every byte. All subsequent outcomes retain it.
        let after = |reason, detail| prediction_failure(reason, detail, true, None);
        loop {
            if self.cancellation_requested(cancelled, poll) {
                return Err(after(NoCandidateReason::Cancelled, "Cancelled"));
            }
            if Instant::now() >= deadline {
                return Err(after(
                    NoCandidateReason::RuntimeFailure,
                    "CPU model write deadline exceeded",
                ));
            }
            match self.writes.recv_timeout(Duration::from_millis(20)) {
                Ok(result) => {
                    result.map_err(|e| {
                        if cancelled.load(Ordering::Relaxed) {
                            after(NoCandidateReason::Cancelled, "Cancelled")
                        } else {
                            prediction_failure(NoCandidateReason::RuntimeFailure, e, true, None)
                        }
                    })?;
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => {
                    return Err(after(
                        if cancelled.load(Ordering::Relaxed) {
                            NoCandidateReason::Cancelled
                        } else {
                            NoCandidateReason::RuntimeFailure
                        },
                        if cancelled.load(Ordering::Relaxed) {
                            "Cancelled"
                        } else {
                            "CPU model input closed"
                        },
                    ));
                }
            }
        }
        let received = self.receive_typed_with_poll(cancelled, deadline, true, poll)?;
        input.validate_adapter_reply(
            received.reply,
            self.adapter.output_role(),
            received.payload_sha256,
        )
    }
}
fn adapter_request(
    adapter: &AdapterPackage,
    input: &Input<'_>,
    id: usize,
) -> Result<Value, PredictionFailure> {
    let c = &input.context;
    let prior_needed = adapter.context().prior_pressure == PriorPressure::Required
        || adapter
            .inputs()
            .values()
            .any(|s| *s == InputSemantic::PreviousPressure);
    if id - 1 < adapter.context().minimum_native_steps || (prior_needed && c.prior().is_none()) {
        return Err(prediction_failure(
            NoCandidateReason::UnsupportedContext,
            "Required native prefix or prior pressure is missing",
            false,
            None,
        ));
    }
    adapter
        .validate_support(
            c.grid(),
            c.density(),
            input.viscosity,
            c.dt(),
            id - 1,
            c.prior().is_some(),
        )
        .map_err(|e| {
            prediction_failure(NoCandidateReason::Unsupported, e.to_string(), false, None)
        })?;
    input.adapter_request(adapter.inputs())
}

impl Drop for ModelProcess {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.sender.take();
        let child = self.slot.lock().ok().and_then(|mut guard| guard.take());
        if let Some(mut child) = child {
            let _ = child.kill();
            let _ = child.wait();
        }
        for reader in self.readers.drain(..) {
            let _ = reader.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn protocol_is_closed_and_rejects_oversized_pressure() {
        assert!(
            serde_json::from_str::<Reply>(r#"{"kind":"loading","phase":"x","extra":1}"#).is_err()
        );
        assert!(serde_json::from_str::<Reply>(r#"{"kind":"something_else"}"#).is_err());
        assert!(
            serde_json::from_str::<Reply>(
                r#"{"kind":"error","code":"unrecognized","id":1,"message":"x"}"#
            )
            .is_err()
        );
        let invalid = json!({"kind":"candidate","role":"full_pressure","id":16,"device":"cpu","optimizer_updates":0,"pressure":vec![0.0;65537]});
        assert!(serde_json::from_value::<Reply>(invalid).is_err());
        assert!(matches!(serde_json::from_str::<Reply>(r#"{"kind":"error","code":"unsupported_context","id":1,"message":"native prefix required"}"#).unwrap(), Reply::Error { code: ErrorCode::UnsupportedContext, id: Some(1), .. }));
    }
}
#[cfg(test)]
#[path = "prediction_contract_tests.rs"]
mod prediction_contract_tests;
