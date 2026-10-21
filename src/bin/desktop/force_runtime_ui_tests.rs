//! In-memory custody/intent probes only; never construct or step a solver.
use super::*;
use rheon::{
    ForceKind, ForceObject, ForceScene, ForceSnapshot, ForceSupport, ForceTime, ForceTransform,
    ForceUnits, SceneObjectId,
};
use run_sets::{FrozenSpec, NativeCarrierSpec, RunSetMember, RunSetRevision, SpecDifference};
const V1_SPEC: &str = r#"{"counts_xyz":[2,2,2],"spacing_m":[0.5,0.5,0.5],"origin_m":[0.0,0.0,0.0],"density_kg_m3":2.0,"pressure_method":"jacobi_pcg_v1","tolerances":{"relative_residual":1.0,"absolute_residual_m3_s2":1.0,"predicted_divergence_per_s":1.0,"actual_divergence_per_s":1.0,"max_courant":1.0,"max_pressure_iterations":4},"requested_dt_s":0.125,"requested_steps":1,"source":null,"simulation_memory_limit_bytes":1048576,"recording":{"max_database_bytes":65536,"max_record_bytes":16384,"max_records":2}}"#;
fn spec() -> FrozenSpec {
    FrozenSpec::new(serde_json::from_str::<NativeCarrierSpec>(V1_SPEC).unwrap()).unwrap()
}
fn scene() -> ForceScene {
    ForceScene {
        version: 1,
        objects: vec![ForceObject {
            id: SceneObjectId(1),
            name: "Recorded force".into(),
            enabled: false,
            transform: ForceTransform {
                translation: [-0.0, 0.5, 0.5],
                ..Default::default()
            },
            units: ForceUnits::ForceDensity,
            kind: ForceKind::Vortex { strength: -0.25 },
            support: ForceSupport::Box {
                half_extent: [0.1, 0.2, 0.3],
                edge_fraction: 0.25,
            },
            time: ForceTime {
                start_seconds: 0.125,
                end_seconds: Some(0.5),
            },
        }],
    }
}
fn metadata(fields: &ForceSnapshot) -> Vec<u8> {
    rheon::recording::encode_force_metadata(fields).unwrap()
}
fn set(specs: Vec<FrozenSpec>, difference: Vec<SpecDifference>) -> RunSetRevision {
    RunSetRevision {
        schema_version: 1,
        set_id: "literal-set".into(),
        parent_sha256: None,
        baseline_member: "member-0".into(),
        members: specs
            .into_iter()
            .enumerate()
            .map(|(i, spec)| RunSetMember {
                member_id: format!("member-{i}"),
                spec,
                differences_from_baseline: if i == 0 { vec![] } else { difference.clone() },
                attempt: None,
            })
            .collect(),
    }
}
#[test]
fn force_intent_legacy_v1_bytes_and_digest_remain_exact() {
    use sha2::{Digest, Sha256};
    let original = spec();
    assert_eq!(serde_json::to_string(&original.spec).unwrap(), V1_SPEC);
    assert_eq!(
        format!("{:x}", Sha256::digest(V1_SPEC.as_bytes())),
        "5969e6f1a25728943a3a1119d4f393f33a4bb45af71a5d99a54551d06d569519"
    );
    assert_eq!(
        original.sha256,
        <[u8; 32]>::from(Sha256::digest(V1_SPEC.as_bytes()))
    );
    let legacy = format!(
        "{{\"schema_version\":1,\"sha256\":{},\"spec\":{V1_SPEC}}}",
        serde_json::to_string(&original.sha256).unwrap()
    );
    assert_eq!(serde_json::to_string(&original).unwrap(), legacy);
    let decoded = FrozenSpec::from_bytes(legacy.as_bytes()).unwrap();
    assert_eq!(decoded, original);
    assert!(decoded.force_snapshot().unwrap().is_none());
    assert_eq!(serde_json::to_vec(&decoded).unwrap(), legacy.as_bytes());
}
#[test]
fn force_intent_v2_freeze_roundtrip_and_mixed_member_differences_are_exact() {
    let mut draft = scene();
    let fields = draft.freeze().unwrap();
    let original = spec();
    let forced = original.clone().with_force_fields(&fields).unwrap();
    assert_eq!(forced.schema_version, 2);
    // Independently constructed with Python struct.pack little-endian scalars,
    // literal tags/bytes and hashlib; not the Rust encoder/decoder roundtrip.
    let literal_hex = "5248454f4e46460001000000010000000100000001000000000000000e0000005265636f7264656420666f726365000000000000000080000000000000e03f000000000000e03f000000000000000000000000000000000000000000000000000000000000f03f0102000000000000d0bf029a9999999999b93f9a9999999999c93f333333333333d33f000000000000d03f000000000000c03f01000000000000e03f";
    let encoded_hex: String = metadata(&fields)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(encoded_hex, literal_hex);
    assert_eq!(
        forced
            .sha256
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>(),
        "fd129900f78968456c52e42562d7a860fae3885f180f6d72717da127a0bfcbcb"
    );
    assert_ne!(forced.sha256, original.sha256);
    let bytes = serde_json::to_vec(&forced).unwrap();
    let restored = FrozenSpec::from_bytes(&bytes).unwrap();
    assert_eq!(
        metadata(&restored.force_snapshot().unwrap().unwrap()),
        metadata(&fields)
    );
    draft.objects[0].enabled = true;
    draft.objects[0].name = "Changed draft".into();
    assert_eq!(
        restored.force_fields.unwrap().objects[0].name,
        "Recorded force"
    );
    let empty = ForceScene::default().freeze().unwrap();
    let empty_spec = spec().with_force_fields(&empty).unwrap();
    assert_ne!(empty_spec.sha256, original.sha256);
    assert!(
        set(
            vec![original.clone(), empty_spec.clone()],
            vec![SpecDifference::ForceMetadata]
        )
        .validate()
        .is_ok()
    );
    assert!(
        set(vec![original.clone(), empty_spec], vec![])
            .validate()
            .is_err()
    );
    let mut ordered = fields.scene().clone();
    let mut second = ordered.objects[0].clone();
    second.id = SceneObjectId(2);
    ordered.objects.push(second);
    let forward = spec()
        .with_force_fields(&ordered.freeze().unwrap())
        .unwrap();
    ordered.objects.reverse();
    let reverse = spec()
        .with_force_fields(&ordered.freeze().unwrap())
        .unwrap();
    assert_ne!(forward.sha256, reverse.sha256);
    assert!(
        set(vec![forward, reverse], vec![SpecDifference::ForceMetadata])
            .validate()
            .is_ok()
    );
    assert!(
        set(
            vec![original.clone(), forced.clone()],
            vec![SpecDifference::ForceMetadata]
        )
        .validate()
        .is_ok()
    );
    assert!(
        set(vec![original, forced.clone()], vec![])
            .validate()
            .is_err()
    );
    for mutation in 0..4 {
        let mut changed = fields.scene().clone();
        match mutation {
            0 => changed.objects[0].transform.translation[0] = 0.0,
            1 => changed.objects[0].enabled = true,
            2 => changed.objects[0].units = ForceUnits::Acceleration,
            _ => changed.objects[0].name = "Renamed".into(),
        };
        let changed = spec()
            .with_force_fields(&changed.freeze().unwrap())
            .unwrap();
        assert_ne!(changed.sha256, forced.sha256);
        assert!(
            set(
                vec![forced.clone(), changed],
                vec![SpecDifference::ForceMetadata]
            )
            .validate()
            .is_ok()
        );
    }
}
#[test]
fn force_intent_rejects_mixed_versions_tamper_unknown_duplicate_and_oversize() {
    let forced = spec()
        .with_force_fields(&scene().freeze().unwrap())
        .unwrap();
    let value = serde_json::to_value(&forced).unwrap();
    for (key, v) in [
        ("schema_version", serde_json::json!(1)),
        ("schema_version", serde_json::json!(3)),
        ("force_fields", serde_json::Value::Null),
    ] {
        let mut bad = value.clone();
        bad[key] = v;
        assert!(FrozenSpec::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    }
    let mut bad = value.clone();
    bad.as_object_mut().unwrap().remove("force_fields");
    assert!(FrozenSpec::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = value.clone();
    bad["force_fields"]["objects"][0]["name"] = serde_json::json!("Tampered");
    assert!(FrozenSpec::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let mut bad = value.clone();
    bad["unknown"] = serde_json::json!(true);
    assert!(FrozenSpec::from_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    let json = serde_json::to_string(&forced).unwrap();
    let duplicate = json.replacen("{", "{\"schema_version\":2,", 1);
    assert!(FrozenSpec::from_bytes(duplicate.as_bytes()).is_err());
    let mut bad = value;
    bad["schema_version"] = serde_json::json!(99);
    assert!(matches!(
        FrozenSpec::from_bytes(&serde_json::to_vec(&bad).unwrap()),
        Err(run_sets::RunSetError::Unsupported(
            "frozen native intent version"
        ))
    ));
    assert!(FrozenSpec::from_bytes(&vec![b' '; 1024 * 1024 + 1]).is_err());
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
        requested_dt: 0.125,
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
#[test]
fn force_request_track_resolver_gates_before_work_and_binds_saved_members() {
    let fields = scene().freeze().unwrap();
    let mut r = request();
    r.force_fields = vec![Some(fields.clone())];
    assert!(
        validate_request(&r)
            .unwrap_err()
            .contains("validation is pending")
    );
    r.force_validation = true;
    assert!(validate_request(&r).is_ok());
    let forced = spec().with_force_fields(&fields).unwrap();
    r.methods = PressureImplementation::ALL.to_vec();
    r.force_fields = vec![None, Some(fields.clone())];
    r.saved_set = Some(set(
        vec![spec(), forced],
        vec![SpecDifference::ForceMetadata],
    ));
    assert!(validate_request(&r).is_ok());
    assert!(r.fields_for_track(0).is_none());
    assert_eq!(metadata(r.fields_for_track(1).unwrap()), metadata(&fields));
    r.force_fields = vec![Some(fields.clone()), None];
    assert!(
        validate_request(&r)
            .unwrap_err()
            .contains("snapshot mismatch")
    );
    r.saved_set = None;
    r.force_fields = vec![Some(fields)];
    assert!(validate_request(&r).unwrap_err().contains("track count"));
    r.force_fields.clear();
    r.force_validation = false;
    assert!(validate_request(&r).is_ok());
}
#[test]
fn force_recording_header_preflight_rejects_missing_wrong_or_double_binding_without_io() {
    let legacy = rheon::recording::RecordingRun {
        identity: rheon::SessionIdentity {
            run_id: 7,
            source_revision: "literal".into(),
        },
        solver: "literal-solver".into(),
        configuration: "literal-config".into(),
        counts: [1; 3],
        spacing: [1.0; 3],
        origin: [0.0; 3],
        provenance: "literal-provenance".into(),
    };
    let fields = scene().freeze().unwrap();
    let bound = inspector::bind_force_fields(legacy.clone(), Some(&fields)).unwrap();
    assert!(inspector::verify_force_fields(&legacy, None).is_ok());
    assert!(inspector::verify_force_fields(&legacy, Some(&fields)).is_err());
    assert!(inspector::verify_force_fields(&bound, None).is_err());
    assert!(inspector::verify_force_fields(&bound, Some(&fields)).is_ok());
    assert!(inspector::bind_force_fields(bound.clone(), Some(&fields)).is_err());
    let empty = ForceScene::default().freeze().unwrap();
    assert!(inspector::verify_force_fields(&bound, Some(&empty)).is_err());
    assert_eq!(legacy.configuration, "literal-config");
}
#[test]
fn force_retained_track_projection_is_readonly_and_clears_same_id_selection() {
    let fields = scene().freeze().unwrap();
    let mut ui = scene_ui::SceneUi::geometry_fixture(ForceScene::default(), false);
    ui.begin_run(false);
    ui.bind_retained_fields(0, Some(&fields)).unwrap();
    ui.select(Some(SceneObjectId(1)));
    assert_eq!(ui.selected(), Some(SceneObjectId(1)));
    assert_eq!(
        metadata(&ui.display_scene().freeze().unwrap()),
        metadata(&fields)
    );
    assert!(ui.selected_force().is_none());
    assert!(!ui.begin_edit());
    ui.bind_retained_fields(1, Some(&fields)).unwrap();
    assert_eq!(ui.selected(), None);
    ui.select(Some(SceneObjectId(1)));
    let mut changed = scene();
    changed.objects[0].name = "Other track document".into();
    ui.bind_retained_fields(1, Some(&changed.freeze().unwrap()))
        .unwrap();
    assert_eq!(ui.selected(), None);
    let ctx = egui::Context::default();
    ui.open = true;
    let out = ctx.run(Default::default(), |ctx| ui.show(ctx, true, false, None));
    assert!(
        out.shapes.iter().any(
            |s| matches!(&s.shape,egui::Shape::Text(t)if t.galley.text().contains("read-only"))
        )
    );
    assert!(ui.draft().objects.is_empty());
    ui.bind_retained_fields(2, None).unwrap();
    assert!(ui.display_scene().objects.is_empty());
}
#[test]
fn force_retained_composition_resolves_once_before_tree_and_defers_track_intents() {
    let mut request = request();
    request.methods = PressureImplementation::ALL.to_vec();
    let first = scene().freeze().unwrap();
    let mut second = scene();
    second.objects[0].name = "Second retained track".into();
    request.force_fields = vec![Some(first), Some(second.freeze().unwrap())];
    // Literal presentation data only: no capture, solver, worker or archive.
    let history = request.preview_history().unwrap();
    for (track, method) in request.methods.iter().enumerate() {
        history
            .publish(
                track,
                Arc::new(PreviewFrame {
                    method: *method,
                    step: 0,
                    time: track as f64,
                    seconds: 0.0,
                    edge: 2,
                    nonzero_tracer_cells: 0,
                    density: vec![0; 8],
                    report: None,
                    particle_report: None,
                    material_color: None,
                }),
            )
            .unwrap();
    }
    let mut app = App::default();
    app.running_request = Some(request);
    app.history = Some(history);
    app.scene.begin_run(false);
    // The active track is 1 after literal publication; follow-live must resolve
    // before binding tree/properties, never half way through painting.
    let (_, frame) = app.prepare_scene_presentation().unwrap();
    assert_eq!(app.track, 1);
    assert_eq!(frame.unwrap().time, 1.0);
    assert_eq!(
        app.scene.display_scene().objects[0].name,
        "Second retained track"
    );
    app.scene.select(Some(SceneObjectId(1)));
    // This is the scalar intent committed by timeline painting. Both cached
    // viewport and already composed properties stay on 1 until next prepare.
    app.follow_live = false;
    app.track = 0;
    assert_eq!(
        app.scene.display_scene().objects[0].name,
        "Second retained track"
    );
    let (_, frame) = app.prepare_scene_presentation().unwrap();
    assert_eq!(frame.unwrap().time, 0.0);
    assert_eq!(app.scene.display_scene().objects[0].name, "Recorded force");
    assert_eq!(app.scene.selected(), None);
    app.scene.context = scene_ui::SceneContext::Draft;
    app.timeline_step = 9;
    assert!(app.prepare_scene_presentation().is_none());
    assert_eq!(app.timeline_step, 9);
    assert!(app.worker.is_none());
}
