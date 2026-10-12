#![cfg(feature = "parallel")]
use rheon::{
    GridGeometry, PressureImplementation, PressureSettings, Simulation, SimulationConfig,
    SmokeSource, StepReport, StepStage,
};

fn simulation(method: PressureImplementation) -> Simulation {
    // Unequal dimensions and a partial final task exercise chunk boundaries.
    Simulation::with_implementation(
        GridGeometry::new([40, 33, 27], [0.04, 0.05, 0.06], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 1.0,
            memory_limit: 8 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-9,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        method,
    )
    .unwrap()
}
fn source() -> Option<SmokeSource> {
    Some(SmokeSource {
        lower: [0.2, 0.1, 0.2],
        upper: [0.7, 0.4, 0.7],
        tracer_rate: 2.0,
        vertical_acceleration: 1.0,
    })
}
fn state(sim: &Simulation) -> Vec<u32> {
    let view = sim.state();
    view.x
        .iter()
        .chain(view.y)
        .chain(view.z)
        .chain(view.tracer)
        .map(|v| v.to_bits())
        .collect()
}
fn report_bits(r: StepReport) -> (usize, Vec<u64>) {
    (
        r.pressure.iterations,
        [
            r.dt,
            r.time,
            r.pressure.true_residual_l2,
            r.pressure.true_residual_max,
            r.pressure.predicted_divergence_max,
            r.actual_divergence_max,
            r.courant,
            r.tracer_integral,
            r.kinetic_energy,
        ]
        .map(f64::to_bits)
        .to_vec(),
    )
}
fn pool(threads: usize) -> rayon::ThreadPool {
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build()
        .unwrap()
}

#[test]
fn thread_counts_preserve_every_state_bit_and_diagnostic_for_both_methods() {
    for method in PressureImplementation::ALL {
        let expected = pool(1).install(|| {
            let mut sim = simulation(method);
            let first = sim.step(0.02, source(), |_| false).unwrap();
            let second = sim.step(0.02, None, |_| false).unwrap();
            (
                state(&sim),
                vec![report_bits(first), report_bits(second)],
                sim.allocated_bytes(),
            )
        });
        for threads in [2, 4] {
            let actual = pool(threads).install(|| {
                let mut sim = simulation(method);
                let first = sim.step(0.02, source(), |_| false).unwrap();
                let second = sim.step(0.02, None, |_| false).unwrap();
                (
                    state(&sim),
                    vec![report_bits(first), report_bits(second)],
                    sim.allocated_bytes(),
                )
            });
            assert_eq!(actual, expected, "{method:?}, {threads} threads");
        }
    }
}

#[test]
fn run_owned_adaptive_feedback_preserves_bits_and_releases_thread_bindings() {
    use std::sync::Arc;
    for method in PressureImplementation::ALL {
        let reference = pool(1).install(|| {
            let mut sim = simulation(method);
            let report = sim.step(0.02, source(), |_| false).unwrap();
            (state(&sim), report_bits(report))
        });
        let scheduler = rheon::CpuScheduler::new();
        let actual = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build_scoped(
                |thread| scheduler.scope(|| thread.run()),
                |pool| {
                    pool.install(|| {
                        let mut sim = simulation(method);
                        let report = sim.step(0.02, source(), |_| false).unwrap();
                        (state(&sim), report_bits(report))
                    })
                },
            )
            .unwrap();
        assert_eq!(actual, reference);
        let samples = scheduler.snapshot();
        assert!(
            samples
                .iter()
                .any(|s| s.stage == rheon::CalculationStage::PressureOperator && s.calls > 1)
        );
        assert!(
            samples.iter().all(|s| s.batch_cells > 0
                && s.tasks > 0
                && s.wall_nanoseconds_per_cell.is_finite())
        );
        assert_eq!(Arc::strong_count(&scheduler), 1);
        let before = samples.iter().map(|s| s.calls).collect::<Vec<_>>();
        pool(2).install(|| simulation(method).step(0.02, source(), |_| false).unwrap());
        assert_eq!(
            before,
            scheduler
                .snapshot()
                .iter()
                .map(|s| s.calls)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn parallel_cancellation_keeps_accepted_state_and_callbacks_on_the_caller() {
    let pool = pool(4);
    for method in PressureImplementation::ALL {
        pool.install(|| {
            let mut sim = simulation(method);
            sim.step(0.02, source(), |_| false).unwrap();
            let before = state(&sim);
            let time = sim.state().time.to_bits();
            let generation = sim.state().generation;
            let caller = std::thread::current().id();
            for stage in [
                StepStage::VelocitySlice,
                StepStage::PressureIteration,
                StepStage::TracerSlice,
                StepStage::BeforeCommit,
            ] {
                let occurrence = match stage {
                    // Pass a completed parallel band before cancelling.
                    StepStage::VelocitySlice | StepStage::TracerSlice => 22,
                    StepStage::PressureIteration => 2,
                    _ => 1,
                };
                // Rc proves the API continues accepting non-Send callbacks.
                let calls = std::rc::Rc::new(std::cell::Cell::new(0));
                let result = sim.step(0.02, None, |at| {
                    assert_eq!(std::thread::current().id(), caller);
                    if at == stage {
                        calls.set(calls.get() + 1);
                    }
                    at == stage && calls.get() == occurrence
                });
                assert!(result.is_err(), "{method:?}, {stage:?}");
                assert_eq!(
                    calls.get(),
                    occurrence,
                    "cancellation checkpoint must be reached"
                );
                assert_eq!(state(&sim), before);
                assert_eq!(sim.state().time.to_bits(), time);
                assert_eq!(sim.state().generation, generation);
            }
            let mut reference = simulation(method);
            reference.step(0.02, source(), |_| false).unwrap();
            reference.step(0.02, None, |_| false).unwrap();
            sim.step(0.02, None, |_| false).unwrap();
            assert_eq!(state(&sim), state(&reference));
        });
    }
}
