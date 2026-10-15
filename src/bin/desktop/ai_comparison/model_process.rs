//! One bounded CPU child, owned readers, explicit cancellation and teardown.
use super::adapter::{AdapterPackage, InputSemantic, OutputRole};
use super::engine::Input;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
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
pub type ChildSlot = Arc<Mutex<Option<Child>>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSpec {
    sha256: String,
    bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Deployment {
    schema: String,
    python: String,
    bridge: String,
    python_sha256: String,
    files: BTreeMap<String, FileSpec>,
    checkpoint_sha256: String,
    normalization_sha256: String,
    source_commit: String,
    training_run_id: String,
    training_updates: usize,
    profile: String,
    physical_contract: Value,
    context_policy: ContextPolicy,
    assumption: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ContextPolicy {
    mode: String,
    minimum_native_steps: usize,
    training_snapshot_indices: Vec<usize>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ErrorCode {
    Invalid,
    Unavailable,
    Unsupported,
    UnsupportedContext,
}
#[derive(Debug, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Reply {
    AdapterReady {
        id: String,
        version: String,
        manifest_sha256: String,
        device: String,
        optimizer_updates: u64,
    },
    Candidate {
        id: usize,
        role: OutputRole,
        #[serde(deserialize_with = "bounded_pressure")]
        pressure: Vec<f64>,
        device: String,
        optimizer_updates: u64,
    },
    Loading {
        phase: String,
    },
    Ready {
        checkpoint_sha256: String,
        training_run_id: String,
        device: String,
        model_dtype: String,
        channels: u32,
        optimizer_updates: u64,
        cuda_initialized: bool,
        minimum_native_steps: usize,
    },
    Pressure {
        id: usize,
        #[serde(deserialize_with = "bounded_pressure")]
        pressure: Vec<f64>,
        device: String,
        optimizer_updates: u64,
    },
    Error {
        code: ErrorCode,
        id: Option<usize>,
        message: String,
    },
}
fn bounded_pressure<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<f64>, D::Error> {
    struct Values;
    impl<'de> serde::de::Visitor<'de> for Values {
        type Value = Vec<f64>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("at most 65,536 finite pressure values")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> Result<Self::Value, A::Error> {
            let mut values = Vec::new();
            while let Some(value) = seq.next_element::<f64>()? {
                if values.len() == 65_536 || !value.is_finite() {
                    return Err(serde::de::Error::custom(
                        "Pressure array exceeds finite value budget",
                    ));
                }
                values.push(value);
            }
            Ok(values)
        }
    }
    deserializer.deserialize_seq(Values)
}
fn is_hex(value: &str, len: usize) -> bool {
    value.len() == len && value.bytes().all(|b| b.is_ascii_hexdigit())
}
fn bridge_error(code: ErrorCode, message: String) -> String {
    format!(
        "CPU bridge {code:?}: {}",
        message
            .chars()
            .filter(|c| !c.is_control())
            .take(2000)
            .collect::<String>()
    )
}
fn file_hash(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let n = file.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        hash.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
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
    sender: &SyncSender<Result<Reply, String>>,
    mut item: Result<Reply, String>,
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
#[derive(Clone, Debug, Serialize)]
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
    adapter: Option<AdapterPackage>,
}
fn runtime_config(raw: &[u8], path: &Path) -> Result<RuntimeConfig, String> {
    let value: Value = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    if value.get("schema").and_then(Value::as_str) == Some(super::adapter::SCHEMA) {
        let adapter = AdapterPackage::load(path).map_err(|e| e.to_string())?;
        return Ok(RuntimeConfig {
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
            adapter: Some(adapter),
        });
    }
    let deployment: Deployment = serde_json::from_slice(raw).map_err(|e| e.to_string())?;
    if deployment.schema != "rheon-pressure-cpu-deployment-v2"
        || deployment.profile != "sealed-water-seven-channel-float32"
        || deployment.bridge != "cpu_bridge.py"
        || deployment.physical_contract["density_kg_m3"] != 1000.0
        || deployment.physical_contract["dynamic_viscosity_pa_s"] != 0.001
        || deployment.physical_contract["cells"] != "cubic"
        || deployment.training_updates == 0
        || !is_hex(&deployment.normalization_sha256, 64)
        || !is_hex(&deployment.checkpoint_sha256, 64)
        || !is_hex(&deployment.training_run_id, 32)
        || !is_hex(&deployment.source_commit, 40)
        || deployment.context_policy.mode != "native_prefix"
        || deployment.context_policy.minimum_native_steps > 512
        || deployment
            .context_policy
            .training_snapshot_indices
            .is_empty()
        || deployment.context_policy.training_snapshot_indices.len() > 512
        || deployment
            .context_policy
            .training_snapshot_indices
            .iter()
            .any(|n| *n > 512)
        || deployment
            .context_policy
            .training_snapshot_indices
            .iter()
            .min()
            != Some(&deployment.context_policy.minimum_native_steps)
        || deployment.assumption.is_empty()
    {
        return Err("Unsupported water CPU deployment contract".into());
    }
    let root = path.parent().ok_or("Deployment parent")?;
    let bridge = root.join(&deployment.bridge);
    let expected = deployment
        .files
        .get(&deployment.bridge)
        .ok_or("Measured bridge required")?;
    if expected.bytes > 2 * 1024 * 1024
        || file_hash(&bridge)? != expected.sha256
        || std::fs::metadata(&bridge).map_err(|e| e.to_string())?.len() != expected.bytes
        || file_hash(Path::new(&deployment.python))? != deployment.python_sha256
    {
        return Err("Interpreter/bridge drift".into());
    }
    Ok(RuntimeConfig {
        python: PathBuf::from(deployment.python),
        bridge,
        identity: ProviderIdentity {
            id: deployment.checkpoint_sha256.clone(),
            version: "legacy-v2".into(),
            manifest_sha256: format!("{:x}", Sha256::digest(raw)),
            checkpoint_sha256: Some(deployment.checkpoint_sha256),
            training_run_id: Some(deployment.training_run_id),
        },
        minimum_native_steps: deployment.context_policy.minimum_native_steps,
        adapter: None,
    })
}
pub struct ModelProcess {
    sender: Option<SyncSender<Vec<u8>>>,
    writes: Receiver<Result<(), String>>,
    receiver: Receiver<Result<Reply, String>>,
    slot: ChildSlot,
    stop: Arc<AtomicBool>,
    readers: Vec<JoinHandle<()>>,
    pub identity: ProviderIdentity,
    adapter: Option<AdapterPackage>,
    pub(super) prediction_timeout: Duration,
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
        if let Some(adapter) = &runtime.adapter {
            adapter
                .validate_support(
                    &grid,
                    super::scene::RHO,
                    super::scene::MU,
                    super::scene::DT,
                    scene.warmup_steps,
                    scene.warmup_steps > 0,
                )
                .map_err(|e| e.to_string())?;
        } else if scene.warmup_steps < runtime.minimum_native_steps {
            return Err(format!(
                "Unsupported testing setup: checkpoint requires at least {} native warm-up steps; configured {}",
                runtime.minimum_native_steps, scene.warmup_steps
            ));
        }
        Ok(runtime.identity)
    }
    pub fn start(
        path: &Path,
        slot: ChildSlot,
        cancelled: &AtomicBool,
        mut progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        let (canonical, runtime) = read_runtime(path)?;
        let path = canonical.as_path();
        if cancelled.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
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
                return Err("Cancelled".into());
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
                            serde_json::from_slice(&bytes).map_err(|e| e.to_string())
                        }
                        Ok(_) => Err("Model output record exceeds bound or is incomplete".into()),
                        Err(e) => Err(e.to_string()),
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
            let message = result.receive(cancelled, deadline)?;
            match message {
                Reply::Loading { phase }
                    if phase.len() <= 2000 && !phase.chars().any(char::is_control) =>
                {
                    progress(&phase)?
                }
                Reply::Ready {
                    checkpoint_sha256,
                    training_run_id,
                    device,
                    model_dtype,
                    channels,
                    optimizer_updates,
                    cuda_initialized,
                    minimum_native_steps,
                } => {
                    if result.adapter.is_some()
                        || Some(&checkpoint_sha256) != result.identity.checkpoint_sha256.as_ref()
                        || Some(&training_run_id) != result.identity.training_run_id.as_ref()
                        || device != "cpu"
                        || model_dtype != "float32"
                        || channels != 7
                        || cuda_initialized
                        || optimizer_updates != 0
                        || minimum_native_steps != result.minimum_native_steps
                    {
                        return Err("CPU model handshake mismatch".into());
                    }
                    return Ok(result);
                }
                Reply::AdapterReady {
                    id,
                    version,
                    manifest_sha256,
                    device,
                    optimizer_updates,
                } if result.adapter.is_some()
                    && id == result.identity.id
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
                } => return Err(bridge_error(code, message)),
                _ => return Err("Unexpected CPU startup record".into()),
            }
        }
    }
    fn receive(&mut self, cancelled: &AtomicBool, deadline: Instant) -> Result<Reply, String> {
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            if Instant::now() >= deadline {
                return Err("CPU model reply deadline exceeded".into());
            }
            match self.receiver.recv_timeout(Duration::from_millis(20)) {
                Ok(v) => return v,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err("CPU model closed its output".into()),
            }
        }
    }
    pub fn predict(
        &mut self,
        input: Input<'_>,
        cancelled: &AtomicBool,
    ) -> Result<Vec<f64>, String> {
        let request_id =
            usize::try_from(input.context.generation()).map_err(|_| "Request id overflow")?;
        if input.context.generation() >= (1_u64 << 53) {
            return Err("Request id exceeds protocol range".into());
        }
        let request = if let Some(adapter) = &self.adapter {
            adapter_request(adapter, &input, request_id)?
        } else {
            json!({"id":request_id,"counts_xyz":input.context.grid().counts(),"cell_size_m":input.context.grid().spacing()[0],"density_kg_m3":input.context.density(),"dt_s":input.context.dt(),"base_pressure":input.initial_guess,"native_input_residual":input.residual})
        };
        let mut raw = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        if raw.len() + 1 > LIMIT {
            return Err("CPU request budget exceeded".into());
        }
        raw.push(b'\n');
        let deadline = Instant::now() + self.prediction_timeout;
        self.sender
            .as_ref()
            .ok_or("Model input closed")?
            .try_send(raw)
            .map_err(|_| "Model input unavailable or full")?;
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err("Cancelled".into());
            }
            if Instant::now() >= deadline {
                return Err("CPU model write deadline exceeded".into());
            }
            match self.writes.recv_timeout(Duration::from_millis(20)) {
                Ok(result) => {
                    result?;
                    break;
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err("CPU model input closed".into()),
            }
        }
        let response = self.receive(cancelled, deadline)?;
        match response {
            Reply::Error {
                code,
                id: Some(id),
                message,
            } if id == request_id => Err(bridge_error(code, message)),
            Reply::Pressure {
                id,
                pressure,
                device,
                optimizer_updates,
            } if self.adapter.is_none()
                && id == request_id
                && device == "cpu"
                && optimizer_updates == 0
                && pressure.len() == input.initial_guess.len()
                && pressure.first() == Some(&0.0) =>
            {
                Ok(pressure)
            }
            Reply::Candidate {
                id,
                role,
                mut pressure,
                device,
                optimizer_updates,
            } if id == request_id
                && self
                    .adapter
                    .as_ref()
                    .is_some_and(|a| a.output_role() == role)
                && device == "cpu"
                && optimizer_updates == 0
                && pressure.len() == input.context.grid().cell_len()
                && pressure.first() == Some(&0.0) =>
            {
                if role == OutputRole::PressureCorrection {
                    let prior = input
                        .context
                        .prior()
                        .ok_or("Correction requires prior pressure")?;
                    for (p, previous) in pressure.iter_mut().zip(prior.values) {
                        *p += *previous;
                        if !p.is_finite() {
                            return Err("Nonfinite reconstructed pressure".into());
                        }
                    }
                }
                Ok(pressure)
            }
            _ => Err("CPU pressure reply shape/gauge/correlation mismatch".into()),
        }
    }
}
fn adapter_request(
    adapter: &AdapterPackage,
    input: &Input<'_>,
    id: usize,
) -> Result<Value, String> {
    let c = &input.context;
    adapter
        .validate_support(
            c.grid(),
            c.density(),
            input.viscosity,
            c.dt(),
            id - 1,
            c.prior().is_some(),
        )
        .map_err(|e| e.to_string())?;
    let counts = c.grid().counts();
    let mut fields = serde_json::Map::new();
    for (name, semantic) in adapter.inputs() {
        let (values, units, scalar) = match semantic {
            InputSemantic::Rhs => (c.rhs().to_vec(), "m3/s2", false),
            InputSemantic::InitialResidual => (input.residual.to_vec(), "m3/s2", false),
            InputSemantic::PreviousPressure => (
                c.prior()
                    .ok_or("Previous pressure unavailable")?
                    .values
                    .to_vec(),
                "Pa",
                false,
            ),
            InputSemantic::Density => (vec![c.density()], "kg/m3", true),
            InputSemantic::Dt => (vec![c.dt()], "s", true),
            InputSemantic::X | InputSemantic::Y | InputSemantic::Z => {
                let axis = match semantic {
                    InputSemantic::X => 0,
                    InputSemantic::Y => 1,
                    _ => 2,
                };
                let mut values = Vec::with_capacity(c.grid().cell_len());
                for k in 0..counts[2] {
                    for j in 0..counts[1] {
                        for i in 0..counts[0] {
                            values.push(
                                c.grid()
                                    .cell_position([i, j, k])
                                    .ok_or("Coordinate unavailable")?[axis],
                            );
                        }
                    }
                }
                (values, "m", false)
            }
        };
        fields.insert(name.clone(),json!({"semantic":semantic,"units":units,"association":if scalar {"global"}else{"cell"},"dimensions":if scalar {[1,1,1]}else{counts},"scalar":"f64","layout":"x_fast","values":values}));
    }
    Ok(
        json!({"kind":"predict","schema":"rheon-pressure-context-request-v1","id":id,"stage":rheon::solver_provider::context::STAGE_CONTRACT,
        "context":{"run_id":c.run_id(),"generation":c.generation(),"mode":match c.mode(){rheon::solver_provider::context::PressureContextMode::AcceptedCandidate=>"accepted_candidate",rheon::solver_provider::context::PressureContextMode::DiagnosticCandidate=>"diagnostic_candidate"},"interval_start_s":c.interval_start(),"interval_end_s":c.interval_end(),"dt_s":c.dt(),"grid":{"counts_xyz":counts,"spacing_xyz_m":c.grid().spacing(),"origin_xyz_m":c.grid().origin()},
        "prior":c.prior().map(|p|json!({"generation":p.generation,"interval_end_s":p.interval_end,"status":match p.status {rheon::solver_provider::context::PressureHistoryStatus::Accepted=>json!({"kind":"accepted"}),rheon::solver_provider::context::PressureHistoryStatus::Diagnostic{qualified}=>json!({"kind":"diagnostic","qualified":qualified})}}))},"inputs":fields}),
    )
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
        let invalid = json!({"kind":"pressure","id":16,"device":"cpu","optimizer_updates":0,"pressure":vec![0.0;65537]});
        assert!(serde_json::from_value::<Reply>(invalid).is_err());
        assert!(matches!(serde_json::from_str::<Reply>(r#"{"kind":"error","code":"unsupported_context","id":1,"message":"native prefix required"}"#).unwrap(), Reply::Error { code: ErrorCode::UnsupportedContext, id: Some(1), .. }));
    }
}
