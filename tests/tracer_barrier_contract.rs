use rheon::{
    Axis, BodyForce, ForceRegion, ForceUnits, GridGeometry, PressureImplementation,
    PressureSettings, ScalarSampler, Simulation, SimulationConfig, SimulationError, SmokeSource,
    StepStage, SurfaceError, SurfaceSettings, SurfaceStamp, TracerBarrierError,
    TracerBarrierSampler, TriangleSurface, advect_tracer, advect_tracer_with_barrier,
};

fn plane(axis: usize, coordinate: f64) -> TriangleSurface {
    let mut a = [-16.0; 3];
    a[axis] = coordinate;
    let mut b = a;
    b[(axis + 1) % 3] = 64.0;
    let mut c = a;
    c[(axis + 2) % 3] = 64.0;
    TriangleSurface::new(
        SurfaceStamp { id: 41, version: 7 },
        vec![a, b, c],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn velocity(grid: &GridGeometry, value: [f32; 3]) -> [Vec<f32>; 3] {
    [Axis::X, Axis::Y, Axis::Z].map(|axis| {
        vec![
            value[match axis {
                Axis::X => 0,
                Axis::Y => 1,
                Axis::Z => 2,
            }];
            grid.face_len(axis)
        ]
    })
}
fn refs(v: &[Vec<f32>; 3]) -> [&[f32]; 3] {
    [&v[0], &v[1], &v[2]]
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 2e-14, "{a} != {b}");
}

#[test]
fn visible_donors_renormalize_on_both_sides_and_remove_thin_sheet_leakage() {
    for axis in 0..3 {
        let mut dims = [1; 3];
        dims[axis] = 2;
        let grid = GridGeometry::new(dims, [1.0; 3], [0.0; 3]).unwrap();
        let surface = plane(axis, 1.0);
        let old = [0.0, 1.0];
        let sampler = TracerBarrierSampler::new(&grid, &old, &surface).unwrap();
        for (coordinate, expected, weight) in [(0.75, 0.0, 0.75), (1.25, 1.0, 0.75)] {
            let mut p = [0.5; 3];
            p[axis] = coordinate;
            let filtered = sampler.sample(p, || false).unwrap();
            assert_eq!(filtered.value, expected);
            near(filtered.visible_weight, weight);
            assert_eq!(filtered.blocked_donors, 1);
            let ordinary = ScalarSampler::cells(&grid, &old)
                .unwrap()
                .sample(p)
                .unwrap();
            assert_ne!(ordinary, expected);
        }
    }
}

#[test]
fn visible_interpolation_preserves_constants_and_visible_range_with_3d_weights() {
    let grid = GridGeometry::new([2, 2, 2], [1.0; 3], [0.0; 3]).unwrap();
    let surface = plane(0, 1.0);
    let constant = [3.25; 8];
    let sampler = TracerBarrierSampler::new(&grid, &constant, &surface).unwrap();
    for x in [0.625, 0.75, 1.25, 1.375] {
        let s = sampler.sample([x, 0.75, 1.25], || false).unwrap();
        assert_eq!(s.value, 3.25);
        assert_eq!(s.blocked_donors, 4);
    }
    // Right-side values are outside the desired visible range and must have
    // exactly zero effect. Left y/z interpolation at (.75,1.25) is 2.75.
    let old = [1.0, 100.0, 2.0, 100.0, 3.0, 100.0, 4.0, 100.0];
    let sample = TracerBarrierSampler::new(&grid, &old, &surface)
        .unwrap()
        .sample([0.75, 0.75, 1.25], || false)
        .unwrap();
    near(sample.value, 2.75);
    assert!((1.0..=4.0).contains(&sample.value));
    assert_eq!(sample.blocked_donors, 4);
}

#[test]
fn blocked_midpoint_and_departure_legs_revert_without_wall_pushout() {
    let grid = GridGeometry::new([4, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let surface = plane(0, 1.0);
    let old = [1.0, 0.0, 0.0, 0.0];
    let v = velocity(&grid, [1.0, 0.0, 0.0]);
    for dt in [0.75, 1.0] {
        let mut ordinary = [-1.0; 4];
        let mut out = ordinary;
        advect_tracer(&grid, &old, refs(&v), dt, &mut ordinary, || false).unwrap();
        let report =
            advect_tracer_with_barrier(&grid, &old, refs(&v), dt, &surface, &mut out, || false)
                .unwrap();
        assert_eq!(out[1], old[1]);
        assert!(ordinary[1] > 0.0);
        assert_eq!(report.reverted_traces, 1);
        assert_eq!(report.surface, surface.stamp());
        assert!(out.iter().all(|v| (0.0..=1.0).contains(v)));
    }
}

#[test]
fn clear_trace_still_filters_a_stencil_that_crosses_the_barrier() {
    let grid = GridGeometry::new([4, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let surface = plane(0, 1.0);
    let old = [1.0, 0.0, 0.0, 0.0];
    let v = velocity(&grid, [1.0, 0.0, 0.0]);
    let mut ordinary = [-1.0; 4];
    let mut out = ordinary;
    advect_tracer(&grid, &old, refs(&v), 0.25, &mut ordinary, || false).unwrap();
    let report =
        advect_tracer_with_barrier(&grid, &old, refs(&v), 0.25, &surface, &mut out, || false)
            .unwrap();
    assert_eq!(ordinary[1], 0.25);
    assert_eq!(out[1], 0.0);
    assert_eq!(report.reverted_traces, 0);
    assert_eq!(report.samples_with_blocked_donors, 1);
    assert_eq!(report.blocked_donors, 1);
}

#[test]
fn no_obstruction_preserves_legacy_midpoint_and_sample_bits() {
    let grid = GridGeometry::new([4, 3, 2], [0.25, 0.5, 0.75], [-0.25, 0.0, 0.0]).unwrap();
    let surface = plane(0, 100.0);
    let old: Vec<f32> = (0..grid.cell_len())
        .map(|i| ((i * 7) % 19) as f32 / 19.0)
        .collect();
    let mut v = velocity(&grid, [0.0; 3]);
    for (d, field) in v.iter_mut().enumerate() {
        for (i, u) in field.iter_mut().enumerate() {
            *u = ((i * 3 + d) % 11) as f32 / 29.0 - 0.1;
        }
    }
    let mut expected = vec![0.0; old.len()];
    let mut out = expected.clone();
    advect_tracer(&grid, &old, refs(&v), 0.2, &mut expected, || false).unwrap();
    let report =
        advect_tracer_with_barrier(&grid, &old, refs(&v), 0.2, &surface, &mut out, || false)
            .unwrap();
    assert_eq!(
        out.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
        expected.iter().map(|v| v.to_bits()).collect::<Vec<_>>()
    );
    assert_eq!(report.reverted_traces, 0);
    assert_eq!(report.blocked_donors, 0);
}

#[test]
fn contacts_no_visible_donor_and_ambiguous_visibility_fail_explicitly() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let old = [0.0, 1.0];
    let sheet = plane(0, 1.0);
    let sampler = TracerBarrierSampler::new(&grid, &old, &sheet).unwrap();
    assert_eq!(
        sampler.sample([1.0, 0.5, 0.5], || false),
        Err(TracerBarrierError::NoVisibleDonor)
    );
    assert_eq!(
        sampler.sample([-0.1, 0.5, 0.5], || false),
        Err(TracerBarrierError::PositionOutsideBox)
    );
    assert_eq!(
        sampler.sample([f64::NAN; 3], || false),
        Err(TracerBarrierError::Sampling(
            rheon::SamplingError::NonFinitePosition
        ))
    );
    let coplanar = plane(2, 0.5);
    assert_eq!(
        TracerBarrierSampler::new(&grid, &old, &coplanar)
            .unwrap()
            .sample([0.75, 0.5, 0.5], || false),
        Err(TracerBarrierError::Surface(
            SurfaceError::AmbiguousIntersection { triangle: 0 }
        ))
    );
    // Coincident arrival/donor requires no visibility segment.
    assert_eq!(sampler.sample([0.5; 3], || false).unwrap().value, 0.0);
}

#[test]
fn low_level_cancellation_can_leave_scratch_partial_but_inputs_retry_unchanged() {
    let grid = GridGeometry::new([4, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let old = [1.0, 0.0, 0.0, 0.0];
    let before = old;
    let v = velocity(&grid, [1.0, 0.0, 0.0]);
    let saved = v.clone();
    let sheet = plane(0, 100.0);
    let mut out = [-777.0; 4];
    let mut polls = 0;
    assert_eq!(
        advect_tracer_with_barrier(&grid, &old, refs(&v), 0.25, &sheet, &mut out, || {
            polls += 1;
            polls == 9
        }),
        Err(TracerBarrierError::Cancelled)
    );
    assert_ne!(out[0], -777.0);
    assert_eq!(out[3], -777.0);
    assert_eq!(old, before);
    assert_eq!(v, saved);
    let a = advect_tracer_with_barrier(&grid, &old, refs(&v), 0.25, &sheet, &mut out, || false)
        .unwrap();
    let mut clean = [0.0; 4];
    let b = advect_tracer_with_barrier(&grid, &old, refs(&v), 0.25, &sheet, &mut clean, || false)
        .unwrap();
    assert_eq!(out, clean);
    assert_eq!(a, b);
}

fn config() -> SimulationConfig {
    SimulationConfig {
        density: 1.0,
        memory_limit: 1024 * 1024,
        pressure: PressureSettings {
            relative_residual: 1e-11,
            absolute_residual: 1e-12,
            divergence_limit: 1e-9,
            max_iterations: 1000,
        },
        actual_divergence_limit: 1e-6,
        max_courant: 1.0,
    }
}
fn sim(method: PressureImplementation) -> Simulation {
    Simulation::with_implementation(
        GridGeometry::new([2, 2, 1], [1.0; 3], [0.0; 3]).unwrap(),
        config(),
        method,
    )
    .unwrap()
}
fn source() -> SmokeSource {
    SmokeSource {
        lower: [0.0; 3],
        upper: [1.0, 2.0, 1.0],
        tracer_rate: 1.0,
        vertical_acceleration: 0.0,
    }
}
fn force() -> BodyForce {
    BodyForce {
        value: [2.0, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: Some(ForceRegion {
            lower: [0.0; 3],
            upper: [2.0, 1.0, 1.0],
        }),
    }
}
fn snapshot(s: &Simulation) -> (Vec<u32>, u64, u64) {
    let s = s.state();
    (
        s.x.iter()
            .chain(s.y)
            .chain(s.z)
            .chain(s.tracer)
            .map(|v| v.to_bits())
            .collect(),
        s.time.to_bits(),
        s.generation,
    )
}

#[test]
fn accepted_simulation_filters_tracer_while_preserving_force_pressure_and_memory() {
    let sheet = plane(0, 1.0);
    for method in PressureImplementation::ALL {
        let mut filtered = sim(method);
        let mut legacy = sim(method);
        filtered.step(0.1, Some(source()), |_| false).unwrap();
        legacy.step(0.1, Some(source()), |_| false).unwrap();
        let memory = filtered.allocated_bytes();
        let report = filtered
            .step_with_tracer_barrier(0.125, None, &[force()], Some(&sheet), |_| false)
            .unwrap();
        let original = legacy
            .step_with_forces(0.125, None, &[force()], |_| false)
            .unwrap();
        let barrier = report.tracer_barrier.unwrap();
        assert_eq!(barrier.surface, sheet.stamp());
        assert!(barrier.blocked_donors > 0);
        assert_eq!(filtered.state().x, legacy.state().x);
        assert_eq!(filtered.state().y, legacy.state().y);
        assert_eq!(filtered.state().z, legacy.state().z);
        assert_eq!(report.step.forces, original.forces);
        assert_eq!(
            report.step.step.pressure.iterations,
            original.step.pressure.iterations
        );
        assert_eq!(
            report.step.step.actual_divergence_max,
            original.step.actual_divergence_max
        );
        assert_eq!(filtered.allocated_bytes(), memory);
        assert_eq!(filtered.state().tracer[1], 0.0);
        assert!(legacy.state().tracer[1] > 0.0);
        assert!(
            filtered
                .state()
                .tracer
                .iter()
                .all(|v| (0.0..=1.0).contains(v))
        );
        assert_eq!(filtered.state().generation, 2);
    }
}

#[test]
fn none_barrier_preserves_every_accepted_bit_report_and_callback_sequence() {
    for method in PressureImplementation::ALL {
        let mut a = sim(method);
        let mut b = sim(method);
        let mut first = vec![];
        let mut second = vec![];
        let original = a
            .step_with_forces(0.125, Some(source()), &[force()], |s| {
                first.push(s);
                false
            })
            .unwrap();
        let report = b
            .step_with_tracer_barrier(0.125, Some(source()), &[force()], None, |s| {
                second.push(s);
                false
            })
            .unwrap();
        assert!(report.tracer_barrier.is_none());
        assert_eq!(snapshot(&a), snapshot(&b));
        assert_eq!(first, second);
        assert_eq!(original.forces, report.step.forces);
        assert_eq!(
            original.step.tracer_integral,
            report.step.step.tracer_integral
        );
    }
}

#[test]
fn barrier_error_and_late_cancellation_preserve_owner_and_allow_clean_retry() {
    for method in PressureImplementation::ALL {
        let mut candidate = sim(method);
        candidate.step(0.1, Some(source()), |_| false).unwrap();
        let before = snapshot(&candidate);
        let memory = candidate.allocated_bytes();
        let ambiguous = plane(2, 0.5);
        assert!(matches!(
            candidate
                .step_with_tracer_barrier(0.125, None, &[force()], Some(&ambiguous), |_| false),
            Err(SimulationError::TracerBarrier(TracerBarrierError::Surface(
                SurfaceError::AmbiguousIntersection { .. }
            )))
        ));
        assert_eq!(snapshot(&candidate), before);
        let outside = plane(0, 100.0);
        let mut polls = 0;
        assert_eq!(
            candidate
                .step_with_tracer_barrier(0.125, None, &[force()], Some(&outside), |stage| {
                    if stage == StepStage::TracerSlice {
                        polls += 1;
                    }
                    stage == StepStage::TracerSlice && polls == 9
                })
                .unwrap_err(),
            SimulationError::Cancelled {
                stage: StepStage::TracerSlice
            }
        );
        assert_eq!(snapshot(&candidate), before);
        assert_eq!(
            candidate
                .step_with_tracer_barrier(0.125, None, &[force()], Some(&outside), |s| s
                    == StepStage::BeforeCommit)
                .unwrap_err(),
            SimulationError::Cancelled {
                stage: StepStage::BeforeCommit
            }
        );
        assert_eq!(snapshot(&candidate), before);
        assert_eq!(candidate.allocated_bytes(), memory);
        candidate
            .step_with_tracer_barrier(0.125, None, &[force()], Some(&outside), |_| false)
            .unwrap();
        let mut fresh = sim(method);
        fresh.step(0.1, Some(source()), |_| false).unwrap();
        fresh
            .step_with_forces(0.125, None, &[force()], |_| false)
            .unwrap();
        assert_eq!(snapshot(&candidate), snapshot(&fresh));
    }
}

#[test]
fn admission_failures_precede_any_output_write() {
    let grid = GridGeometry::new([2, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let sheet = plane(0, 1.0);
    let v = velocity(&grid, [0.0; 3]);
    let mut out = [-99.0; 2];
    assert_eq!(
        advect_tracer_with_barrier(&grid, &[0.0; 2], refs(&v), 0.0, &sheet, &mut out, || false),
        Err(TracerBarrierError::Sampling(
            rheon::SamplingError::InvalidTimeStep
        ))
    );
    assert_eq!(out, [-99.0; 2]);
    assert_eq!(
        advect_tracer_with_barrier(
            &grid,
            &[f32::NAN; 2],
            refs(&v),
            0.1,
            &sheet,
            &mut out,
            || false
        ),
        Err(TracerBarrierError::Sampling(
            rheon::SamplingError::NonFiniteField
        ))
    );
    assert_eq!(out, [-99.0; 2]);
    assert_eq!(
        advect_tracer_with_barrier(&grid, &[0.0; 2], refs(&v), 0.1, &sheet, &mut [], || false),
        Err(TracerBarrierError::OutputLengthMismatch)
    );
}

#[test]
fn zero_weight_extremes_cannot_relax_the_positive_donor_range() {
    let grid = GridGeometry::new([2, 2, 2], [1.0; 3], [0.0; 3]).unwrap();
    let surface = plane(0, 100.0);
    let old = [3.25, 100.0, 3.25, 100.0, 3.25, 100.0, 3.25, 100.0];
    let sample = TracerBarrierSampler::new(&grid, &old, &surface)
        .unwrap()
        .sample([0.5, 0.51, 0.65], || false)
        .unwrap();
    assert_eq!(sample.blocked_donors, 0);
    assert_eq!(sample.value, 3.25);
}
