// Appended observation only: unchanged evaluator, no Newton loop or owner.
include!("native/forecast_bits.rs");
pub(super) fn forecast_input_layout() {
    println!(
        "{{\"event\":\"forecast_input_layout\",\"unknown_buffer_bytes\":{},\"forecast_bits\":{:?}}}",
        std::mem::size_of::<[f64; V]>(),
        FORECAST_BITS
    );
}
#[inline(never)]
pub(super) fn case41_forecast_capture(
    c: &super::FixedCase,
    chart: &mut super::scalar::ChartWorkspace,
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
    let (q, eta) = work.paired_coordinates(&old, Some(&mut *chart)).unwrap();
    assert_eq!(q.map(f64::to_bits), c.q.map(f64::to_bits));
    assert_eq!(eta.map(f64::to_bits), c.eta.map(f64::to_bits));
    work.paired_attach_chart(Some(chart));
    let forecast = FORECAST_BITS.map(f64::from_bits);
    for observation in 0usize..2 {
        let unknown = if observation == 0 {
            &c.unknown
        } else {
            &forecast
        };
        // One reused Result return slot, borrowed serialization after return.
        let result = work.case41_paired_equation(c, &old, &mass, unknown);
        match &result {
            Ok(e) => {
                if observation == 0 {
                    // Mismatch aborts before any forecast evaluator call.
                    assert_eq!(e.rate.map(f64::to_bits), FROZEN_E2_RATE_BITS);
                }
                work.forecast_serialize(c, unknown, observation, e);
            }
            Err(error) => {
                println!(
                    "{{\"event\":\"forecast_refusal\",\"observation\":{observation},\"error\":\"{error:?}\",\"owners\":0,\"controller_corrections\":0,\"published\":false}}"
                );
                // Any refusal is terminal; no forecast after baseline refusal.
                return;
            }
        }
    }
    println!(
        "{{\"event\":\"forecast_complete\",\"observations\":2,\"owners\":0,\"controller_corrections\":0,\"published\":false}}"
    );
}
impl PairedWork<'_> {
    #[inline(never)]
    fn forecast_serialize(
        &self,
        _c: &super::FixedCase,
        unknown: &[f64; V],
        observation: usize,
        e: &Equation,
    ) {
        let role = if observation == 0 {
            "original_baseline"
        } else {
            "single_rounded_coordinate_forecast"
        };
        // Every original stored observation is retained. Additional stored fields
        // are borrowed too; no point/partition replay or equation reevaluation.
        println!(
            "{{\"event\":\"forecast_equation\",\"case\":41,\"order\":16,\"observation\":{observation},\"role\":\"{role}\",\"unknown\":{unknown:?},\"r\":{:?},\"rate\":{:?},\"direct_rate\":{:?},\"start_z\":{:?},\"start_mass\":{:?},\"end_z\":{:?},\"end_velocity\":{:?},\"end_mass\":{:?},\"end_force\":{:?},\"end_q\":{:?},\"end_eta\":{:?},\"end_d\":{:?},\"end_b\":{:?},\"pairs\":{:?},\"plus\":{:?},\"minus\":{:?},\"maximum_constraints\":{:?},\"sign_roots\":{},\"start_velocity\":{:?},\"start_q\":{:?},\"start_eta\":{:?},\"start_d\":{:?},\"start_b\":{:?},\"start_force\":{:?},\"start_ddot\":{:?},\"end_ddot\":{:?},\"physical_convection\":{:?},\"endpoint_convection\":{:?},\"owners\":0,\"corrections\":0,\"published\":false}}",
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
