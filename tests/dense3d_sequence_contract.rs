use rheon::*;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

#[derive(Default)]
struct SinkState {
    bytes: Vec<u8>,
    fail_at: Option<usize>,
    fail_flush: bool,
}
#[derive(Clone, Default)]
struct Sink(Rc<RefCell<SinkState>>);
impl Write for Sink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut state = self.0.borrow_mut();
        let available = state
            .fail_at
            .map_or(bytes.len(), |n| n.saturating_sub(state.bytes.len()));
        if available == 0 {
            return Err(io::Error::other("injected write failure"));
        }
        let n = bytes.len().min(available);
        state.bytes.extend_from_slice(&bytes[..n]);
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.0.borrow().fail_flush {
            Err(io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}
fn fixture(
    counts: [u64; 3],
    version: u64,
) -> (
    LiquidTransportSimulation,
    BoxFluxStepWorkspace,
    BoxFluxStepBoundary,
) {
    let g = GridGeometry::new(
        counts,
        [
            1.0 / counts[0] as f64,
            1.0 / counts[1] as f64,
            1.0 / counts[2] as f64,
        ],
        [0.0; 3],
    )
    .unwrap();
    let mut f = vec![0.0; g.cell_len()];
    for k in 0..counts[2] as usize {
        for j in 0..counts[1] as usize {
            for i in counts[0] as usize / 4..counts[0] as usize / 2 {
                f[g.cell_index([i, j, k]).unwrap()] = 0.5;
            }
        }
    }
    let c = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1000.0,
            memory_limit: 16 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-12,
                absolute_residual: 1e-12,
                divergence_limit: 1e-9,
                max_iterations: 10000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
        represented_density: 800.0,
        volume_stamp: VolumeStamp {
            id: u64::MAX,
            version,
        },
        carrier_id: u64::MAX - 1,
    };
    let method = PressureImplementation::JacobiPcgV1;
    (
        LiquidTransportSimulation::new(g.clone(), c, method, f).unwrap(),
        BoxFluxStepWorkspace::new(g, c.carrier.density, 16 * 1024 * 1024, method).unwrap(),
        BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp { id: 47, version: 0 },
                [[-0.25, 0.25], [0.0; 2], [0.0; 2]],
            )
            .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        },
    )
}
fn inputs() -> LiquidStepInputs<'static> {
    LiquidStepInputs {
        requested_dt: 0.0625,
        smoke_source: None,
        forces: &[],
        inlet: LiquidInlet::new(VolumeStamp { id: 53, version: 0 }, [[0.0; 2]; 3]).unwrap(),
        source: None,
        volume: LiquidVolumeSettings {
            max_outward_courant: 1.0,
            actual_divergence_limit: 1e-5,
        },
    }
}
fn limits(frames: usize) -> Dense3dLimits {
    Dense3dLimits {
        max_cells: 512,
        max_frames: frames,
        max_bytes: 2 * 1024 * 1024,
    }
}
fn field<'a>(line: &'a str, name: &str) -> &'a str {
    line.split_once(&format!("\"{name}\":["))
        .unwrap()
        .1
        .split_once(']')
        .unwrap()
        .0
}
fn compare(line: &str, state: LiquidTransportView<'_>) {
    for (name, values) in [
        ("velocity_x", state.carrier.x),
        ("velocity_y", state.carrier.y),
        ("velocity_z", state.carrier.z),
        ("tracer", state.carrier.tracer),
    ] {
        let parsed: Vec<_> = field(line, name)
            .split(',')
            .map(|s| s.parse::<f32>().unwrap().to_bits())
            .collect();
        assert_eq!(
            parsed,
            values.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            "{name}"
        );
    }
    for (name, values) in [
        ("fraction", state.liquid.fraction),
        ("pressure", state.pressure),
    ] {
        let parsed: Vec<_> = field(line, name)
            .split(',')
            .map(|s| s.parse::<f64>().unwrap().to_bits())
            .collect();
        assert_eq!(
            parsed,
            values.iter().map(|v| v.to_bits()).collect::<Vec<_>>(),
            "{name}"
        );
    }
    assert!(line.contains(&format!(
        "\"carrier_stamp\":{{\"id\":\"{}\",\"version\":\"{}\"}}",
        state.carrier_stamp.id, state.carrier_stamp.version
    )));
    assert!(line.contains(&format!(
        "\"liquid_stamp\":{{\"id\":\"{}\",\"version\":\"{}\"}}",
        state.liquid.stamp.id, state.liquid.stamp.version
    )));
}
fn bits(state: LiquidTransportView<'_>) -> Vec<u64> {
    state
        .carrier
        .x
        .iter()
        .chain(state.carrier.y)
        .chain(state.carrier.z)
        .chain(state.carrier.tracer)
        .map(|v| u64::from(v.to_bits()))
        .chain(
            state
                .liquid
                .fraction
                .iter()
                .chain(state.pressure)
                .map(|v| v.to_bits()),
        )
        .chain([
            state.carrier.time.to_bits(),
            state.liquid.time.to_bits(),
            state.carrier_stamp.version,
            state.liquid.stamp.version,
        ])
        .collect()
}
#[test]
fn nine_frames_round_trip_every_accepted_native_bit_and_decimal_u64_stamp() {
    let (s, mut w, b) = fixture([16, 8, 4], 0);
    let sink = Sink::default();
    let mut export = Dense3dSequence::new(s, sink.clone(), limits(9)).unwrap();
    assert!(
        String::from_utf8(sink.0.borrow().bytes.clone())
            .unwrap()
            .contains("\"diagnostics\":null")
    );
    for frame in 0..9 {
        let content = String::from_utf8(sink.0.borrow().bytes.clone()).unwrap();
        let lines: Vec<_> = content.lines().collect();
        assert_eq!(lines.len(), frame + 1);
        compare(lines[frame], export.state());
        assert!(lines[frame].contains(&format!("\"frame\":{frame},")));
        assert_eq!(export.state().carrier.time, frame as f64 * 0.0625);
        assert_eq!(export.state().liquid.stamp.version, frame as u64);
        if frame < 8 {
            let r = export
                .step_with_box_flux(inputs(), &mut w, b, |_| false)
                .unwrap();
            assert_eq!(r.carrier.step.dt, 0.0625);
            assert_eq!(r.liquid.time, export.state().carrier.time);
        }
    }
    export.flush().unwrap();
    assert_eq!(export.frame_count(), 9);
    assert_eq!(export.bytes_written(), sink.0.borrow().bytes.len());
    assert_eq!(export.grid().face_counts(Axis::X), [17, 8, 4]);
    assert_eq!(export.grid().face_counts(Axis::Y), [16, 9, 4]);
    assert_eq!(export.grid().face_counts(Axis::Z), [16, 8, 5]);
}
#[test]
fn failed_cancelled_and_capped_steps_emit_no_frames_and_cannot_retry() {
    for mode in 0..4 {
        let (s, mut w, b) = fixture([3, 1, 1], 0);
        let sink = Sink::default();
        let mut export = Dense3dSequence::new(
            s,
            sink.clone(),
            Dense3dLimits {
                max_bytes: if mode == 3 { 4096 } else { 2 * 1024 * 1024 },
                ..limits(if mode == 2 { 1 } else { 9 })
            },
        )
        .unwrap();
        let before = bits(export.state());
        let bytes = sink.0.borrow().bytes.clone();
        let mut input = inputs();
        if mode == 0 {
            input.requested_dt = f64::NAN;
        }
        let result = export.step_with_box_flux(input, &mut w, b, |stage| {
            mode == 1 && stage == LiquidStepStage::BeforeCommit
        });
        assert!(result.is_err());
        assert!(export.is_aborted());
        assert_eq!(bits(export.state()), before);
        assert_eq!(sink.0.borrow().bytes, bytes);
        assert!(matches!(
            export.step_with_box_flux(inputs(), &mut w, b, |_| false),
            Err(Dense3dError::Aborted)
        ));
    }
}
#[test]
fn output_failure_after_physics_keeps_committed_state_and_aborts_without_rerun() {
    let (s, mut w, b) = fixture([3, 1, 1], 0);
    let sink = Sink::default();
    let mut export = Dense3dSequence::new(s, sink.clone(), limits(9)).unwrap();
    sink.0.borrow_mut().fail_at = Some(export.bytes_written() + 50);
    assert!(matches!(
        export.step_with_box_flux(inputs(), &mut w, b, |_| false),
        Err(Dense3dError::Io(_))
    ));
    assert_eq!(export.state().carrier.generation, 1);
    assert_eq!(export.state().liquid.stamp.version, 1);
    assert_eq!(export.state().carrier.time, 0.0625);
    assert_eq!(export.frame_count(), 1);
    let accepted = bits(export.state());
    let bytes = sink.0.borrow().bytes.clone();
    assert!(!bytes.ends_with(b"\n"));
    assert!(matches!(
        export.step_with_box_flux(inputs(), &mut w, b, |_| false),
        Err(Dense3dError::Aborted)
    ));
    assert_eq!(bits(export.state()), accepted);
    assert_eq!(sink.0.borrow().bytes, bytes);
    assert!(export.flush().is_err());
}
#[test]
fn malformed_nonfinite_and_wrong_axis_views_fail_before_output() {
    let (s, _, _) = fixture([3, 2, 1], 0);
    let g = s.grid();
    for mode in 0..5 {
        let mut state = s.state();
        let bad = [f64::NAN; 6];
        let velocity = [f32::INFINITY; 8];
        match mode {
            0 => state.pressure = &bad,
            1 => state.liquid.fraction = &bad,
            2 => state.carrier.x = &velocity,
            3 => state.carrier.x = state.carrier.y,
            4 => state.carrier.time = f64::NAN,
            _ => unreachable!(),
        }
        let mut out = vec![];
        assert!(write_dense3d_frame(&mut out, g, state, 0, None).is_err());
        assert!(out.is_empty());
    }
}
#[test]
fn flush_failure_is_terminal_and_initial_cell_cap_is_checked() {
    let (s, mut w, b) = fixture([3, 1, 1], 0);
    let sink = Sink::default();
    let mut export = Dense3dSequence::new(s, sink.clone(), limits(9)).unwrap();
    sink.0.borrow_mut().fail_flush = true;
    assert!(matches!(export.flush(), Err(Dense3dError::Io(_))));
    assert!(matches!(
        export.step_with_box_flux(inputs(), &mut w, b, |_| false),
        Err(Dense3dError::Aborted)
    ));
    assert_eq!(export.state().carrier.generation, 0);
    let (s, _, _) = fixture([3, 1, 1], 0);
    assert!(matches!(
        Dense3dSequence::new(
            s,
            Vec::new(),
            Dense3dLimits {
                max_cells: 2,
                ..limits(9)
            }
        ),
        Err(Dense3dError::Limit)
    ));
}

// Ownership prerequisites for a future, separately versioned metadata export.
// These use the existing three-cell fixture; they define no new JSON fields.
#[test]
fn transition_ownership_accepted_snapshot_follows_actual_control_report() {
    let (simulation, mut workspace, _) = fixture([3, 1, 1], 0);
    let sink = Sink::default();
    let mut sequence = Dense3dSequence::new(simulation, sink.clone(), limits(3)).unwrap();
    let mut expected_tracer = 0.0_f32;
    for interval in 0..2 {
        let scale = (interval + 1) as f64;
        let rates = [0.0001 * scale; 3];
        let forces = [BodyForce {
            value: [0.25 * scale, 0.0, 0.0],
            units: ForceUnits::Acceleration,
            region: None,
        }];
        let speed = 0.125 * scale as f32;
        let inlet_fraction = 0.125 + 0.125 * scale;
        let before_time = sequence.state().carrier.time;
        let before_carrier = sequence.state().carrier_stamp;
        let before_liquid = sequence.state().liquid.stamp;
        let before_output = sink.0.borrow().bytes.clone();
        let boundary_stamp = BoxFluxStamp {
            id: 701,
            version: 7 + interval * 2,
        };
        let inlet_stamp = VolumeStamp {
            id: 702,
            version: 12 + interval * 12,
        };
        let source_stamp = VolumeStamp {
            id: 703,
            version: 11 + interval * 12,
        };
        let boundary = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(boundary_stamp, [[-speed, speed], [0.0; 2], [0.0; 2]])
                .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        let input = LiquidStepInputs {
            requested_dt: if interval == 0 { 0.0625 } else { 0.03125 },
            smoke_source: Some(SmokeSource {
                lower: [0.0; 3],
                upper: [1.0; 3],
                tracer_rate: 0.125 * scale,
                vertical_acceleration: 0.0,
            }),
            forces: &forces,
            inlet: LiquidInlet::new(inlet_stamp, [[inlet_fraction; 2]; 3]).unwrap(),
            source: Some(LiquidVolumeSource::new(source_stamp, &rates).unwrap()),
            volume: inputs().volume,
        };
        let mut reached_final_checkpoint = false;
        let report = sequence
            .step_with_box_flux(input, &mut workspace, boundary, |stage| {
                // No constructor-plus-candidate output can escape before commit.
                assert_eq!(sink.0.borrow().bytes, before_output);
                reached_final_checkpoint |= stage == LiquidStepStage::BeforeCommit;
                false
            })
            .unwrap();
        assert!(reached_final_checkpoint);
        let state = sequence.state();
        assert_eq!(report.boundary.unwrap().projection.boundary, boundary_stamp);
        assert_eq!(report.liquid.inlet, inlet_stamp);
        assert_eq!(report.liquid.source, Some(source_stamp));
        assert!(report.carrier.forces.unwrap().applied_work > 0.0);
        assert_eq!(state.carrier.x.first().unwrap().to_bits(), speed.to_bits());
        assert_eq!(state.carrier.x.last().unwrap().to_bits(), speed.to_bits());
        let expected_inward = report.liquid.dt * f64::from(speed) * inlet_fraction;
        assert!(
            (report.liquid.inward_boundary_volume - expected_inward).abs()
                <= 4.0 * f64::EPSILON * expected_inward
        );
        let expected_source = report.liquid.dt * rates[0] * 3.0;
        assert!(
            (report.liquid.source_volume - expected_source).abs()
                <= 4.0 * f64::EPSILON * expected_source
        );
        assert_eq!(report.liquid.flow, state.carrier_stamp);
        assert_eq!(report.liquid.stamp, state.liquid.stamp);
        assert_eq!(state.carrier_stamp.id, before_carrier.id);
        assert_eq!(state.carrier_stamp.version, before_carrier.version + 1);
        assert_eq!(state.liquid.stamp.id, before_liquid.id);
        assert_eq!(state.liquid.stamp.version, before_liquid.version + 1);
        assert_eq!(report.carrier.step.dt, input.requested_dt);
        assert_eq!(report.liquid.dt, report.carrier.step.dt);
        assert_eq!(
            report.carrier.step.time,
            before_time + report.carrier.step.dt
        );
        assert_eq!(state.carrier.time, report.carrier.step.time);
        assert_eq!(state.liquid.time, state.carrier.time);
        let output = String::from_utf8(sink.0.borrow().bytes.clone()).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(lines.len(), interval as usize + 2);
        compare(lines.last().unwrap(), state);
        assert_eq!(sequence.frame_count(), lines.len());
        expected_tracer += (input.requested_dt * 0.125 * scale) as f32;
        assert!(
            state
                .carrier
                .tracer
                .iter()
                .all(|v| v.to_bits() == expected_tracer.to_bits())
        );
    }
}

#[test]
fn transition_ownership_accepted_dt_is_not_the_requested_dt() {
    let (simulation, mut workspace, boundary) = fixture([3, 1, 1], 0);
    let sink = Sink::default();
    let mut sequence = Dense3dSequence::new(simulation, sink.clone(), limits(2)).unwrap();
    let mut input = inputs();
    input.requested_dt = 2.0;
    let report = sequence
        .step_with_box_flux(input, &mut workspace, boundary, |_| false)
        .unwrap();
    assert!(report.carrier.step.dt > 0.0);
    assert!(report.carrier.step.dt < input.requested_dt);
    assert_eq!(report.carrier.step.dt, report.liquid.dt);
    assert_eq!(report.carrier.step.time, report.carrier.step.dt);
    assert_eq!(sequence.state().carrier.time, report.carrier.step.time);
    let output = String::from_utf8(sink.0.borrow().bytes.clone()).unwrap();
    let line = output.lines().last().unwrap();
    assert!(line.contains(&format!("\"dt_s\":{:.17e}", report.carrier.step.dt)));
    assert!(!line.contains(&format!("\"dt_s\":{:.17e}", input.requested_dt)));
    compare(line, sequence.state());
}

#[test]
fn transition_ownership_rejected_new_controls_do_not_relabel_prior_acceptance() {
    for cancel_at_commit in [false, true] {
        let (simulation, mut workspace, boundary) = fixture([3, 1, 1], 0);
        let sink = Sink::default();
        let mut sequence = Dense3dSequence::new(simulation, sink.clone(), limits(3)).unwrap();
        let accepted = sequence
            .step_with_box_flux(inputs(), &mut workspace, boundary, |_| false)
            .unwrap();
        let before = bits(sequence.state());
        let before_output = sink.0.borrow().bytes.clone();
        let mut rejected = inputs();
        rejected.requested_dt = if cancel_at_commit { 0.03125 } else { f64::NAN };
        rejected.inlet = LiquidInlet::new(
            VolumeStamp {
                id: 999,
                version: 8,
            },
            [[0.75; 2]; 3],
        )
        .unwrap();
        let new_boundary = BoxFluxStepBoundary {
            flux: PrescribedBoxFlux::new(
                BoxFluxStamp {
                    id: 998,
                    version: 9,
                },
                [[-0.125, 0.125], [0.0; 2], [0.0; 2]],
            )
            .unwrap(),
            tracer: BoxFluxTracerPolicy::ClampedAppearance,
        };
        let mut reached_commit = false;
        let result = sequence.step_with_box_flux(rejected, &mut workspace, new_boundary, |stage| {
            reached_commit |= stage == LiquidStepStage::BeforeCommit;
            cancel_at_commit && stage == LiquidStepStage::BeforeCommit
        });
        if cancel_at_commit {
            assert!(reached_commit);
            assert!(matches!(
                result,
                Err(Dense3dError::Step(LiquidStepError::Cancelled))
            ));
        } else {
            assert!(!reached_commit);
            assert!(matches!(
                result,
                Err(Dense3dError::Step(LiquidStepError::Carrier(
                    SimulationError::InvalidTimeStep
                )))
            ));
        }
        assert_eq!(bits(sequence.state()), before);
        assert_eq!(sink.0.borrow().bytes, before_output);
        assert_eq!(sequence.state().carrier.time, accepted.carrier.step.time);
        assert_eq!(sequence.state().liquid.stamp, accepted.liquid.stamp);
        assert_eq!(sequence.frame_count(), 2);
        assert!(sequence.is_aborted());
        assert!(matches!(
            sequence.step_with_box_flux(inputs(), &mut workspace, boundary, |_| false),
            Err(Dense3dError::Aborted)
        ));
        assert_eq!(sink.0.borrow().bytes, before_output);
    }
}
