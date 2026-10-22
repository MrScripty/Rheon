//! Explicit post-smoke audit only: protected clean paths, no concurrent writer,
//! actual App/display already closed. Reuses the reviewed no-recovery-sidecar
//! closed-pin reader; no solver, publication, pin or OS read-only guarantee.
use crate::run_sets::{file_digest, voxel_material::DraftRevision};
use rheon::{
    SnapshotStage,
    recording::{RecordedVoxelMaterial, VOXEL_MATERIAL_SOLVER},
};
use std::path::PathBuf;

#[test]
#[ignore = "requires the observed ordinary-App smoke artifacts; reads existing data only"]
fn ordinary_native_artifacts_match_saved_seed_and_one_interval() {
    let set_path =
        PathBuf::from(std::env::var_os("RHEON_ORDINARY_SET").expect("existing finished.json"));
    let draft_path =
        PathBuf::from(std::env::var_os("RHEON_ORDINARY_DRAFT").expect("existing saved draft"));
    let root = set_path.parent().unwrap();
    let inputs = [
        set_path.clone(),
        draft_path.clone(),
        root.join("intent.json"),
        root.join("started.json"),
    ];
    let unchanged = inputs.map(|path| {
        let hash = file_digest(&path, 1024 * 1024).unwrap();
        (path, hash)
    });
    let mut archives = Vec::new();
    let (set, reader) = super::audit::open_finished(&set_path, &mut archives);
    let draft = DraftRevision::open(&draft_path).unwrap();
    let editor = draft.restore().unwrap();
    let captured = set.members[0].spec.initial_editor().unwrap();
    let expected: [u16; 64] =
        std::array::from_fn(|i| if [37, 38, 41, 42].contains(&i) { 1 } else { 2 });
    assert_eq!(editor.document().cells(), expected);
    assert_eq!(captured.document().cells(), expected);
    assert_eq!(captured.document().id(), editor.document().id());
    assert_eq!(captured.document().revision(), editor.document().revision());
    assert_eq!(
        captured.document().materials(),
        editor.document().materials()
    );
    for document in [editor.document(), captured.document()] {
        assert_eq!(document.geometry().counts(), [4; 3]);
        assert_eq!(
            document.geometry().spacing().map(f64::to_bits),
            [0.25f64.to_bits(); 3]
        );
        assert_eq!(
            document.geometry().origin().map(f64::to_bits),
            [0.0f64.to_bits(); 3]
        );
        assert_eq!(document.counts(), [0, 4, 60]);
        document.validate().unwrap();
    }
    let policy = set.members[0].spec.execution();
    assert_eq!(policy.requested_steps, 1);
    assert_eq!(policy.requested_dt_s.to_bits(), 0.001f64.to_bits());
    assert_eq!(policy.gravity_m_s2.map(f64::to_bits), [0.0f64.to_bits(); 3]);
    assert_eq!(policy.numerical_memory_limit_bytes, 128 * 1024 * 1024);
    assert_eq!(policy.executor.id, "rheon-desktop-voxel-material-serial-v1");
    assert_eq!(reader.run().solver, VOXEL_MATERIAL_SOLVER);
    let status = reader.status().unwrap();
    assert_eq!(status.outcome, "completed");
    assert_eq!(status.accepted_generation, Some(1));
    assert_eq!(reader.count().unwrap(), 2);
    for ordinal in 0..2 {
        let raw = reader.select(ordinal).unwrap();
        let fields = RecordedVoxelMaterial::decode(reader.run(), &raw).unwrap();
        assert_eq!(
            fields.metadata.initial_input_sha256,
            set.members[0].spec.digest()
        );
        assert_eq!(raw.generation, ordinal as u64);
        assert_eq!(
            raw.time.to_bits(),
            (if ordinal == 0 { 0.0f64 } else { 0.001 }).to_bits()
        );
        assert_eq!(
            raw.dt.map(f64::to_bits),
            if ordinal == 0 {
                None
            } else {
                Some(0.001f64.to_bits())
            }
        );
        assert_eq!(
            raw.stage,
            if ordinal == 0 {
                SnapshotStage::Constructor
            } else {
                SnapshotStage::Accepted
            }
        );
        for (&fraction, id) in fields.liquid_fraction.iter().zip(expected) {
            assert_eq!(
                fraction.to_bits(),
                if id == 1 { 1.0f64.to_bits() } else { 0 }
            );
        }
        assert!(
            fields
                .velocity
                .iter()
                .flatten()
                .all(|&v| v.is_finite() && v == 0.0)
        );
        if ordinal == 0 {
            assert!(fields.interval_pressure.is_none());
        } else {
            let (start, pressure) = fields.interval_pressure.as_ref().unwrap();
            assert_eq!(start.to_bits(), 0.0f64.to_bits());
            assert_eq!(pressure.len(), 64);
            assert!(pressure.iter().all(|&p| p.is_finite() && p == 0.0));
            assert_eq!(pressure[0].to_bits(), 0.0f64.to_bits());
        }
        let pair = fields.metadata.materials;
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
    }
    println!(
        "ordinary native: run_id={} status={status:?} publications=2 constructor/accepted=4Water60Air exact rest; executor={:?}",
        reader.run().identity.run_id,
        policy.executor
    );
    drop(reader);
    for (path, hash) in unchanged {
        assert_eq!(file_digest(&path, 1024 * 1024).unwrap(), hash);
        println!("unchanged input {} sha256={hash:02x?}", path.display());
    }
    for (path, hash, limit) in archives {
        assert_eq!(file_digest(&path, limit).unwrap(), hash);
        println!(
            "unchanged pinned archive {} sha256={hash:02x?}",
            path.display()
        );
    }
}
