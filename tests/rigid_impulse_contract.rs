use rheon::*;
fn surface() -> TriangleSurface {
    TriangleSurface::new(
        SurfaceStamp { id: 17, version: 4 },
        vec![[0., 0., 0.], [2., 0., 0.], [0., 1., 0.]],
        vec![[0, 1, 2]],
        SurfaceSettings::default(),
    )
    .unwrap()
}
fn state() -> RigidSnapshot {
    RigidSnapshot {
        stamp: RigidStamp {
            id: 7,
            generation: 2,
        },
        surface: SurfaceStamp { id: 17, version: 4 },
        center_of_mass: [0.; 3],
        mass_kg: 2.,
        inertia_kg_m2: [3., 4., 5.],
        velocity_m_s: [1., 2., 3.],
        angular_velocity_rad_s: [4., 5., 6.],
    }
}
fn close(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-12, "{a} != {b}");
}
#[test]
fn varying_load_changes_actual_twist_and_signed_work() {
    let s = surface();
    let traction = [[[0.; 3], [0., 0., 3.], [0.; 3]]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&traction),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let before = state();
    let mut body = FrozenRigidBody::new(before).unwrap();
    let r = body
        .apply_mesh_impulse(before.stamp, &load, 0.5, |_, _| false)
        .unwrap();
    assert_eq!(body.snapshot(), r.after);
    assert_eq!(r.before, before);
    assert_eq!(r.after.stamp.generation, 3);
    assert_eq!(r.after.surface, before.surface);
    assert_eq!(r.after.center_of_mass, before.center_of_mass);
    assert_eq!(r.force_n, [0., 0., 1.]);
    assert_eq!(r.torque_n_m, [0.25, -1., 0.]);
    assert_eq!(r.after.velocity_m_s, [1., 2., 3.25]);
    close(r.after.angular_velocity_rad_s[0], 4. + 1. / 24.);
    assert_eq!(r.after.angular_velocity_rad_s[1], 4.875);
    close(r.kinetic_before_j, 178.);
    close(r.energy_defect_j, 0.);
    close(r.update_energy_defect_j, 0.);
}
#[test]
fn pure_couple_negative_work_and_reverse_kick() {
    let s = surface();
    let values = [[[0., 0., -3.], [0., 0., 3.], [0.; 3]]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let mut body = FrozenRigidBody::new(state()).unwrap();
    let before = body.snapshot();
    let r = body
        .apply_mesh_impulse(before.stamp, &load, 1., |_, _| false)
        .unwrap();
    assert_eq!(r.force_n, [0.; 3]);
    assert_eq!(r.torque_n_m, [0., -0.5, 0.]);
    assert_eq!(r.after.velocity_m_s, before.velocity_m_s);
    assert!(r.impulse_work_j < 0.);
    assert!(r.kinetic_after_j < r.kinetic_before_j);
    let opposite = [[[0., 0., 3.], [0., 0., -3.], [0.; 3]]];
    let reversed = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&opposite),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let r2 = body
        .apply_mesh_impulse(r.after.stamp, &reversed, 1., |_, _| false)
        .unwrap();
    assert_eq!(r2.after.velocity_m_s, before.velocity_m_s);
    assert_eq!(
        r2.after.angular_velocity_rad_s,
        before.angular_velocity_rad_s
    );
    close(r.impulse_work_j + r2.impulse_work_j, 0.);
}
#[test]
fn split_impulses_telescope_without_pose_evolution() {
    let s = surface();
    let values = [[[1., 2., 3.]; 3]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let mut single = FrozenRigidBody::new(state()).unwrap();
    let mut split = FrozenRigidBody::new(state()).unwrap();
    let all = single
        .apply_mesh_impulse(state().stamp, &load, 1., |_, _| false)
        .unwrap();
    let a = split
        .apply_mesh_impulse(state().stamp, &load, 0.25, |_, _| false)
        .unwrap();
    let b = split
        .apply_mesh_impulse(a.after.stamp, &load, 0.75, |_, _| false)
        .unwrap();
    for i in 0..3 {
        close(all.after.velocity_m_s[i], b.after.velocity_m_s[i]);
        close(
            all.after.angular_velocity_rad_s[i],
            b.after.angular_velocity_rad_s[i],
        );
    }
    close(all.impulse_work_j, a.impulse_work_j + b.impulse_work_j);
}
#[test]
fn cancellation_at_every_phase_is_atomic() {
    let s = surface();
    let values = [[[1., 2., 3.]; 3]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    for phase in [
        RigidImpulseStage::Admission,
        RigidImpulseStage::Reduction,
        RigidImpulseStage::Proposal,
        RigidImpulseStage::Publication,
    ] {
        let mut body = FrozenRigidBody::new(state()).unwrap();
        let before = body.snapshot();
        assert_eq!(
            body.apply_mesh_impulse(before.stamp, &load, 1., |stage, _| stage == phase),
            Err(RigidImpulseError::Cancelled {
                stage: phase,
                triangle: 0
            })
        );
        assert_eq!(body.snapshot(), before);
    }
}
#[test]
fn ownership_duration_and_overflow_errors_are_atomic() {
    let s = surface();
    let values = [[[1., 2., 3.]; 3]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let mut body = FrozenRigidBody::new(state()).unwrap();
    let before = body.snapshot();
    let stale = RigidStamp {
        generation: 1,
        ..before.stamp
    };
    assert_eq!(
        body.apply_mesh_impulse(stale, &load, 1., |_, _| false),
        Err(RigidImpulseError::StaleBody)
    );
    for duration in [0., -1., f64::NAN, f64::INFINITY] {
        assert_eq!(
            body.apply_mesh_impulse(before.stamp, &load, duration, |_, _| false),
            Err(RigidImpulseError::InvalidDuration)
        );
        assert_eq!(body.snapshot(), before);
    }
    let shifted = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [1., 0., 0.],
        1,
        |_, _| false,
    )
    .unwrap();
    assert_eq!(
        body.apply_mesh_impulse(before.stamp, &shifted, 1., |_, _| false),
        Err(RigidImpulseError::ReferenceMismatch)
    );
    let mut changed = state();
    changed.surface.version += 1;
    let mut other = FrozenRigidBody::new(changed).unwrap();
    assert_eq!(
        other.apply_mesh_impulse(changed.stamp, &load, 1., |_, _| false),
        Err(RigidImpulseError::StaleSurface)
    );
    assert_eq!(other.snapshot(), changed);
    let mut last = state();
    last.stamp.generation = u64::MAX;
    let mut exhausted = FrozenRigidBody::new(last).unwrap();
    assert_eq!(
        exhausted.apply_mesh_impulse(last.stamp, &load, 1., |_, _| false),
        Err(RigidImpulseError::GenerationOverflow)
    );
    assert_eq!(exhausted.snapshot(), last);
    assert_eq!(
        body.apply_mesh_impulse(before.stamp, &load, f64::MAX, |_, _| false),
        Err(RigidImpulseError::ArithmeticFailure)
    );
    assert_eq!(body.snapshot(), before);
}
#[test]
fn physical_inertia_and_body_admission() {
    for inertia in [
        [1., 1., 3.],
        [0., 1., 1.],
        [-1., 1., 1.],
        [f64::NAN, 1., 1.],
        [f64::INFINITY, 1., 1.],
    ] {
        assert!(
            FrozenRigidBody::new(RigidSnapshot {
                inertia_kg_m2: inertia,
                ..state()
            })
            .is_err()
        );
    }
    let zero = RigidSnapshot {
        velocity_m_s: [0.; 3],
        angular_velocity_rad_s: [0.; 3],
        ..state()
    };
    for inertia in [[1., 1., 2.], [f64::MAX; 3], [f64::from_bits(1); 3]] {
        assert!(
            FrozenRigidBody::new(RigidSnapshot {
                inertia_kg_m2: inertia,
                ..zero
            })
            .is_ok()
        );
    }
    // Rounded 1 + tiny would equal 1; exact represented inequality must reject.
    let next = f64::from_bits(1f64.to_bits() + 1);
    assert!(
        FrozenRigidBody::new(RigidSnapshot {
            inertia_kg_m2: [next, 1., f64::EPSILON / 2.],
            ..zero
        })
        .is_err()
    );
    for mass in [0., -1., f64::INFINITY, f64::NAN] {
        assert!(
            FrozenRigidBody::new(RigidSnapshot {
                mass_kg: mass,
                ..zero
            })
            .is_err()
        );
    }
    assert!(
        FrozenRigidBody::new(RigidSnapshot {
            velocity_m_s: [f64::MAX; 3],
            ..state()
        })
        .is_err()
    );
}
#[test]
fn absorbed_increment_is_reported_from_actual_storage() {
    let s = surface();
    let values = [[[0., 0., 3.]; 3]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let initial = RigidSnapshot {
        mass_kg: 1.,
        velocity_m_s: [0., 0., 1e16],
        angular_velocity_rad_s: [0.; 3],
        ..state()
    };
    let mut body = FrozenRigidBody::new(initial).unwrap();
    let r = body
        .apply_mesh_impulse(initial.stamp, &load, 0.01, |_, _| false)
        .unwrap();
    assert_eq!(body.snapshot().velocity_m_s, initial.velocity_m_s);
    assert_eq!(r.momentum_defect[2], -r.impulse_n_s[2]);
    assert_ne!(r.energy_defect_j, 0.);
}
#[test]
fn late_diagnostic_overflow_and_impulse_underflow_are_atomic() {
    let s = surface();
    let values = [[[0., 0., 3.]; 3]];
    let load = TriangleMeshLoad::new(
        &s,
        s.stamp(),
        SurfaceLoading::Traction(&values),
        [0.; 3],
        1,
        |_, _| false,
    )
    .unwrap();
    let mut body = FrozenRigidBody::new(state()).unwrap();
    let before = body.snapshot();
    assert_eq!(
        body.apply_mesh_impulse(before.stamp, &load, f64::from_bits(1), |_, _| false),
        Err(RigidImpulseError::ArithmeticFailure)
    );
    assert_eq!(body.snapshot(), before);
    let initial = RigidSnapshot {
        mass_kg: 1.,
        velocity_m_s: [0.; 3],
        angular_velocity_rad_s: [0.; 3],
        ..state()
    };
    let mut body = FrozenRigidBody::new(initial).unwrap();
    assert_eq!(
        body.apply_mesh_impulse(initial.stamp, &load, 1e154, |_, _| false),
        Err(RigidImpulseError::ArithmeticFailure)
    );
    assert_eq!(body.snapshot(), initial);
}
