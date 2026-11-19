//! Saved-input component checks; actual subprocess, no native worker/UI claim.
use super::*;
use rheon::initial_contents::{BrushDepth, BrushOperation, ContentsBrush, ContentsMaterial};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};
fn directory(label: &str) -> PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let path = PathBuf::from(".generated/voxel-material/m4").join(format!(
        "{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
fn editor(fill: ContentsMaterial) -> ContentsEditor {
    ContentsEditor::new(
        InitialContents::new(
            SceneObjectId(13),
            GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
            fill,
        )
        .unwrap(),
    )
}
fn paint(editor: &mut ContentsEditor, op: BrushOperation, material: ContentsMaterial) {
    let brush = ContentsBrush {
        operation: op,
        material,
        radius: 0.1,
        axis: 2,
        layer: 1,
        depth: BrushDepth::Layer,
    };
    let p = editor.document().geometry().cell_position([1; 3]).unwrap();
    editor.begin(brush).unwrap();
    editor.segment(p, p).unwrap();
    editor.commit().unwrap();
}
fn policy() -> ExecutionPolicy {
    ExecutionPolicy {
        requested_dt_s: 0.001,
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
            id: "rheon-native-serial-voxel-v1".into(),
            source_revision: "test-declared-source".into(),
            build_provenance: None,
        },
    }
}
#[test]
fn exact_draft_roundtrip_preserves_incomplete_revision_and_resets_only_session_history() {
    let root = directory("draft");
    let path = root.join("revision.json");
    let mut e = editor(ContentsMaterial::Vacuum);
    paint(&mut e, BrushOperation::Add, ContentsMaterial::Water);
    e.undo().unwrap();
    let redo = e.redo_label();
    let revision = e.document().revision();
    let draft = DraftRevision::capture(&e).unwrap();
    draft.publish(&path).unwrap();
    assert_eq!(e.redo_label(), redo);
    assert_eq!(e.document().revision(), revision);
    let loaded = DraftRevision::open(&path).unwrap();
    assert_eq!(loaded, draft);
    let restored = loaded.restore().unwrap();
    assert_eq!(DraftRevision::capture(&restored).unwrap(), draft);
    assert_eq!(restored.document().cells(), e.document().cells());
    assert_eq!(restored.document().revision(), revision);
    assert_eq!(restored.document().id(), SceneObjectId(13));
    assert_eq!(restored.undo_label(), None);
    assert_eq!(restored.redo_label(), None);
    assert!(restored.document().validate().is_err());
    assert!(matches!(
        FrozenInput::capture(&e, policy()),
        Err(RunSetError::Unsupported(
            "Vacuum initial cells; fill with Air or Water"
        ))
    ));
    let original = fs::read(&path).unwrap();
    assert!(draft.publish(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), original);
}
#[test]
fn freeze_is_owned_exact_and_refuses_active_strokes_or_unsupported_policies() {
    let mut e = editor(ContentsMaterial::Air);
    paint(&mut e, BrushOperation::Replace, ContentsMaterial::Water);
    let frozen = FrozenInput::capture(&e, policy()).unwrap();
    let digest = frozen.digest();
    paint(&mut e, BrushOperation::Remove, ContentsMaterial::Air);
    let original = frozen.initial_editor().unwrap();
    assert_eq!(original.document().counts(), [0, 1, 63]);
    assert_eq!(original.document().revision(), 1);
    assert_eq!(frozen.digest(), digest);
    assert_eq!(frozen.execution().gravity_m_s2, [0.0; 3]);
    let sim = frozen.construct().unwrap();
    assert_eq!(
        sim.state()
            .liquid_fraction()
            .iter()
            .filter(|&&f| f == 1.0)
            .count(),
        1
    );
    let mut bad = policy();
    bad.numerical_memory_limit_bytes = 1;
    assert!(FrozenInput::capture(&original, bad).is_err());
    let mut bad = policy();
    bad.recording.max_records = 1;
    assert!(FrozenInput::capture(&original, bad).is_err());
    let mut active = editor(ContentsMaterial::Air);
    let brush = ContentsBrush {
        operation: BrushOperation::Replace,
        material: ContentsMaterial::Water,
        radius: 0.1,
        axis: 2,
        layer: 1,
        depth: BrushDepth::Layer,
    };
    active.begin(brush).unwrap();
    assert!(matches!(
        DraftRevision::capture(&active),
        Err(RunSetError::Invalid("active contents stroke"))
    ));
    assert!(FrozenInput::capture(&active, policy()).is_err());
}
fn write_draft(
    path: &Path,
    mut document: InitialDocument,
    edit: impl FnOnce(&mut InitialDocument),
) {
    edit(&mut document);
    let sha256 = digest(&document).unwrap();
    let bytes = serde_json::to_vec(&DraftEnvelope {
        schema_version: DRAFT_VERSION,
        kind: DraftKind::VoxelInitialContents,
        document,
        sha256,
    })
    .unwrap();
    fs::write(path, bytes).unwrap();
}
#[test]
fn malformed_and_self_consistently_hashed_unsupported_drafts_refuse_at_their_owner() {
    let root = directory("invalid");
    let path = root.join("input.json");
    let d = DraftRevision::capture(&editor(ContentsMaterial::Air)).unwrap();
    write_draft(&path, d.document.clone(), |d| {
        d.cell_ids_base64.pop().map(|_| ()).unwrap()
    });
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Invalid("initial payload encoded length"))
    ));
    write_draft(&path, d.document.clone(), |d| {
        d.cell_ids_base64 = STANDARD.encode([3u8; 64])
    });
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Invalid("initial payload IDs/length"))
    ));
    write_draft(&path, d.document.clone(), |d| {
        d.palette[0].density_kg_m3 = 900.0
    });
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Invalid("initial document/palette"))
    ));
    write_draft(&path, d.document.clone(), |d| {
        d.palette[0].color_srgb[0] = 0
    });
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Unsupported("initial voxel palette"))
    ));
    write_draft(&path, d.document.clone(), |d| d.counts_xyz = [65, 4, 4]);
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Unsupported("initial voxel counts"))
    ));
    let valid = DraftEnvelope {
        schema_version: 1,
        kind: DraftKind::VoxelInitialContents,
        document: d.document.clone(),
        sha256: d.sha256,
    };
    let bytes = serde_json::to_vec(&valid).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    fs::write(
        &path,
        text.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
    )
    .unwrap();
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Codec(_))
    ));
    fs::write(
        &path,
        text.replacen(
            "\"schema_version\":1",
            "\"unknown\":0,\"schema_version\":1",
            1,
        ),
    )
    .unwrap();
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Codec(_))
    ));
    fs::write(
        &path,
        text.replacen("\"schema_version\":1", "\"schema_version\":9", 1),
    )
    .unwrap();
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Unsupported("manifest version"))
    ));
    let mut wrong = valid;
    wrong.sha256[0] ^= 1;
    fs::write(&path, serde_json::to_vec(&wrong).unwrap()).unwrap();
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Integrity("initial draft digest"))
    ));
    fs::write(&path, vec![b' '; MAX_BYTES as usize + 1]).unwrap();
    assert!(matches!(
        DraftRevision::open(&path),
        Err(RunSetError::Invalid("manifest byte limit"))
    ));
}
#[test]
fn fresh_process_opens_original_frozen_constructor_and_takes_its_bounded_step() {
    let root = directory("cold");
    let path = root.join("frozen.json");
    let mut e = editor(ContentsMaterial::Air);
    paint(&mut e, BrushOperation::Replace, ContentsMaterial::Water);
    let frozen = FrozenInput::capture(&e, policy()).unwrap();
    frozen.publish(&path).unwrap();
    paint(&mut e, BrushOperation::Remove, ContentsMaterial::Air);
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "run_sets::voxel_material::tests::fresh_process_child",
            "--ignored",
            "--nocapture",
        ])
        .env("RHEON_VOXEL_INPUT_CHILD", fs::canonicalize(&path).unwrap())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("COLD_VOXEL_OK"));
    assert_eq!(FrozenInput::open(&path).unwrap(), frozen);
    assert_eq!(e.document().counts(), [1, 0, 63]);
}
#[test]
#[ignore = "child-process entrypoint; executed explicitly by its parent fixture"]
fn fresh_process_child() {
    let path = std::env::var_os("RHEON_VOXEL_INPUT_CHILD").expect("owned child fixture input");
    let f = FrozenInput::open(Path::new(&path)).unwrap();
    let e = f.initial_editor().unwrap();
    assert_eq!(e.document().id(), SceneObjectId(13));
    assert_eq!(e.document().revision(), 1);
    assert_eq!(e.document().counts(), [0, 1, 63]);
    let mut sim = f.construct().unwrap();
    assert_eq!(sim.state().generation(), 0);
    let pair = sim.state().materials();
    assert_eq!(
        [
            pair.liquid().density(),
            pair.liquid().viscosity(),
            pair.gas().density(),
            pair.gas().viscosity()
        ]
        .map(f64::to_bits),
        [
            0x408f31999999999a,
            0x3f506ab14ec204f3,
            0x3ff3441c9a98fd61,
            0x3ef315867a02249d
        ]
    );
    let r = sim.step(f.execution().requested_dt_s, |_| false).unwrap();
    assert_eq!(r.generation, 1);
    assert_eq!(
        sim.state()
            .liquid_fraction()
            .iter()
            .filter(|&&x| x == 1.0)
            .count(),
        1
    );
    println!("COLD_VOXEL_OK");
}

#[test]
fn fresh_process_reopens_complete_and_incomplete_drafts_without_numerical_work() {
    let root = directory("cold-drafts-only");
    let complete_path = root.join("complete-draft.json");
    let incomplete_path = root.join("incomplete-draft.json");
    let mut e = editor(ContentsMaterial::Air);
    paint(&mut e, BrushOperation::Replace, ContentsMaterial::Water);
    assert_eq!(e.document().counts(), [0, 1, 63]);
    let complete = DraftRevision::capture(&e).unwrap();
    complete.publish(&complete_path).unwrap();
    paint(&mut e, BrushOperation::Remove, ContentsMaterial::Air);
    assert_eq!(e.document().counts(), [1, 0, 63]);
    let incomplete = DraftRevision::capture(&e).unwrap();
    incomplete.publish(&incomplete_path).unwrap();
    let original_complete = fs::read(&complete_path).unwrap();
    let original_incomplete = fs::read(&incomplete_path).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "run_sets::voxel_material::tests::fresh_process_draft_child",
            "--ignored",
            "--nocapture",
        ])
        .env(
            "RHEON_VOXEL_COMPLETE_DRAFT_CHILD",
            fs::canonicalize(&complete_path).unwrap(),
        )
        .env(
            "RHEON_VOXEL_INCOMPLETE_DRAFT_CHILD",
            fs::canonicalize(&incomplete_path).unwrap(),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child stderr: {}\nchild stdout: {}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("COLD_COMPLETE_AND_INCOMPLETE_DRAFTS_OK")
    );
    assert_eq!(fs::read(&complete_path).unwrap(), original_complete);
    assert_eq!(fs::read(&incomplete_path).unwrap(), original_incomplete);
    assert_eq!(DraftRevision::open(&complete_path).unwrap(), complete);
    assert_eq!(DraftRevision::open(&incomplete_path).unwrap(), incomplete);
}

#[test]
#[ignore = "draft-only child entrypoint; executed explicitly with two parent-owned files"]
fn fresh_process_draft_child() {
    for (key, incomplete) in [
        ("RHEON_VOXEL_COMPLETE_DRAFT_CHILD", false),
        ("RHEON_VOXEL_INCOMPLETE_DRAFT_CHILD", true),
    ] {
        let path = std::env::var_os(key).expect("owned draft-only child input");
        let saved = DraftRevision::open(Path::new(&path)).unwrap();
        let restored = saved.restore().unwrap();
        assert_eq!(restored.document().id(), SceneObjectId(13));
        assert_eq!(
            restored.document().revision(),
            if incomplete { 2 } else { 1 }
        );
        let expected = (0..64)
            .map(|i| {
                if i == 21 {
                    if incomplete { 0 } else { 1 }
                } else {
                    2
                }
            })
            .collect::<Vec<u16>>();
        assert_eq!(restored.document().cells(), expected);
        assert_eq!(
            restored.document().counts(),
            if incomplete { [1, 0, 63] } else { [0, 1, 63] }
        );
        let geometry = restored.document().geometry();
        assert_eq!(geometry.counts(), [4; 3]);
        assert_eq!(geometry.spacing().map(f64::to_bits), [0.25f64.to_bits(); 3]);
        assert_eq!(geometry.origin().map(f64::to_bits), [0.0f64.to_bits(); 3]);
        assert_eq!(restored.undo_label(), None);
        assert_eq!(restored.redo_label(), None);
        assert_eq!(restored.history_bytes(), 0);
        assert_eq!(restored.evicted_commands(), 0);
        assert!(!restored.active());
        assert_eq!(saved.document.palette_version, 1);
        assert_eq!(
            [saved.document.palette[0].id, saved.document.palette[1].id],
            [1, 2]
        );
        assert_eq!(
            [
                saved.document.palette[0].role,
                saved.document.palette[1].role
            ],
            [MaterialRole::Water, MaterialRole::Air]
        );
        assert_eq!(
            [
                saved.document.palette[0].color_srgb,
                saved.document.palette[1].color_srgb
            ],
            [[45, 125, 215], [135, 103, 49]]
        );
        let pair = restored.document().materials();
        assert_eq!(
            [
                pair.liquid().density(),
                pair.liquid().viscosity(),
                pair.gas().density(),
                pair.gas().viscosity(),
            ]
            .map(f64::to_bits),
            [
                0x408f31999999999a,
                0x3f506ab14ec204f3,
                0x3ff3441c9a98fd61,
                0x3ef315867a02249d,
            ]
        );
        if incomplete {
            let error = restored.document().validate().unwrap_err();
            assert_eq!(
                error,
                rheon::initial_contents::ContentsError::Vacuum {
                    count: 1,
                    first: [1; 3],
                }
            );
            assert!(error.to_string().contains("Fill vacuum with Air or Water"));
        } else {
            restored.document().validate().unwrap();
        }
        assert_eq!(DraftRevision::capture(&restored).unwrap(), saved);
    }
    println!("COLD_COMPLETE_AND_INCOMPLETE_DRAFTS_OK");
}

#[test]
fn frozen_geometry_refuses_unrepresentable_scales_before_constructor_admission() {
    let root = directory("geometry");
    let path = root.join("input.json");
    let valid = FrozenInput::capture(&editor(ContentsMaterial::Water), policy()).unwrap();
    // Cell mass overflow, subnormal volume/mass, finite cell but overflowing
    // domain mass, and finite volume with unrepresentable pressure weights.
    for spacing in [
        [1e102; 3],
        [1e-103; 3],
        [1.5e101; 3],
        [1e200, 1e100, 1e-300],
    ] {
        let grid = GridGeometry::new([4; 3], spacing, [0.0; 3]).unwrap();
        let e = ContentsEditor::new(
            InitialContents::new(SceneObjectId(13), grid, ContentsMaterial::Water).unwrap(),
        );
        // A draft may be saved to repair its geometry; it is not a run recipe.
        assert!(DraftRevision::capture(&e).is_ok());
        assert!(matches!(
            FrozenInput::capture(&e, policy()),
            Err(RunSetError::Invalid(
                "voxel numerical geometry representability"
            ))
        ));
        let mut input = valid.input.clone();
        input.initial.spacing_m = spacing;
        let sha256 = digest(&input).unwrap();
        fs::write(
            &path,
            serde_json::to_vec(&InputEnvelope {
                schema_version: INPUT_VERSION,
                input,
                sha256,
            })
            .unwrap(),
        )
        .unwrap();
        assert!(matches!(
            FrozenInput::open(&path),
            Err(RunSetError::Invalid(
                "voxel numerical geometry representability"
            ))
        ));
    }
    let grid = GridGeometry::new([4; 3], [0.2, 0.3, 0.4], [-0.0, 0.125, -2.5]).unwrap();
    let e = ContentsEditor::new(
        InitialContents::new(SceneObjectId(13), grid, ContentsMaterial::Air).unwrap(),
    );
    let input = FrozenInput::capture(&e, policy()).unwrap();
    let path = root.join("valid.json");
    input.publish(&path).unwrap();
    let loaded = FrozenInput::open(&path).unwrap();
    assert_eq!(loaded, input);
    assert_eq!(
        loaded
            .initial_editor()
            .unwrap()
            .document()
            .geometry()
            .origin()[0]
            .to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(loaded.construct().unwrap().state().generation(), 0);
}
