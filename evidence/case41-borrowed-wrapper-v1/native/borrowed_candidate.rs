// Appended only to the frozen private E2 candidate module in an isolated harness.
// Seven order16 equations: one fresh fixed baseline, six original FD probes.
// Stream one column at a time. No Jacobian array, linear solve or owner exists.
#[inline(never)]
pub(super) fn case41_fd_capture(c: &super::FixedCase, chart: &mut super::scalar::ChartWorkspace) {
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
    let mut work = Work {
        research: None,
        geometry,
        r: [[[0.; V]; 2]; N],
        diagnostic: [Default::default(); N],
        pressure: [0.; P],
    };
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
    let (q, eta) = coordinates(&work.geometry, &old, &work.r, Some(chart)).unwrap();
    assert_eq!(q.map(f64::to_bits), c.q.map(f64::to_bits));
    assert_eq!(eta.map(f64::to_bits), c.eta.map(f64::to_bits));
    work.research = Some(chart);
    let mut result = work.case41_numerical(c, &old, &mass, &c.unknown);
    let mut baseline = [0.; V];
    let mut perturbed = c.unknown;
    let mut values = [0.; V];
    match &result {
        Ok(e) => baseline.copy_from_slice(&e.rate),
        Err(error) => {
            work.case41_serialize(c, &c.unknown, None, 0., &result);
            println!(
                "{{\"event\":\"fd_complete\",\"case\":41,\"baseline_failed\":true,\"error\":{:?},\"equations_attempted\":1,\"corrections\":0,\"owners\":0,\"published\":false}}",
                format!("{error:?}")
            );
            return;
        }
    }
    work.case41_serialize(c, &c.unknown, None, 0., &result);
    for j in 0..6 {
        let delta = mul(1e-6, add(c.unknown[j].abs(), 0.01).unwrap()).unwrap();
        perturbed.copy_from_slice(&c.unknown);
        perturbed[j] = add(perturbed[j], delta).unwrap();
        result = work.case41_numerical(c, &old, &mass, &perturbed);
        work.case41_serialize(c, &perturbed, Some(j), delta, &result);
        match &result {
            Ok(e) => {
                let column = (|| -> Result<(), CoupledDiscreteError> {
                    for i in 0..V {
                        values[i] = div(add(e.rate[i], -baseline[i])?, delta)?;
                    }
                    Ok(())
                })();
                match column {
                    Ok(()) => println!(
                        "{{\"event\":\"fd_column\",\"case\":41,\"column\":{j},\"delta\":{delta:?},\"native_column\":{values:?},\"corrections\":0,\"owners\":0}}"
                    ),
                    Err(error) => println!(
                        "{{\"event\":\"fd_column_unavailable\",\"case\":41,\"column\":{j},\"error\":{:?},\"owners\":0,\"corrections\":0}}",
                        format!("{error:?}")
                    ),
                }
            }
            Err(error) => println!(
                "{{\"event\":\"fd_column_unavailable\",\"case\":41,\"column\":{j},\"error\":{:?},\"owners\":0,\"corrections\":0}}",
                format!("{error:?}")
            ),
        }
    }
    println!(
        "{{\"event\":\"fd_complete\",\"case\":41,\"baseline_failed\":false,\"equations_attempted\":7,\"corrections\":0,\"owners\":0,\"published\":false}}"
    );
}
impl Work<'_> {
    #[inline(never)]
    fn case41_numerical(
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
        result: &Result<Equation, CoupledDiscreteError>,
    ) {
        let e = match result {
            Ok(e) => e,
            Err(error) => {
                println!(
                    "{{\"event\":\"fd_equation_refusal\",\"case\":41,\"column\":{},\"delta\":{delta:?},\"unknown\":{unknown:?},\"error\":{:?},\"owners\":0,\"corrections\":0}}",
                    column.map_or(-1, |j| j as i32),
                    format!("{error:?}")
                );
                return;
            }
        };
        println!(
            "{{\"event\":\"fd_equation\",\"case\":41,\"order\":16,\"column\":{},\"delta\":{delta:?},\"unknown\":{unknown:?},\"r\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"start_z\":{:?},\"start_mass\":{:?},\"end_z\":{:?},\"end_velocity\":{:?},\"end_mass\":{:?},\"end_force\":{:?},\"end_q\":{:?},\"end_eta\":{:?},\"end_d\":{:?},\"end_b\":{:?},\"pairs\":{:?},\"plus\":{:?},\"minus\":{:?},\"maximum_constraints\":{:?},\"sign_roots\":{},\"owners\":0,\"corrections\":0,\"published\":false}}",
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
            e.sign_roots
        );
    }
}
pub(super) fn borrowed_layout() -> [usize; 5] {
    [
        std::mem::size_of::<Work>(),
        std::mem::size_of::<Equation>(),
        std::mem::size_of::<Result<Equation, CoupledDiscreteError>>(),
        std::mem::size_of::<Point>(),
        std::mem::size_of::<Integrals>(),
    ]
}
