use rheon::{
    AdvectionError, Axis, GridGeometry, SamplingError, ScalarSampler, VelocitySampler,
    advect_tracer, advect_velocity,
};

#[test]
fn skipped_tracer_regions_match_dense_rk2_bits_with_clamping_and_tiny_donors() {
    let g = GridGeometry::new([41, 29, 17], [0.07, 0.11, 0.19], [-2.0, 4.0, 0.3]).unwrap();
    let velocity: [Vec<f32>; 3] = std::array::from_fn(|d| {
        (0..g.face_len([Axis::X, Axis::Y, Axis::Z][d]))
            .map(|i| (0.8 * (i as f64 * 0.017 + d as f64).sin()) as f32)
            .collect()
    });
    let fields = std::array::from_fn(|d| velocity[d].as_slice());
    let [nx, ny, nz] = g.counts();
    let mut old = vec![0.0; g.cell_len()];
    old[nx * (8 + ny * 5)] = 1.0;
    old[nx - 1 + nx * (9 + ny * 6)] = -0.5;
    old[18 + nx * (9 + ny * 6)] = f32::from_bits(1);
    let scalar = ScalarSampler::cells(&g, &old).unwrap();
    let sampler = VelocitySampler::new(&g, fields).unwrap();
    for dt in [0.01, 0.5, 3.0] {
        let mut actual = vec![f32::NAN; g.cell_len()];
        advect_tracer(&g, &old, fields, dt, &mut actual, || false).unwrap();
        for z in 0..nz {
            for y in 0..ny {
                for x in 0..nx {
                    let position = g.cell_position([x, y, z]).unwrap();
                    let expected = scalar
                        .sample(sampler.backtrace(position, dt).unwrap())
                        .unwrap() as f32;
                    assert_eq!(
                        actual[x + nx * (y + ny * z)].to_bits(),
                        expected.to_bits(),
                        "{x},{y},{z} dt={dt}"
                    );
                }
            }
        }
    }
}

#[test]
fn empty_tracer_fast_path_preserves_validation_and_cancellation() {
    let g = GridGeometry::new([8; 3], [0.125; 3], [0.0; 3]).unwrap();
    let mut u = vec![1.0; g.face_len(Axis::X)];
    let v = vec![0.0; g.face_len(Axis::Y)];
    let w = vec![0.0; g.face_len(Axis::Z)];
    let old = vec![-0.0; g.cell_len()];
    let mut out = vec![99.0; g.cell_len()];
    advect_tracer(&g, &old, [&u, &v, &w], 0.1, &mut out, || false).unwrap();
    assert!(out.iter().all(|v| v.to_bits() == 0));
    assert_eq!(
        advect_tracer(&g, &old, [&u, &v, &w], 0.1, &mut out, || true),
        Err(AdvectionError::Cancelled)
    );
    u[0] = f32::NAN;
    assert_eq!(
        advect_tracer(&g, &old, [&u, &v, &w], 0.1, &mut out, || false),
        Err(AdvectionError::Sampling(SamplingError::NonFiniteField))
    );
    u.fill(f32::MAX);
    assert_eq!(
        advect_tracer(&g, &old, [&u, &v, &w], f64::MAX, &mut out, || false),
        Err(AdvectionError::Sampling(SamplingError::ArithmeticFailure))
    );
}
#[test]
fn constant_translation_advects_an_interior_pulse_one_cell() {
    let g = GridGeometry::new([8, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let u = vec![1.0; g.face_len(Axis::X)];
    let v = vec![0.0; g.face_len(Axis::Y)];
    let w = vec![0.0; g.face_len(Axis::Z)];
    let old = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let mut out = [-1.0; 8];
    advect_tracer(&g, &old, [&u, &v, &w], 1.0, &mut out, || false).unwrap();
    assert_eq!(out, [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0]);
}
#[test]
fn rest_state_is_preserved_on_degenerate_grid() {
    let g = GridGeometry::new([1, 1, 1], [0.5, 0.25, 0.125], [0.0; 3]).unwrap();
    let old = [0.0; 2];
    let (mut u, mut v, mut w) = ([1.0; 2], [1.0; 2], [1.0; 2]);
    advect_velocity(
        &g,
        [&old, &old, &old],
        0.1,
        [&mut u, &mut v, &mut w],
        || false,
    )
    .unwrap();
    assert_eq!([u, v, w], [[0.0; 2]; 3]);
    let mut tracer = [0.0];
    advect_tracer(&g, &[0.37], [&u, &v, &w], 0.1, &mut tracer, || false).unwrap();
    assert_eq!(tracer, [0.37]);
}
#[test]
fn cancellation_and_output_shape_are_explicit() {
    let g = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let u = [0.0; 3];
    let v = [0.0; 4];
    let w = [0.0; 4];
    let mut out = [-1.0; 2];
    assert_eq!(
        advect_tracer(&g, &[1.0; 2], [&u, &v, &w], 0.1, &mut out, || true),
        Err(AdvectionError::Cancelled)
    );
    assert_eq!(out, [-1.0; 2]);
    assert_eq!(
        advect_tracer(&g, &[1.0; 2], [&u, &v, &w], 0.1, &mut [], || false),
        Err(AdvectionError::OutputLengthMismatch)
    );
}
