use rheon::*;
use std::{
    cell::RefCell,
    io::{self, Write},
    rc::Rc,
};

#[derive(Clone, Default)]
struct Sink(Rc<RefCell<(Vec<u8>, bool)>>);
impl Write for Sink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut s = self.0.borrow_mut();
        if s.1 {
            return Err(io::Error::other("injected output failure"));
        }
        s.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.0.borrow().1 {
            Err(io::Error::other("injected flush failure"))
        } else {
            Ok(())
        }
    }
}
fn fixture() -> (
    LiquidTransportSimulation,
    BoxFluxStepWorkspace,
    BoxFluxStepBoundary,
) {
    fixture_with_spacing(1.0 / 3.0)
}
fn fixture_with_spacing(
    spacing_x: f64,
) -> (
    LiquidTransportSimulation,
    BoxFluxStepWorkspace,
    BoxFluxStepBoundary,
) {
    let grid = GridGeometry::new([3, 1, 1], [spacing_x, 1.0, 1.0], [0.0; 3]).unwrap();
    let config = LiquidTransportConfig {
        carrier: SimulationConfig {
            density: 1000.0,
            memory_limit: 1_000_000,
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
        volume_stamp: VolumeStamp {
            id: u64::MAX,
            version: 0,
        },
        carrier_id: u64::MAX - 1,
    };
    let method = PressureImplementation::JacobiPcgV1;
    (
        LiquidTransportSimulation::new(grid.clone(), config, method, vec![0.5, 0.0, 0.0]).unwrap(),
        BoxFluxStepWorkspace::new(grid, 1000.0, 1_000_000, method).unwrap(),
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
#[test]
fn clipped_accepted_dt_is_retained_before_postcommit_metadata_refusal() {
    let (s, mut w, b) = fixture_with_spacing(1.0 / 1024.0);
    let frames = Sink::default();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, frames.clone(), limits(), 65536).unwrap();
    let constructor = frames.0.borrow().0.clone();
    assert!(matches!(
        seq.step_with_box_flux(inputs(), &mut w, b, |_| false),
        Err(Dense3dError::InvalidState("accepted controls continuity"))
    ));
    assert!(seq.is_aborted());
    assert_eq!(seq.state().carrier.time, 1.0 / 512.0);
    assert_eq!(seq.state().liquid.time, 1.0 / 512.0);
    assert_eq!(
        seq.state().carrier_stamp,
        VolumeStamp {
            id: u64::MAX - 1,
            version: 1
        }
    );
    assert_eq!(
        seq.state().liquid.stamp,
        VolumeStamp {
            id: u64::MAX,
            version: 1
        }
    );
    assert_eq!(seq.producer_controls_interval_count(), 1);
    assert_eq!(seq.frame_count(), 1);
    assert_eq!(frames.0.borrow().0, constructor);
    let accepted = snapshot(seq.state());
    assert!(matches!(
        seq.step_with_box_flux(inputs(), &mut w, b, |_| panic!("retry physics")),
        Err(Dense3dError::Aborted)
    ));
    assert!(matches!(
        seq.write_producer_controls_intervals(Vec::new()),
        Err(Dense3dError::Aborted)
    ));
    assert!(matches!(seq.flush(), Err(Dense3dError::Aborted)));
    assert_eq!(snapshot(seq.state()), accepted);
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
fn limits() -> Dense3dLimits {
    Dense3dLimits {
        max_cells: 3,
        max_frames: 10,
        max_bytes: 1_000_000,
    }
}
fn snapshot(s: LiquidTransportView<'_>) -> Vec<u64> {
    s.carrier
        .x
        .iter()
        .chain(s.carrier.y)
        .chain(s.carrier.z)
        .chain(s.carrier.tracer)
        .map(|v| u64::from(v.to_bits()))
        .chain(
            s.liquid
                .fraction
                .iter()
                .chain(s.pressure)
                .map(|v| v.to_bits()),
        )
        .chain([
            s.carrier.time.to_bits(),
            s.liquid.time.to_bits(),
            s.carrier_stamp.id,
            s.carrier_stamp.version,
            s.liquid.stamp.id,
            s.liquid.stamp.version,
        ])
        .collect()
}
#[test]
fn capture_is_explicit_and_keeps_v1_frame_bytes_identical() {
    let (s, mut w, b) = fixture();
    let (s1, mut w1, b1) = fixture();
    let out = Sink::default();
    let old = Sink::default();
    let mut capture =
        Dense3dSequence::new_with_producer_controls(s, out.clone(), limits(), 65536).unwrap();
    let mut v1 = Dense3dSequence::new(s1, old.clone(), limits()).unwrap();
    assert_eq!(capture.producer_controls_interval_count(), 0);
    assert!(capture.producer_controls_payload_bytes() > 0);
    assert_eq!(
        v1.producer_controls_payload_bytes(),
        capture.producer_controls_payload_bytes()
    );
    for _ in 0..2 {
        capture
            .step_with_box_flux(inputs(), &mut w, b, |_| false)
            .unwrap();
        v1.step_with_box_flux(inputs(), &mut w1, b1, |_| false)
            .unwrap();
    }
    assert_eq!(out.0.borrow().0, old.0.borrow().0);
    assert_eq!(snapshot(capture.state()), snapshot(v1.state()));
    let mut controls = Vec::new();
    let count = capture
        .write_producer_controls_intervals(&mut controls)
        .unwrap();
    assert_eq!(count, controls.len());
    let json = String::from_utf8(controls).unwrap();
    assert_eq!(json.matches("\"start_frame\"").count(), 2);
    assert!(
        json.contains("\"start_frame\":0,\"end_frame\":1,\"start_time_s\":0.00000000000000000e0")
    );
    assert!(
        json.contains("\"start_frame\":1,\"end_frame\":2,\"start_time_s\":6.25000000000000000e-2")
    );
    assert!(
        json.contains("\"carrier_before\":{\"id\":\"18446744073709551614\",\"version\":\"0\"}")
    );
    assert!(json.contains("\"liquid_after\":{\"id\":\"18446744073709551615\",\"version\":\"2\"}"));
    assert!(json.contains(
        "\"source_mode\":\"none\",\"source_rate_m3_s\":0,\"body_acceleration_m_s2\":[0,0,0]"
    ));
    assert_eq!(
        json.matches("\"boundary_stamp\":{\"id\":\"47\",\"version\":\"0\"}")
            .count(),
        2
    );
    let before = snapshot(capture.state());
    assert!(
        capture
            .step_with_box_flux(inputs(), &mut w, b, |_| false)
            .is_err()
    );
    assert_eq!(before, snapshot(capture.state()));
}
#[test]
fn unsupported_controls_and_some_zero_source_refuse_before_physics() {
    let zero = [0.0; 3];
    let force = [BodyForce {
        value: [0.0; 3],
        units: ForceUnits::Acceleration,
        region: None,
    }];
    for mode in 0..8 {
        let (s, mut w, mut b) = fixture();
        let sink = Sink::default();
        let mut sequence =
            Dense3dSequence::new_with_producer_controls(s, sink.clone(), limits(), 65536).unwrap();
        let before = snapshot(sequence.state());
        let bytes = sink.0.borrow().0.clone();
        let mut input = inputs();
        match mode {
            0 => input.requested_dt = 0.03125,
            1 => {
                input.source = Some(
                    LiquidVolumeSource::new(VolumeStamp { id: 99, version: 0 }, &zero).unwrap(),
                )
            }
            2 => input.forces = &force,
            3 => {
                input.inlet =
                    LiquidInlet::new(VolumeStamp { id: 53, version: 1 }, [[0.0; 2]; 3]).unwrap()
            }
            4 => {
                input.inlet =
                    LiquidInlet::new(VolumeStamp { id: 53, version: 0 }, [[0.1; 2]; 3]).unwrap()
            }
            5 => {
                b.flux = PrescribedBoxFlux::new(
                    BoxFluxStamp { id: 48, version: 0 },
                    [[-0.25, 0.25], [0.0; 2], [0.0; 2]],
                )
                .unwrap()
            }
            6 => {
                b.flux = PrescribedBoxFlux::new(
                    BoxFluxStamp { id: 47, version: 0 },
                    [[-0.125, 0.125], [0.0; 2], [0.0; 2]],
                )
                .unwrap()
            }
            _ => {
                input.smoke_source = Some(SmokeSource {
                    lower: [0.0; 3],
                    upper: [1.0; 3],
                    tracer_rate: 0.0,
                    vertical_acceleration: 0.0,
                })
            }
        }
        assert!(matches!(
            sequence.step_with_box_flux(input, &mut w, b, |_| panic!("physics callback reached")),
            Err(Dense3dError::InvalidState(_))
        ));
        assert!(sequence.is_aborted());
        assert_eq!(before, snapshot(sequence.state()));
        assert_eq!(bytes, sink.0.borrow().0);
        assert_eq!(sequence.producer_controls_interval_count(), 0);
    }
}
#[test]
fn capture_capacity_and_budget_are_preflighted() {
    let (s, _, _) = fixture();
    let sink = Sink::default();
    assert!(matches!(
        Dense3dSequence::new_with_producer_controls(s, sink.clone(), limits(), 2049),
        Err(Dense3dError::Limit)
    ));
    assert!(sink.0.borrow().0.is_empty());
    let (s, mut w, b) = fixture();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, Vec::new(), limits(), 2050).unwrap();
    seq.step_with_box_flux(inputs(), &mut w, b, |_| false)
        .unwrap();
    let before = snapshot(seq.state());
    assert!(matches!(
        seq.step_with_box_flux(inputs(), &mut w, b, |_| panic!("physics callback reached")),
        Err(Dense3dError::Limit)
    ));
    assert_eq!(before, snapshot(seq.state()));
    let (s, mut w, b) = fixture();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, Vec::new(), limits(), 65536).unwrap();
    for _ in 0..8 {
        seq.step_with_box_flux(inputs(), &mut w, b, |_| false)
            .unwrap();
    }
    let before = snapshot(seq.state());
    assert_eq!(seq.producer_controls_interval_count(), 8);
    assert!(matches!(
        seq.step_with_box_flux(inputs(), &mut w, b, |_| panic!("physics callback reached")),
        Err(Dense3dError::Limit)
    ));
    assert_eq!(before, snapshot(seq.state()));
}
#[test]
fn cancellation_has_no_interval_and_output_failure_preserves_accepted_physics() {
    let (s, mut w, b) = fixture();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, Vec::new(), limits(), 65536).unwrap();
    let before = snapshot(seq.state());
    assert!(matches!(
        seq.step_with_box_flux(inputs(), &mut w, b, |_| true),
        Err(Dense3dError::Step(_))
    ));
    assert_eq!(before, snapshot(seq.state()));
    assert_eq!(seq.producer_controls_interval_count(), 0);
    let (s, mut w, b) = fixture();
    let sink = Sink::default();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, sink.clone(), limits(), 65536).unwrap();
    sink.0.borrow_mut().1 = true;
    assert!(matches!(
        seq.step_with_box_flux(inputs(), &mut w, b, |_| false),
        Err(Dense3dError::Io(_))
    ));
    assert_eq!(seq.state().carrier.generation, 1);
    assert_eq!(
        seq.state().carrier_stamp,
        VolumeStamp {
            id: u64::MAX - 1,
            version: 1
        }
    );
    assert_eq!(
        seq.state().liquid.stamp,
        VolumeStamp {
            id: u64::MAX,
            version: 1
        }
    );
    assert_eq!(seq.producer_controls_interval_count(), 1);
    assert!(matches!(
        seq.write_producer_controls_intervals(Vec::new()),
        Err(Dense3dError::Aborted)
    ));
    let (s, mut w, b) = fixture();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, Vec::new(), limits(), 65536).unwrap();
    seq.step_with_box_flux(inputs(), &mut w, b, |_| false)
        .unwrap();
    let before = snapshot(seq.state());
    let broken = Sink::default();
    broken.0.borrow_mut().1 = true;
    assert!(matches!(
        seq.write_producer_controls_intervals(broken),
        Err(Dense3dError::Io(_))
    ));
    assert!(seq.is_aborted());
    assert_eq!(before, snapshot(seq.state()));
    struct FailFlush;
    impl Write for FailFlush {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Err(io::Error::other("injected controls flush failure"))
        }
    }
    let (s, mut w, b) = fixture();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, Vec::new(), limits(), 65536).unwrap();
    seq.step_with_box_flux(inputs(), &mut w, b, |_| false)
        .unwrap();
    let before = snapshot(seq.state());
    assert!(matches!(
        seq.write_producer_controls_intervals(FailFlush),
        Err(Dense3dError::Io(_))
    ));
    assert!(seq.is_aborted());
    assert_eq!(before, snapshot(seq.state()));
}
#[test]
fn eight_tiny_accepted_intervals_bind_actual_frames() {
    let (s, mut w, b) = fixture();
    let frames = Sink::default();
    let mut seq =
        Dense3dSequence::new_with_producer_controls(s, frames.clone(), limits(), 65536).unwrap();
    for _ in 0..8 {
        seq.step_with_box_flux(inputs(), &mut w, b, |_| false)
            .unwrap();
    }
    seq.flush().unwrap();
    let mut controls = Vec::new();
    seq.write_producer_controls_intervals(&mut controls)
        .unwrap();
    let text = std::str::from_utf8(&controls).unwrap();
    assert_eq!(text.matches("\"start_frame\"").count(), 8);
    for frame in 0..8 {
        assert!(text.contains(&format!("\"start_frame\":{frame},\"end_frame\":{},\"start_time_s\":{:.17e},\"end_time_s\":{:.17e},\"dt_s\":{:.17e}",frame+1,frame as f64*0.0625,(frame+1) as f64*0.0625,0.0625)));
    }
    assert_eq!(seq.state().carrier_stamp.version, 8);
    assert_eq!(seq.state().liquid.stamp.version, 8);
    if let Some(path) = std::env::var_os("RHEON_TINY_CONTROLS_EVIDENCE") {
        let directory = std::path::Path::new(&path);
        assert!(directory.is_absolute());
        let parent = directory.parent().unwrap().canonicalize().unwrap();
        let git = std::process::Command::new("git")
            .args(["-C"])
            .arg(parent)
            .args(["rev-parse", "--is-inside-work-tree"])
            .output()
            .unwrap();
        assert!(!(git.status.success() && git.stdout.starts_with(b"true")));
        std::fs::create_dir(directory).unwrap();
        for (name, bytes) in [
            ("frames.jsonl", frames.0.borrow().0.as_slice()),
            ("intervals.json", controls.as_slice()),
        ] {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(directory.join(name))
                .unwrap();
            file.write_all(bytes).unwrap();
            file.sync_all().unwrap();
        }
        println!(
            "limited tiny native ownership evidence: geometry [3,1,1], 9 frames, 8 captures; not a full pilot or completed v2 dataset"
        );
    }
}
