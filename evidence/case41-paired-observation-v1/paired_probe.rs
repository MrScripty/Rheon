// Identical include on both sides. Equation, inputs and arithmetic are untouched.
include!("native/expected_baseline.rs");

pub(super) fn paired_layout() -> [usize; 5] {
    [
        std::mem::size_of::<PairedWork<'_>>(),
        std::mem::size_of::<Equation>(),
        std::mem::size_of::<Result<Equation, CoupledDiscreteError>>(),
        std::mem::size_of::<Point>(),
        std::mem::size_of::<[f64; V]>(),
    ]
}

#[inline(never)]
pub(super) fn case41_paired_capture(
    c: &super::FixedCase,
    mut chart: Option<&mut super::scalar::ChartWorkspace>,
    require_e2_baseline: bool,
) {
    assert_eq!(c.index, 41);
    let old = c.accepted.velocity.map(|u| u.map(f64::from_bits));
    let mass = c.accepted.mass.map(f64::from_bits);
    let geometry = FittedHeightWorkspace::new(
        FittedHeightGeometry {
            cap: &[
                [c.q[0], c.q[2]],
                [c.q[1], 2.25 - c.q[2]],
                [c.q[0] + 1., c.q[2]],
            ],
            bottom_x: &BOTTOM,
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        },
        Default::default(),
        |_| false,
    )
    .unwrap();
    for i in 0..19 {
        assert_eq!(
            geometry.nodes()[i].position.map(f64::to_bits),
            c.accepted.positions[i]
        );
    }
    for i in 0..N {
        assert_eq!(geometry.nodal_mass()[i].to_bits(), c.accepted.mass[i]);
    }
    let mut work = paired_new_work(geometry);
    for i in 0..N {
        for d in 0..2 {
            let e = work.geometry.velocity_embedding(i, d).unwrap();
            for k in 0..2 {
                if let Some(j) = e.columns[k] {
                    work.r[i][d][j] = add(work.r[i][d][j], e.weights[k]).unwrap();
                }
            }
        }
    }
    // Both wrappers use the same pre-equation setup; no accepted owner exists.
    let (q, eta) = work.paired_coordinates(&old, chart.as_deref_mut()).unwrap();
    assert_eq!(q.map(f64::to_bits), c.q.map(f64::to_bits));
    assert_eq!(eta.map(f64::to_bits), c.eta.map(f64::to_bits));
    work.paired_attach_chart(chart);
    let mut baseline = [0.; V];
    let mut perturbed = c.unknown;
    let mut values = [0.; V];
    for evaluation in 0usize..7 {
        let column = evaluation.checked_sub(1);
        let delta = if let Some(j) = column {
            mul(1e-6, add(c.unknown[j].abs(), 0.01).unwrap()).unwrap()
        } else {
            0.
        };
        perturbed.copy_from_slice(&c.unknown);
        if let Some(j) = column {
            perturbed[j] = add(perturbed[j], delta).unwrap();
        }
        // This one Result slot is the return destination. Match borrows it;
        // serialization cannot add a full Equation/Point copy to the call peak.
        let result = work.case41_paired_equation(c, &old, &mass, &perturbed);
        match &result {
            Ok(e) => {
                work.case41_serialize(c, &perturbed, column, delta, e);
                if let Some(j) = column {
                    let column_result = (|| -> Result<(), CoupledDiscreteError> {
                        for i in 0..V {
                            values[i] = div(add(e.rate[i], -baseline[i])?, delta)?;
                        }
                        Ok(())
                    })();
                    match column_result {
                        Ok(()) => println!(
                            "{{\"event\":\"fd_column\",\"case\":41,\"column\":{j},\"delta\":{delta:?},\"native_column\":{values:?},\"owners\":0,\"corrections\":0}}"
                        ),
                        Err(error) => println!(
                            "{{\"event\":\"fd_column_unavailable\",\"case\":41,\"column\":{j},\"error\":\"{error:?}\",\"owners\":0,\"corrections\":0}}"
                        ),
                    }
                } else {
                    baseline.copy_from_slice(&e.rate);
                    if require_e2_baseline {
                        // First later authorized baseline must match frozen E2
                        // exactly, before any of the six perturbations execute.
                        assert_eq!(baseline.map(f64::to_bits), FROZEN_E2_RATE_BITS);
                    }
                }
            }
            Err(error) => {
                println!(
                    "{{\"event\":\"fd_equation_refusal\",\"case\":41,\"column\":{},\"delta\":{delta:?},\"unknown\":{perturbed:?},\"error\":\"{error:?}\",\"owners\":0,\"corrections\":0}}",
                    column.map_or(-1, |j| j as i32)
                );
                if column.is_none() {
                    println!(
                        "{{\"event\":\"fd_complete\",\"case\":41,\"baseline_failed\":true,\"equations_attempted\":1,\"owners\":0,\"corrections\":0,\"published\":false}}"
                    );
                    return;
                }
            }
        }
    }
    println!(
        "{{\"event\":\"fd_complete\",\"case\":41,\"baseline_failed\":false,\"equations_attempted\":7,\"owners\":0,\"corrections\":0,\"published\":false}}"
    );
}

impl PairedWork<'_> {
    #[inline(never)]
    fn case41_paired_equation(
        &mut self,
        c: &super::FixedCase,
        old: &[[f64; 3]; N],
        mass: &[f64; N],
        unknown: &[f64; V],
    ) -> Result<Equation, CoupledDiscreteError> {
        self.equation(
            c.q,
            c.eta,
            unknown,
            c.h,
            16,
            old,
            mass,
            c.acceleration,
            &mut |_| false,
        )
    }

    #[inline(never)]
    fn case41_serialize(
        &self,
        _c: &super::FixedCase,
        unknown: &[f64; V],
        column: Option<usize>,
        delta: f64,
        e: &Equation,
    ) {
        // Every original FD observation is retained. Additional stored fields
        // are borrowed too; no point/partition replay or equation reevaluation.
        println!(
            "{{\"event\":\"fd_equation\",\"case\":41,\"order\":16,\"column\":{},\"delta\":{delta:?},\"unknown\":{unknown:?},\"r\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"start_z\":{:?},\"start_mass\":{:?},\"end_z\":{:?},\"end_velocity\":{:?},\"end_mass\":{:?},\"end_force\":{:?},\"end_q\":{:?},\"end_eta\":{:?},\"end_d\":{:?},\"end_b\":{:?},\"pairs\":{:?},\"plus\":{:?},\"minus\":{:?},\"maximum_constraints\":{:?},\"sign_roots\":{},\"start_velocity\":{:?},\"start_q\":{:?},\"start_eta\":{:?},\"start_d\":{:?},\"start_b\":{:?},\"start_force\":{:?},\"start_ddot\":{:?},\"end_ddot\":{:?},\"physical_convection\":{:?},\"endpoint_convection\":{:?},\"owners\":0,\"corrections\":0,\"published\":false}}",
            column.map_or(-1, |j| j as i32),
            self.r,
            e.rate,
            e.direct_rate,
            e.start.z,
            e.start.mass,
            e.end.z,
            e.end.u,
            e.end.mass,
            e.end.force,
            e.end.q,
            e.end.eta,
            e.end.d,
            e.end.b,
            &e.end.pairs[..e.end.faces],
            &e.integral.plus[..e.end.faces],
            &e.integral.minus[..e.end.faces],
            e.integral.max_constraints,
            e.sign_roots,
            e.start.u,
            e.start.q,
            e.start.eta,
            e.start.d,
            e.start.b,
            e.start.force,
            e.start.ddot,
            e.end.ddot,
            e.integral.physical_convection,
            e.endpoint_convection
        );
    }
}
