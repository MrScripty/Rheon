//! Cold immutable manifest/archive composition, not native worker acceptance.
use super::super::{ExecutionPolicy, PressurePolicy};
use super::*;
use crate::run_sets::ArchiveBudget;
use crate::run_sets::source::SavedRunSet;
use rheon::{
    GridGeometry, SceneObjectId,
    initial_contents::{ContentsEditor, ContentsMaterial, InitialContents},
    recording::{StepRecorder, voxel_material_publication},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
fn root() -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = PathBuf::from(".generated/voxel-material/m4-set").join(format!(
        "{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    root
}
fn spec() -> FrozenInput {
    let e = ContentsEditor::new(
        InitialContents::new(
            SceneObjectId(19),
            GridGeometry::new([4; 3], [0.25; 3], [-0.0; 3]).unwrap(),
            ContentsMaterial::Air,
        )
        .unwrap(),
    );
    FrozenInput::capture(
        &e,
        ExecutionPolicy {
            requested_dt_s: 0.1,
            requested_steps: 1,
            gravity_m_s2: [0.0; 3],
            pressure: PressurePolicy {
                relative_residual: 1e-11,
                absolute_residual: 1e-12,
                divergence_limit: 1e-8,
                max_iterations: 2000,
            },
            numerical_memory_limit_bytes: 64 * 1024 * 1024,
            recording: ArchiveBudget {
                max_database_bytes: 1024 * 1024,
                max_record_bytes: 128 * 1024,
                max_records: 2,
            },
            executor: ExecutorDeclaration {
                id: "test-voxel-serial".into(),
                source_revision: "test-voxel-source".into(),
                build_provenance: None,
            },
        },
    )
    .unwrap()
}
fn attempt() -> RunAttempt {
    RunAttempt {
        attempt_id: "attempt-one".into(),
        archive_relative_path: "run.sqlite".into(),
        expected_run_id: 401,
        expected_source_revision: "test-voxel-source".into(),
        finalized: None,
    }
}
fn revision(spec: FrozenInput, attempt: Option<RunAttempt>) -> VoxelRunSetRevision {
    VoxelRunSetRevision {
        schema_version: 3,
        kind: VoxelSetKind::VoxelMaterialSet,
        set_id: "voxel-test".into(),
        parent_sha256: None,
        baseline_member: "voxel".into(),
        members: vec![VoxelRunSetMember {
            member_id: "voxel".into(),
            spec,
            attempt,
        }],
    }
}
fn write_closed(root: &Path, spec: &FrozenInput, dt: f64) -> RunAttempt {
    let a = attempt();
    let mut sim = spec.construct().unwrap();
    let run = bound_run(
        spec,
        &sim,
        SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
    )
    .unwrap();
    let mut writer = StepRecorder::create(
        &root.join("run.sqlite"),
        run,
        spec.execution().recording.limits(),
    )
    .unwrap();
    writer
        .append(&voxel_material_publication(&sim, 0, 128 * 1024).unwrap())
        .unwrap();
    sim.step(dt, |_| false).unwrap();
    writer
        .append(&voxel_material_publication(&sim, 1, 128 * 1024).unwrap())
        .unwrap();
    writer
        .finish("completed", "one declared component interval", Some(1))
        .unwrap();
    drop(writer);
    a
}
#[test]
fn explicit_voxel_dispatch_and_cold_member_states_bind_the_closed_artifact() {
    let root = root();
    let spec = spec();
    let mut set = revision(spec.clone(), None);
    assert!(matches!(
        open_member(&root, &set.members[0]),
        MemberState::NotRun
    ));
    set.members[0].attempt = Some(attempt());
    assert!(matches!(
        open_member(&root, &set.members[0]),
        MemberState::Missing
    ));
    let mut sim = spec.construct().unwrap();
    let a = set.members[0].attempt.as_ref().unwrap();
    let run = bound_run(
        &spec,
        &sim,
        SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
    )
    .unwrap();
    let mut writer = StepRecorder::create(
        &root.join("run.sqlite"),
        run,
        spec.execution().recording.limits(),
    )
    .unwrap();
    writer
        .append(&voxel_material_publication(&sim, 0, 128 * 1024).unwrap())
        .unwrap();
    assert!(matches!(
        open_member(&root, &set.members[0]),
        MemberState::Incomplete {
            recorded_count: 1,
            ..
        }
    ));
    sim.step(0.1, |_| false).unwrap();
    writer
        .append(&voxel_material_publication(&sim, 1, 128 * 1024).unwrap())
        .unwrap();
    writer.finish("completed", "one interval", Some(1)).unwrap();
    drop(writer);
    drop(sim);
    assert!(matches!(
        open_member(&root, &set.members[0]),
        MemberState::Unpinned {
            recorded_count: 2,
            ..
        }
    ));
    pin_closed(set.members[0].attempt.as_mut().unwrap(), &root, &spec).unwrap();
    match open_member(&root, &set.members[0]) {
        MemberState::Ready(reader) => {
            let cold =
                RecordedVoxelMaterial::decode(reader.run(), &reader.select(1).unwrap()).unwrap();
            assert_eq!(cold.liquid_fraction, vec![0.0; 64]);
            assert_eq!(cold.metadata.initial_input_sha256, spec.digest());
        }
        _ => panic!("expected exact pinned ready artifact"),
    }
    let path = root.join("complete.runset");
    set.save_new(&path).unwrap();
    let loaded = SavedRunSet::open(&path).unwrap();
    assert_eq!(loaded, SavedRunSet::Voxel(set.clone()));
    assert_eq!(loaded.schema_version(), 3);
    assert_eq!(loaded.members().count(), 1);
    assert!(matches!(
        loaded.member("voxel").unwrap().open(&root),
        MemberState::Ready(_)
    ));
    let original = fs::read(&path).unwrap();
    assert!(set.save_new(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
    let mut bytes = fs::read(root.join("run.sqlite")).unwrap();
    bytes.extend_from_slice(b"changed");
    fs::write(root.join("run.sqlite"), bytes).unwrap();
    assert!(matches!(
        open_member(&root, &set.members[0]),
        MemberState::Invalid(_)
    ));
}
#[test]
fn false_schedule_or_captured_binding_cannot_be_pinned() {
    let root = root();
    let spec = spec();
    let mut a = write_closed(&root, &spec, 0.2); // Finite accepted step above the captured request.
    assert!(matches!(
        pin_closed(&mut a, &root, &spec),
        Err(RunSetError::Integrity("voxel accepted interval schedule"))
    ));
    assert!(a.finalized.is_none());
    let mut wrong = spec.clone();
    wrong.input.execution.gravity_m_s2[0] = -0.0;
    wrong.sha256 = digest(&wrong.input).unwrap();
    assert!(matches!(
        pin_closed(&mut a, &root, &wrong),
        Err(RunSetError::Integrity(
            "voxel archive captured configuration"
        ))
    ));
}
#[test]
fn strict_manifest_dispatch_preserves_invalid_original_bytes() {
    let root = root();
    let path = root.join("intent.runset");
    let set = revision(spec(), None);
    set.save_new(&path).unwrap();
    let original = fs::read_to_string(&path).unwrap();
    fs::write(
        &path,
        original.replacen(
            "\"schema_version\":3",
            "\"schema_version\":3,\"schema_version\":3",
            1,
        ),
    )
    .unwrap();
    assert!(matches!(
        SavedRunSet::open(&path),
        Err(RunSetError::Codec(_))
    ));
    fs::write(
        &path,
        original.replacen(
            "\"schema_version\":3",
            "\"schema_version\":3,\"unknown\":1",
            1,
        ),
    )
    .unwrap();
    assert!(SavedRunSet::open(&path).is_err());
    let mut e: Envelope = serde_json::from_str(&original).unwrap();
    e.sha256[0] ^= 1;
    fs::write(&path, serde_json::to_vec(&e).unwrap()).unwrap();
    assert!(matches!(
        SavedRunSet::open(&path),
        Err(RunSetError::Integrity("voxel run-set digest"))
    ));
}

#[test]
fn self_consistent_record_with_wrong_original_materials_cannot_be_ready() {
    let root = root();
    let spec = spec();
    let original = spec.construct().unwrap();
    let a = attempt();
    let header = bound_run(
        &spec,
        &original,
        SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
    )
    .unwrap();
    let wrong = VoxelMaterialSimulation::new(
        original.state().geometry().clone(),
        original.state().materials(),
        &[1.0; 64],
        original.config(),
    )
    .unwrap();
    assert!(matches!(
        bound_run(
            &spec,
            &wrong,
            SessionIdentity {
                run_id: a.expected_run_id,
                source_revision: a.expected_source_revision.clone()
            }
        ),
        Err(RunSetError::Integrity(
            "voxel constructor differs from input"
        ))
    ));
    let mut writer = StepRecorder::create(
        &root.join("run.sqlite"),
        header,
        spec.execution().recording.limits(),
    )
    .unwrap();
    // Generic SQLite integrity alone cannot prove agreement with the recipe.
    writer
        .append(&voxel_material_publication(&wrong, 0, 128 * 1024).unwrap())
        .unwrap();
    writer
        .finish("failed", "constructor mismatch fixture", Some(0))
        .unwrap();
    drop(writer);
    let mut a = a;
    assert!(matches!(
        pin_closed(&mut a, &root, &spec),
        Err(RunSetError::Integrity("voxel original constructor fields"))
    ));
    assert!(a.finalized.is_none());
}

#[test]
fn actual_ratio_shortened_solver_interval_can_be_pinned_and_reopened() {
    let root = root();
    let base = spec();
    let original = base.initial_editor().unwrap();
    let ids = (0..64)
        .map(|i| if i % 4 < 2 { 1 } else { 2 })
        .collect::<Vec<_>>();
    let e = ContentsEditor::new(
        InitialContents::restore(
            SceneObjectId(19),
            1,
            original.document().geometry().clone(),
            original.document().materials(),
            &ids,
        )
        .unwrap(),
    );
    let mut policy = base.execution().clone();
    policy.requested_dt_s = 1.0;
    policy.gravity_m_s2 = [0.0, -9.81, 0.0];
    policy.pressure.relative_residual = 1e-12;
    policy.pressure.max_iterations = 3000;
    let spec = FrozenInput::capture(&e, policy).unwrap();
    let mut a = attempt();
    let mut sim = spec.construct().unwrap();
    let run = bound_run(
        &spec,
        &sim,
        SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        },
    )
    .unwrap();
    let mut writer = StepRecorder::create(
        &root.join("run.sqlite"),
        run,
        spec.execution().recording.limits(),
    )
    .unwrap();
    writer
        .append(&voxel_material_publication(&sim, 0, 128 * 1024).unwrap())
        .unwrap();
    let report = sim
        .step(spec.execution().requested_dt_s, |_| false)
        .unwrap();
    assert!(report.dt > 0.0 && report.dt < 1.0 && report.attempts > 1 && report.attempts <= 8);
    writer
        .append(&voxel_material_publication(&sim, 1, 128 * 1024).unwrap())
        .unwrap();
    writer
        .finish("completed", "actual solver-shortened fixture", Some(1))
        .unwrap();
    drop(writer);
    drop(sim);
    pin_closed(&mut a, &root, &spec).unwrap();
    let member = VoxelRunSetMember {
        member_id: "voxel".into(),
        spec,
        attempt: Some(a),
    };
    match open_member(&root, &member) {
        MemberState::Ready(reader) => assert_eq!(
            reader.select(1).unwrap().dt.unwrap().to_bits(),
            report.dt.to_bits()
        ),
        _ => panic!("actual shortened interval must remain ready"),
    }
}
#[test]
fn signed_zero_cannot_change_exact_constructor_fraction_or_velocity_bits() {
    for velocity in [false, true] {
        let root = root();
        let spec = spec();
        let original = spec.construct().unwrap();
        let a = attempt();
        let identity = || SessionIdentity {
            run_id: a.expected_run_id,
            source_revision: a.expected_source_revision.clone(),
        };
        let header = bound_run(&spec, &original, identity()).unwrap();
        let wrong = VoxelMaterialSimulation::new(
            original.state().geometry().clone(),
            original.state().materials(),
            &[-0.0; 64],
            original.config(),
        )
        .unwrap();
        assert!(matches!(
            bound_run(&spec, &wrong, identity()),
            Err(RunSetError::Integrity(
                "voxel constructor differs from input"
            ))
        ));
        let mut publication = voxel_material_publication(&original, 0, 128 * 1024).unwrap();
        let field = if velocity { 0 } else { 3 };
        publication.fields[field].data[..8].copy_from_slice(&(-0.0_f64).to_le_bytes());
        let mut writer = StepRecorder::create(
            &root.join("run.sqlite"),
            header,
            spec.execution().recording.limits(),
        )
        .unwrap();
        writer.append(&publication).unwrap();
        writer
            .finish("failed", "signed-zero constructor fixture", Some(0))
            .unwrap();
        drop(writer);
        let mut a = a;
        let error = pin_closed(&mut a, &root, &spec).unwrap_err();
        assert!(
            matches!(error,RunSetError::Integrity(r) if r==if velocity {"voxel original constructor velocity"} else {"voxel original constructor fields"})
        );
        assert!(a.finalized.is_none());
    }
}
