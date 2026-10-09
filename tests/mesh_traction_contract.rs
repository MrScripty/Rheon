use rheon::{
    MeshLoadError, MeshLoadStage, SurfaceLoading, SurfaceSettings, SurfaceStamp, TriangleMeshLoad,
    TriangleSurface,
};
const STAMP: SurfaceStamp = SurfaceStamp { id: 17, version: 4 };
fn surface(scale: f64) -> TriangleSurface {
    TriangleSurface::new(
        STAMP,
        vec![[0.0; 3], [2.0 * scale, 0.0, 0.0], [0.0, scale, 0.0]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-12, "{a} != {b}");
}
#[test]
fn varying_traction_retains_corner_moments_and_virtual_work() {
    let s = surface(1.0);
    let traction = [[[0.0; 3], [0.0, 0.0, 3.0], [0.0; 3]]];
    let load = TriangleMeshLoad::new(
        &s,
        STAMP,
        SurfaceLoading::Traction(&traction),
        [0.0; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let t = load.triangle_load(0).unwrap();
    assert_eq!(t.area_m2, 1.0);
    assert_eq!(t.force, [0.0, 0.0, 1.0]);
    assert_eq!(
        t.nodal_force,
        [[0.0, 0.0, 0.25], [0.0, 0.0, 0.5], [0.0, 0.0, 0.25]]
    );
    assert_eq!(t.torque, [0.25, -1.0, 0.0]);
    let r = load
        .reduce([1.0, -2.0, 3.0], [4.0, 5.0, -6.0], |_, _| false)
        .unwrap();
    assert_eq!(r.rigid_power, -1.0);
    assert_eq!(r.nodal_power, -1.0);
    assert_eq!(r.power_defect, 0.0);
    assert_ne!(r.torque, [1.0 / 3.0, -2.0 / 3.0, 0.0]);
}
#[test]
fn oblique_pressure_has_declared_winding_and_three_dimensional_torque() {
    let s = TriangleSurface::new(
        STAMP,
        vec![[0.0; 3], [2.0, 0.0, 0.0], [0.0, 0.6, 0.8]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap();
    let p = [[1.0, 2.0, 4.0]];
    let load = TriangleMeshLoad::new(
        &s,
        STAMP,
        SurfaceLoading::Pressure(&p),
        [0.0; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let r = load
        .reduce([1.0, -2.0, 3.0], [4.0, 5.0, -6.0], |_, _| false)
        .unwrap();
    for (x, y) in r.force.into_iter().zip([0.0, 28.0 / 15.0, -7.0 / 5.0]) {
        near(x, y);
    }
    for (x, y) in r.torque.into_iter().zip([-11.0 / 12.0, 0.9, 1.2]) {
        near(x, y);
    }
    near(r.rigid_power, -14.3);
    near(r.nodal_power, -14.3);
    near(r.power_defect, 0.0);
}
#[test]
fn stamps_counts_and_work_caps_are_checked_before_scan() {
    let s = surface(1.0);
    let p = [[1.0; 3]];
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            SurfaceStamp { id: 17, version: 3 },
            SurfaceLoading::Pressure(&p),
            [0.0; 3],
            1,
            |_, _| panic!("must not scan")
        ),
        Err(MeshLoadError::StaleSurface)
    ));
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            STAMP,
            SurfaceLoading::Pressure(&[]),
            [0.0; 3],
            1,
            |_, _| panic!("must not scan")
        ),
        Err(MeshLoadError::LoadCount)
    ));
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            STAMP,
            SurfaceLoading::Pressure(&p),
            [0.0; 3],
            0,
            |_, _| panic!("must not scan")
        ),
        Err(MeshLoadError::TriangleLimit {
            required: 1,
            limit: 0
        })
    ));
}
#[test]
fn cancellation_polls_each_facet_and_leaves_inputs_reusable() {
    let s = TriangleSurface::new(
        STAMP,
        vec![[0.0; 3], [2.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        vec![[0, 1, 2]; 3],
        SurfaceSettings::default(),
    )
    .unwrap();
    let p = [[1.0; 3]; 3];
    let mut polls = Vec::new();
    let rejected = TriangleMeshLoad::new(
        &s,
        STAMP,
        SurfaceLoading::Pressure(&p),
        [0.0; 3],
        3,
        |stage, i| {
            polls.push((stage, i));
            i == 1
        },
    );
    assert!(matches!(
        rejected,
        Err(MeshLoadError::Cancelled {
            stage: MeshLoadStage::Admission,
            triangle: 1
        })
    ));
    assert_eq!(
        polls,
        vec![(MeshLoadStage::Admission, 0), (MeshLoadStage::Admission, 1)]
    );
    let load = TriangleMeshLoad::new(
        &s,
        STAMP,
        SurfaceLoading::Pressure(&p),
        [0.0; 3],
        3,
        |_, _| false,
    )
    .unwrap();
    assert!(matches!(
        load.reduce([0.0; 3], [0.0; 3], |_, i| i == 2),
        Err(MeshLoadError::Cancelled {
            stage: MeshLoadStage::Reduction,
            triangle: 2
        })
    ));
    assert_eq!(
        load.reduce([0.0; 3], [0.0; 3], |_, _| false).unwrap().force,
        [0.0, 0.0, -3.0]
    );
    assert_eq!(s.stamp(), STAMP);
    assert_eq!(p, [[1.0; 3]; 3]);
}
#[test]
fn unsupported_physical_area_refuses_even_if_surface_queries_admit() {
    for scale in [1e-200, 1e200] {
        let s = surface(scale);
        let p = [[1.0; 3]];
        assert!(matches!(
            TriangleMeshLoad::new(
                &s,
                STAMP,
                SurfaceLoading::Pressure(&p),
                [0.0; 3],
                1,
                |_, _| false
            ),
            Err(MeshLoadError::ArithmeticFailure { triangle: 0 })
        ));
    }
}
#[test]
fn nonfinite_data_and_overflow_refuse_without_changing_surface() {
    let s = surface(1.0);
    let p = [[f64::NAN, 0.0, 0.0]];
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            STAMP,
            SurfaceLoading::Pressure(&p),
            [0.0; 3],
            1,
            |_, _| false
        ),
        Err(MeshLoadError::NonFiniteLoad { triangle: 0 })
    ));
    let p = [[f64::MAX; 3]];
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            STAMP,
            SurfaceLoading::Pressure(&p),
            [0.0; 3],
            1,
            |_, _| false
        ),
        Err(MeshLoadError::ArithmeticFailure { triangle: 0 })
    ));
    let p = [[1.0; 3]];
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            STAMP,
            SurfaceLoading::Pressure(&p),
            [f64::INFINITY; 3],
            1,
            |_, _| false
        ),
        Err(MeshLoadError::InvalidReference)
    ));
    let load = TriangleMeshLoad::new(
        &s,
        STAMP,
        SurfaceLoading::Pressure(&p),
        [0.0; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    assert!(matches!(
        load.reduce([f64::NAN; 3], [0.0; 3], |_, _| false),
        Err(MeshLoadError::InvalidTwist)
    ));
    assert!(matches!(
        load.triangle_load(1),
        Err(MeshLoadError::InvalidTriangle)
    ));
    assert_eq!(s.vertices()[1], [2.0, 0.0, 0.0]);
}
#[test]
fn moment_and_virtual_velocity_overflow_fail_closed() {
    let s = surface(1.0);
    let p = [[10.0; 3]];
    assert!(matches!(
        TriangleMeshLoad::new(
            &s,
            STAMP,
            SurfaceLoading::Pressure(&p),
            [f64::MAX, 0.0, 0.0],
            1,
            |_, _| false
        ),
        Err(MeshLoadError::ArithmeticFailure { triangle: 0 })
    ));
    let load = TriangleMeshLoad::new(
        &s,
        STAMP,
        SurfaceLoading::Pressure(&p),
        [0.0; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    assert!(matches!(
        load.reduce([0.0; 3], [0.0, f64::MAX, 0.0], |_, _| false),
        Err(MeshLoadError::ArithmeticFailure { triangle: 0 })
    ));
}
