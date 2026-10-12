#![cfg(feature = "work-observation")]
use rheon::{
    CalculationObserver, GridGeometry, PressureSettings, Simulation, SimulationConfig, SmokeSource,
};
use std::sync::Arc;
fn simulation() -> Simulation {
    Simulation::new(
        GridGeometry::new([8; 3], [0.125; 3], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 1.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-9,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
    )
    .unwrap()
}
fn step() -> Vec<u32> {
    let mut s = simulation();
    s.step(
        0.02,
        Some(SmokeSource {
            lower: [0.25; 3],
            upper: [0.75; 3],
            tracer_rate: 2.0,
            vertical_acceleration: 1.0,
        }),
        |_| false,
    )
    .unwrap();
    let state = s.state();
    state
        .x
        .iter()
        .chain(state.y)
        .chain(state.z)
        .chain(state.tracer)
        .map(|v| v.to_bits())
        .collect()
}
#[test]
fn diagnostics_are_real_per_worker_isolated_and_do_not_change_results_or_leak_bindings() {
    let g = GridGeometry::new([8; 3], [0.125; 3], [0.0; 3]).unwrap();
    let observer = CalculationObserver::new(&g, 2).unwrap();
    let expected = step();
    let results = rayon::ThreadPoolBuilder::new()
        .num_threads(2)
        .build_scoped(
            |thread| observer.observe(|| thread.run()),
            |pool| pool.broadcast(|_| step()),
        )
        .unwrap();
    for result in results {
        assert_eq!(result, expected);
    }
    let markers = observer.snapshot();
    assert_eq!(markers.len(), 2);
    for marker in markers {
        assert!(!marker.active);
        assert!(marker.coordinate.iter().all(|v| *v < 8));
    }
    assert_eq!(Arc::strong_count(&observer), 1);
    let before = observer
        .snapshot()
        .iter()
        .map(|m| m.coordinate)
        .collect::<Vec<_>>();
    step();
    assert_eq!(
        before,
        observer
            .snapshot()
            .iter()
            .map(|m| m.coordinate)
            .collect::<Vec<_>>()
    );
    assert!(CalculationObserver::new(&g, 0).is_err());
}
