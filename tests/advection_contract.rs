use rheon::{AdvectionError, Axis, GridGeometry, advect_tracer, advect_velocity};
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
