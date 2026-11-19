//! Existing App/Worker adoption boundary. No normal launch exposure is admitted here.
use crate::{App, PreviewFrame, Request, Worker, scene_ui};
use std::sync::Arc;

pub(super) struct PendingVoxel {
    pub request: Request,
    view: Option<(usize, Option<Arc<PreviewFrame>>)>,
}
impl PendingVoxel {
    pub fn view(&self) -> (usize, Option<Arc<PreviewFrame>>) {
        self.view.clone().unwrap_or((0, None))
    }
}
fn add(total: &mut usize, bytes: usize) -> Result<(), String> {
    *total = total
        .checked_add(bytes)
        .ok_or("Retained voxel admission capacity overflow")?;
    Ok(())
}
fn vector<T>(capacity: usize) -> Result<usize, String> {
    capacity
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(|| "Retained vector capacity overflow".into())
}
fn request_payload(request: &Request) -> Result<usize, String> {
    let mut bytes = std::mem::size_of::<Request>();
    add(
        &mut bytes,
        vector::<rheon::PressureImplementation>(request.methods.capacity())?,
    )?;
    add(
        &mut bytes,
        vector::<Option<rheon::ForceSnapshot>>(request.force_fields.capacity())?,
    )?;
    add(&mut bytes, vector::<u64>(request.prior_run_ids.capacity())?)?;
    add(&mut bytes, request.recording_directory.capacity())?;
    for fields in request.force_fields.iter().flatten() {
        add(&mut bytes, scene_ui::force_payload_bytes(fields.scene())?)?;
        add(&mut bytes, 2 * std::mem::size_of::<usize>())?;
    }
    if let Some(mesh) = &request.collision_mesh {
        add(&mut bytes, mesh.allocated_bytes())?;
    }
    if let Some(set) = &request.saved_set {
        add(&mut bytes, set.owned_capacity().map_err(|e| e.to_string())?)?;
    }
    if let Some(configured) = &request.configured_set {
        if configured.runtime.is_some() {
            return Err("Retained configured runtime handles are unavailable for voxel admission; close the prior worker first".into());
        }
        add(&mut bytes, configured.project_root.capacity())?;
        add(
            &mut bytes,
            configured
                .revision
                .owned_capacity()
                .map_err(|e| e.to_string())?,
        )?;
    }
    if let Some(voxel) = &request.voxel {
        add(
            &mut bytes,
            voxel.revision.owned_capacity().map_err(|e| e.to_string())?,
        )?;
        add(&mut bytes, 2 * std::mem::size_of::<usize>())?;
    }
    if let Some(receipt) = &request.manifest_receipt {
        let path = receipt
            .lock()
            .map_err(|_| "Retained receipt owner failed")?;
        add(
            &mut bytes,
            path.capacity()
                .checked_add(std::mem::size_of_val(&*path) + 2 * std::mem::size_of::<usize>())
                .ok_or("Retained receipt capacity overflow")?,
        )?;
    }
    Ok(bytes)
}
impl App {
    /// Enumerated retained numerical/immutable payload; allocator, SQLite, egui/GPU,
    /// inactive independent pages and UI-only text buffers are outside this policy.
    pub(super) fn retained_payload_for_voxel(&self) -> Result<usize, String> {
        let mut bytes = self.scene.retained_payload_bytes()?;
        add(
            &mut bytes,
            self.viewport.retained_payload_bytes_for_voxel()?,
        )?;
        if let Some(mesh) = &self.collision.mesh {
            add(&mut bytes, mesh.allocated_bytes())?;
        }
        if let Some(history) = &self.history {
            add(&mut bytes, history.retained_payload_bytes()?)?;
        }
        if let Some(request) = &self.running_request {
            add(&mut bytes, request_payload(request)?)?;
        }
        add(
            &mut bytes,
            vector::<crate::ResultRow>(self.rows.capacity())?,
        )?;
        for row in &self.rows {
            add(&mut bytes, row.pixels.capacity())?;
            add(&mut bytes, row.label.capacity())?;
        }
        Ok(bytes)
    }
    pub(super) fn launch_voxel_request(&mut self, mut request: Request) -> Result<(), String> {
        if self.scene.contents.create_requested || self.scene.contents.draft_action.is_some() {
            return Err("Finish the queued initial-contents create/save/load command before preparing a voxel run".into());
        }
        if self.worker.is_some()
            || self.pending_voxel.is_some()
            || self.closing
            || self.scene.editing()
            || self.scene.contents.busy()
            || self
                .scene
                .contents
                .editor
                .as_ref()
                .is_some_and(|e| e.active())
            || self.ai_comparison.is_some()
            || self.archive.active
            || self.run_sets.active
            || self.saved_comparison_active
        {
            return Err("Stop the worker, finish or cancel editing, and return to the simulation page before preparing a voxel run".into());
        }
        let source = request.voxel.as_ref().ok_or("Voxel request required")?;
        let digest = source
            .revision
            .members
            .first()
            .ok_or("Voxel member required")?
            .spec
            .digest();
        let mut retained = self.retained_payload_for_voxel()?;
        add(
            &mut retained,
            self.run_sets
                .related_voxel_payload_bytes(&source.revision)?,
        )?;
        add(
            &mut retained,
            self.archive.inspector.related_voxel_payload_bytes(digest)?,
        )?;
        request
            .voxel
            .as_mut()
            .ok_or("Voxel request required")?
            .retained_payload_bytes = retained;
        crate::validate_request(&request)?;
        // Capture without selecting/decode/clamping. Existing selected owners stay fixed.
        let view = if self.scene.context == scene_ui::SceneContext::Retained {
            Some(self.history.as_ref().map_or(Ok((0, None)), |h| {
                h.view(self.track, self.timeline_step, self.follow_live)
            })?)
        } else {
            None
        };
        // Mutable per-attempt custody must never alias a retained Request receipt.
        request.manifest_receipt = Some(Arc::new(std::sync::Mutex::new(Default::default())));
        let worker = Worker::start(request.clone())
            .map_err(|e| format!("Cannot prepare voxel worker: {e}"))?;
        self.pending_voxel = Some(PendingVoxel { request, view });
        self.worker = Some(worker);
        self.status = "Preparing immutable voxel Water/Air run; current result and selection held until its verified constructor is ready.".into();
        Ok(())
    }
    /// Must run before finished/join handling, including a fast terminal worker.
    pub(super) fn acknowledge_voxel_constructor(&mut self) -> Result<bool, String> {
        let Some(pending) = &self.pending_voxel else {
            return Ok(false);
        };
        let worker = self.worker.as_ref().ok_or("Pending voxel worker missing")?;
        let (_, frame) = worker.history.view(0, 0, false)?;
        let Some(frame) = frame else {
            return Ok(false);
        };
        if frame.step != 0
            || frame.time.to_bits() != 0.0_f64.to_bits()
            || frame.voxel.as_ref().is_none_or(|v| v.report.is_some())
            || frame.report.is_some()
            || frame.particle_report.is_some()
        {
            return Ok(false);
        }
        let digest = pending
            .request
            .voxel
            .as_ref()
            .ok_or("Pending voxel recipe missing")?
            .revision
            .members[0]
            .spec
            .digest();
        if !worker.history.carrier_inspectors[0].voxel_constructor_matches(digest)? {
            return Ok(false);
        }
        let pending = self
            .pending_voxel
            .take()
            .ok_or("Pending voxel custody missing")?;
        self.history = Some(Arc::clone(&worker.history));
        self.scheduler = Some(Arc::clone(&worker.scheduler));
        self.running_request = Some(pending.request);
        self.scene.begin_run(false);
        self.scene.bind_retained_no_forces(0);
        self.rows.clear();
        self.textures.clear();
        self.timeline_step = 0;
        self.track = 0;
        self.follow_live = true;
        self.playing = false;
        self.play_time = 0.0;
        self.status =
            "Voxel constructor recorded and verified; accepted intervals appear live.".into();
        Ok(true)
    }
    pub(super) fn pending_voxel_terminal(
        &mut self,
        outcome: Result<Result<Vec<crate::ResultRow>, String>, Box<dyn std::any::Any + Send>>,
    ) {
        let Some(pending) = self.pending_voxel.take() else {
            return;
        };
        self.status = match outcome {
            Ok(Ok(_)) => "Voxel worker ended without a verified constructor; previous result retained".into(),
            Ok(Err(error)) => format!("Voxel preparation stopped: {error}. Previous result and selection retained"),
            Err(_) => "Voxel worker failed before verified adoption; previous result and selection retained".into(),
        };
        if let Some(receipt) = &pending.request.manifest_receipt {
            match receipt.lock() {
                Ok(path) if !path.as_os_str().is_empty() => self
                    .status
                    .push_str(&format!(" New attempt revision: {}", path.display())),
                Ok(_) => self
                    .status
                    .push_str("; no new revision publication acknowledged"),
                Err(_) => self
                    .status
                    .push_str("; new attempt receipt unavailable; preserve its output directory"),
            }
        }
        self.status.push_str(&format!(
            ". New output target: {}",
            pending.request.recording_directory
        ));
        // The old request/path/playback never moved. Do not adopt its receipt as the new attempt.
    }
}

#[cfg(test)]
#[path = "voxel_material_launch_tests.rs"]
mod tests;
