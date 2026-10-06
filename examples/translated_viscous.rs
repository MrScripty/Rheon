//! Actual repeated material translation and third-component viscosity. Pressure
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
    let mut flow = TranslatedViscousFlow::new(
        geometry(),
        &initial,
        Default::default(),
        Default::default(),
        29,
    )?;
    drop(initial);
    println!(
        "{{\"scope\":\"restricted translating varying-height flow with Neumann third viscosity and analytic zero pressure\",\"dt\":{dt:?},\"allocated_bytes\":{},\"mass\":{:?},\"nodes\":{:?},\"triangles\":{:?},\"states\":[",
        flow.allocated_bytes(),
        flow.state().template.nodal_mass(),
        flow.state()
            .template
            .nodes()
            .iter()
            .map(|n| [n.position[0], n.position[1], n.periodic_index as f64])
            .collect::<Vec<_>>(),
        flow.state()
            .template
            .triangles()
            .iter()
            .map(|t| t.nodes)
            .collect::<Vec<_>>()
    );
    print_state(&flow);
    for i in 0..steps {
        // All four cancellation stages exercise the same accepted state at
        // every step. Failure never publishes a geometry/velocity half-state.
        for stage in [
            TranslatedViscousStage::BeforeSolve,
            TranslatedViscousStage::Iteration,
            TranslatedViscousStage::BeforeAcceptance,
            TranslatedViscousStage::BeforePublish,
        ] {
            let before = flow.state();
            let stamp = before.stamp;
            let time = before.time;
            let offset = before.offset;
            let velocity = before.velocity.to_vec();
            let error = flow.step(dt, |s| s == stage).unwrap_err();
            if error != (TranslatedViscousError::Cancelled { stage })
                || flow.state().stamp != stamp
                || flow.state().time != time
                || flow.state().offset != offset
                || flow.state().velocity != velocity
            {
                return Err("rollback failed".into());
            }
        }
        let r = flow.step(dt, |_| false)?;
        println!(
            ",{{\"report\":{{\"mass\":{:?},\"momentum_before\":{:?},\"momentum_after\":{:?},\"energy_before\":{:?},\"energy_after\":{:?},\"increment_energy\":{:?},\"strain_power\":{:?},\"residual\":{:?},\"residual_work\":{:?},\"work_error\":{:?},\"divergence\":{:?},\"iterations\":{}}},\"state\":",
            r.liquid_mass,
            r.momentum_before,
            r.momentum_after,
            r.energy_before,
            r.energy_after,
            r.increment_energy,
            r.strain_power,
            r.true_residual,
            r.residual_work,
            r.work_identity_error,
            r.divergence_max,
            r.iterations
        );
        print_state(&flow);
        print!("}}");
        if i + 1 == steps {
            println!();
        }
    }
    println!(
        "],\"rollback_trials\":{},\"pressure_model\":\"analytic zero; no pressure iteration\"}}",
        4 * steps
    );
    Ok(())
}
fn print_state(flow: &TranslatedViscousFlow) {
    let s = flow.state();
    print!(
        "{{\"version\":{},\"time\":{:?},\"offset\":{:?},\"pressure\":{:?},\"velocity\":{:?},\"liquid_volume\":{:?},\"physical_nodes\":{:?}}}",
        s.stamp.version,
        s.time,
        s.offset,
        s.relative_pressure(),
        s.velocity,
        (0..s.template.nodal_mass().len())
            .map(|i| s.nodal_liquid_volume(i).unwrap())
            .collect::<Vec<_>>(),
        (0..s.template.nodes().len())
            .map(|i| s.physical_node(i).unwrap())
            .collect::<Vec<_>>()
    );
}
