use rheon::{
    Axis, BodyForce, BoxFluxStamp, BoxFluxStepBoundary, BoxFluxStepWorkspace, BoxFluxTracerPolicy,
    ForceUnits, GridGeometry, LiquidInlet, LiquidStepError, LiquidStepInputs, LiquidStepStage,
    LiquidTransportConfig, LiquidTransportSimulation, LiquidTransportView, LiquidVolumeError,
    LiquidVolumeSettings, LiquidVolumeSource, PrescribedBoxFlux, PressureImplementation,
    PressureSettings, SimulationConfig, SimulationError, SmokeSource, StepStage, VolumeStage,
    VolumeStamp,
};

fn config() -> LiquidTransportConfig {
    LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1000.0,
            memory_limit: 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 1000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 800.0,
        volume_stamp: VolumeStamp { id: 11, version: 4 },
        carrier_id: 17,
    }
}
fn make(
    g: &GridGeometry,
    method: PressureImplementation,
    initial: Vec<f64>,
) -> LiquidTransportSimulation {
    LiquidTransportSimulation::new(g.clone(), config(), method, initial).unwrap()
}
fn workspace(g: &GridGeometry, method: PressureImplementation) -> BoxFluxStepWorkspace {
    BoxFluxStepWorkspace::new(g.clone(), config().carrier.density, 1024 * 1024, method).unwrap()
}
fn boundary(axis: usize, speed: f32) -> BoxFluxStepBoundary {
    let mut outward = [[0.0; 2]; 3];
    outward[axis] = [-speed, speed];
    BoxFluxStepBoundary {
        flux: PrescribedBoxFlux::new(BoxFluxStamp { id: 13, version: 7 }, outward).unwrap(),
        tracer: BoxFluxTracerPolicy::ClampedAppearance,
    }
}
fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.5,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 19, version: 8 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
#[derive(Debug, PartialEq)]
struct Snapshot {
    fields: Vec<u32>,
    pressure: Vec<u64>,
    fractions: Vec<u64>,
    carrier_time: u64,
    volume_time: u64,
    carrier_stamp: VolumeStamp,
    volume_stamp: VolumeStamp,
}
fn snapshot(s: LiquidTransportView<'_>) -> Snapshot {
    Snapshot {
        fields: s
            .carrier
            .x
            .iter()
            .chain(s.carrier.y)
            .chain(s.carrier.z)
            .chain(s.carrier.tracer)
            .map(|v| v.to_bits())
            .collect(),
        pressure: s.pressure.iter().map(|v| v.to_bits()).collect(),
        fractions: s.liquid.fraction.iter().map(|v| v.to_bits()).collect(),
        carrier_time: s.carrier.time.to_bits(),
        volume_time: s.liquid.time.to_bits(),
        carrier_stamp: s.carrier_stamp,
        volume_stamp: s.liquid.stamp,
    }
}
fn near(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 1e-9 * a.abs().max(b.abs()).max(1.0),
        "{a} != {b}"
    );
}

#[test]
fn signed_axis_transport_publishes_hand_pressure_and_synchronized_states() {
    for method in PressureImplementation::ALL {
        for axis in 0..3 {
            for speed in [-0.25, 0.25] {
                let mut dims = [1; 3];
                dims[axis] = 3;
                let g = GridGeometry::new(dims, [1.0; 3], [0.0; 3]).unwrap();
                let mut s = make(&g, method, vec![0.0, 1.0, 0.0]);
                let mut w = workspace(&g, method);
                let r = s
                    .step_with_box_flux(inputs(), &mut w, boundary(axis, speed), |_| false)
                    .unwrap();
                assert_eq!(
                    s.state().liquid.fraction,
                    if speed > 0.0 {
                        [0.0, 0.875, 0.125]
                    } else {
                        [0.125, 0.875, 0.0]
                    }
                );
                let state = s.state();
                for (i, &p) in state.pressure.iter().enumerate() {
                    near(p, -(i as f64) * 1000.0 * f64::from(speed) / 0.5);
                }
                assert_eq!(state.carrier.time, 0.5);
                assert_eq!(state.liquid.time, 0.5);
                assert_eq!(state.carrier_stamp, VolumeStamp { id: 17, version: 1 });
                assert_eq!(state.liquid.stamp, VolumeStamp { id: 11, version: 5 });
                assert_eq!(r.liquid.flow, state.carrier_stamp);
                assert_eq!(r.liquid.stamp, state.liquid.stamp);
                assert_eq!(r.liquid.liquid_volume_after, 1.0);
                assert_eq!(r.liquid.liquid_mass_after, 800.0);
                assert_eq!(r.liquid.actual_divergence_max, 0.0);
                assert_eq!(
                    r.total_array_bytes,
                    s.allocated_bytes() + w.allocated_bytes()
                );
            }
        }
    }
}

#[test]
fn liquid_failure_after_new_pressure_prepare_preserves_all_accepted_bits_and_retry() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let mut s = make(&g, method, vec![0.5; 3]);
        let mut w = workspace(&g, method);
        let b = boundary(0, 0.25);
        let mut request = inputs();
        request.inlet =
            LiquidInlet::new(VolumeStamp { id: 19, version: 9 }, [[0.5; 2]; 3]).unwrap();
        s.step_with_box_flux(request, &mut w, b, |_| false).unwrap();
        assert!(s.state().pressure.iter().any(|&p| p != 0.0));
        let before = snapshot(s.state());
        let rates = [2.0; 3];
        let mut failed = request;
        failed.source =
            Some(LiquidVolumeSource::new(VolumeStamp { id: 23, version: 2 }, &rates).unwrap());
        assert!(matches!(
            s.step_with_box_flux(failed, &mut w, b, |_| false),
            Err(LiquidStepError::Volume(
                LiquidVolumeError::FractionBounds { .. }
            ))
        ));
        assert_eq!(snapshot(s.state()), before);
        let r = s.step_with_box_flux(request, &mut w, b, |_| false).unwrap();
        let mut fresh = make(&g, method, vec![0.5; 3]);
        let mut fresh_w = workspace(&g, method);
        fresh
            .step_with_box_flux(request, &mut fresh_w, b, |_| false)
            .unwrap();
        let expected = fresh
            .step_with_box_flux(request, &mut fresh_w, b, |_| false)
            .unwrap();
        assert_eq!(snapshot(s.state()), snapshot(fresh.state()));
        assert_eq!(
            r.liquid.volume_balance_error,
            expected.liquid.volume_balance_error
        );
        assert!(s.state().pressure.iter().all(|&p| p == 0.0));
    }
}

#[test]
fn every_exercised_carrier_volume_and_final_checkpoint_rolls_back_and_replays() {
    let g = GridGeometry::new([3, 2, 2], [1.0; 3], [0.0; 3]).unwrap();
    let forces = [BodyForce {
        value: [0.125, 0.0, 0.0],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    for method in PressureImplementation::ALL {
        let b = boundary(0, 0.25);
        let mut request = inputs();
        request.forces = &forces;
        request.smoke_source = Some(SmokeSource {
            lower: g.origin(),
            upper: g.upper(),
            tracer_rate: 0.125,
            vertical_acceleration: 0.0,
        });
        request.inlet =
            LiquidInlet::new(VolumeStamp { id: 19, version: 8 }, [[0.375; 2]; 3]).unwrap();
        let mut reference = make(&g, method, vec![0.375; g.cell_len()]);
        let mut rw = workspace(&g, method);
        reference
            .step_with_box_flux(request, &mut rw, b, |_| false)
            .unwrap();
        let mut stages = vec![];
        reference
            .step_with_box_flux(request, &mut rw, b, |stage| {
                stages.push(stage);
                false
            })
            .unwrap();
        assert!(stages.contains(&LiquidStepStage::Carrier(StepStage::PressureIteration)));
        assert!(stages.contains(&LiquidStepStage::Volume(VolumeStage::CellUpdateSlice)));
        assert_eq!(stages.last(), Some(&LiquidStepStage::BeforeCommit));
        let mut unique = vec![];
        for &stage in &stages {
            if !unique.contains(&stage) {
                unique.push(stage);
            }
        }
        for stage in unique {
            let count = stages.iter().filter(|&&v| v == stage).count();
            for occurrence in [1, count] {
                let mut s = make(&g, method, vec![0.375; g.cell_len()]);
                let mut w = workspace(&g, method);
                s.step_with_box_flux(request, &mut w, b, |_| false).unwrap();
                let before = snapshot(s.state());
                let mut seen = 0;
                let result = s.step_with_box_flux(request, &mut w, b, |actual| {
                    if actual == stage {
                        seen += 1;
                    }
                    actual == stage && seen == occurrence
                });
                assert!(result.is_err(), "{method:?} {stage:?} #{occurrence}");
                assert_eq!(snapshot(s.state()), before);
                s.step_with_box_flux(request, &mut w, b, |_| false).unwrap();
                assert_eq!(snapshot(s.state()), snapshot(reference.state()));
            }
        }
    }
}

#[test]
fn carrier_failure_and_pause_preserve_volume_pressure_and_clocks() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
        let mut s = make(&g, method, vec![0.0, 0.5, 0.0]);
        let mut w = workspace(&g, method);
        let b = boundary(0, 0.25);
        s.step_with_box_flux(inputs(), &mut w, b, |_| false)
            .unwrap();
        let before = snapshot(s.state());
        s.set_paused(true);
        assert!(matches!(
            s.step(inputs(), |_| false),
            Err(LiquidStepError::Carrier(SimulationError::Paused))
        ));
        assert_eq!(snapshot(s.state()), before);
        s.set_paused(false);
        let mut bad = inputs();
        bad.requested_dt = f64::NAN;
        assert!(
            s.step_with_box_flux(bad, &mut w, b, |_| panic!("invalid input reached callback"))
                .is_err()
        );
        assert_eq!(snapshot(s.state()), before);
        let bad_boundary = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp { id: 5, version: 2 },
                [[0.0, 0.25], [0.0; 2], [0.0; 2]],
            )
            .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        assert!(
            s.step_with_box_flux(inputs(), &mut w, bad_boundary, |_| false)
                .is_err()
        );
        assert_eq!(snapshot(s.state()), before);
    }
}

#[test]
fn accepted_actual_dt_controls_volume_interval_and_closed_step_is_coherent() {
    for method in PressureImplementation::ALL {
        let g = GridGeometry::new([3, 1, 1], [0.5, 1.0, 1.0], [0.0; 3]).unwrap();
        let mut s = make(&g, method, vec![0.0, 1.0, 0.0]);
        let mut w = workspace(&g, method);
        let mut request = inputs();
        request.requested_dt = 20.0;
        let r = s
            .step_with_box_flux(request, &mut w, boundary(0, 2.0), |_| false)
            .unwrap();
        assert_eq!(r.carrier.step.dt, 0.125);
        assert_eq!(r.liquid.dt, 0.125);
        assert_eq!(s.state().liquid.fraction, [0.0, 0.5, 0.5]);
        assert_eq!(s.state().carrier.time, s.state().liquid.time);
        let mut closed = make(&g, method, vec![0.5; 3]);
        let r = closed.step(inputs(), |_| false).unwrap();
        assert!(r.boundary.is_none());
        assert_eq!(r.boundary_workspace_array_bytes, 0);
        assert_eq!(r.total_array_bytes, closed.allocated_bytes());
        assert_eq!(closed.state().liquid.fraction, [0.5; 3]);
        assert_eq!(closed.state().pressure, [0.0; 3]);
        assert_eq!(closed.state().carrier.time, closed.state().liquid.time);
    }
}

#[test]
fn combined_capacity_cap_counts_transferred_capacity_and_pressure() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let faces = [Axis::X, Axis::Y, Axis::Z]
        .into_iter()
        .map(|a| g.face_len(a))
        .sum::<usize>();
    let nominal = 16 * faces + 80 * g.cell_len();
    let mut c = config();
    c.carrier.memory_limit = nominal;
    let s = LiquidTransportSimulation::new(
        g.clone(),
        c,
        PressureImplementation::JacobiPcgV1,
        vec![0.5; 3],
    )
    .unwrap();
    assert_eq!(s.allocated_bytes(), nominal);
    c.carrier.memory_limit -= 1;
    assert!(
        matches!(LiquidTransportSimulation::new(g.clone(),c,PressureImplementation::JacobiPcgV1,vec![0.5;3]),Err(LiquidStepError::BufferLimit{required,limit}) if required==nominal && limit==nominal-1)
    );
    let mut initial = Vec::with_capacity(100);
    initial.resize(3, 0.5);
    c.carrier.memory_limit = nominal;
    assert!(
        matches!(LiquidTransportSimulation::new(g,c,PressureImplementation::JacobiPcgV1,initial),Err(LiquidStepError::BufferLimit{required,..}) if required>=nominal+97*8)
    );
}

#[test]
fn volume_input_failure_and_version_overflow_precede_carrier_callbacks() {
    let g = GridGeometry::new([3, 1, 1], [1.0; 3], [0.0; 3]).unwrap();
    let mut s = make(&g, PressureImplementation::JacobiPcgV1, vec![0.5; 3]);
    let before = snapshot(s.state());
    let mut request = inputs();
    request.volume.max_outward_courant = 0.0;
    assert!(matches!(
        s.step(request, |_| panic!("invalid settings reached callback")),
        Err(LiquidStepError::Volume(LiquidVolumeError::InvalidSettings))
    ));
    assert_eq!(snapshot(s.state()), before);
    let mut c = config();
    c.volume_stamp.version = u64::MAX;
    let mut s =
        LiquidTransportSimulation::new(g, c, PressureImplementation::JacobiPcgV1, vec![0.5; 3])
            .unwrap();
    let before = snapshot(s.state());
    assert!(matches!(
        s.step(inputs(), |_| panic!("version overflow reached callback")),
        Err(LiquidStepError::Volume(LiquidVolumeError::VersionOverflow))
    ));
    assert_eq!(snapshot(s.state()), before);
}

#[cfg(feature = "png-export")]
#[test]
fn liquid_guidance_integrates_accepted_fraction_with_explicit_pixel_budget() {
    use rheon::{ExportError, liquid_guidance_pixels, write_liquid_guidance_png};
    use std::io::{BufReader, Cursor};
    let g = GridGeometry::new([2; 3], [1.0; 3], [0.0; 3]).unwrap();
    let mut fraction = vec![0.0; g.cell_len()];
    fraction[g.cell_index([0, 0, 0]).unwrap()] = 0.25;
    fraction[g.cell_index([0, 0, 1]).unwrap()] = 0.5;
    fraction[g.cell_index([1, 1, 0]).unwrap()] = 0.125;
    let s = make(&g, PressureImplementation::JacobiPcgV1, fraction);
    assert!(matches!(
        liquid_guidance_pixels(&s, 3),
        Err(ExportError::PixelBudget {
            required: 4,
            limit: 3
        })
    ));
    let expected = [0, 161, 254, 0];
    assert_eq!(liquid_guidance_pixels(&s, 4).unwrap(), expected);
    let mut bytes = vec![];
    assert_eq!(write_liquid_guidance_png(&s, &mut bytes, 4).unwrap(), 4);
    let decoder = png::Decoder::new(BufReader::new(Cursor::new(bytes)));
    let mut reader = decoder.read_info().unwrap();
    let mut raw = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut raw).unwrap();
    assert_eq!((info.width, info.height), (2, 2));
    assert_eq!(&raw[..info.buffer_size()], &expected);
}
