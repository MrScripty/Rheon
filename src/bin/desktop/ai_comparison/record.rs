//! Atomic external observations and immutable diagnostic frames, never Run custody.
use super::{engine::Frame, model_process::ProviderIdentity, scene::Scene};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
pub struct Record {
    pub directory: PathBuf,
    root: PathBuf,
    _lease: File,
    scene: Scene,
    identity: Option<ProviderIdentity>,
    sequence: u64,
}
fn atomic(path: &Path, value: &Value) -> Result<(), String> {
    let bytes = serde_json::to_vec(value).map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("Publication exceeds 8 MiB".into());
    }
    let temporary = path.with_extension(format!("{}.tmp", std::process::id()));
    std::fs::write(&temporary, bytes).map_err(|e| e.to_string())?;
    std::fs::rename(temporary, path).map_err(|e| e.to_string())
}
fn stamp() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let z = (seconds / 86400) as i64 + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}+00:00",
        seconds / 3600 % 24,
        seconds / 60 % 60,
        seconds % 60
    )
}
impl Record {
    pub fn begin(root: &Path, scene: &Scene) -> Result<Self, String> {
        std::fs::create_dir_all(root).map_err(|e| e.to_string())?;
        let lease = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(root.join("active.lock"))
            .map_err(|e| e.to_string())?;
        lease
            .try_lock()
            .map_err(|_| "Another comparison owns this project directory".to_owned())?;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let directory = root.join(format!("run-{nanos}-{}", std::process::id()));
        std::fs::create_dir(&directory).map_err(|e| e.to_string())?;
        let result = Self {
            directory,
            root: root.to_owned(),
            _lease: lease,
            scene: scene.clone(),
            identity: None,
            sequence: 0,
        };
        atomic(
            &result.directory.join("scene.json"),
            &json!({"schema":"rheon-live-water-comparison-scene-v1","scene":scene,"density_kg_m3":1000.0,"dynamic_viscosity_pa_s":0.001,"boundary":"sealed_free_slip","dt_s":super::scene::DT,"direction_phase":scene.direction_phase(),"scope":"Shared qualified native prefix, then independent diagnostic AI trajectory; failed pressure states are not accepted Simulation publications"}),
        )?;
        Ok(result)
    }
    pub fn model(&mut self, identity: &ProviderIdentity, deployment: &Path) -> Result<(), String> {
        self.identity = Some(identity.clone());
        atomic(
            &self.directory.join("model.json"),
            &json!({"schema":"rheon-provider-binding-v1","identity":identity,"checkpoint_sha256":identity.checkpoint_sha256,"training_run_id":identity.training_run_id,"deployment":deployment,"inference_device":"cpu","optimizer_updates":0}),
        )
    }
    pub fn activity(&mut self, state: &str, message: &str, done: usize) -> Result<(), String> {
        self.sequence += 1;
        atomic(
            &self.root.join("activity.json"),
            &json!({"schema_version":1,"producer":"Rheon live water comparison","sequence":self.sequence,"updated_at":stamp(),"state":state,"phase":"Fresh solver / AI comparison","message":message.chars().filter(|c| !c.is_control()).take(900).collect::<String>(),"progress":{"completed":done,"total":self.scene.steps}}),
        )
    }
    pub fn frame(&mut self, frame: &Frame) -> Result<(), String> {
        let value = json!({"schema":if self.identity.as_ref().is_some_and(|p|p.checkpoint_sha256.is_some()) {"rheon-live-water-comparison-frame-v2"}else{"rheon-live-water-comparison-frame-v3"},"run_directory":self.directory,"counts_xyz":self.scene.counts,"cell_size_m":self.scene.h(),"extents_m":self.scene.extents(),"provider":self.identity,"checkpoint_sha256":self.identity.as_ref().and_then(|p|p.checkpoint_sha256.as_ref()),"frame":frame,"difference":"p_ai - p_reference, Pa; equal gauge at cell zero; shared native warm-up, then independent histories"});
        let file = self.directory.join(format!("frame-{:04}.json", frame.step));
        if file.exists() {
            return Err("Frame already published".into());
        }
        atomic(&file, &value)?;
        atomic(&self.root.join("latest.json"), &value)?;
        Ok(())
    }
    pub fn finish(&mut self, state: &str, message: &str, steps: usize) -> Result<(), String> {
        atomic(
            &self.directory.join("completion.json"),
            &json!({"state":state,"message":message,"steps":steps,"provider":self.identity,"checkpoint_sha256":self.identity.as_ref().and_then(|p|p.checkpoint_sha256.as_ref()),"ai_scope":"raw diagnostic, check per-frame verdicts; no fallback or qualification claim"}),
        )?;
        self.activity(state, message, steps)
    }
}
