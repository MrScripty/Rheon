use rheon_solver::flow::{FlowSimulation, FlowSnapshot};

const METHODS: [&str; 2] = ["jacobi-pcg-v1", "sgs-pcg-v1"];

fn close(a: f64, b: f64, atol: f64) {
    assert!((a - b).abs() <= atol, "{a:.17e} != {b:.17e}");
}

// Separate equations and indexing from the adapter's diagnostic calculation.
fn independent_measures(s: &FlowSnapshot, dt: f64) -> (f64, f64, f64, f64) {
    let n = 24;
    let mut maximum_divergence = 0.0_f64;
    for z in 0..n {
        for y in 0..n {
            for x in 0..n {
                let a = x + 25 * (y + n * z);
                let b = x + n * (y + 25 * z);
                let c = x + n * (y + n * z);
                let div =
                    ((s.x[a + 1] - s.x[a]) + (s.y[b + n] - s.y[b]) + (s.z[c + n * n] - s.z[c]))
                        * 24.0;
                maximum_divergence = maximum_divergence.max(div.abs());
            }
        }
    }
    let velocity = [&s.x, &s.y, &s.z];
    let energy = 0.6 / 24.0_f64.powi(3)
        * velocity
            .iter()
            .flat_map(|v| v.iter())
            .map(|v| v * v)
            .sum::<f64>();
    let tracer = s.tracer.iter().sum::<f64>() / 24.0_f64.powi(3);
    let courant = dt
        * 24.0
        * velocity
            .iter()
            .map(|v| v.iter().fold(0.0_f64, |m, x| m.max(x.abs())))
            .sum::<f64>();
    (maximum_divergence, energy, tracer, courant)
}

fn check_fields(s: &FlowSnapshot) {
    assert_eq!(s.grid.counts, [24; 3]);
    for (values, bits, length) in [
        (&s.x, &s.bits.x, 14400),
        (&s.y, &s.bits.y, 14400),
        (&s.z, &s.bits.z, 14400),
        (&s.tracer, &s.bits.tracer, 13824),
    ] {
        assert_eq!(values.len(), length);
        assert_eq!(bits.len(), length);
        for (&value, &bits) in values.iter().zip(bits) {
            assert!(value.is_finite());
            assert_eq!(value, f64::from(f32::from_bits(bits)));
        }
    }
    assert!(s.tracer.iter().all(|x| (0.0..=1.0).contains(x)));
    // Prescribed wall-normal faces remain exactly zero on every outer wall.
    for j in 0..24 {
        for i in 0..24 {
            assert_eq!(s.x[25 * (i + 24 * j)], 0.0);
            assert_eq!(s.x[24 + 25 * (i + 24 * j)], 0.0);
            assert_eq!(s.y[i + 24 * 25 * j], 0.0);
            assert_eq!(s.y[i + 24 * (24 + 25 * j)], 0.0);
            assert_eq!(s.z[i + 24 * j], 0.0);
            assert_eq!(s.z[i + 24 * (j + 24 * 24)], 0.0);
        }
    }
}

#[test]
fn flow_construction_and_snapshot_do_not_advance_or_invent_fields() {
    assert!(FlowSimulation::new("unsupported").is_err());
    for method in METHODS {
        let flow = FlowSimulation::new(method).unwrap();
        let state = flow.state();
        check_fields(&state);
        assert_eq!(state.accepted_steps, 0);
        assert_eq!(state.max_steps, 80);
        assert_eq!(state.source_steps, 30);
        assert_eq!(state.generation, 0);
        assert_eq!(state.time_bits, "0000000000000000");
        assert!(state.next_source_active);
        assert!(state.metrics.tracer_centroid.is_none());
        assert!(state.tracer_integral_at_source_cutoff.is_none());
        assert_eq!(flow.snapshot(), serde_json::to_string(&state).unwrap());
        assert_eq!(flow.state(), state);
    }
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "bounded 24-cubed native transaction; run this contract in release"
)]
fn cancellation_and_retry_preserve_fields_metadata_and_source_schedule() {
    for method in METHODS {
        let mut flow = FlowSimulation::new(method).unwrap();
        let baseline = flow.state();
        let cancel = flow.advance(true);
        assert_eq!(cancel.code, "cancelled_before_commit");
        assert_eq!(cancel.state, baseline);
        let accepted = flow.advance(false);
        assert_eq!(accepted.status, "accepted");
        let cancel = flow.advance(true);
        assert_eq!(cancel.code, "cancelled_before_commit");
        assert_eq!(cancel.state, accepted.state);
        let retried = flow.advance(false);
        assert_eq!(retried.status, "accepted");
        let mut uninterrupted = FlowSimulation::new(method).unwrap();
        uninterrupted.advance(false);
        let expected = uninterrupted.advance(false);
        assert_eq!(retried.state, expected.state);
        assert_eq!(
            serde_json::to_string(&retried.report).unwrap(),
            serde_json::to_string(&expected.report).unwrap()
        );
    }
}

#[test]
#[cfg_attr(
    debug_assertions,
    ignore = "bounded 160 accepted 24-cubed steps; run this contract in release"
)]
fn complete_bounded_sequence_has_independent_gates_cutoff_cap_and_reset() {
    for method in METHODS {
        let mut flow = FlowSimulation::new(method).unwrap();
        let mut first = None;
        let mut time = 0.0;
        let mut cutoff = None;
        let mut reduced = false;
        for step in 1..=80 {
            let outcome = flow.advance(false);
            assert_eq!(
                outcome.status, "accepted",
                "{method}, step{step}: {:?}",
                outcome.detail
            );
            let s = outcome.state;
            let r = outcome.report.unwrap();
            check_fields(&s);
            let (div, energy, tracer, courant) = independent_measures(&s, r.dt);
            close(div, r.actual_divergence_max, 1e-12);
            close(div, s.metrics.actual_divergence_max, 1e-12);
            close(energy, r.kinetic_energy, 1e-12);
            close(
                energy,
                s.metrics.component_kinetic_energy.iter().sum(),
                1e-12,
            );
            close(tracer, r.tracer_integral, 1e-12);
            close(tracer, s.metrics.tracer_integral, 1e-12);
            close(courant, r.courant, 1e-12);
            assert!(div <= 1e-5 && courant <= 1.0);
            assert!(r.dt > 0.0 && r.dt <= 0.02);
            reduced |= r.dt < 0.02;
            time += r.dt;
            assert_eq!(s.time_bits, format!("{:016x}", time.to_bits()));
            assert_eq!(s.accepted_steps, step);
            assert_eq!(s.generation, step as u64);
            assert_eq!(s.next_source_active, step < 30);
            if step == 30 {
                cutoff = Some(r.tracer_integral);
            }
            assert_eq!(s.tracer_integral_at_source_cutoff, cutoff);
            if step >= 10 {
                assert!(s.metrics.component_kinetic_energy.iter().all(|v| *v > 1e-5));
                assert!(
                    s.metrics
                        .tracer_adjacent_slice_rms
                        .iter()
                        .all(|v| *v > 1e-3)
                );
            }
            if step == 1 {
                first = Some(s);
            }
        }
        assert!(
            reduced,
            "native policy must expose shortened dt in this run"
        );
        let last = flow.state();
        assert!(last.metrics.tracer_outside_source_fraction.unwrap() > 0.9);
        // This explicitly catches an accidental mass-conservation claim: the
        // transported appearance tracer loses integral after injection ends.
        assert!(last.metrics.tracer_integral < 0.95 * cutoff.unwrap());
        for cancel in [false, true] {
            let limited = flow.advance(cancel);
            assert_eq!(limited.code, "step_limit");
            assert_eq!(limited.state, last);
        }
        let reset = flow.reset_state().unwrap();
        check_fields(&reset);
        assert_eq!(reset.generation, 81);
        assert_eq!(reset.accepted_steps, 0);
        assert_eq!(reset.time, 0.0);
        assert!(reset.last_accepted_dt.is_none());
        assert!(reset.tracer_integral_at_source_cutoff.is_none());
        let retry = flow.advance(false);
        let first = first.unwrap();
        assert_eq!(retry.state.generation, 82);
        assert_eq!(retry.state.bits, first.bits);
        assert_eq!(retry.state.time_bits, first.time_bits);
    }
}
