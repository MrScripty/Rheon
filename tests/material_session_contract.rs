use rheon::*;
use std::time::Instant;
fn grid() -> GridGeometry {
    GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap()
}
fn config() -> ParticleFluidConfig {
    ParticleFluidConfig {
        samples_per_axis: 4,
        max_particles: 512,
        memory_limit: 1024 * 1024,
        fill: DomainFill::EmittedFluid,
        gravity: [0.0; 3],
        ..Default::default()
    }
}
fn session(max_steps: usize) -> MaterialSession {
    MaterialSession::new(
        SessionIdentity {
            run_id: 71,
            source_revision: "authored-material-test".into(),
        },
        grid(),
        config(),
        max_steps,
    )
    .unwrap()
}
fn controls() -> MaterialStepControls {
    MaterialStepControls {
        requested_dt: 0.001,
        emitter: None,
    }
}

#[test]
fn canonical_particle_capture_and_actual_controls_match_direct_solver() {
    let mut session = session(4);
    let mut reference = ParticleFluidSimulation::new(grid(), config()).unwrap();
    assert_eq!(session.snapshot().stage(), SnapshotStage::Constructor);
    assert!(session.snapshot().particle_diagnostics().is_none());
    assert!(session.snapshot().accepted_controls().is_none());
    assert!(session.snapshot().diagnostic_stage().is_none());
    assert!(!session.capabilities().grid_pressure_provider);
    for generation in 1..=4 {
        let mut controls = controls();
        controls.requested_dt = 0.2; // stability-bound step must shorten this
        controls.emitter = Some(MaterialEmitterControl {
            source_id: 9,
            source_version: generation,
            emitter: RotatingCylinderEmitter {
                pressure_heads: [0.0; 2],
                ..Default::default()
            },
        });
        let before_time = session.snapshot().time();
        let expected = reference
            .step(
                controls.requested_dt,
                controls.emitter.map(|c| c.emitter),
                || false,
            )
            .unwrap();
        let report = session.step(controls, None, || false).unwrap();
        let snapshot = session.snapshot();
        assert_eq!(snapshot.stage(), SnapshotStage::Accepted);
        assert_eq!(snapshot.identity().run_id, 71);
        assert_eq!(snapshot.generation(), generation);
        assert_eq!(snapshot.particles(), reference.particles());
        assert_eq!(
            snapshot.particle_diagnostics(),
            reference.particle_diagnostics()
        );
        assert_eq!(
            snapshot.diagnostic_stage(),
            Some(ParticleEvaluationStage::PostEmissionPreIntegration)
        );
        assert_eq!(snapshot.particle_diagnostics().unwrap().0, before_time);
        assert_eq!(snapshot.time(), before_time + report.dt);
        assert_eq!(report.dt, expected.dt);
        assert!(report.dt < controls.requested_dt);
        assert_eq!(snapshot.accepted_controls(), Some(controls));
    }
    assert!(matches!(
        session.step(controls(), None, || false),
        Err(MaterialSessionError::StepLimit)
    ));
    session.stop();
    assert!(session.is_stopped());
    assert!(matches!(
        session.step(controls(), None, || false),
        Err(MaterialSessionError::Stopped)
    ));
    assert_eq!(session.snapshot().generation(), 4);
}

#[test]
fn every_checkpoint_and_invalid_request_preserve_diagnostics_and_controls() {
    let mut probe = session(4);
    probe.step(controls(), None, || false).unwrap();
    let mut count = 0;
    probe
        .step(controls(), None, || {
            count += 1;
            false
        })
        .unwrap();
    assert!(count > 0);
    for selected in 1..=count {
        let mut session = session(4);
        session.step(controls(), None, || false).unwrap();
        let before = session.snapshot().particles().to_vec();
        let (time, diag) = session.snapshot().particle_diagnostics().unwrap();
        let diag = diag.to_vec();
        let accepted_time = session.snapshot().time();
        let mut visits = 0;
        assert!(matches!(
            session.step(controls(), None, || {
                visits += 1;
                visits == selected
            }),
            Err(MaterialSessionError::Physics(ParticleFluidError::Cancelled))
        ));
        let mut invalid = controls();
        invalid.requested_dt = f64::NAN;
        assert!(session.step(invalid, None, || false).is_err());
        assert!(matches!(
            session.step(controls(), Some(Instant::now()), || false),
            Err(MaterialSessionError::Deadline)
        ));
        let snapshot = session.snapshot();
        assert_eq!(snapshot.particles(), before);
        assert_eq!(
            snapshot.particle_diagnostics(),
            Some((time, diag.as_slice()))
        );
        assert_eq!(snapshot.time(), accepted_time);
        assert_eq!(snapshot.generation(), 1);
        assert_eq!(snapshot.accepted_controls(), Some(controls()));
        session.step(controls(), None, || false).unwrap();
        assert_eq!(session.snapshot().generation(), 2);
    }
}

#[test]
fn constructor_budgets_identity_and_zero_quota_are_enforced() {
    let id = || SessionIdentity {
        run_id: 1,
        source_revision: "test".into(),
    };
    let mut invalid = id();
    invalid.source_revision = "\n".into();
    assert!(matches!(
        MaterialSession::new(invalid, grid(), config(), 1),
        Err(MaterialSessionError::InvalidIdentity)
    ));
    let mut tiny = config();
    tiny.memory_limit = 1;
    assert!(matches!(
        MaterialSession::new(id(), grid(), tiny, 1),
        Err(MaterialSessionError::Physics(
            ParticleFluidError::BufferLimit { .. }
        ))
    ));
    let mut session = session(0);
    assert!(matches!(
        session.step(controls(), None, || false),
        Err(MaterialSessionError::StepLimit)
    ));
    assert_eq!(session.snapshot().generation(), 0);
}
