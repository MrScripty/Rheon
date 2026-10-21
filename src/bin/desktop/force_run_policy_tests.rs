//! Pure target/metadata preflight only; no Worker, solver, SQLite or cleanup.
use super::*;
use force_run_policy::{
    PENDING, PRODUCTION_QUALIFICATION, Qualification, Target, UNSUPPORTED, check,
};
fn fields() -> rheon::ForceSnapshot {
    rheon::ForceScene {
        version: 1,
        objects: vec![rheon::ForceObject {
            id: rheon::SceneObjectId(1),
            name: "Disabled zero field".into(),
            enabled: false,
            transform: Default::default(),
            units: rheon::ForceUnits::Acceleration,
            kind: rheon::ForceKind::Directional { vector: [0.0; 3] },
            support: rheon::ForceSupport::Global,
            time: Default::default(),
        }],
    }
    .freeze()
    .unwrap()
}
fn request() -> Request {
    Request {
        force_fields: Vec::new(),
        force_validation: false,
        collision_mesh: None,
        size: 2,
        steps: 1,
        tight: false,
        threads: 1,
        methods: vec![PressureImplementation::JacobiPcgV1],
        emitter: None,
        observe: false,
        history_budget_mib: 16,
        preview_resolution: history::PreviewResolution::Edge(2),
        domain_side: 1.0,
        requested_dt: 1e-6,
        material: None,
        material_color: [0; 3],
        recording_limits: rheon::recording::RecordingLimits {
            max_database_bytes: 65536,
            max_record_bytes: 16384,
            max_records: 2,
        },
        recording_directory: "unused-no-output".into(),
        saved_set: None,
        configured_set: None,
        prior_run_ids: vec![],
        manifest_receipt: None,
        progress_gate: None,
        failure_gate: None,
    }
}
fn unsupported_request() -> ConfiguredRequest {
    // Deliberately incomplete metadata sentinel: target refusal must happen before
    // configured provider/model preparation. No fixture builder or filesystem.
    ConfiguredRequest {
        revision: run_sets::configured::ConfiguredRunSetRevision {
            schema_version: 1,
            kind: run_sets::configured::ConfiguredSetKind::ConfiguredAcceptedSessionSet,
            set_id: "unused".into(),
            parent_sha256: None,
            baseline_member: "unused".into(),
            members: vec![],
        },
        project_root: "unused".into(),
        runtime: None,
    }
}
#[test]
fn force_policy_qualification_never_grants_unsupported_owner() {
    assert_eq!(PRODUCTION_QUALIFICATION, Qualification::Pending);
    for qualification in [Qualification::Pending, Qualification::Validated] {
        assert_eq!(
            check(Target::Unsupported, true, qualification),
            Err(UNSUPPORTED)
        );
        for target in [
            Target::NativeCarrier,
            Target::ParticleMaterial,
            Target::Unsupported,
        ] {
            assert_eq!(check(target, false, qualification), Ok(()));
        }
    }
    for target in [Target::NativeCarrier, Target::ParticleMaterial] {
        assert_eq!(check(target, true, PRODUCTION_QUALIFICATION), Err(PENDING));
        assert_eq!(check(target, true, Qualification::Validated), Ok(()));
    }
}
#[test]
fn force_policy_request_and_setup_resolve_effective_targets() {
    assert_eq!(
        SetupWorkflow::Preview.force_target(false),
        Target::NativeCarrier
    );
    assert_eq!(
        SetupWorkflow::Preview.force_target(true),
        Target::ParticleMaterial
    );
    for material in [false, true] {
        assert_eq!(
            SetupWorkflow::AcceptedSession.force_target(material),
            Target::Unsupported
        );
    }
    for snapshot in [fields(), rheon::ForceScene::default().freeze().unwrap()] {
        let mut r = request();
        r.force_fields = vec![Some(snapshot)];
        assert_eq!(r.force_target(), Target::NativeCarrier);
        assert_eq!(validate_request(&r), Err(PENDING.to_owned()));
        r.force_validation = true;
        assert!(validate_request(&r).is_ok());
        r.material = Some(Default::default());
        assert_eq!(r.force_target(), Target::ParticleMaterial);
        assert!(validate_request(&r).is_ok());
        r.configured_set = Some(unsupported_request());
        assert_eq!(
            r.force_target(),
            Target::Unsupported,
            "configured takes precedence over material"
        );
        assert_eq!(validate_request(&r), Err(UNSUPPORTED.to_owned()));
        r.force_validation = false;
        assert_eq!(validate_request(&r), Err(UNSUPPORTED.to_owned()));
        r.methods = PressureImplementation::ALL.to_vec();
        r.force_fields.push(r.force_fields[0].clone());
        assert_eq!(
            validate_request(&r),
            Err(UNSUPPORTED.to_owned()),
            "configured target owns refusal even with multiple material tracks"
        );
    }
}
#[test]
fn force_policy_scene_readiness_uses_valid_draft_not_retained_display() {
    let fields = fields();
    let draft = scene_ui::SceneUi::geometry_fixture(fields.scene().clone(), false);
    for target in [Target::NativeCarrier, Target::ParticleMaterial] {
        assert_eq!(draft.launch_readiness(target), Err(PENDING.to_owned()));
    }
    assert_eq!(
        draft.launch_readiness(Target::Unsupported),
        Err(UNSUPPORTED.to_owned())
    );
    let mut invalid = fields.scene().clone();
    invalid.objects[0].support = rheon::ForceSupport::Sphere {
        radius: 0.0,
        edge_fraction: 0.0,
    };
    let invalid = scene_ui::SceneUi::geometry_fixture(invalid, false);
    assert!(
        invalid
            .launch_readiness(Target::NativeCarrier)
            .unwrap_err()
            .contains("Invalid authored force")
    );
    let mut empty = scene_ui::SceneUi::geometry_fixture(Default::default(), false);
    empty.begin_run(false);
    empty.bind_retained_fields(0, Some(&fields)).unwrap();
    assert!(!empty.display_scene().objects.is_empty());
    assert!(empty.draft().objects.is_empty());
    assert_eq!(empty.launch_readiness(Target::Unsupported), Ok(()));
    assert_eq!(empty.launch_readiness(Target::NativeCarrier), Ok(()));
}
