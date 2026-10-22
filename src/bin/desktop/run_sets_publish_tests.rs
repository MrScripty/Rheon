//! Injected I/O boundaries on clean caller-owned files, not power-loss tests.
use super::*;
use rheon::{
    GridGeometry, SceneObjectId,
    initial_contents::{ContentsEditor, ContentsMaterial, InitialContents},
};

fn directory(label: &str) -> std::path::PathBuf {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".generated/voxel-material")
        .join(format!(
            "publication-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&root).unwrap();
    root
}
fn draft(material: ContentsMaterial) -> voxel_material::DraftRevision {
    let initial = InitialContents::new(
        SceneObjectId(1),
        GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
        material,
    )
    .unwrap();
    voxel_material::DraftRevision::capture(&ContentsEditor::new(initial)).unwrap()
}
#[test]
fn publication_stage_failures_preserve_confirmed_old_and_reconcile_new_without_retry() {
    use PublicationFailure::*;
    for stage in [
        BeforeStage,
        BeforeFlush,
        BeforeLink,
        FirstDirectorySync,
        RemoveTemporary,
        FinalDirectorySync,
    ] {
        let root = directory("fault");
        let old = root.join("confirmed.json");
        let target = root.join("new.json");
        draft(ContentsMaterial::Air).publish(&old).unwrap();
        let original = fs::read(&old).unwrap();
        let new = draft(ContentsMaterial::Water);
        let reference = root.join("reference-new.json");
        new.publish(&reference).unwrap();
        let bytes = fs::read(reference).unwrap();
        let error = publish_new_impl(&target, &bytes, 1024 * 1024, Some(stage)).unwrap_err();
        assert_eq!(
            fs::read(&old).unwrap(),
            original,
            "old confirmed document changed at {stage:?}"
        );
        let old_document = voxel_material::DraftRevision::open(&old)
            .unwrap()
            .restore()
            .unwrap();
        assert_eq!(old_document.document().counts(), [0, 0, 64]);
        match stage {
            BeforeStage | BeforeFlush | BeforeLink => {
                assert!(matches!(error, RunSetError::Io(_)), "{stage:?}: {error}");
                assert!(!target.exists(), "pre-link target became authoritative");
                if stage == BeforeStage {
                    assert_eq!(fs::read_dir(&root).unwrap().count(), 2);
                }
            }
            FirstDirectorySync | RemoveTemporary | FinalDirectorySync => {
                assert!(
                    matches!(error, RunSetError::PublicationUncertain(_)),
                    "{stage:?}: {error}"
                );
                assert_eq!(fs::read(&target).unwrap(), bytes);
                let restored = voxel_material::DraftRevision::open(&target)
                    .unwrap()
                    .restore()
                    .unwrap();
                assert_eq!(restored.document().counts(), [0, 64, 0]);
                // Only read-only reconciliation: do not retry uncertain publication.
            }
        }
    }
}
#[test]
fn ordinary_publication_guards_and_complete_success_preserve_no_replace() {
    let root = directory("normal");
    let path = root.join("confirmed.json");
    let new = draft(ContentsMaterial::Water);
    let reference = root.join("reference-new.json");
    new.publish(&reference).unwrap();
    let bytes = fs::read(reference).unwrap();
    assert!(publish_new(&path, &bytes, 0).is_err());
    assert!(publish_new(&path, &bytes, bytes.len() as u64 - 1).is_err());
    assert!(!path.exists());
    assert!(publish_new(&root.join("missing/file.json"), &bytes, 1024 * 1024).is_err());
    publish_new(&path, &bytes, 1024 * 1024).unwrap();
    assert_eq!(voxel_material::DraftRevision::open(&path).unwrap(), new);
    let original = fs::read(&path).unwrap();
    assert!(
        matches!(draft(ContentsMaterial::Air).publish(&path),Err(RunSetError::Io(e)) if e.kind()==std::io::ErrorKind::AlreadyExists)
    );
    assert_eq!(fs::read(path).unwrap(), original);
}
