//! Actual repeated fixed-bottom ALE transport and third-component viscosity. Pressure
//! is analytically zero for this invariant family, not a general pressure solve.
use rheon::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dt: f64 = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "0.025".into())
        .parse()?;
    let steps: usize = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "20".into())
        .parse()?;
    let cap = [[0., 1.], [0.25, 1.25], [0.5, 1.], [0.75, 1.5], [1., 1.]];
    let bottom = [0., 0.25, 0.5, 0.75, 1.];
    let geometry = || FittedHeightGeometry {
        cap: &cap,
        bottom_x: &bottom,
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: 0.05,
    };
    let frame = FittedHeightWorkspace::new(geometry(), Default::default(), |_| false)?;
    let mut reduced = vec![0.; frame.plan().reduced_velocity_unknowns];
    for n in frame.nodes() {
        for d in [0, 2] {
            let row = frame.velocity_embedding(n.periodic_index, d).unwrap();
            if row.weights[0] == 1.
                && let Some(j) = row.columns[0]
            {
                reduced[j] = if d == 0 {
                    0.25
                } else {
                    n.position[1] * n.position[1]
                };
            }
        }
    }
    let mut initial = vec![[0.; 3]; frame.plan().periodic_nodes];
    frame.embed_velocity(&reduced, &mut initial)?;
    drop(frame);
    drop(reduced);
    let mut flow = FixedBottomAleFlow::new(
        geometry(),
        &initial,
        Default::default(),
        Default::default(),
        29,
    )?;
    drop(initial);
    println!(
        "{{\"scope\":\"fixed-bottom ALE: actual nonzero relative transport, constrained donor and third viscosity; analytic zero pressure\",\"dt\":{dt:?},\"allocated_bytes\":{},\"nodes\":{:?},\"triangles\":{:?},\"states\":[",
        flow.allocated_bytes(),
        flow.state()
            .geometry
            .nodes()
            .iter()
            .map(|n| [n.position[0], n.position[1], n.periodic_index as f64])
            .collect::<Vec<_>>(),
        flow.state()
            .geometry
            .triangles()
            .iter()
            .map(|t| t.nodes)
            .collect::<Vec<_>>()
    );
    print_state(&flow);
    for i in 0..steps {
        // Exercise all six cancellation stages after the first accepted step.
        // Failure never publishes a geometry/velocity half-state.
        if i == 1 {
            for stage in [
                FixedBottomAleStage::BeforeGeometry,
                FixedBottomAleStage::Quadrature,
                FixedBottomAleStage::BeforeSolve,
                FixedBottomAleStage::Iteration,
                FixedBottomAleStage::BeforeAcceptance,
                FixedBottomAleStage::BeforePublish,
            ] {
                let before = flow.state();
                let stamp = before.stamp;
                let time = before.time;
                let offset = before.cap_offset;
                let velocity = before.velocity.to_vec();
                let nodes = before
                    .geometry
                    .nodes()
                    .iter()
                    .map(|n| n.position)
                    .collect::<Vec<_>>();
                let masses = before.geometry.nodal_mass().to_vec();
                let error = flow.step(dt, |s| s == stage).unwrap_err();
                let after = flow.state();
                if error != (FixedBottomAleError::Cancelled { stage })
                    || after.stamp != stamp
                    || after.time != time
                    || after.cap_offset != offset
                    || after.velocity != velocity
                    || after.geometry.nodal_mass() != masses
                    || after
                        .geometry
                        .nodes()
                        .iter()
                        .map(|n| n.position)
                        .collect::<Vec<_>>()
                        != nodes
                {
                    return Err("accepted joint state changed on cancellation".into());
                }
            }
        }
        let actual_dt = dt.min((0.125 - flow.state().cap_offset) / 0.25);
        let r = flow.step(actual_dt, |_| false)?;
        println!(
            ",{{\"report\":{{\"dt\":{:?},\"mass_before\":{:?},\"mass_after\":{:?},\"momentum_before\":{:?},\"momentum_after\":{:?},\"energy_before\":{:?},\"energy_after\":{:?},\"increment_loss\":{:?},\"advection_loss\":{:?},\"strain_power\":{:?},\"residual\":{:?},\"residual_work\":{:?},\"gcl_work\":{:?},\"work_error\":{:?},\"gcl_max\":{:?},\"quadrature_error\":{:?},\"relative_transfer_l1\":{:?},\"divergence\":{:?},\"iterations\":{}}},\"transfers\":{:?},\"state\":",
            r.dt,
            r.mass_before,
            r.mass_after,
            r.momentum_before,
            r.momentum_after,
            r.energy_before,
            r.energy_after,
            r.increment_loss,
            r.advection_loss,
            r.strain_power,
            r.true_residual,
            r.residual_work,
            r.gcl_work,
            r.work_error,
            r.gcl_max,
            r.quadrature_error_max,
            r.relative_transfer_l1,
            r.divergence_max,
            r.iterations,
            flow.transfer_scratch()
                .iter()
                .map(|f| [
                    f.nodes[0] as f64,
                    f.nodes[1] as f64,
                    f.positive,
                    f.negative,
                    f.coarse_positive,
                    f.coarse_negative
                ])
                .collect::<Vec<_>>()
        );
        print_state(&flow);
        print!("}}");
        if i + 1 == steps {
            println!();
        }
    }
    println!(
        "],\"rollback_trials\":{},\"pressure_model\":\"analytic zero; no pressure iteration\"}}",
        if steps > 1 { 6 } else { 0 }
    );
    Ok(())
}
fn print_state(flow: &FixedBottomAleFlow) {
    let s = flow.state();
    print!(
        "{{\"version\":{},\"time\":{:?},\"offset\":{:?},\"pressure\":{:?},\"velocity\":{:?},\"mass\":{:?},\"liquid_volume\":{:?},\"physical_nodes\":{:?}}}",
        s.stamp.version,
        s.time,
        s.cap_offset,
        s.relative_pressure(),
        s.velocity,
        s.geometry.nodal_mass(),
        (0..s.geometry.nodal_mass().len())
            .map(|i| s.nodal_liquid_volume(i).unwrap())
            .collect::<Vec<_>>(),
        (0..s.geometry.nodes().len())
            .map(|i| s.physical_node(i).unwrap())
            .collect::<Vec<_>>()
    );
}
