use super::*;
use serde_json::json;

fn fixture(sequence: u64) -> serde_json::Value {
    json!({"schema_version":1,"session":"test-session","sequence":sequence,"state":"running",
        "phase":"Teacher diagnostic","label":"Read-only fixture","frame":16,"time_s":0.25,
        "counts_xyz":[2,2,2],"extents_m":[1.0,1.0,1.0],
        "tracer":[0.0,0.1,0.2,0.3,0.4,0.5,0.6,0.7],
        "pressure":[-4.0,-3.0,-2.0,-1.0,1.0,2.0,3.0,4.0],
        "pressure_label":"Proposal only; native rejection/fallback remains in source evidence"})
}
fn decode(value: &serde_json::Value) -> Preview {
    let frame: Preview = serde_json::from_value(value.clone()).unwrap();
    frame.validate().unwrap();
    frame
}
struct Output(PathBuf);
impl Output {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rheon-live-view-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn publish(&self, value: &serde_json::Value) {
        let pending = self.0.join("pending.json");
        std::fs::write(&pending, serde_json::to_vec(value).unwrap()).unwrap();
        std::fs::rename(pending, self.0.join("frame.json")).unwrap();
    }
}
impl Drop for Output {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn rectangular_cubic_geometry_and_field_order_remain_exact() {
    let mut value = fixture(1);
    value["counts_xyz"] = json!([4, 3, 2]);
    value["extents_m"] = json!([1.0, 0.75, 0.5]);
    value["cell_size_m"] = json!(0.25);
    value["tracer"] = json!((0..24).map(|v| v as f32).collect::<Vec<_>>());
    value["pressure"] = json!((0..24).map(|v| -(v as f64)).collect::<Vec<_>>());
    let frame = decode(&value);
    assert!(frame.cubic());
    assert_eq!(frame.spacing(), [0.25; 3]);
    assert_eq!(frame.index(3, 2, 1), 23);
    assert_eq!(
        frame.pressure.as_ref().unwrap()[frame.index(3, 2, 1)],
        -23.0
    );
    assert_eq!(frame.display([0.0; 3]), [0.0, 0.125, 0.25]);
    assert_eq!(frame.display([1.0; 3]), [1.0, 0.875, 0.75]);
    value["cell_size_m"] = json!(0.5);
    assert!(
        serde_json::from_value::<Preview>(value)
            .unwrap()
            .validate()
            .is_err()
    );
}

#[test]
fn pressure_only_has_no_fabricated_tracer_and_non_cubic_observation_stays_visible() {
    let mut value = fixture(1);
    value.as_object_mut().unwrap().remove("tracer");
    value["extents_m"] = json!([1.5, 1.0, 0.75]);
    value["forcing"] = json!({"regime":"steady","amplitude_m_s2":0.5});
    let frame = decode(&value);
    assert!(frame.tracer.is_none());
    assert!(!frame.cubic());
    assert_eq!(frame.forcing.as_ref().unwrap().amplitude_m_s2, 0.5);
    assert_eq!(frame.payload_bytes(), 8 * 8);
}

#[test]
fn malformed_shape_nonfinite_geometry_and_oversized_fields_are_rejected() {
    for change in ["shape", "geometry", "budget", "state", "forcing", "spacing"] {
        let mut value = fixture(1);
        match change {
            "shape" => value["pressure"] = json!([0.0]),
            "geometry" => value["extents_m"] = json!([0.0, 1.0, 1.0]),
            "budget" => value["counts_xyz"] = json!([128, 128, 128]),
            "state" => value["state"] = json!("accepted"),
            "forcing" => value["forcing"] = json!({"regime":"steady","amplitude_m_s2":-1.0}),
            "spacing" => value["cell_size_m"] = json!(0.01),
            _ => unreachable!(),
        }
        assert!(
            serde_json::from_value::<Preview>(value)
                .unwrap()
                .validate()
                .is_err(),
            "{change}"
        );
    }
    let mut frame = decode(&fixture(1));
    frame.pressure.as_mut().unwrap()[0] = f64::NAN;
    assert!(frame.validate().is_err());
    let raw = serde_json::to_string(&fixture(1)).unwrap().replace(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
    );
    assert!(serde_json::from_str::<Preview>(&raw).is_err());
}

#[test]
fn current_activity_has_bounded_truthful_progress_and_utc_timestamp() {
    let value = json!({"schema_version":1,"producer":"Actual fixture","sequence":3,
        "updated_at":"2026-10-16T22:00:00.123456+00:00","state":"failed","phase":"Native gate",
        "message":"Raw proposal rejected; native fallback is separate", "progress":{"completed":1,"total":2}});
    let mut activity: Activity = serde_json::from_value(value.clone()).unwrap();
    activity.validate().unwrap();
    activity.progress.total = Some(0);
    assert!(activity.validate().is_err());
    assert!(!utc_timestamp("2026-02-30T22:00:00+00:00"));
    assert!(!utc_timestamp("2026-10-16T22:00:00-07:00"));
    assert!(utc_timestamp("2024-02-29T22:00:00Z"));
    let mut missing_total = value.clone();
    missing_total["progress"]
        .as_object_mut()
        .unwrap()
        .remove("total");
    assert!(serde_json::from_value::<Activity>(missing_total).is_err());
    let mut obsolete = value;
    obsolete["activity_path"] = json!("old.json");
    assert!(serde_json::from_value::<Activity>(obsolete).is_err());
}

#[test]
fn repeated_publications_are_lazy_and_invalid_update_keeps_last_valid_frame() {
    let output = Output::new();
    output.publish(&fixture(2));
    let mut view = LiveProject::new(output.0.join("frame.json"), None);
    view.poll();
    let pointer = view
        .preview
        .as_ref()
        .unwrap()
        .pressure
        .as_ref()
        .unwrap()
        .as_ptr();
    view.poll();
    assert_eq!(
        view.preview
            .as_ref()
            .unwrap()
            .pressure
            .as_ref()
            .unwrap()
            .as_ptr(),
        pointer
    );
    output.publish(&fixture(1));
    view.poll();
    assert_eq!(view.preview.as_ref().unwrap().sequence, 2);
    let mut malformed = fixture(3);
    malformed["pressure"] = json!([1.0]);
    output.publish(&malformed);
    view.poll();
    assert_eq!(view.preview.as_ref().unwrap().sequence, 2);
    assert!(!view.feedback.is_empty());
    let mut next = fixture(1);
    next["session"] = json!("new-session");
    output.publish(&next);
    view.poll();
    assert_eq!(view.preview.as_ref().unwrap().session, "new-session");
    assert_eq!(view.preview.as_ref().unwrap().sequence, 1);
}

#[test]
fn input_byte_limit_and_symlinks_are_refused() {
    let output = Output::new();
    let path = output.0.join("frame.json");
    std::fs::write(&path, vec![b' '; FRAME_BYTES as usize + 1]).unwrap();
    assert!(read_changed::<Preview>(&path, None, FRAME_BYTES).is_err());
    #[cfg(unix)]
    {
        std::fs::remove_file(&path).unwrap();
        std::fs::write(
            output.0.join("actual.json"),
            serde_json::to_vec(&fixture(1)).unwrap(),
        )
        .unwrap();
        std::os::unix::fs::symlink(output.0.join("actual.json"), &path).unwrap();
        assert!(read_changed::<Preview>(&path, None, FRAME_BYTES).is_err());
    }
}

#[test]
fn many_updates_retain_only_one_dense_field_and_real_egui_draws_pressure_and_failure() {
    let output = Output::new();
    let mut view = LiveProject::new(output.0.join("frame.json"), None);
    for sequence in 0..80 {
        output.publish(&fixture(sequence));
        view.poll();
        assert_eq!(view.preview.as_ref().unwrap().payload_bytes(), 8 * 12);
    }
    let mut value = fixture(81);
    value["state"] = json!("failed");
    value["tracer"] = serde_json::Value::Null;
    output.publish(&value);
    view.poll();
    view.next_poll = Instant::now() + Duration::from_secs(60);
    let ctx = egui::Context::default();
    let rendered = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 900.0),
            )),
            ..Default::default()
        },
        |ctx| {
            assert!(view.show(ctx));
        },
    );
    let has = |expected: &str| {
        rendered.shapes.iter().any(|shape|
        matches!(&shape.shape, egui::Shape::Text(text) if text.galley.job.text.contains(expected)))
    };
    for text in [
        "Tracer not supplied",
        "Force metadata not supplied",
        "native rejection/fallback",
        "XY voxel field",
        "frame 16",
        "failed",
    ] {
        assert!(has(text), "Missing painted text {text}");
    }
    assert_eq!(color(-4.0, 4.0), egui::Color32::from_rgb(45, 105, 225));
    assert_eq!(color(4.0, 4.0), egui::Color32::from_rgb(225, 65, 65));
}

#[test]
#[ignore = "requires an explicitly retained current publisher frame fixture"]
fn current_pressure_only_publisher_contract_and_real_egui_paint() {
    let path = PathBuf::from(
        std::env::var("RHEON_LIVE_PREVIEW_FIXTURE").expect("Select exact retained current frame"),
    );
    let (frame, _) = read_changed::<Preview>(&path, None, FRAME_BYTES)
        .unwrap()
        .unwrap();
    frame.validate().unwrap();
    assert!(frame.pressure.is_some());
    assert!(
        frame.tracer.is_none(),
        "Current scientific capture has no tracer; never fabricate one"
    );
    assert!(frame.cubic());
    assert!(frame.source.as_ref().unwrap()["forcing_parameters"].is_object());
    assert!(
        frame.source.as_ref().unwrap()["sha256"]
            .as_str()
            .is_some_and(|s| s.len() == 64)
    );
    let mut view = LiveProject::new(path, None);
    view.preview = Some(frame);
    view.next_poll = Instant::now() + Duration::from_secs(60);
    let ctx = egui::Context::default();
    let output = ctx.run(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 900.0),
            )),
            ..Default::default()
        },
        |ctx| {
            assert!(view.show(ctx));
        },
    );
    assert!(!output.shapes.is_empty());
    assert!(
        output
            .shapes
            .iter()
            .any(|shape| matches!(&shape.shape, egui::Shape::Text(text)
        if text.galley.job.text.contains("Tracer not supplied")))
    );
    assert!(
        output
            .shapes
            .iter()
            .any(|shape| matches!(&shape.shape, egui::Shape::Text(text)
        if text.galley.job.text.contains("Declared source forcing parameters")))
    );
}
