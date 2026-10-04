use rheon::{Axis, GridGeometry, SamplingError, ScalarSampler, VelocitySampler};
fn field(g: &GridGeometry, axis: Axis, f: impl Fn([f64; 3]) -> f32) -> Vec<f32> {
    let dims = g.face_counts(axis);
    let mut v = Vec::new();
    for z in 0..dims[2] {
        for y in 0..dims[1] {
            for x in 0..dims[0] {
                v.push(f(g.face_position(axis, [x, y, z]).unwrap()));
            }
        }
    }
    v
}
#[test]
fn component_offsets_reproduce_affine_fields() {
    let g = GridGeometry::new([4, 5, 6], [0.5, 0.25, 0.125], [-1.0, -0.5, 0.0]).unwrap();
    let f = |p: [f64; 3]| (p[0] + 2.0 * p[1] - 3.0 * p[2] + 2.0) as f32;
    for axis in [Axis::X, Axis::Y, Axis::Z] {
        let values = field(&g, axis, f);
        let sampler = ScalarSampler::faces(&g, axis, &values).unwrap();
        let p = [-0.2, 0.2, 0.4];
        assert!(
            (sampler.sample(p).unwrap() - (p[0] + 2.0 * p[1] - 3.0 * p[2] + 2.0)).abs() < 1e-12
        );
    }
}
#[test]
fn convex_bounds_and_singleton_axes_hold() {
    let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let s = ScalarSampler::cells(&g, &[-2.0, 8.0]).unwrap();
    assert_eq!(s.sample([1.0, 0.5, 0.5]).unwrap(), 3.0);
    assert_eq!(s.sample([-f64::MAX; 3]).unwrap(), -2.0);
    assert_eq!(s.sample([f64::MAX; 3]).unwrap(), 8.0);
    assert_eq!(
        s.sample([f64::NAN; 3]),
        Err(SamplingError::NonFinitePosition)
    );
}
#[test]
fn midpoint_backtrace_matches_constant_translation() {
    let g = GridGeometry::new([8, 8, 8], [1.0; 3], [0.0; 3]).unwrap();
    let u = field(&g, Axis::X, |_| 1.0);
    let v = field(&g, Axis::Y, |_| -2.0);
    let w = field(&g, Axis::Z, |_| 0.5);
    let s = VelocitySampler::new(&g, [&u, &v, &w]).unwrap();
    assert_eq!(s.backtrace([4.0; 3], 0.5).unwrap(), [3.5, 5.0, 3.75]);
    assert_eq!(s.backtrace([0.1, 7.9, 0.1], 0.5).unwrap(), [0.0, 8.0, 0.0]);
    assert_eq!(
        s.backtrace([4.0; 3], 0.0),
        Err(SamplingError::InvalidTimeStep)
    );
}
#[test]
fn midpoint_rotation_matches_independent_polynomial() {
    let g = GridGeometry::new([8, 8, 1], [1.0; 3], [-4.0, -4.0, 0.0]).unwrap();
    let u = field(&g, Axis::X, |p| -p[1] as f32);
    let v = field(&g, Axis::Y, |p| p[0] as f32);
    let w = field(&g, Axis::Z, |_| 0.0);
    let s = VelocitySampler::new(&g, [&u, &v, &w]).unwrap();
    let dt = 0.1;
    let actual = s.backtrace([1.0, 0.0, 0.5], dt).unwrap();
    assert!((actual[0] - (1.0 - dt * dt / 2.0)).abs() < 1e-14);
    assert!((actual[1] + dt).abs() < 1e-14);
}
#[test]
fn invalid_input_is_rejected_at_sampler_construction() {
    let g = GridGeometry::new([1, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    assert!(matches!(
        ScalarSampler::cells(&g, &[]),
        Err(SamplingError::LengthMismatch)
    ));
    assert!(matches!(
        ScalarSampler::cells(&g, &[f32::INFINITY]),
        Err(SamplingError::NonFiniteField)
    ));
}
