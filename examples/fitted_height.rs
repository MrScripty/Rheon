//! Machine-readable native instantaneous assembly; deliberately no time stepping.
use rheon::*;
fn field(w: &FittedHeightWorkspace, f: impl Fn([f64; 2]) -> [f64; 3]) -> Vec<[f64; 3]> {
    let mut z = vec![0.0; w.plan().reduced_velocity_unknowns];
    for n in w.nodes() {
        for d in 0..3 {
            let row = w.velocity_embedding(n.periodic_index, d).unwrap();
            if row.weights[0] == 1.0
                && let Some(col) = row.columns[0]
            {
                z[col] = f(n.position)[d];
            }
        }
    }
    let mut u = vec![[0.0; 3]; w.plan().periodic_nodes];
    w.embed_velocity(&z, &mut u).unwrap();
    u
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cap = [
        [0.0, 1.0],
        [0.25, 1.25],
        [0.5, 1.0],
        [0.75, 1.5],
        [1.0, 1.0],
    ];
    let bottom = [0.0, 0.25, 0.5, 0.75, 1.0];
    let mut w = FittedHeightWorkspace::new(
        FittedHeightGeometry {
            cap: &cap,
            bottom_x: &bottom,
            extrusion_width: 1.0,
            density: 3.0,
            dynamic_viscosity: 0.5,
        },
        FittedHeightSettings::default(),
        |_| false,
    )?;
    let plan = w.plan();
    println!(
        "{{\"scope\":\"native instantaneous fitted assembly; no physical time advance\",\"allocated_bytes\":{},\"pressure_modes\":{},\"pressure_terms\":{},\"pivot_ratio\":{:.17e},",
        w.allocated_bytes(),
        plan.pressure_modes,
        w.pressure_basis().len(),
        w.pivot_ratio()
    );
    println!("\"nodes\":[");
    for (i, n) in w.nodes().iter().enumerate() {
        println!(
            "[{:?},{:?},{}]{}",
            n.position[0],
            n.position[1],
            n.periodic_index,
            if i + 1 == w.nodes().len() { "" } else { "," }
        );
    }
    println!(
        "],\"triangles\":{:?},\"mass\":{:?},\"samples\":[",
        w.triangles().iter().map(|t| t.nodes).collect::<Vec<_>>(),
        w.nodal_mass()
    );
    for case in 0..2 {
        let u = if case == 0 {
            field(&w, |[_, y]| [0.25, 0.0, y * y])
        } else {
            field(&w, |[_, y]| [0.0, y, 0.125])
        };
        let p = (0..plan.pressure_modes)
            .map(|i| {
                if case == 0 {
                    0.0
                } else {
                    ((i * 7) % 13) as f64 / 11.0 - 0.5
                }
            })
            .collect::<Vec<_>>();
        let mut out = vec![FittedHeightNodeDiagnostic::default(); plan.periodic_nodes];
        let r = w.inspect(
            FittedHeightInputs {
                velocity: &u,
                pressure_coefficients: &p,
            },
            &mut out,
            |_| false,
        )?;
        println!(
            "{{\"case\":\"{}\",\"volume\":{:.17e},\"mass\":{:.17e},\"divergence_max\":{:.17e},\"continuity_max\":{:.17e},\"geometric_identity_error\":{:.17e},\"strain_power\":{:.17e},\"advection_dissipation\":{:.17e},\"pressure_work\":{:.17e},\"pressure_adjoint_error\":{:.17e},\"strain_work_error\":{:.17e},\"convection_work_error\":{:.17e},\"total_mass_rate\":{:.17e},",
            if case == 0 {
                "translation-third"
            } else {
                "divergence-defect-pressure-work"
            },
            r.liquid_volume,
            r.total_mass,
            r.divergence_max,
            r.continuity_defect_max,
            r.geometric_identity_error,
            r.strain_power,
            r.advection_dissipation,
            r.pressure_work,
            r.pressure_adjoint_error,
            r.strain_work_error,
            r.convection_work_error,
            r.total_mass_rate
        );
        println!(
            "\"velocity\":{:?},\"mass_rates\":{:?},\"mass_flux_sums\":{:?},\"pressure_values\":{:?},\"convection\":{:?},\"strain_force\":{:?},\"pressure_force\":{:?},\"flux\":[",
            u,
            out.iter().map(|o| o.mass_rate).collect::<Vec<_>>(),
            out.iter().map(|o| o.mass_flux_sum).collect::<Vec<_>>(),
            w.triangle_scratch()
                .iter()
                .map(|s| s[1])
                .collect::<Vec<_>>(),
            out.iter().map(|o| o.convection).collect::<Vec<_>>(),
            out.iter().map(|o| o.strain_force).collect::<Vec<_>>(),
            out.iter().map(|o| o.pressure_force).collect::<Vec<_>>()
        );
        for (i, f) in w.shared_flux_scratch().iter().enumerate() {
            println!(
                "[{},{},{:?}]{}",
                f.nodes[0],
                f.nodes[1],
                f.flux,
                if i + 1 == w.shared_flux_scratch().len() {
                    ""
                } else {
                    ","
                }
            );
        }
        println!("]}}{}", if case == 0 { "," } else { "" });
    }
    println!("]}}");
    Ok(())
}
