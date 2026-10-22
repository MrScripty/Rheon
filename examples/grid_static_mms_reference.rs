//! CV3-A private static polygon manufactured-solution preparation.
//! Execution requires the separately frozen source/oracle/resource admission.
#[allow(dead_code)]
mod grid_cut_variational_reference {
    #[path = "../grid_cut_variational_reference/assembly.rs"]
    pub mod assembly;
    #[path = "../grid_cut_variational_reference/basis.rs"]
    pub mod basis;
    #[path = "../grid_cut_variational_reference/clip.rs"]
    pub mod clip;
    #[path = "../grid_geometry_reference/exact.rs"]
    pub mod exact;
    #[path = "../grid_cut_variational_reference/export.rs"]
    pub mod export;
    #[path = "../grid_geometry_reference/geometry.rs"]
    pub mod geometry;
    #[path = "../grid_cut_variational_reference/observe.rs"]
    pub mod observe;
    #[path = "../grid_cut_variational_reference/trace.rs"]
    pub mod trace;
}
#[allow(dead_code)]
mod grid_cut_coupled_reference {
    #[path = "../grid_cut_coupled_reference/solver.rs"]
    pub mod solver;
}
mod grid_static_mms_reference {
    pub mod cases;
    pub mod experiment;
    pub mod field;
    pub mod loads;
}
const RHO: f64 = 1.0;
const MU: f64 = 0.1;
const GATE: f64 = 1e-10;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2 {
        return Err("fresh output directory and one exact CV3-A case required".into());
    }
    let name = args[1].to_str().ok_or("UTF8 fixed case required")?;
    let case = grid_static_mms_reference::cases::cases()
        .into_iter()
        .find(|c| c.name == name)
        .ok_or("case is not in the frozen nine-case corpus")?;
    let root = std::path::Path::new(&args[0]);
    std::fs::create_dir(root)?;
    match grid_static_mms_reference::experiment::run(root, &case) {
        Ok(()) => Ok(()),
        Err(e) => {
            std::fs::write(root.join("refusal.txt"), &e)?;
            Err(e.into())
        }
    }
}

#[cfg(test)]
mod preparation_contracts {
    use super::{
        grid_cut_coupled_reference::solver,
        grid_cut_variational_reference::{
            basis::{GAUSS_WEIGHTS, Layout, gauss_nodes},
            geometry::Polygon,
        },
        grid_static_mms_reference::{cases, field},
    };
    #[test]
    #[ignore = "CV3-A preparation: requires the frozen oracle/resource execution admission"]
    fn n16_ceiling_retains_complete_lattice_and_refuses_above_ceiling_before_allocation() {
        let source = Polygon::new(cases::cases()[0].vertices.clone()).unwrap();
        assert_eq!(
            Layout::new(16, [0.0; 2], 1.0, &source).unwrap().nodes.len(),
            544
        );
        for n in [0, 1, 17, usize::MAX] {
            assert!(
                Layout::new(n, [0.0; 2], 1.0, &source)
                    .err()
                    .unwrap()
                    .contains("n2..16")
            );
        }
        let mut above_cap = vec![vec![0.0; 545]; 545];
        for (i, row) in above_cap.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        assert!(
            solver::cholesky(&above_cap)
                .unwrap_err()
                .contains("bounded")
        );
        assert!(
            solver::qr(&vec![vec![1.0]; 545])
                .unwrap_err()
                .contains("bounded")
        );
    }
    #[test]
    #[ignore = "CV3-A preparation: requires the frozen oracle/resource execution admission"]
    fn new_trace_rule_resolves_degree_four_that_old_two_point_rule_misses() {
        let actual: f64 = gauss_nodes()
            .iter()
            .zip(GAUSS_WEIGHTS)
            .map(|(t, w)| w * t.powi(4))
            .sum();
        let old: f64 = [
            (1.0 - 1.0 / 3.0_f64.sqrt()) / 2.0,
            (1.0 + 1.0 / 3.0_f64.sqrt()) / 2.0,
        ]
        .iter()
        .map(|t| 0.5 * t.powi(4))
        .sum();
        assert!((actual - 1.0 / 5.0).abs() < 1e-14);
        assert!((old - 1.0 / 5.0).abs() > 1e-3);
    }
    #[test]
    #[ignore = "CV3-A preparation: requires the frozen oracle/resource execution admission"]
    fn continuum_keeps_absolute_pressure_and_outward_traction_sign() {
        let point = [0.375, 0.625];
        let gradient = field::gradient(point);
        assert_eq!(gradient[0][0] + gradient[1][1], 0.0);
        let sigma = field::stress(point);
        assert_eq!(field::traction(point, [1.0, 0.0]), [sigma[0], sigma[1]]);
        assert_eq!(field::traction(point, [-1.0, 0.0]), [-sigma[0], -sigma[1]]);
        assert!(field::pressure(point) > 1.0);
        assert_eq!(cases::cases().len(), 9);
    }
}
