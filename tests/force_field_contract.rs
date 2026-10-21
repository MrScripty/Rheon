//! Pure authoring/evaluator tests: no solver, filesystem, subprocess or workloads.
use rheon::*;
fn object(kind: ForceKind) -> ForceObject {
    ForceObject {
        id: SceneObjectId(1),
        name: "Field".into(),
        enabled: true,
        transform: ForceTransform::default(),
        units: ForceUnits::Acceleration,
        kind,
        support: ForceSupport::Global,
        time: ForceTime::default(),
    }
}
fn snapshot(objects: Vec<ForceObject>) -> ForceSnapshot {
    ForceScene {
        objects,
        ..Default::default()
    }
    .freeze()
    .unwrap()
}
fn sample(o: ForceObject, p: [f64; 3]) -> [f64; 3] {
    snapshot(vec![o]).acceleration(p, 0.0, 2.0).unwrap()
}
fn close(a: [f64; 3], b: [f64; 3]) {
    for d in 0..3 {
        assert!((a[d] - b[d]).abs() < 1e-12, "{a:?} != {b:?}");
    }
}
#[test]
fn directional_units_superposition_and_density() {
    let a = object(ForceKind::Directional {
        vector: [2.0, -4.0, 0.0],
    });
    let mut f = a.clone();
    f.id = SceneObjectId(2);
    f.units = ForceUnits::ForceDensity;
    let s = snapshot(vec![a, f]);
    assert_eq!(
        s.acceleration([0.0; 3], 0.0, 2.0).unwrap(),
        [3.0, -6.0, 0.0]
    );
    assert_eq!(
        s.acceleration([0.0; 3], 0.0, 4.0).unwrap(),
        [2.5, -5.0, 0.0]
    );
}
#[test]
fn translation_rotation_and_local_box_support() {
    let mut o = object(ForceKind::Directional {
        vector: [2.0, 0.0, 0.0],
    });
    o.transform.translation = [3.0, 4.0, 0.0];
    let h = 0.5_f64.sqrt();
    o.transform.rotation = [0.0, 0.0, h, h];
    o.support = ForceSupport::Box {
        half_extent: [2.0, 0.5, 1.0],
        edge_fraction: 0.0,
    };
    close(sample(o.clone(), [3.0, 5.0, 0.0]), [0.0, 2.0, 0.0]);
    assert_eq!(sample(o, [4.0, 4.0, 0.0]), [0.0; 3]);
}
#[test]
fn radial_sign_is_outward_and_centre_is_zero() {
    let mut o = object(ForceKind::Radial { strength: 10.0 });
    assert_eq!(sample(o.clone(), [3.0, 4.0, 0.0]), [6.0, 8.0, 0.0]);
    assert_eq!(sample(o.clone(), [0.0; 3]), [0.0; 3]);
    o.kind = ForceKind::Radial { strength: -10.0 };
    assert_eq!(sample(o, [3.0, 4.0, 0.0]), [-6.0, -8.0, 0.0]);
}
#[test]
fn vortex_handedness_axis_and_rotated_axis() {
    let mut o = object(ForceKind::Vortex { strength: 2.0 });
    assert_eq!(sample(o.clone(), [1.0, 0.0, 0.0]), [0.0, 0.0, -2.0]);
    assert_eq!(sample(o.clone(), [0.0, 10.0, 0.0]), [0.0; 3]);
    let h = 0.5_f64.sqrt();
    o.transform.rotation = [h, 0.0, 0.0, h];
    close(sample(o, [1.0, 0.0, 0.0]), [0.0, 2.0, 0.0]);
}
#[test]
fn support_boundaries_and_smoothstep_weights() {
    let mut o = object(ForceKind::Directional {
        vector: [8.0, 0.0, 0.0],
    });
    o.support = ForceSupport::Sphere {
        radius: 2.0,
        edge_fraction: 0.5,
    };
    assert_eq!(sample(o.clone(), [0.0; 3]), [8.0, 0.0, 0.0]);
    assert_eq!(sample(o.clone(), [1.5, 0.0, 0.0]), [4.0, 0.0, 0.0]);
    assert_eq!(sample(o.clone(), [2.0, 0.0, 0.0]), [0.0; 3]);
    o.support = ForceSupport::Box {
        half_extent: [2.0; 3],
        edge_fraction: 0.0,
    };
    assert_eq!(sample(o.clone(), [-2.0, 0.0, 0.0]), [8.0, 0.0, 0.0]);
    assert_eq!(sample(o.clone(), [2.0, 0.0, 0.0]), [0.0; 3]);
    o.support = ForceSupport::Box {
        half_extent: [2.0; 3],
        edge_fraction: 0.5,
    };
    assert_eq!(sample(o, [1.5, 0.0, 0.0]), [4.0, 0.0, 0.0]);
}
#[test]
fn enabled_and_half_open_time_use_static_samples() {
    let mut o = object(ForceKind::Radial { strength: 2.0 });
    o.time = ForceTime {
        start_seconds: 1.0,
        end_seconds: Some(2.0),
    };
    let s = snapshot(vec![o.clone()]);
    for t in [0.0, 0.999, 2.0, 3.0] {
        assert_eq!(s.acceleration([1.0, 0.0, 0.0], t, 1.0).unwrap(), [0.0; 3]);
    }
    assert_eq!(
        s.acceleration([1.0, 0.0, 0.0], 1.0, 1.0).unwrap(),
        [2.0, 0.0, 0.0]
    );
    o.enabled = false;
    assert_eq!(
        snapshot(vec![o])
            .acceleration([1.0, 0.0, 0.0], 1.5, 1.0)
            .unwrap(),
        [0.0; 3]
    );
}
#[test]
fn frozen_run_owns_original_identity_and_values() {
    let mut scene = ForceScene {
        objects: vec![object(ForceKind::Radial { strength: 2.0 })],
        ..Default::default()
    };
    let frozen = scene.freeze().unwrap();
    scene.objects[0].name = "Edited".into();
    scene.objects[0].enabled = false;
    assert_eq!(frozen.scene().objects[0].id, SceneObjectId(1));
    assert_eq!(frozen.scene().objects[0].name, "Field");
    assert_eq!(
        frozen.acceleration([1.0, 0.0, 0.0], 0.0, 1.0).unwrap(),
        [2.0, 0.0, 0.0]
    );
}
#[test]
fn invalid_schema_ids_and_count_are_rejected() {
    let o = object(ForceKind::Radial { strength: 1.0 });
    let mut s = ForceScene {
        objects: vec![o.clone()],
        ..Default::default()
    };
    s.version += 1;
    assert_eq!(s.freeze().unwrap_err(), ForceFieldError::UnsupportedVersion);
    s.version = FORCE_SCENE_VERSION;
    s.objects[0].id = SceneObjectId(0);
    assert_eq!(s.freeze().unwrap_err(), ForceFieldError::InvalidIdentity);
    s.objects = vec![o.clone(), o.clone()];
    assert_eq!(s.freeze().unwrap_err(), ForceFieldError::InvalidIdentity);
    s.objects = vec![o; MAX_FORCE_OBJECTS + 1];
    assert_eq!(s.freeze().unwrap_err(), ForceFieldError::TooManyObjects);
}
#[test]
fn invalid_disabled_objects_are_also_rejected() {
    let base = object(ForceKind::Radial { strength: 1.0 });
    let mut cases = vec![];
    let mut o = base.clone();
    o.transform.rotation = [0.0; 4];
    cases.push(o);
    let mut o = base.clone();
    o.transform.translation[0] = f64::NAN;
    cases.push(o);
    let mut o = base.clone();
    o.kind = ForceKind::Radial {
        strength: f64::INFINITY,
    };
    cases.push(o);
    let mut o = base.clone();
    o.time.end_seconds = Some(0.0);
    cases.push(o);
    let mut o = base.clone();
    o.time.start_seconds = -1.0;
    cases.push(o);
    let mut o = base.clone();
    o.support = ForceSupport::Sphere {
        radius: 0.0,
        edge_fraction: 0.0,
    };
    cases.push(o);
    let mut o = base.clone();
    o.support = ForceSupport::Box {
        half_extent: [1.0; 3],
        edge_fraction: 1.1,
    };
    cases.push(o);
    let mut o = base;
    o.name = "x".repeat(257);
    cases.push(o);
    for mut o in cases {
        o.enabled = false;
        assert_eq!(
            ForceScene {
                objects: vec![o],
                ..Default::default()
            }
            .freeze()
            .unwrap_err(),
            ForceFieldError::InvalidObject
        );
    }
}
#[test]
fn nonfinite_inputs_overflow_and_tiny_distance_are_safe() {
    let s = snapshot(vec![object(ForceKind::Radial { strength: 2.0 })]);
    for rho in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert_eq!(
            s.acceleration([0.0; 3], 0.0, rho).unwrap_err(),
            ForceFieldError::InvalidSample
        );
    }
    assert_eq!(
        s.acceleration([f64::NAN; 3], 0.0, 1.0).unwrap_err(),
        ForceFieldError::InvalidSample
    );
    assert_eq!(
        s.acceleration([0.0; 3], -1.0, 1.0).unwrap_err(),
        ForceFieldError::InvalidSample
    );
    assert_eq!(
        s.acceleration([1e-300, 0.0, 0.0], 0.0, 1.0).unwrap(),
        [2.0, 0.0, 0.0]
    );
    let mut o = object(ForceKind::Directional {
        vector: [f64::MAX, 0.0, 0.0],
    });
    o.units = ForceUnits::ForceDensity;
    assert_eq!(
        snapshot(vec![o])
            .acceleration([0.0; 3], 0.0, 0.5)
            .unwrap_err(),
        ForceFieldError::ArithmeticFailure
    );
}
#[cfg(feature = "force-serde")]
#[test]
fn serialization_roundtrip_preserves_ids_and_requires_validation() {
    let scene = snapshot(vec![object(ForceKind::Vortex { strength: -3.0 })])
        .scene()
        .clone();
    let json = serde_json::to_string(&scene).unwrap();
    let decoded: ForceScene = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, scene);
    decoded.freeze().unwrap();
    let unsupported = json.replace("\"version\":1", "\"version\":99");
    assert_eq!(
        serde_json::from_str::<ForceScene>(&unsupported)
            .unwrap()
            .freeze()
            .unwrap_err(),
        ForceFieldError::UnsupportedVersion
    );
    assert!(
        serde_json::from_str::<ForceScene>(
            &json.replace("\"version\":1", "\"scale\":2,\"version\":1")
        )
        .is_err()
    );
}

#[test]
fn radial_and_vortex_normalization_handles_subnormal_and_huge_vectors() {
    let tiny = f64::from_bits(1);
    let diagonal = 2.0_f64.sqrt();
    close(
        sample(
            object(ForceKind::Radial { strength: 2.0 }),
            [tiny, tiny, 0.0],
        ),
        [diagonal, diagonal, 0.0],
    );
    close(
        sample(
            object(ForceKind::Radial { strength: 2.0 }),
            [f64::MAX, f64::MAX, 0.0],
        ),
        [diagonal, diagonal, 0.0],
    );
    close(
        sample(
            object(ForceKind::Vortex { strength: 2.0 }),
            [tiny, 0.0, tiny],
        ),
        [diagonal, 0.0, -diagonal],
    );
}
