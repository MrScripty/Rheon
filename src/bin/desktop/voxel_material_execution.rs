//! The filled-box voxel method inside the existing desktop Worker. Normal App
//! exposure remains gated by the canonical M4/A5/A6 adoption/native milestones.
use crate::{
    Request, ResultRow, build_identity,
    history::{LiveHistory, PreviewFrame},
    identity, new_recording_path, publish_set_revision, run_sets,
};
use rheon::{
    PressureImplementation,
    recording::{StepRecorder, voxel_material_publication, voxel_material_publication_bytes},
};
use run_sets::{
    configured::ExecutorDeclaration,
    voxel_material::run_set::{self, VoxelRunSetRevision},
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::Instant,
};
const MIB: usize = 1024 * 1024;
const ADMISSION_BUDGET: usize = 384 * MIB;
pub(crate) const METHOD: &str = "Voxel Water/Air · matched donor / strain / Jacobi v1";
#[derive(Clone)]
pub(crate) struct VoxelRequest {
    pub revision: Arc<VoxelRunSetRevision>,
    /// Actual overlapping retained host payload; production App calculation and
    /// atomic constructor adoption are a separate mandatory admission gate.
    pub retained_payload_bytes: usize,
}
pub(crate) fn executor() -> ExecutorDeclaration {
    ExecutorDeclaration {
        id: "rheon-desktop-voxel-material-serial-v1".into(),
        source_revision: build_identity::executor_revision("desktop-voxel-material-v1"),
        build_provenance: build_identity::material_sha256().map(str::to_owned),
    }
}
pub(crate) fn validate(request: &Request) -> Result<(), String> {
    let voxel = request.voxel.as_ref().ok_or("Voxel request required")?;
    voxel.revision.validate().map_err(|e| e.to_string())?;
    if request.material.is_some()
        || request.saved_set.is_some()
        || request.configured_set.is_some()
        || request.emitter.is_some()
        || request.collision_mesh.is_some()
        || request.observe
        || request.force_fields.iter().any(Option::is_some)
        || (!request.force_fields.is_empty() && request.force_fields.len() != 1)
        || request.methods != [PressureImplementation::JacobiPcgV1]
        || request.threads != 1
    {
        return Err("Voxel Water/Air requires one serial Jacobi track, without particle/carrier profiles, forces, emitter, mesh, observer or provider".into());
    }
    let spec = &voxel.revision.members[0].spec;
    let p = spec.execution();
    if p.executor != executor()
        || p.requested_steps != request.steps
        || p.requested_dt_s.to_bits() != request.requested_dt.to_bits()
        || p.recording.limits() != request.recording_limits
    {
        return Err("Voxel immutable schedule, recording policy or compiled executor declaration differs; inspect original intent or capture a new supported run".into());
    }
    // This temporary restores only immutable authoring IDs, never solver arrays.
    let initial = spec.initial_editor().map_err(|e| e.to_string())?;
    let grid = initial.document().geometry();
    let n = grid.counts();
    let h = grid.spacing();
    let side = h[0] * n[0] as f64;
    if n != [request.size as usize; 3]
        || n[1] != n[0]
        || n[2] != n[0]
        || h[1].to_bits() != h[0].to_bits()
        || h[2].to_bits() != h[0].to_bits()
        || grid.origin().iter().any(|&v| v != 0.0)
        || !(0.001..=100.0).contains(&side)
        || side.to_bits() != request.domain_side.to_bits()
    {
        return Err("Voxel native execution requires a 4–64 cubic equal-spacing zero-origin domain of side 0.001–100 m; original data remain inspectable without rescaling".into());
    }
    if !(16..=96).contains(&request.history_budget_mib) || request.recording_directory.is_empty() {
        return Err(
            "Voxel execution requires a 16–96 MiB preview budget and caller-owned output directory"
                .into(),
        );
    }
    let history_budget = request.history_budget_mib * MIB;
    LiveHistory::plan_resolution(
        request.preview_edge()?,
        request.steps,
        1,
        history_budget
            .checked_sub(8192)
            .ok_or("Voxel history budget")?,
        request.preview_resolution.exact(),
    )?;
    let publication = voxel_material_publication_bytes(grid, true).map_err(|e| e.to_string())?;
    if publication > request.recording_limits.max_record_bytes {
        return Err("Voxel accepted publication exceeds captured per-record limit".into());
    }
    // Four recipe/manifest owners at the deepest no-replace serialization
    // boundary, two bounded JSON buffers; preparation uses the existing8MiB
    // candidate reserve after capture has refused a live authoring gesture.
    let request_capacity = request
        .methods
        .capacity()
        .checked_mul(std::mem::size_of::<PressureImplementation>())
        .and_then(|n| {
            request
                .force_fields
                .capacity()
                .checked_mul(std::mem::size_of::<Option<rheon::ForceSnapshot>>())
                .and_then(|c| n.checked_add(c))
        })
        .and_then(|n| {
            request
                .prior_run_ids
                .capacity()
                .checked_mul(std::mem::size_of::<u64>())
                .and_then(|c| n.checked_add(c))
        })
        .and_then(|n| n.checked_add(request.recording_directory.capacity()))
        .and_then(|n| n.checked_add(std::mem::size_of::<Request>()))
        .ok_or("Voxel request capacity overflow")?;
    let staging = voxel
        .revision
        .owned_capacity()
        .map_err(|e| e.to_string())?
        .checked_add(request_capacity)
        .and_then(|n| n.checked_mul(4))
        .and_then(|n| n.checked_add(2 * MIB + 40 * MIB + 512 * 1024))
        .ok_or("Voxel staging capacity overflow")?;
    if staging > 48 * MIB {
        return Err("Voxel immutable input/staging reserve exceeded".into());
    }
    let reserved = [
        p.numerical_memory_limit_bytes,
        history_budget,
        publication,
        publication,
        publication,
        publication,
        128 * MIB,
        voxel.retained_payload_bytes,
    ]
    .into_iter()
    .try_fold(0_usize, |n, c| n.checked_add(c))
    .ok_or("Voxel admission capacity overflow")?;
    if reserved > ADMISSION_BUDGET {
        return Err(format!(
            "Voxel enumerated reserved payload {reserved} exceeds 384 MiB; reduce new preview/input policy or close retained results explicitly"
        ));
    }
    // Lower bound only: SQLite metadata and filesystem capacity can still fail.
    let minimum = grid
        .cell_len()
        .checked_mul(8)
        .and_then(|n| n.checked_mul(request.steps + 1))
        .ok_or("Voxel archive minimum overflow")?;
    if minimum as u64 > request.recording_limits.max_database_bytes {
        return Err("Voxel archive cannot retain even the required fraction fields within captured database budget".into());
    }
    Ok(())
}
fn publish(request: &Request, set: &VoxelRunSetRevision, name: &str) -> Result<(), String> {
    publish_set_revision(request, name, |path| set.save_new(path))
}
pub(crate) fn run(
    mut request: Request,
    cancel: &AtomicBool,
    progress: &AtomicUsize,
    history: &LiveHistory,
) -> Result<Vec<ResultRow>, String> {
    validate(&request)?;
    let voxel = request.voxel.as_ref().ok_or("Voxel request required")?;
    let mut set = (*voxel.revision).clone();
    let parent = set.digest().map_err(|e| e.to_string())?;
    let old_id = set.members[0].attempt.as_ref().map(|a| a.expected_run_id);
    let mut id = identity::new_identity("voxel-material-v1");
    while request.prior_run_ids.contains(&id.run_id) || old_id == Some(id.run_id) {
        id = identity::new_identity("voxel-material-v1");
    }
    set.members[0].attempt = None;
    set.parent_sha256 = Some(parent);
    // Construct from captured geometry/coefficient bits, never scalar side/n.
    let mut sim = set.members[0].spec.construct().map_err(|e| e.to_string())?;
    let run =
        run_set::bound_run(&set.members[0].spec, &sim, id.clone()).map_err(|e| e.to_string())?;
    let publication_limit = voxel_material_publication_bytes(sim.state().geometry(), true)
        .map_err(|e| e.to_string())?;
    if cancel.load(Ordering::Relaxed) {
        return Err("Cancelled before voxel output preparation; previous results retained".into());
    }
    // This existing helper may create the requested parent directory. All
    // policy/executor/resource refusal above precedes that side effect.
    let root =
        new_recording_path(&request.recording_directory, "voxel-set")?.with_extension("voxelset");
    std::fs::create_dir(&root)
        .map_err(|e| format!("Voxel attempt directory {}: {e}", root.display()))?;
    request.recording_directory = root.to_string_lossy().into_owned();
    publish(&request, &set, "intent.json")?;
    let archive = root.join("run.sqlite");
    set.parent_sha256 = Some(set.digest().map_err(|e| e.to_string())?);
    set.members[0].attempt = Some(run_sets::RunAttempt {
        attempt_id: format!("voxel-run-{}", id.run_id),
        archive_relative_path: "run.sqlite".into(),
        expected_run_id: id.run_id,
        expected_source_revision: id.source_revision,
        finalized: None,
    });
    publish(&request, &set, "started.json")?;
    let mut recorder =
        StepRecorder::create(&archive, run, request.recording_limits).map_err(|e| {
            format!(
                "Voxel recording create {}: {e}; immutable started intent retained",
                archive.display()
            )
        })?;
    let inspector = &history.carrier_inspectors[0];
    let mut seconds = 0.0;
    let mut iterations = 0;
    let mut divergence = 0.0_f64;
    let mut cancelled = false;
    let mut pixels = Vec::new();
    let result: Result<(), String> = (|| {
        let append_and_present = |sim: &rheon::voxel_material::VoxelMaterialSimulation,
                                  recorder: &mut StepRecorder,
                                  elapsed| {
            let step =
                voxel_material_publication(sim, recorder.recorded_count(), publication_limit)
                    .map_err(|e| e.to_string())?;
            recorder.append(&step).map_err(|e| {
                format!(
                    "Voxel generation {} append not confirmed at {}: {e}; do not retry",
                    sim.state().generation(),
                    archive.display()
                )
            })?;
            drop(step);
            let frame = PreviewFrame::capture_voxel(sim, history.preview_edge, elapsed)
                .and_then(|frame| history.publish(0, frame));
            frame.map_err(|e| {
                format!(
                    "Voxel generation {} recorded but not presented: {e}",
                    sim.state().generation()
                )
            })
        };
        append_and_present(&sim, &mut recorder, 0.0)?;
        inspector.reload(&archive, request.recording_limits)?;
        inspector.ensure_timestep(0)?;
        for step in 0..request.steps {
            if cancel.load(Ordering::Relaxed) {
                cancelled = true;
                return Err(
                    "Cancelled; acknowledged voxel recording/appearance prefix retained".into(),
                );
            }
            history.begin_step(0, step)?;
            let start = Instant::now();
            let advanced = sim.step(request.requested_dt, |_| cancel.load(Ordering::Relaxed));
            history.end_step()?;
            let report = advanced.map_err(|e| {
                cancelled = matches!(e, rheon::voxel_material::VoxelMaterialError::Cancelled);
                e.to_string()
            })?;
            let elapsed = start.elapsed().as_secs_f64();
            seconds += elapsed;
            iterations += report.projection.pressure.iterations;
            divergence = divergence.max(report.projection.actual_divergence_max);
            append_and_present(&sim, &mut recorder, elapsed)?;
            progress.fetch_add(1, Ordering::Relaxed);
            #[cfg(test)]
            if progress.load(Ordering::Relaxed) == 1
                && let Some(gate) = &request.progress_gate
            {
                gate.reached.send(()).map_err(|e| e.to_string())?;
                gate.release
                    .lock()
                    .map_err(|e| e.to_string())?
                    .recv_timeout(std::time::Duration::from_secs(10))
                    .map_err(|e| e.to_string())?;
            }
        }
        let n = sim.state().geometry().counts()[0];
        pixels
            .try_reserve_exact(n * n)
            .map_err(|e| format!("Voxel thumbnail allocation: {e}"))?;
        if pixels.capacity() > 64 * 64 {
            return Err("Voxel thumbnail actual-capacity budget".into());
        }
        pixels.resize(n * n, 0);
        let f = sim.state().liquid_fraction();
        for k in 0..n {
            for j in 0..n {
                for i in 0..n {
                    pixels[i + n * k] =
                        pixels[i + n * k].max((255.0 * f[i + n * (j + n * k)]).round() as u8);
                }
            }
        }
        Ok(())
    })();
    let outcome = if result.is_ok() {
        "completed"
    } else if cancelled {
        "cancelled"
    } else {
        "failed"
    };
    let detail = match &result {
        Ok(()) => format!(
            "{} accepted voxel intervals; actual elapsed {}s; maximum requested dt{}s; no restart claim",
            request.steps,
            sim.state().time(),
            request.requested_dt
        ),
        Err(e) => format!(
            "{e}; accepted generation {}; acknowledged publications {}; accepted tail may be unrecorded",
            sim.state().generation(),
            recorder.recorded_count()
        ),
    };
    recorder
        .finish(outcome, &detail, Some(sim.state().generation()))
        .map_err(|e| {
            format!(
                "{detail}; voxel terminal status not confirmed at {}: {e}; preserve exact path",
                archive.display()
            )
        })?;
    drop(recorder);
    let parent = set.digest().map_err(|e| e.to_string())?;
    let member = &mut set.members[0];
    let pin = run_set::pin_closed(
        member.attempt.as_mut().ok_or("Voxel attempt missing")?,
        &root,
        &member.spec,
    );
    set.parent_sha256 = Some(parent);
    publish(&request, &set, "finished.json")?;
    pin.map_err(|e| {
        format!(
            "Voxel closed archive not pinned at {}: {e}; retain immutable manifests",
            archive.display()
        )
    })?;
    result?;
    Ok(vec![ResultRow {
        method: PressureImplementation::JacobiPcgV1,
        label: METHOD.into(),
        seconds,
        iterations,
        divergence,
        bytes: sim.allocated_bytes(),
        time: sim.state().time(),
        pixels,
        particle_report: None,
    }])
}
