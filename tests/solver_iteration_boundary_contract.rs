#[path = "common/iteration_boundary_fixture.rs"]
mod fixture;
use fixture::*;
use rheon::*;

#[test]
fn final_authorized_correction_matches_larger_budget_without_extra_solve() {
    for h in [0.05, 0.0015625] {
        let mut reference = owner(7, false);
        let expected = reference
            .step_extruded_with_forces(h, &[force(true)], |_| false)
            .unwrap();
        let mut bounded = owner(3, false);
        let report = bounded
            .step_extruded_with_forces(h, &[force(true)], |_| false)
            .unwrap();
        assert_eq!(snapshot(&bounded), snapshot(&reference));
        assert_eq!(report.step.planar.unknowns, expected.step.planar.unknowns);
        assert_eq!(report.step.planar.equation_evaluations, 22);
        assert_eq!(report.step.planar.iterations, 3);
        assert_eq!(expected.step.planar.iterations, 4);
        assert!(report.step.planar.finite_momentum_rate_norm <= 1e-13);
        assert!(report.step.planar.ledger_error.abs() <= report.step.planar.work_allowance);
        assert_eq!(bounded.allocated_bytes(), 474768);
        for budget in [8, 512] {
            let mut large = owner(budget, false);
            let r = large
                .step_extruded_with_forces(h, &[force(true)], |_| false)
                .unwrap();
            assert_eq!(snapshot(&large), snapshot(&reference));
            assert_eq!(r.step.planar.equation_evaluations, 22);
        }
    }
}

#[test]
fn insufficient_corrections_and_seed_controls_preserve_accepted_state() {
    for budget in [1, 2] {
        let mut o = owner(budget, false);
        let before = snapshot(&o);
        for _ in 0..2 {
            assert_eq!(
                o.step_extruded_with_forces(0.05, &[force(true)], |_| false)
                    .unwrap_err(),
                CoupledDiscreteError::IterationLimit
            );
            assert_eq!(snapshot(&o), before);
        }
    }
    let mut o = owner(1, true);
    let r = o.step_extruded_with_forces(0.025, &[], |_| false).unwrap();
    assert_eq!(r.step.planar.iterations, 1);
    assert_eq!(r.step.planar.equation_evaluations, 1);
    let before = snapshot(&o);
    assert_eq!(
        o.step_extruded_with_forces(0.05, &[force(false)], |_| false)
            .unwrap_err(),
        CoupledDiscreteError::IterationLimit
    );
    assert_eq!(snapshot(&o), before);
}

#[test]
fn terminal_check_cancellation_preserves_state_and_retries_identically() {
    let mut reference = owner(3, false);
    reference
        .step_extruded_with_forces(0.05, &[force(true)], |_| false)
        .unwrap();
    let expected = snapshot(&reference);
    let mut o = owner(3, false);
    let before = snapshot(&o);
    let mut visits = 0;
    assert_eq!(
        o.step_extruded_with_forces(0.05, &[force(true)], |stage| {
            if stage == CoupledDiscreteStage::Iteration {
                visits += 1;
            }
            stage == CoupledDiscreteStage::Iteration && visits == 4
        })
        .unwrap_err(),
        CoupledDiscreteError::Cancelled {
            stage: CoupledDiscreteStage::Iteration
        }
    );
    assert_eq!(visits, 4);
    assert_eq!(snapshot(&o), before);
    o.step_extruded_with_forces(0.05, &[force(true)], |_| false)
        .unwrap();
    assert_eq!(snapshot(&o), expected);
}

#[test]
fn all_downstream_barriers_preserve_nonzero_accepted_state_and_resume() {
    let mut reference = owner(3, false);
    reference
        .step_extruded_with_forces(0.05, &[force(true)], |_| false)
        .unwrap();
    reference
        .step_extruded_with_forces(0.025, &[force(true)], |_| false)
        .unwrap();
    let expected = snapshot(&reference);
    for target in [
        CoupledDiscreteStage::BeforeAcceptance,
        CoupledDiscreteStage::BeforeThirdSolve,
        CoupledDiscreteStage::ThirdAssembly,
        CoupledDiscreteStage::AfterThirdSolve,
        CoupledDiscreteStage::BeforePublish,
    ] {
        let mut o = owner(3, false);
        o.step_extruded_with_forces(0.05, &[force(true)], |_| false)
            .unwrap();
        let before = snapshot(&o);
        assert!(o.state().velocity.iter().any(|u| u[2] != 0.));
        assert_eq!(
            o.step_extruded_with_forces(0.025, &[force(true)], |s| s == target)
                .unwrap_err(),
            CoupledDiscreteError::Cancelled { stage: target }
        );
        assert_eq!(snapshot(&o), before);
        o.step_extruded_with_forces(0.025, &[force(true)], |_| false)
            .unwrap();
        assert_eq!(snapshot(&o), expected);
        assert_eq!(o.allocated_bytes(), 474768);
    }
}

#[test]
fn fixed_workspace_reservation_and_strict_tolerances_are_unchanged() {
    let mut o = owner(3, false);
    let u: [[f64; 3]; 16] = o.state().velocity.try_into().unwrap();
    let bytes = CoupledDiscreteFlow::nominal_forced_extruded_bytes().unwrap();
    assert_eq!(bytes, 474768);
    assert!(matches!(
        CoupledDiscreteFlow::new_forced_extruded(
            geometry(),
            &u,
            Default::default(),
            TranslatedViscousSettings {
                memory_limit: bytes - 1,
                ..Default::default()
            },
            131
        ),
        Err(CoupledDiscreteError::Flow(
            TranslatedViscousError::BufferLimit { .. }
        ))
    ));
    let before = snapshot(&o);
    assert!(
        o.step_extruded_with_forces(0.051, &[force(true)], |_| false)
            .is_err()
    );
    assert_eq!(before, snapshot(&o));
    assert!(
        CoupledDiscreteFlow::new_forced_extruded(
            geometry(),
            &u,
            Default::default(),
            TranslatedViscousSettings {
                absolute_residual: 1e-12,
                ..Default::default()
            },
            131
        )
        .is_err()
    );
}
