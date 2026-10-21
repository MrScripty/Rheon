use rheon::{
    Axis, GridGeometry, LiquidFlowInterval, LiquidInlet, LiquidVolumeError, LiquidVolumeSettings,
    LiquidVolumeSource, LiquidVolumeState, VolumeStage, VolumeStamp,
};
fn stamp(id: u64) -> VolumeStamp {
    VolumeStamp { id, version: 0 }
}
fn settings() -> LiquidVolumeSettings {
    LiquidVolumeSettings {
        max_outward_courant: 1.0,
        actual_divergence_limit: 0.0,
    }
}
fn fields(g: &GridGeometry, u: [f32; 3]) -> [Vec<f32>; 3] {
    std::array::from_fn(|d| vec![u[d]; g.face_len([Axis::X, Axis::Y, Axis::Z][d])])
}
#[test]
fn periodic_matches_independent_multiaxis_upwind_and_retains_boundary_identity() {
    let g = GridGeometry::new([2, 2, 2], [0.5; 3], [0.0; 3]).unwrap();
    let initial = vec![0.0, 0.25, 0.5, 1.0, 0.75, 0.125, 0.625, 0.875];
    let mut state =
        LiquidVolumeState::new(g.clone(), 998.2, stamp(1), initial.clone(), 1 << 20).unwrap();
    let u = fields(&g, [0.25, -0.125, 0.0625]);
    let dt = 0.25;
    let flow = LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, dt).unwrap();
    let r = state
        .advance_periodic(flow, stamp(9), None, settings(), |_| false)
        .unwrap();
    for k in 0..2 {
        for j in 0..2 {
            for i in 0..2 {
                let p = [i, j, k];
                let idx = i + 2 * j + 4 * k;
                let mut exact = initial[idx];
                for d in 0..3 {
                    let mut donor = p;
                    donor[d] = 1 - p[d];
                    let q = donor[0] + 2 * donor[1] + 4 * donor[2];
                    let c = dt * f64::from([0.25_f32, -0.125, 0.0625][d]).abs() / 0.5;
                    exact += c * (initial[q] - initial[idx]);
                }
                assert!((state.state().fraction[idx] - exact).abs() < 1e-14);
            }
        }
    }
    assert!(r.periodic);
    assert_eq!(r.inlet, stamp(9));
    assert_eq!(r.inward_boundary_volume, 0.0);
    assert_eq!(r.outward_boundary_volume, 0.0);
    assert!(r.volume_balance_error.abs() < 1e-14);
}
#[test]
fn one_cell_periodic_seams_cancel_and_source_is_not_boundary_flux() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut state = LiquidVolumeState::new(g.clone(), 2.0, stamp(1), vec![0.25], 1 << 20).unwrap();
    let u = fields(&g, [-0.5, 0.25, -0.125]);
    let flow = LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, 0.25).unwrap();
    let rate = [0.5];
    let source = LiquidVolumeSource::new(stamp(3), &rate).unwrap();
    let r = state
        .advance_periodic(flow, stamp(4), Some(source), settings(), |_| false)
        .unwrap();
    assert_eq!(state.state().fraction, [0.375]);
    assert_eq!(r.source_volume, 0.125);
    assert_eq!(r.volume_balance_error, 0.0);
    assert_eq!(r.inward_boundary_volume + r.outward_boundary_volume, 0.0);
}
#[test]
fn seam_mismatch_and_every_cancellation_stage_preserve_accepted_state() {
    let g = GridGeometry::new([2; 3], [0.5; 3], [0.0; 3]).unwrap();
    let mut state =
        LiquidVolumeState::new(g.clone(), 998.2, stamp(1), vec![0.5; 8], 1 << 20).unwrap();
    let before = state.state().fraction.to_vec();
    let mut u = fields(&g, [0.25, 0.0, 0.0]);
    u[0][g.face_index(Axis::X, [2, 0, 0]).unwrap()] = 0.125;
    let flow = LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, 0.25).unwrap();
    assert!(matches!(
        state.advance_periodic(flow, stamp(3), None, settings(), |_| false),
        Err(LiquidVolumeError::PeriodicVelocityMismatch { axis: Axis::X, .. })
    ));
    assert_eq!(state.state().fraction, before);
    assert_eq!(state.state().time, 0.0);
    u[0].fill(0.25);
    for stage in [
        VolumeStage::BeforeFlux,
        VolumeStage::FaceFluxSlice,
        VolumeStage::CellUpdateSlice,
        VolumeStage::BeforeCommit,
    ] {
        let flow = LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, 0.25).unwrap();
        assert!(
            matches!(state.advance_periodic(flow,stamp(3),None,settings(),|s|s==stage),Err(LiquidVolumeError::Cancelled{stage:s}) if s==stage)
        );
        assert_eq!(state.state().fraction, before);
        assert_eq!(state.state().time, 0.0);
        assert_eq!(state.state().stamp, stamp(1));
    }
    let flow = LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, 10.0).unwrap();
    assert!(matches!(
        state.advance_periodic(flow, stamp(3), None, settings(), |_| false),
        Err(LiquidVolumeError::CourantLimit { .. })
    ));
    assert_eq!(state.state().fraction, before);
}
#[test]
fn original_box_policy_remains_explicitly_nonperiodic() {
    let g = GridGeometry::new([1; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut state = LiquidVolumeState::new(g.clone(), 1.0, stamp(1), vec![0.0], 1 << 20).unwrap();
    let u = fields(&g, [0.5, 0.0, 0.0]);
    let flow = LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, 0.25).unwrap();
    let inlet = LiquidInlet::new(stamp(3), [[1.0, 0.0], [0.0; 2], [0.0; 2]]).unwrap();
    let r = state
        .advance(flow, inlet, None, settings(), |_| false)
        .unwrap();
    assert!(!r.periodic);
    assert_eq!(r.inlet, stamp(3));
    assert_eq!(r.inward_boundary_volume, 0.125);
    assert_eq!(state.state().fraction, [0.125]);
}
#[test]
fn three_cell_impulse_distinguishes_each_periodic_donor_direction() {
    for d in 0..3 {
        for sign in [-1.0_f32, 1.0] {
            let mut counts = [1; 3];
            counts[d] = 3;
            let mut spacing = [1.0; 3];
            spacing[d] = 1.0 / 3.0;
            let g = GridGeometry::new(counts, spacing, [0.0; 3]).unwrap();
            let mut state =
                LiquidVolumeState::new(g.clone(), 1.0, stamp(1), vec![1.0, 0.0, 0.0], 1 << 20)
                    .unwrap();
            let mut speed = [0.0; 3];
            speed[d] = sign;
            let u = fields(&g, speed);
            let flow =
                LiquidFlowInterval::new(&g, stamp(2), [&u[0], &u[1], &u[2]], 0.0, spacing[d] * 0.5)
                    .unwrap();
            state
                .advance_periodic(flow, stamp(3), None, settings(), |_| false)
                .unwrap();
            let exact = if sign > 0.0 {
                [0.5, 0.5, 0.0]
            } else {
                [0.5, 0.0, 0.5]
            };
            for (a, b) in state.state().fraction.iter().zip(exact) {
                assert!((a - b).abs() < 1e-14);
            }
        }
    }
}
