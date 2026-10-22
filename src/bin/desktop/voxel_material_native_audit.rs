//! Ignored, explicit post-observation audit of existing native-session artifacts.
//! Reads authoring/manifest/recording owners only. It creates no App, Worker,
//! numerical owner, accepted interval, publication, pin or replacement artifact.
use crate::run_sets::{
    MemberState, TerminalOutcome, archive_path, file_digest,
    voxel_material::{
        DraftRevision,
        run_set::{VoxelRunSetRevision, open_member},
    },
};
use rheon::{
    SnapshotStage,
    recording::{RecordedVoxelMaterial, RecordingReader},
};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Oracle {
    kind: String,
    counts: [u64; 3],
    spacing_m: [f64; 3],
    water_brush: Brush,
    gas_pocket: Pocket,
    expected_ids_after_redo: Vec<u16>,
    baseline: Schedule,
    cancellation: Cancellation,
    expected_occupied_volume_m3: f64,
    expected_initial_phase_volumes_m3: [f64; 2],
}
#[derive(Deserialize)]
struct Brush {
    mode: String,
    depth: String,
    center_m: [f64; 3],
    radius_m: f64,
    water_cells: usize,
}
#[derive(Deserialize)]
struct Pocket {
    mode: String,
    depth: String,
    center_m: [f64; 3],
    radius_m: f64,
    flat_index: usize,
    water_cells_after: usize,
    air_cells_after: usize,
    all_six_neighbors_remain_water: bool,
}
#[derive(Deserialize)]
struct Schedule {
    maximum_dt_s: f64,
    requested_intervals: usize,
    gravity_m_s2: [f64; 3],
}
#[derive(Deserialize)]
struct Cancellation {
    counts: [u64; 3],
    maximum_requested_intervals: usize,
    maximum_dt_s: f64,
    gravity_m_s2: [f64; 3],
}

fn env_path(key: &str) -> PathBuf {
    PathBuf::from(
        std::env::var_os(key)
            .unwrap_or_else(|| panic!("{key} must name an existing observed artifact")),
    )
}
fn hex(value: [u8; 32]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(64);
    for byte in value {
        write!(&mut text, "{byte:02x}").unwrap();
    }
    text
}
fn metric_mask(oracle: &Oracle) -> Vec<u16> {
    assert_eq!(
        oracle.kind,
        "predeclared_metric_cell_center_oracle_not_observed_native_state"
    );
    assert_eq!(oracle.counts, [8; 3]);
    assert_eq!(oracle.spacing_m.map(f64::to_bits), [0.125f64.to_bits(); 3]);
    assert_eq!(
        (&*oracle.water_brush.mode, &*oracle.water_brush.depth),
        ("replace", "volume")
    );
    assert_eq!(
        (&*oracle.gas_pocket.mode, &*oracle.gas_pocket.depth),
        ("replace", "volume")
    );
    let inside = |index: usize, center: [f64; 3], radius: f64| {
        let p = [index % 8, index / 8 % 8, index / 64];
        let squared_distance: f64 = (0..3)
            .map(|d| {
                let offset = (p[d] as f64 + 0.5) * oracle.spacing_m[d] - center[d];
                offset * offset
            })
            .sum();
        squared_distance <= radius * radius
    };
    // Independent metric-center sphere descriptions; no brush implementation,
    // viewport color, mesh or saved observed IDs determine this expected mask.
    let mut ids: Vec<u16> = (0..512)
        .map(|i| {
            if inside(i, oracle.water_brush.center_m, oracle.water_brush.radius_m) {
                1
            } else {
                2
            }
        })
        .collect();
    assert_eq!(
        ids.iter().filter(|&&id| id == 1).count(),
        oracle.water_brush.water_cells
    );
    for (i, id) in ids.iter_mut().enumerate() {
        if inside(i, oracle.gas_pocket.center_m, oracle.gas_pocket.radius_m) {
            *id = 2;
        }
    }
    assert_eq!(ids, oracle.expected_ids_after_redo);
    assert_eq!(ids.iter().filter(|&&id| id == 1).count(), 95);
    assert_eq!(ids.iter().filter(|&&id| id == 2).count(), 417);
    assert_eq!(oracle.gas_pocket.water_cells_after, 95);
    assert_eq!(oracle.gas_pocket.air_cells_after, 417);
    assert_eq!(oracle.gas_pocket.flat_index, 283);
    assert_eq!(ids[283], 2);
    assert!(oracle.gas_pocket.all_six_neighbors_remain_water);
    for neighbor in [282, 284, 275, 291, 219, 347] {
        assert_eq!(ids[neighbor], 1);
    }
    assert_eq!(
        oracle.expected_occupied_volume_m3.to_bits(),
        1.0f64.to_bits()
    );
    assert_eq!(
        oracle.expected_initial_phase_volumes_m3.map(f64::to_bits),
        [95.0f64 / 512.0, 417.0f64 / 512.0].map(f64::to_bits)
    );
    ids
}

// Caller owns protected paths and confirms all producers have closed. This is
// not a hostile-filesystem race guarantee or an OS read-only SQLite handle.
fn open_finished(
    path: &Path,
    archives: &mut Vec<(PathBuf, [u8; 32], u64)>,
) -> (VoxelRunSetRevision, Box<RecordingReader>) {
    assert_eq!(
        path.file_name().and_then(|n| n.to_str()),
        Some("finished.json")
    );
    let revision = VoxelRunSetRevision::open(path).unwrap();
    assert_eq!(revision.members.len(), 1);
    let root = path.parent().unwrap();
    let intent = VoxelRunSetRevision::open(&root.join("intent.json")).unwrap();
    let started = VoxelRunSetRevision::open(&root.join("started.json")).unwrap();
    assert!(intent.members[0].attempt.is_none());
    assert!(
        started.members[0]
            .attempt
            .as_ref()
            .unwrap()
            .finalized
            .is_none()
    );
    assert_eq!(started.parent_sha256, Some(intent.digest().unwrap()));
    assert_eq!(revision.parent_sha256, Some(started.digest().unwrap()));
    assert_eq!(
        intent.members[0].spec.digest(),
        revision.members[0].spec.digest()
    );
    assert_eq!(
        started.members[0].spec.digest(),
        revision.members[0].spec.digest()
    );
    assert!(
        revision.members[0]
            .attempt
            .as_ref()
            .unwrap()
            .finalized
            .is_some(),
        "require the already-published closed pin; never create a pin during audit"
    );
    let attempt = revision.members[0].attempt.as_ref().unwrap();
    let archive = archive_path(root, &attempt.archive_relative_path).unwrap();
    for suffix in ["-journal", "-wal", "-shm"] {
        let mut sidecar = archive.as_os_str().to_os_string();
        sidecar.push(suffix);
        assert!(
            matches!(std::fs::symlink_metadata(Path::new(&sidecar)), Err(e) if e.kind()==std::io::ErrorKind::NotFound),
            "Recovery sidecar present or cannot be excluded; do not recover or repair during audit"
        );
    }
    let limit = revision.members[0]
        .spec
        .execution()
        .recording
        .max_database_bytes;
    let original_hash = file_digest(&archive, limit).unwrap();
    assert_eq!(
        original_hash,
        attempt.finalized.as_ref().unwrap().sha256,
        "original closed archive differs from its published pin before open"
    );
    archives.push((archive, original_hash, limit));
    let reader = match open_member(root, &revision.members[0]) {
        MemberState::Ready(reader) => reader,
        MemberState::Invalid(error) => panic!("native closed pin invalid: {error}"),
        MemberState::Unsupported(reason) => panic!("native closed pin unsupported: {reason}"),
        _ => panic!("native archive missing, incomplete, unpinned or not run"),
    };
    (revision, reader)
}

fn audit_archive(
    label: &str,
    set: &VoxelRunSetRevision,
    reader: &RecordingReader,
    cancelled: bool,
) {
    let member = &set.members[0];
    let policy = member.spec.execution();
    let initial = member.spec.initial_editor().unwrap();
    let document = initial.document();
    let ids = document.cells();
    assert_eq!(document.geometry().counts(), [8; 3]);
    assert_eq!(
        document.geometry().spacing().map(f64::to_bits),
        [0.125f64.to_bits(); 3]
    );
    assert!(document.geometry().origin().iter().all(|&v| v == 0.0));
    assert_eq!(ids.len(), 512);
    assert!(ids.iter().all(|&id| id == 1 || id == 2));
    let status = reader.status().unwrap();
    let pin = member.attempt.as_ref().unwrap().finalized.as_ref().unwrap();
    assert_eq!(
        status.outcome,
        if cancelled { "cancelled" } else { "completed" }
    );
    assert_eq!(
        pin.outcome,
        if cancelled {
            TerminalOutcome::Cancelled
        } else {
            TerminalOutcome::Completed
        }
    );
    let accepted = status.accepted_generation.unwrap();
    let count = reader.count().unwrap();
    assert!(
        count >= 1,
        "native audit requires recorded constructor custody"
    );
    assert_eq!(pin.recorded_count, count);
    assert_eq!(pin.accepted_generation, Some(accepted));
    assert!(count <= policy.requested_steps + 1);
    if cancelled {
        assert!(
            accepted < policy.requested_steps as u64,
            "completed execution is not observed Cancel evidence"
        );
        assert!(accepted >= (count - 1) as u64 && accepted <= count as u64);
    } else {
        assert_eq!(count, 9);
        assert_eq!(accepted, 8);
    }
    let initial_volumes = [
        ids.iter().filter(|&&id| id == 1).count() as f64 / 512.0,
        ids.iter().filter(|&&id| id == 2).count() as f64 / 512.0,
    ];
    if !cancelled {
        assert!(initial_volumes.iter().all(|&v| v > 0.0));
    }
    let mut max_volume_drift = [0.0f64; 2];
    let mut max_mass_drift = [0.0f64; 2];
    let mut time = 0.0f64;
    let mut last_controls = String::new();
    let mut last_volumes = initial_volumes;
    for ordinal in 0..count {
        let raw = reader.select(ordinal).unwrap();
        let typed = RecordedVoxelMaterial::decode(reader.run(), &raw).unwrap();
        assert_eq!(typed.metadata.initial_input_sha256, member.spec.digest());
        let pair = typed.metadata.materials;
        assert_eq!(
            [
                pair.liquid().density(),
                pair.liquid().viscosity(),
                pair.gas().density(),
                pair.gas().viscosity()
            ]
            .map(f64::to_bits),
            [998.2f64, 0.001002, 1.2041288413018594, 0.0000182].map(f64::to_bits)
        );
        assert_eq!(typed.generation, ordinal as u64);
        assert!(
            typed
                .liquid_fraction
                .iter()
                .all(|f| f.is_finite() && (0.0..=1.0).contains(f))
        );
        assert!(typed.velocity.iter().flatten().all(|v| v.is_finite()));
        if ordinal == 0 {
            assert_eq!(raw.stage, SnapshotStage::Constructor);
            assert_eq!(raw.time.to_bits(), 0.0f64.to_bits());
            assert!(raw.dt.is_none() && typed.interval_pressure.is_none());
            assert!(typed.velocity.iter().flatten().all(|v| v.to_bits() == 0));
            for (&fraction, &id) in typed.liquid_fraction.iter().zip(ids) {
                assert_eq!(
                    fraction.to_bits(),
                    if id == 1 { 1.0f64.to_bits() } else { 0 }
                );
            }
        } else {
            assert_eq!(raw.stage, SnapshotStage::Accepted);
            let dt = raw.dt.unwrap();
            assert!(dt.is_normal() && dt > 0.0 && dt <= policy.requested_dt_s);
            assert_eq!(raw.time.to_bits(), (time + dt).to_bits());
            let (start, pressure) = typed.interval_pressure.as_ref().unwrap();
            assert_eq!(start.to_bits(), time.to_bits());
            assert_eq!(pressure.len(), 512);
            assert!(pressure.iter().all(|v| v.is_finite()));
            assert_eq!(pressure[0].to_bits(), 0.0f64.to_bits());
        }
        let volumes = [
            typed.liquid_fraction.iter().sum::<f64>() / 512.0,
            typed.liquid_fraction.iter().map(|&f| 1.0 - f).sum::<f64>() / 512.0,
        ];
        let densities = [pair.liquid().density(), pair.gas().density()];
        for phase in 0..2 {
            // Same 1e-8 relative ceiling and independent summation allowance as
            // tests/voxel_material_step_contract.rs. This archive-only check
            // does not parse Debug receipts or reconstruct internal budgets.
            let reduction =
                2.0 * (512 + 4) as f64 * f64::EPSILON * (initial_volumes[phase] + volumes[phase]);
            let bound = 1e-8 * initial_volumes[phase] + reduction;
            let drift = (volumes[phase] - initial_volumes[phase]).abs();
            assert!(
                drift <= bound,
                "{label} ordinal {ordinal} phase {phase} volume drift {drift} > {bound}"
            );
            let mass = volumes[phase] * densities[phase];
            let initial_mass = initial_volumes[phase] * densities[phase];
            let mass_drift = (mass - initial_mass).abs();
            let mass_bound = bound * densities[phase] + 4.0 * f64::EPSILON * (mass + initial_mass);
            assert!(mass.is_finite() && mass >= 0.0 && mass_drift <= mass_bound);
            if !cancelled {
                assert!(mass > 0.0);
            }
            max_volume_drift[phase] = max_volume_drift[phase].max(drift);
            max_mass_drift[phase] = max_mass_drift[phase].max(mass_drift);
        }
        assert!((volumes.iter().sum::<f64>() - 1.0).abs() <= 4.0 * 516.0 * f64::EPSILON);
        time = raw.time;
        last_volumes = volumes;
        last_controls = raw.controls;
    }
    println!(
        "{label}: run_id={} outcome={} publications={count} accepted_generation={accepted} recorded_generation={} unrecorded_accepted_tail={} time_s={time:.17e} initial_phase_volumes_m3={initial_volumes:?} final_phase_volumes_m3={last_volumes:?} max_volume_drifts_m3={max_volume_drift:?} max_mass_drifts_kg={max_mass_drift:?} archive_sha256={}",
        reader.run().identity.run_id,
        status.outcome,
        count - 1,
        accepted - (count - 1) as u64,
        hex(pin.sha256)
    );
    println!("{label} terminal_detail: {}", status.detail);
    println!(
        "{label} last_recorded_producer_controls (retained declaration, not parsed validation): {last_controls}"
    );
}

#[test]
#[ignore = "requires completed actual native baseline/rerun and explicit artifact paths; reads only"]
fn actual_native_artifacts_match_metric_draft_and_cold_replay() {
    let baseline_path = env_path("RHEON_NATIVE_BASELINE");
    let rerun_path = env_path("RHEON_NATIVE_RERUN");
    let draft_path = env_path("RHEON_NATIVE_DRAFT");
    let oracle_path = env_path("RHEON_NATIVE_ORACLE");
    let cancel_path = std::env::var_os("RHEON_NATIVE_CANCEL").map(PathBuf::from);
    let mut unchanged: Vec<_> = [&baseline_path, &rerun_path, &draft_path, &oracle_path]
        .into_iter()
        .map(|path| (path.clone(), file_digest(path, 1024 * 1024).unwrap()))
        .collect();
    if let Some(path) = &cancel_path {
        unchanged.push((path.clone(), file_digest(path, 1024 * 1024).unwrap()));
    }
    let oracle: Oracle = serde_json::from_slice(&std::fs::read(&oracle_path).unwrap()).unwrap();
    let expected = metric_mask(&oracle);
    assert_eq!(oracle.baseline.requested_intervals, 8);
    assert_eq!(oracle.baseline.maximum_dt_s.to_bits(), 0.001f64.to_bits());
    assert_eq!(
        oracle.baseline.gravity_m_s2.map(f64::to_bits),
        [0.0f64, -9.80665, 0.0].map(f64::to_bits)
    );
    let draft = DraftRevision::open(&draft_path).unwrap();
    let editor = draft.restore().unwrap();
    assert_eq!(editor.document().cells(), expected);
    assert_eq!(editor.document().geometry().counts(), [8; 3]);
    assert_eq!(
        editor.document().geometry().spacing().map(f64::to_bits),
        [0.125f64.to_bits(); 3]
    );
    let mut archives = Vec::new();
    let (baseline, before) = open_finished(&baseline_path, &mut archives);
    let (rerun, after) = open_finished(&rerun_path, &mut archives);
    let captured = baseline.members[0].spec.initial_editor().unwrap();
    assert_eq!(captured.document().cells(), expected);
    assert_eq!(captured.document().id(), editor.document().id());
    assert_eq!(captured.document().revision(), editor.document().revision());
    let policy = baseline.members[0].spec.execution();
    assert_eq!(policy.requested_steps, oracle.baseline.requested_intervals);
    assert_eq!(
        policy.requested_dt_s.to_bits(),
        oracle.baseline.maximum_dt_s.to_bits()
    );
    assert_eq!(
        policy.gravity_m_s2.map(f64::to_bits),
        oracle.baseline.gravity_m_s2.map(f64::to_bits)
    );
    assert_eq!(baseline.members[0].spec, rerun.members[0].spec);
    assert_eq!(
        baseline.members[0].spec.digest(),
        rerun.members[0].spec.digest()
    );
    let rerun_intent =
        VoxelRunSetRevision::open(&rerun_path.parent().unwrap().join("intent.json")).unwrap();
    assert_eq!(rerun_intent.parent_sha256, Some(baseline.digest().unwrap()));
    assert_ne!(before.run().identity.run_id, after.run().identity.run_id);
    assert_eq!(before.count().unwrap(), after.count().unwrap());
    for ordinal in 0..before.count().unwrap() {
        let a = before.select(ordinal).unwrap();
        let b = after.select(ordinal).unwrap();
        assert_eq!(a.ordinal, b.ordinal);
        assert_eq!(a.generation, b.generation);
        assert_eq!(a.stage, b.stage);
        assert_eq!(a.time.to_bits(), b.time.to_bits());
        assert_eq!(a.dt.map(f64::to_bits), b.dt.map(f64::to_bits));
        assert_eq!(
            a.fields, b.fields,
            "cold rerun raw f64 payload/field metadata differs at ordinal {ordinal}"
        );
        for (a, b) in a.fields.iter().zip(&b.fields) {
            assert_eq!(
                a.evaluation_time.map(f64::to_bits),
                b.evaluation_time.map(f64::to_bits)
            );
        }
    }
    audit_archive("native baseline", &baseline, &before, false);
    audit_archive("fresh-process native rerun", &rerun, &after, false);
    if let Some(path) = &cancel_path {
        let (cancel, reader) = open_finished(path, &mut archives);
        let policy = cancel.members[0].spec.execution();
        assert_eq!(oracle.cancellation.counts, [8; 3]);
        assert_eq!(oracle.cancellation.maximum_requested_intervals, 1024);
        assert!(
            (1..=oracle.cancellation.maximum_requested_intervals).contains(&policy.requested_steps)
        );
        assert_eq!(
            policy.requested_dt_s.to_bits(),
            oracle.cancellation.maximum_dt_s.to_bits()
        );
        assert_eq!(
            oracle.cancellation.maximum_dt_s.to_bits(),
            0.001f64.to_bits()
        );
        assert_eq!(
            oracle.cancellation.gravity_m_s2.map(f64::to_bits),
            [0.0f64.to_bits(); 3]
        );
        assert_eq!(
            policy.gravity_m_s2.map(f64::to_bits),
            oracle.cancellation.gravity_m_s2.map(f64::to_bits)
        );
        assert_ne!(reader.run().identity.run_id, before.run().identity.run_id);
        assert_ne!(reader.run().identity.run_id, after.run().identity.run_id);
        audit_archive("observed native Cancel", &cancel, &reader, true);
    } else {
        println!(
            "Native Cancel artifact not supplied: actual cancellation acceptance remains unassessed by this audit."
        );
    }
    for (path, hash, limit) in archives {
        assert_eq!(
            file_digest(&path, limit).unwrap(),
            hash,
            "archive changed across audit: {}",
            path.display()
        );
        println!(
            "unchanged pinned archive {} sha256={}",
            path.display(),
            hex(hash)
        );
    }
    for (path, hash) in unchanged {
        assert_eq!(
            file_digest(&path, 1024 * 1024).unwrap(),
            hash,
            "audit input changed: {}",
            path.display()
        );
        println!("unchanged input {} sha256={}", path.display(), hex(hash));
    }
    println!(
        "Read-only native artifact integration audit complete; no numerical owner, step, process, publication or scientific/performance validation was executed by this audit."
    );
}
