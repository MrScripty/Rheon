#[path = "../examples/session_support/mod.rs"]
mod support;
use rheon::*;
use std::io::{self, Write};

fn particle() -> FluidParticle {
    FluidParticle {
        position: [0.125, -2.5, 3.75],
        velocity: [-0.0, 1.0 / 3.0, -5.25],
        mass: 0.12345678901234567,
        temperature_kelvin: 291.125,
        material: 1,
    }
}
fn identity() -> SessionIdentity {
    SessionIdentity {
        run_id: u64::MAX,
        source_revision: "fixture<&\"'>–源".into(),
    }
}
#[test]
fn constructor_pressure_is_unavailable_then_accepted_interval_is_named() {
    let (mut session, boundary) = support::fixture([3, 2, 4], 0);
    let mut bytes = Vec::new();
    write_session_vti(&mut bytes, &session.snapshot()).unwrap();
    let xml = String::from_utf8(bytes).unwrap();
    assert!(!xml.contains("Name=\"interval_pressure_pa\""));
    assert!(!xml.contains("Name=\"pressure_gauge_cell\""));
    session
        .step(support::inputs(), boundary, None, |_| false)
        .unwrap();
    let mut bytes = Vec::new();
    write_session_vti(&mut bytes, &session.snapshot()).unwrap();
    let xml = String::from_utf8(bytes).unwrap();
    assert!(xml.contains("Name=\"interval_pressure_pa\""));
    assert!(xml.contains("Name=\"pressure_interval_s\""));
    assert!(xml.contains("Name=\"velocity_x_dimensions\""));
    assert!(xml.contains("WholeExtent=\"0 3 0 2 0 4\""));
}
#[test]
fn particle_diagnostics_are_optional_and_not_point_data() {
    let mut bytes = Vec::new();
    write_particles_vtp(&mut bytes, &identity(), 0, 0.0, &[], None).unwrap();
    let xml = String::from_utf8(bytes).unwrap();
    assert!(xml.contains("NumberOfPoints=\"0\""));
    assert!(!xml.contains("Name=\"preintegration_pressure_pa\""));
    let mut bytes = Vec::new();
    write_particles_vtp(
        &mut bytes,
        &identity(),
        2,
        0.5,
        &[particle()],
        Some((
            0.25,
            &[ParticleDiagnostic {
                density: 999.1234567890123,
                pressure: 1.23456789012345,
                acceleration: [-1.0, 0.0, 2.0],
            }],
        )),
    )
    .unwrap();
    let xml = String::from_utf8(bytes).unwrap();
    let split = xml.find("<PointData").unwrap();
    assert!(xml[..split].contains("Name=\"preintegration_pressure_pa\""));
    assert!(!xml[split..].contains("preintegration_pressure"));
    assert!(
        !xml.contains(&identity().source_revision),
        "Metadata is byte encoded, not raw XML"
    );
    assert!(xml.contains("18446744073709551615"));
    assert!(xml.contains("0.12345678901234566") || xml.contains("0.12345678901234567"));
}
#[test]
fn invalid_particle_payloads_fail_before_output() {
    let good = particle();
    for bad in [
        FluidParticle {
            material: 2,
            ..good
        },
        FluidParticle {
            position: [f64::NAN, 0.0, 0.0],
            ..good
        },
        FluidParticle {
            velocity: [0.0, f64::INFINITY, 0.0],
            ..good
        },
        FluidParticle { mass: 0.0, ..good },
        FluidParticle {
            temperature_kelvin: -1.0,
            ..good
        },
    ] {
        let mut bytes = Vec::new();
        assert_eq!(
            write_particles_vtp(&mut bytes, &identity(), 1, 1.0, &[bad], None)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(bytes.is_empty());
    }
    for (time, diagnostics) in [
        (f64::NAN, None),
        (1.0, Some((0.5, &[][..]))),
        (
            1.0,
            Some((
                2.0,
                &[ParticleDiagnostic {
                    density: 1.0,
                    ..Default::default()
                }][..],
            )),
        ),
        (
            1.0,
            Some((
                0.5,
                &[ParticleDiagnostic {
                    density: f64::INFINITY,
                    ..Default::default()
                }][..],
            )),
        ),
    ] {
        let mut bytes = Vec::new();
        assert_eq!(
            write_particles_vtp(&mut bytes, &identity(), 1, time, &[good], diagnostics)
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidInput
        );
        assert!(bytes.is_empty());
    }
    let mut bytes = Vec::new();
    let bad_identity = SessionIdentity {
        source_revision: "bad\0source".into(),
        ..identity()
    };
    assert!(write_particles_vtp(&mut bytes, &bad_identity, 0, 0.0, &[], None).is_err());
    assert!(bytes.is_empty());
}
struct FailedWriter {
    remaining: usize,
}
impl Write for FailedWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.remaining == 0 {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "test sink"));
        }
        let n = bytes.len().min(self.remaining);
        self.remaining -= n;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn partial_writer_failure_propagates_without_mutating_publication() {
    let (mut session, boundary) = support::fixture([3, 2, 4], 0);
    assert_eq!(
        write_session_vti(&mut FailedWriter { remaining: 80 }, &session.snapshot())
            .unwrap_err()
            .kind(),
        io::ErrorKind::BrokenPipe
    );
    assert_eq!(session.snapshot().frame(), 0);
    session
        .step(support::inputs(), boundary, None, |_| false)
        .unwrap();
    assert_eq!(
        write_particles_vtp(
            &mut FailedWriter { remaining: 80 },
            &identity(),
            1,
            0.5,
            &[particle()],
            None
        )
        .unwrap_err()
        .kind(),
        io::ErrorKind::BrokenPipe
    );
}

#[test]
fn canonical_material_snapshot_adapter_preserves_both_publication_stages() {
    let mut session = MaterialSession::new(
        identity(),
        GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap(),
        ParticleFluidConfig {
            fill: DomainFill::EmittedFluid,
            samples_per_axis: 4,
            max_particles: 64,
            memory_limit: 1024 * 1024,
            ..Default::default()
        },
        2,
    )
    .unwrap();
    for generation in 0..=1 {
        if generation > 0 {
            session
                .step(
                    MaterialStepControls {
                        requested_dt: 1e-5,
                        emitter: None,
                    },
                    None,
                    || false,
                )
                .unwrap();
        }
        let snapshot = session.snapshot();
        let mut actual = Vec::new();
        write_material_vtp(&mut actual, &snapshot).unwrap();
        let mut expected = Vec::new();
        write_particles_vtp(
            &mut expected,
            snapshot.identity(),
            snapshot.generation(),
            snapshot.time(),
            snapshot.particles(),
            snapshot.particle_diagnostics(),
        )
        .unwrap();
        assert_eq!(actual, expected);
        assert_eq!(snapshot.generation(), generation);
        assert_eq!(snapshot.particle_diagnostics().is_some(), generation > 0);
        assert_eq!(
            write_material_vtp(&mut FailedWriter { remaining: 80 }, &snapshot)
                .unwrap_err()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }
}
