//! One bounded CPU child, owned readers, explicit cancellation and teardown.
use super::engine::Input;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
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
pub struct ModelProcess {
    stdin: Option<ChildStdin>,
    receiver: Receiver<Result<Reply, String>>,
    slot: ChildSlot,
    stop: Arc<AtomicBool>,
    readers: Vec<JoinHandle<()>>,
    pub checkpoint: String,
    pub training_run: String,
    pub deployment_path: PathBuf,
    pub minimum_native_steps: usize,
}
impl ModelProcess {
    pub fn start(
        path: &Path,
        slot: ChildSlot,
        cancelled: &AtomicBool,
        mut progress: impl FnMut(&str) -> Result<(), String>,
    ) -> Result<Self, String> {
        let canonical = std::fs::canonicalize(path).map_err(|e| e.to_string())?;
        let path = canonical.as_path();
        let mut raw = Vec::new();
        std::fs::File::open(path)
            .map_err(|e| e.to_string())?
            .take(1024 * 1024 + 1)
            .read_to_end(&mut raw)
            .map_err(|e| e.to_string())?;
        if raw.len() > 1024 * 1024 {
            return Err("Deployment descriptor exceeds 1 MiB".into());
        }
        let deployment: Deployment = serde_json::from_slice(&raw).map_err(|e| e.to_string())?;
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
        if cancelled.load(Ordering::Relaxed) {
            return Err("Cancelled".into());
        }
        let mut child = Command::new(&deployment.python)
            .args(["-I", "-S", "-B"])
            .arg(&bridge)
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
        let stdin = child.stdin.take().ok_or("Model stdin")?;
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
        let flag = Arc::clone(&stop);
        let owner = Arc::clone(&slot);
        let mut result = Self {
            stdin: Some(stdin),
            receiver,
            slot: Arc::clone(&slot),
            stop: Arc::clone(&stop),
            readers: Vec::new(),
            checkpoint: deployment.checkpoint_sha256,
            training_run: deployment.training_run_id,
            deployment_path: path.to_owned(),
            minimum_native_steps: deployment.context_policy.minimum_native_steps,
        };
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
                    if checkpoint_sha256 != result.checkpoint
                        || training_run_id != result.training_run
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
        let request = json!({"id":input.step,"counts_xyz":input.scene.counts,"cell_size_m":input.scene.h(),"density_kg_m3":1000.0,"dt_s":super::scene::DT,"base_pressure":input.base,"native_input_residual":input.residual});
        let mut raw = serde_json::to_vec(&request).map_err(|e| e.to_string())?;
        if raw.len() > LIMIT {
            return Err("CPU request budget exceeded".into());
        }
        raw.push(b'\n');
        self.stdin
            .as_mut()
            .ok_or("Model input closed")?
            .write_all(&raw)
            .map_err(|e| e.to_string())?;
        let response = self.receive(cancelled, Instant::now() + Duration::from_secs(30))?;
        match response {
            Reply::Error {
                code,
                id: Some(id),
                message,
            } if id == input.step => Err(bridge_error(code, message)),
            Reply::Pressure {
                id,
                pressure,
                device,
                optimizer_updates,
            } if id == input.step
                && device == "cpu"
                && optimizer_updates == 0
                && pressure.len() == input.base.len()
                && pressure.first() == Some(&0.0) =>
            {
                Ok(pressure)
            }
            _ => Err("CPU pressure reply shape/gauge/correlation mismatch".into()),
        }
    }
}
impl Drop for ModelProcess {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.stdin.take();
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
