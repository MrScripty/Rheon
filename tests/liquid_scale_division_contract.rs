use rheon::{
    Axis, GridGeometry, LiquidFlowInterval, LiquidInlet, LiquidVolumeError, LiquidVolumeSettings,
    LiquidVolumeSource, LiquidVolumeState, VolumeStage, VolumeStamp,
};

#[test]
fn unsupported_candidate_courant_and_divergence_quotients_preserve_accepted_state() {
    let cases = [
        ([2.0, 1.0, 1.0], 1.0, [0.0, 0.0], f64::MIN_POSITIVE),
        (
            [2.0_f64.powi(1000), 1.0, 1.0],
            1.0,
            [0.0, 0.0],
            2.0_f64.powi(-100),
        ),
        (
            [2.0 * f64::MIN_POSITIVE, 1.0, 1.0],
            2.0_f64.powi(100),
            [1.0, 1.0],
            0.0,
        ),
        (
            [1e303, 1.0, 1.0],
            1.0,
            [1.0, f32::from_bits(1.0_f32.to_bits() + 1)],
            0.0,
        ),
    ];
    // Respectively: subnormal fraction, nonzero fraction rounded to zero,
    // overflowing Courant quotient, subnormal divergence quotient. Products
    // before the cell loop are supported; rejection must reach that loop.
    for (h, dt, x, rate) in cases {
        let grid = GridGeometry::new([1; 3], h, [0.0; 3]).unwrap();
        let stamp = VolumeStamp { id: 1, version: 0 };
        let mut state = LiquidVolumeState::new(grid.clone(), 1.0, stamp, vec![0.0], 64).unwrap();
        let velocity = [
            x.to_vec(),
            vec![0.0; grid.face_len(Axis::Y)],
            vec![0.0; grid.face_len(Axis::Z)],
        ];
        let flow = LiquidFlowInterval::new(
            &grid,
            VolumeStamp { id: 2, version: 0 },
            [&velocity[0], &velocity[1], &velocity[2]],
            0.0,
            dt,
        )
        .unwrap();
        let rates = [rate];
        let source = LiquidVolumeSource::new(VolumeStamp { id: 3, version: 0 }, &rates).unwrap();
        let inlet = LiquidInlet::new(VolumeStamp { id: 4, version: 0 }, [[0.0; 2]; 3]).unwrap();
        for _ in 0..2 {
            let mut reached_cell = false;
            let error = state
                .advance(
                    flow,
                    inlet,
                    Some(source),
                    LiquidVolumeSettings {
                        max_outward_courant: 1.0,
                        actual_divergence_limit: 1e-5,
                    },
                    |stage| {
                        reached_cell |= stage == VolumeStage::CellUpdateSlice;
                        false
                    },
                )
                .unwrap_err();
            assert!(reached_cell);
            assert_eq!(error, LiquidVolumeError::ArithmeticFailure);
            assert_eq!(state.state().fraction[0].to_bits(), 0.0_f64.to_bits());
            assert_eq!(state.state().time.to_bits(), 0.0_f64.to_bits());
            assert_eq!(state.state().stamp, stamp);
        }
    }
}
