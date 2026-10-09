// Appended observation code: unchanged equation/point implementations above.
#[inline(never)]
pub(super) fn fixed_capture(c: &super::FixedCase, chart: &mut super::scalar::ChartWorkspace) {
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
    let (q, eta) = coordinates(&work.geometry, &old, &work.r).unwrap();
    let _ = chart;
    assert_eq!(q.map(f64::to_bits), c.q.map(f64::to_bits));
    assert_eq!(eta.map(f64::to_bits), c.eta.map(f64::to_bits));

    work.capture_equations(c, &old, &mass);
    println!(
        "{{\"event\":\"fixed_capture_complete\",\"index\":{},\"mode\":\"native-reference\",\"owner_advances\":0,\"controller_iterations\":0,\"equation_evaluations\":2,\"published\":false}}",
        c.index
    );
}
impl Work {
    #[inline(never)]
    fn capture_equations(&mut self, c: &super::FixedCase, old: &[[f64; 3]; N], mass: &[f64; N]) {
        let coarse = self.equation(
            c.q,
            c.eta,
            &c.unknown,
            c.h,
            16,
            old,
            mass,
            c.acceleration,
            &mut |_| false,
        );
        let fine = self.equation(
            c.q,
            c.eta,
            &c.unknown,
            c.h,
            32,
            old,
            mass,
            c.acceleration,
            &mut |_| false,
        );
        if let (Ok(coarse), Ok(fine)) = (&coarse, &fine) {
            let after = TranslatedViscousStamp {
                id: c.accepted.stamp.id,
                version: c.accepted.stamp.version + 1,
            };
            let gates = qualify(
                fine,
                coarse,
                c.h,
                c.unknown,
                c.accepted.stamp,
                after,
                add(f64::from_bits(c.accepted.time), c.h).unwrap(),
                0,
                2,
                c.acceleration,
            );
            println!(
                "{{\"event\":\"fixed_planar_gates\",\"index\":{},\"mode\":\"native-reference\",\"result\":{:?},\"candidate_metadata_only\":true,\"third_solve_executed\":false,\"owner_advances\":0,\"published\":false}}",
                c.index,
                format!("{gates:?}")
            );
        }
        for (order, result) in [(16, &coarse), (32, &fine)] {
            let e = match result {
                Ok(e) => e,
                Err(error) => {
                    println!(
                        "{{\"event\":\"fixed_equation_refusal\",\"index\":{},\"mode\":\"native-reference\",\"order\":{},\"error\":{:?},\"owner_advances\":0,\"published\":false}}",
                        c.index,
                        order,
                        format!("{error:?}")
                    );
                    continue;
                }
            };
            let rate_norm = norm(&e.rate).unwrap();
            println!(
                "{{\"event\":\"fixed_equation\",\"index\":{},\"mode\":\"native-reference\",\"order\":{},\"r\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"norm\":{:?},\"direct_norm\":{:?},\"start_z\":{:?},\"start_velocity\":{:?},\"start_mass\":{:?},\"end_z\":{:?},\"end_velocity\":{:?},\"end_mass\":{:?},\"end_force\":{:?},\"end_q\":{:?},\"end_eta\":{:?},\"end_d\":{:?},\"end_b\":{:?},\"pairs\":{:?},\"plus\":{:?},\"minus\":{:?},\"physical_convection\":{:?},\"endpoint_convection\":{:?},\"maximum_constraints\":{:?},\"sign_roots\":{},\"owner_advances\":0,\"published\":false}}",
                c.index,
                order,
                self.r,
                e.rate,
                e.direct_rate,
                rate_norm,
                norm(&e.direct_rate).unwrap(),
                e.start.z,
                e.start.u,
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
                e.integral.physical_convection,
                e.endpoint_convection,
                e.integral.max_constraints,
                e.sign_roots
            );
            // Point capture does not change the frozen candidate or accepted data.
            for (label, time) in [("start", 0.), ("end", c.h)] {
                let p = self
                    .point(c.q, c.eta, &c.unknown, time, &mut |_| false)
                    .unwrap();
                self.capture_point(c, order, label, time, 0., &p);
            }
            let (partition, count) = self
                .partition(c.q, c.eta, &c.unknown, c.h, &mut |_| false)
                .unwrap();
            assert!(count <= 66);
            println!(
                "{{\"event\":\"fixed_partition\",\"index\":{},\"mode\":\"native-reference\",\"order\":{},\"partition\":{:?}}}",
                c.index,
                order,
                &partition[..count]
            );
            for segment in 0..count - 1 {
                let width = add(partition[segment + 1], -partition[segment]).unwrap();
                for k in 0..order {
                    let (node, weight) = rule(order, k).unwrap();
                    let time = add(
                        partition[segment],
                        mul(mul(width, 0.5).unwrap(), add(1., node).unwrap()).unwrap(),
                    )
                    .unwrap();
                    let factor = mul(mul(width, 0.5).unwrap(), weight).unwrap();
                    let p = self
                        .point(c.q, c.eta, &c.unknown, time, &mut |_| false)
                        .unwrap();
                    self.capture_point(c, order, "quadrature", time, factor, &p);
                }
            }
        }
    }
    #[inline(never)]
    fn capture_point(
        &self,
        c: &super::FixedCase,
        order: usize,
        label: &str,
        time: f64,
        factor: f64,
        p: &Point,
    ) {
        for k in 0..6 {
            assert_eq!(
                p.eta[k].to_bits(),
                add(c.eta[k], mul(time, c.unknown[k]).unwrap())
                    .unwrap()
                    .to_bits()
            );
        }
        for (k, &j) in KNOWN.iter().enumerate() {
            let value = if k == 6 {
                -p.eta[COORD[k]]
            } else {
                p.eta[COORD[k]]
            };
            assert_eq!(p.z[j].to_bits(), value.to_bits());
        }
        // Debug structures contain only fixed names and numeric arrays, so
        // stream them inside JSON strings without allocating native snapshots.
        print!(
            "{{\"event\":\"fixed_point\",\"index\":{},\"mode\":\"native-reference\",\"order\":{},\"label\":\"{}\",\"time\":{:?},\"factor\":{:?},\"q\":{:?},\"eta\":{:?},\"z\":{:?},\"velocity\":{:?},\"mass\":{:?},\"d\":{:?},\"force\":{:?},\"flux\":{:?},\"pairs\":{:?},\"diagnostic\":\"{:?}\",\"triangles\":\"{:?}\",\"pressure_terms\":\"{:?}\",\"chart_endpoint_parts\":[",
            c.index,
            order,
            label,
            time,
            factor,
            p.q,
            p.eta,
            p.z,
            p.u,
            p.mass,
            p.d,
            p.force,
            &p.flux[..p.faces],
            &p.pairs[..p.faces],
            self.diagnostic,
            self.geometry.triangles(),
            self.geometry.pressure_basis()
        );

        println!("],\"owner_advances\":0,\"published\":false}}");
    }
}
