use rheon_solver::{CycleSimulation, DT};

const METHODS: [&str; 2] = ["jacobi-pcg-v1", "sgs-pcg-v1"];

#[test]
fn accepted_state_matches_independent_cycle_projection_and_energy() {
    // In the four interior-face basis, div-free c=(1,-1,-1,1).
    // Forced u=(1/2,0,0,0), so c*(u.c)/(c.c)=c/8.
    for method in METHODS {
        let mut sim = CycleSimulation::new(method).unwrap();
        let outcome = sim.advance(DT, false);
        assert_eq!(outcome.status, "accepted");
        let state = outcome.state;
        assert_eq!(state.x, [0., 0.125, 0., 0., -0.125, 0.]);
        assert_eq!(state.y, [0., 0., -0.125, 0.125, 0., 0.]);
        assert_eq!(state.z, [0.; 8]);
        assert_eq!(state.tracer, [0.; 4]);
        // All four cells have exactly zero divergence, including wall faces.
        for j in 0..2 {
            for i in 0..2 {
                assert_eq!(
                    state.x[i + 1 + 3 * j] - state.x[i + 3 * j] + state.y[i + 2 * (j + 1)]
                        - state.y[i + 2 * j],
                    0.
                );
            }
        }
        let report = outcome.report.unwrap();
        let force = report.forces.unwrap();
        assert_eq!(force.kinetic_energy_before, 0.);
        assert_eq!(force.kinetic_energy_after, 0.125);
        assert_eq!(force.applied_work, 0.125);
        assert_eq!(report.kinetic_energy, 0.03125);
        assert_eq!(report.courant, 0.0625);
        assert_eq!(report.actual_divergence_max, 0.);
        assert_eq!(report.tracer_integral, 0.);
        assert_eq!(state.time, DT);
        assert_eq!(state.generation, 1);
        assert!(state.accepted_since_reset);
    }
}

#[test]
fn cancelled_zero_state_is_bit_identical_and_retry_matches_uninterrupted() {
    for method in METHODS {
        let mut sim = CycleSimulation::new(method).unwrap();
        let initial = sim.state();
        let cancelled = sim.advance(DT, true);
        assert_eq!(cancelled.code, "cancelled_before_commit");
        assert!(cancelled.report.is_none());
        assert_eq!(cancelled.state, initial);
        let actual = sim.advance(DT, false);
        let expected = CycleSimulation::new(method).unwrap().advance(DT, false);
        assert_eq!(actual.state, expected.state);
        assert_eq!(
            serde_json::to_value(actual.report).unwrap(),
            serde_json::to_value(expected.report).unwrap()
        );
    }
}

#[test]
fn cancelled_nonzero_candidate_preserves_accepted_arrays_time_and_generation() {
    for method in METHODS {
        let mut sim = CycleSimulation::new(method).unwrap();
        let accepted = sim.advance(DT, false).state;
        for _ in 0..2 {
            let cancelled = sim.advance(DT, true);
            assert_eq!(cancelled.code, "cancelled_before_commit");
            assert_eq!(cancelled.state, accepted);
            assert!(cancelled.report.is_none());
        }
        let repeated = sim.advance(DT, false);
        assert_eq!(repeated.code, "reset_required");
        assert_eq!(repeated.state, accepted);
    }
}

#[test]
fn invalid_parameters_cannot_mutate_either_zero_or_nonzero_state() {
    assert!(CycleSimulation::new("other").is_err());
    for method in METHODS {
        let mut sim = CycleSimulation::new(method).unwrap();
        for accepted in [false, true] {
            if accepted {
                assert_eq!(sim.advance(DT, false).status, "accepted");
            }
            let before = sim.state();
            for dt in [
                0.,
                -0.25,
                0.125,
                0.5,
                f64::NAN,
                f64::INFINITY,
                f64::NEG_INFINITY,
            ] {
                for cancel in [false, true] {
                    let invalid = sim.advance(dt, cancel);
                    assert_eq!(invalid.code, "invalid_time_step");
                    assert_eq!(invalid.state, before);
                }
            }
        }
    }
}

#[test]
fn reset_preserves_native_generation_semantics_and_allows_exact_retry() {
    for method in METHODS {
        let mut sim = CycleSimulation::new(method).unwrap();
        let first = sim.advance(DT, false).state;
        let reset = sim.reset_state().unwrap();
        assert_eq!(reset.generation, 2);
        assert_eq!(reset.generation_u64, "2");
        assert_eq!(reset.time_bits, "0000000000000000");
        assert!(!reset.accepted_since_reset);
        assert!(
            reset
                .bits
                .x
                .iter()
                .chain(&reset.bits.y)
                .chain(&reset.bits.z)
                .chain(&reset.bits.tracer)
                .all(|b| *b == 0)
        );
        assert_eq!(sim.advance(DT, true).state, reset);
        let retry = sim.advance(DT, false).state;
        assert_eq!(retry.generation, 3);
        assert_eq!(retry.time_bits, first.time_bits);
        assert_eq!(retry.bits, first.bits);
    }
}

#[test]
fn json_facade_and_native_reference_expose_identical_wrapper_results() {
    let reference = rheon_solver::reference().unwrap();
    assert_eq!(reference.runs.len(), 2);
    for run in reference.runs {
        let mut sim = CycleSimulation::new(run.method).unwrap();
        assert_eq!(
            sim.snapshot(),
            serde_json::to_string(&run.baseline).unwrap()
        );
        for (dt, cancel, expected) in [
            (0., false, run.invalid_dt),
            (DT, true, run.cancelled_rest),
            (DT, false, run.accepted),
            (DT, true, run.cancelled_nonzero),
            (DT, false, run.second_accept),
        ] {
            assert_eq!(
                sim.step(dt, cancel),
                serde_json::to_string(&expected).unwrap()
            );
        }
        assert_eq!(
            sim.reset().unwrap(),
            serde_json::to_string(&run.reset).unwrap()
        );
        assert_eq!(
            sim.step(DT, false),
            serde_json::to_string(&run.after_reset).unwrap()
        );
    }
}
