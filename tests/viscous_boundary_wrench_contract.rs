#[allow(dead_code)]
#[path = "../examples/viscous_boundary_wrench.rs"]
mod fixture;
use fixture::{axis, fields, geometry, rigid_component, CAP, OUTER_TWIST, SOLID_TWIST};
use rheon::{
    AlignedStrain, AlignedStrainBoundary, AlignedStrainError, AlignedViscousBoundaryWrench,
    ObstacleFlowError, ObstacleFlowStage,
};
fn close(a: f64, b: f64) {
    assert!(
        (a - b).abs() <= 2e-11 * (1.0 + a.abs() + b.abs()),
        "{a} != {b}"
    );
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
#[test]
fn stationary_force_is_exactly_the_existing_action_and_virtual_work_matches() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    let w = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], CAP, |_, _| false).unwrap();
    for (name, u) in fields(&op, [1.5; 3]) {
        let mut expected = vec![0.0; u.len()];
        let old = op.diagnose(&u, &mut expected, |_, _| false).unwrap();
        let mut actual = vec![0.0; u.len()];
        let report = w.diagnose(&u, &mut actual, |_, _| false).unwrap();
        assert_eq!(report.surface_stamp, owner.stamp());
        assert_eq!(report.reference, w.reference());
        assert_eq!(report.density, op.density());
        assert_eq!(report.viscosity, op.viscosity());
        assert_eq!(
            actual.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            expected.iter().map(|x| x.to_bits()).collect::<Vec<_>>(),
            "{name}"
        );
        assert_eq!(report.dissipation.to_bits(), old.dissipation.to_bits());
        assert_eq!(report.force_work.to_bits(), old.force_work.to_bits());
        assert_eq!(report.work_defect.to_bits(), old.identity_error.to_bits());
        for d in report.balance_defect {
            close(d, 0.0);
        }
        let v = w
            .virtual_boundary_work(&u, SOLID_TWIST, OUTER_TWIST, |_, _| false)
            .unwrap();
        close(v.wrench_work, v.row_work);
        close(v.defect, 0.0);
    }
}
#[test]
fn stationary_force_and_wrench_are_density_independent_and_scale_with_viscosity() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    let dense = AlignedStrain::new(&owner, 7.25, 1.0, CAP, |_, _| false).unwrap();
    let double = AlignedStrain::new(&owner, 7.25, 2.0, CAP, |_, _| false).unwrap();
    let reference = [1.5; 3];
    let w = AlignedViscousBoundaryWrench::new(&op, reference, CAP, |_, _| false).unwrap();
    let wd = AlignedViscousBoundaryWrench::new(&dense, reference, CAP, |_, _| false).unwrap();
    let wm = AlignedViscousBoundaryWrench::new(&double, reference, CAP, |_, _| false).unwrap();
    for (_, u) in fields(&op, reference) {
        let mut force = vec![0.0; u.len()];
        let mut dense_force = force.clone();
        let mut double_force = force.clone();
        let r = w.diagnose(&u, &mut force, |_, _| false).unwrap();
        let rd = wd.diagnose(&u, &mut dense_force, |_, _| false).unwrap();
        let rm = wm.diagnose(&u, &mut double_force, |_, _| false).unwrap();
        for ((f, fd), fm) in force.iter().zip(&dense_force).zip(&double_force) {
            assert_eq!(f.to_bits(), fd.to_bits());
            assert_eq!((2.0 * f).to_bits(), fm.to_bits());
        }
        for (a, b, c) in [
            (r.solid_wrench, rd.solid_wrench, rm.solid_wrench),
            (r.outer_wrench, rd.outer_wrench, rm.outer_wrench),
            (r.fluid_wrench, rd.fluid_wrench, rm.fluid_wrench),
        ] {
            for k in 0..6 {
                assert_eq!(a[k].to_bits(), b[k].to_bits());
                assert_eq!((2.0 * a[k]).to_bits(), c[k].to_bits());
            }
        }
        for (a, b, c) in [
            (r.dissipation, rd.dissipation, rm.dissipation),
            (r.force_work, rd.force_work, rm.force_work),
            (r.work_defect, rd.work_defect, rm.work_defect),
        ] {
            assert_eq!(a.to_bits(), b.to_bits());
            assert_eq!((2.0 * a).to_bits(), c.to_bits());
        }
    }
}
#[test]
fn normal_empty_rows_restore_solid_and_outer_virtual_traces() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    let w = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], CAP, |_, _| false).unwrap();
    let mut count = 0;
    for (r, lift) in op.rows().iter().zip(w.rows()) {
        if r.boundary == AlignedStrainBoundary::Normal && r.terms().is_empty() {
            count += 1;
            assert!(lift.solid[..3].iter().any(|v| *v != 0.0));
            assert!(lift.outer[..3].iter().any(|v| *v != 0.0));
            for k in 0..6 {
                assert_eq!(lift.solid[k] + lift.outer[k], 0.0);
            }
        }
    }
    assert_eq!(count, 6);
}
#[test]
fn detached_report_retains_actual_large_stamp_reference_and_material() {
    let stamp = rheon::SurfaceStamp {
        id: u64::MAX,
        version: u64::MAX - 1,
    };
    let owner =
        fixture::geometry_with_stamp([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3], stamp).unwrap();
    let op = AlignedStrain::new(&owner, 7.25, 0.625, CAP, |_, _| false).unwrap();
    let reference = [-0.25, 2.5, 0.75];
    let w = AlignedViscousBoundaryWrench::new(&op, reference, CAP, |_, _| false).unwrap();
    for (_, u) in fields(&op, reference) {
        let mut force = vec![0.0; u.len()];
        let report = w.diagnose(&u, &mut force, |_, _| false).unwrap();
        assert_eq!(report.surface_stamp, owner.stamp());
        assert_eq!(report.surface_stamp, stamp);
        assert_eq!(report.reference, reference);
        assert_eq!(report.density, op.density());
        assert_eq!(report.density, 7.25);
        assert_eq!(report.viscosity, op.viscosity());
        assert_eq!(report.viscosity, 0.625);
    }
}
#[test]
fn rigid_sample_cancels_only_the_restored_flat_and_corner_lifts() {
    for (h, origin) in [
        ([1.0; 3], [0.0; 3]),
        ([0.5, 0.75, 1.25], [0.0; 3]),
        ([0.3, 0.7, 1.1], [1e12, -1e12, 0.1]),
    ] {
        let owner = geometry([3; 3], h, origin, [1; 3], [2; 3]).unwrap();
        let op = AlignedStrain::new(&owner, 2.0, 0.375, CAP, |_, _| false).unwrap();
        let (lo, hi) = owner.box_bounds();
        let reference = std::array::from_fn(|d| lo[d] + 0.5 * (hi[d] - lo[d]));
        let w = AlignedViscousBoundaryWrench::new(&op, reference, CAP, |_, _| false).unwrap();
        let mut corners = std::collections::BTreeSet::new();
        for (r, lift) in op.rows().iter().zip(w.rows()) {
            if matches!(
                r.boundary,
                AlignedStrainBoundary::ObstacleFlat | AlignedStrainBoundary::ObstacleCorner
            ) {
                if r.boundary == AlignedStrainBoundary::ObstacleCorner {
                    corners.insert(([axis(r.axes[0]), axis(r.axes[1])], r.coordinates));
                }
                for k in 0..6 {
                    let mut twist = [0.0; 6];
                    twist[k] = 1.0;
                    let er: f64 = r
                        .terms()
                        .iter()
                        .map(|t| {
                            let f = &op.active_faces()[t.active];
                            t.coefficient
                                * rigid_component(axis(f.axis), f.position, reference, twist)
                        })
                        .sum();
                    close(er + lift.solid[k], 0.0);
                    assert_eq!(lift.outer[k], 0.0);
                }
            }
        }
        assert_eq!(corners.len(), 12);
        let u = fields(&op, reference)
            .into_iter()
            .find(|(name, _)| name == "rigid-sample")
            .unwrap()
            .1;
        let mut force = vec![0.0; u.len()];
        assert!(
            w.diagnose(&u, &mut force, |_, _| false)
                .unwrap()
                .dissipation
                > 0.0
        );
    }
}
#[test]
fn reference_covariance_preserves_force_and_changes_torque_by_moment_arm() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    let a = [1.5; 3];
    let b = [-0.25, 2.5, 0.75];
    let d = std::array::from_fn(|i| b[i] - a[i]);
    let wa = AlignedViscousBoundaryWrench::new(&op, a, CAP, |_, _| false).unwrap();
    let wb = AlignedViscousBoundaryWrench::new(&op, b, CAP, |_, _| false).unwrap();
    let u = fields(&op, a)
        .into_iter()
        .find(|(name, _)| name == "crosscomponent")
        .unwrap()
        .1;
    let mut fa = vec![0.0; u.len()];
    let mut fb = fa.clone();
    let ra = wa.diagnose(&u, &mut fa, |_, _| false).unwrap();
    let rb = wb.diagnose(&u, &mut fb, |_, _| false).unwrap();
    assert_eq!(fa, fb);
    assert_eq!(ra.dissipation, rb.dissipation);
    for (x, y) in [
        (ra.solid_wrench, rb.solid_wrench),
        (ra.outer_wrench, rb.outer_wrench),
        (ra.fluid_wrench, rb.fluid_wrench),
    ] {
        let arm = cross(d, [x[0], x[1], x[2]]);
        for k in 0..3 {
            assert_eq!(x[k], y[k]);
            close(y[k + 3], x[k + 3] - arm[k]);
        }
    }
}
#[test]
fn constant_translation_and_local_rotation_are_not_stationary_global_null_modes() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    let w = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], CAP, |_, _| false).unwrap();
    for (name, u) in fields(&op, [1.5; 3]) {
        if name == "translation" || name == "compatible-local-rotation" {
            let mut force = vec![0.0; u.len()];
            let r = w.diagnose(&u, &mut force, |_, _| false).unwrap();
            assert!(r.dissipation > 0.0);
            if name == "translation" {
                assert!(r.solid_wrench[0] != 0.0);
            } else {
                let local: Vec<_> = op
                    .rows()
                    .iter()
                    .filter(|r| {
                        r.axes == [rheon::Axis::X, rheon::Axis::Y] && r.coordinates == [1, 1, 0]
                    })
                    .collect();
                assert_eq!(local.len(), 4);
                for row in local {
                    assert_eq!(
                        row.terms()
                            .iter()
                            .map(|t| t.coefficient * u[t.active])
                            .sum::<f64>(),
                        0.0
                    );
                }
            }
        }
    }
}
#[test]
fn combined_capacity_and_each_used_cancellation_stage_refuse() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    let w = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], CAP, |_, _| false).unwrap();
    let cap = w.combined_operator_bytes();
    assert_eq!(std::mem::size_of::<rheon::ViscousBoundaryLift>(), 96);
    assert_eq!(w.row_capacity(), op.rows().len());
    assert_eq!(
        w.allocated_bytes(),
        w.row_capacity() * std::mem::size_of::<rheon::ViscousBoundaryLift>()
    );
    assert_eq!(cap, op.allocated_bytes() + w.allocated_bytes());
    assert!(AlignedViscousBoundaryWrench::new(&op, [1.5; 3], cap, |_, _| false).is_ok());
    assert!(matches!(
        AlignedViscousBoundaryWrench::new(&op, [1.5; 3], cap - 1, |_, _| false),
        Err(AlignedStrainError::Flow(
            ObstacleFlowError::BufferLimit { .. }
        ))
    ));
    assert!(matches!(
        AlignedViscousBoundaryWrench::new(&op, [1.5; 3], cap, |stage, _| stage
            == ObstacleFlowStage::Assembly),
        Err(AlignedStrainError::Flow(ObstacleFlowError::Cancelled {
            stage: ObstacleFlowStage::Assembly,
            ..
        }))
    ));
    let u = vec![1.0; op.active_faces().len()];
    for stage in [ObstacleFlowStage::Correction, ObstacleFlowStage::Acceptance] {
        let mut force = vec![0.0; u.len()];
        assert!(
            matches!(w.diagnose(&u,&mut force,|s,_|s==stage),Err(AlignedStrainError::Flow(ObstacleFlowError::Cancelled{stage:s,..})) if s==stage)
        );
        assert!(
            matches!(w.virtual_boundary_work(&u,SOLID_TWIST,OUTER_TWIST,|s,_|s==stage),Err(AlignedStrainError::Flow(ObstacleFlowError::Cancelled{stage:s,..})) if s==stage)
        );
    }
}
#[test]
fn malformed_nonfinite_overflow_and_nonzero_underflow_refuse() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
    assert!(matches!(
        AlignedViscousBoundaryWrench::new(&op, [f64::NAN, 0.0, 0.0], CAP, |_, _| false),
        Err(AlignedStrainError::Flow(ObstacleFlowError::NonFiniteInput))
    ));
    let w = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], CAP, |_, _| false).unwrap();
    let mut force = vec![0.0; op.active_faces().len()];
    assert!(matches!(
        w.diagnose(&[], &mut force, |_, _| false),
        Err(AlignedStrainError::Flow(ObstacleFlowError::ShapeMismatch))
    ));
    for v in [f64::NAN, f64::INFINITY] {
        let u = vec![v; force.len()];
        assert!(matches!(
            w.diagnose(&u, &mut force, |_, _| false),
            Err(AlignedStrainError::Flow(ObstacleFlowError::NonFiniteInput))
        ));
    }
    for v in [f64::MAX, 2.0_f64.powi(-600)] {
        let u = vec![v; force.len()];
        assert!(matches!(
            w.diagnose(&u, &mut force, |_, _| false),
            Err(AlignedStrainError::Flow(
                ObstacleFlowError::ArithmeticFailure
            ))
        ));
    }
    assert!(matches!(
        w.virtual_boundary_work(
            &vec![0.0; force.len()],
            [f64::INFINITY; 6],
            OUTER_TWIST,
            |_, _| false
        ),
        Err(AlignedStrainError::Flow(ObstacleFlowError::NonFiniteInput))
    ));
}
#[test]
fn inherited_alignment_admission_refuses_unaligned_geometry() {
    let owner = geometry([3; 3], [1.0; 3], [0.0; 3], [1; 3], [2; 3]).unwrap();
    let vertices = vec![
        [1.1, 1.0, 1.0],
        [2.0, 1.0, 1.0],
        [1.1, 2.0, 1.0],
        [2.0, 2.0, 1.0],
        [1.1, 1.0, 2.0],
        [2.0, 1.0, 2.0],
        [1.1, 2.0, 2.0],
        [2.0, 2.0, 2.0],
    ];
    let surface = rheon::TriangleSurface::new(
        rheon::SurfaceStamp { id: 99, version: 1 },
        vertices,
        vec![
            [0, 2, 3],
            [0, 3, 1],
            [4, 5, 7],
            [4, 7, 6],
            [0, 1, 5],
            [0, 5, 4],
            [2, 6, 7],
            [2, 7, 3],
            [0, 4, 6],
            [0, 6, 2],
            [1, 3, 7],
            [1, 7, 5],
        ],
        rheon::SurfaceSettings::default(),
    )
    .unwrap();
    let unaligned =
        rheon::StaticObstacleGeometry::new(owner.grid().clone(), surface, CAP, |_, _| false)
            .unwrap();
    assert!(matches!(
        AlignedStrain::new(&unaligned, 1.0, 1.0, CAP, |_, _| false),
        Err(AlignedStrainError::UnsupportedGeometry)
    ));
}
#[test]
fn polynomial_specimens_expose_the_coarse_sampling_limit_without_accuracy_claims() {
    for (counts, spacing, lo, hi) in [
        ([3; 3], [1.0; 3], [1; 3], [2; 3]),
        ([6; 3], [0.5; 3], [2; 3], [4; 3]),
    ] {
        let owner = geometry(counts, spacing, [0.0; 3], lo, hi).unwrap();
        assert_eq!(owner.box_bounds(), ([1.0; 3], [2.0; 3]));
        let op = AlignedStrain::new(&owner, 1.0, 1.0, CAP, |_, _| false).unwrap();
        let w = AlignedViscousBoundaryWrench::new(&op, [1.5; 3], CAP, |_, _| false).unwrap();
        for (name, u) in fields(&op, [1.5; 3]) {
            if name.starts_with("polynomial-") {
                let mut force = vec![0.0; u.len()];
                let report = w.diagnose(&u, &mut force, |_, _| false).unwrap();
                if counts == [3; 3] {
                    assert!(u.iter().all(|v| *v == 0.0));
                    assert_eq!(report.solid_wrench, [0.0; 6]);
                    assert_eq!(report.dissipation, 0.0);
                    // Both continuum polynomial specimens have nonzero torque;
                    // these zero discrete samples cannot resolve that load.
                } else {
                    assert!(report.dissipation > 0.0);
                    assert!(report.solid_wrench.iter().any(|v| *v != 0.0));
                }
            }
        }
    }
}
