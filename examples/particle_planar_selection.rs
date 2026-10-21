//! Pre-admission P1 native numerical experiment. Exit 0 means measurements
//! completed; it never means the fixed scientific acceptance gates passed.
#[path = "particle_planar_selection/cloud.rs"]
mod cloud;
#[path = "particle_planar_selection/eos.rs"]
mod eos;
#[path = "particle_planar_selection/fixtures.rs"]
mod fixtures;
#[path = "particle_planar_selection/isolation.rs"]
mod isolation;
#[path = "particle_planar_selection/kernel.rs"]
mod kernel;
#[path = "particle_planar_selection/matrix.rs"]
mod matrix;
#[path = "particle_planar_selection/planar.rs"]
mod planar;
use planar::Method;
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};
/// Reproducible signed uniform sample in [-1,1), independent of platform RNG.
fn noise(index: u64, stream: u64) -> f64 {
    let mut x = index
        .wrapping_add(stream.wrapping_mul(0x9e3779b97f4a7c15))
        .wrapping_add(0x123456789abcdef0);
    x = (x ^ (x >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94d049bb133111eb);
    x ^= x >> 31;
    2.0 * ((x >> 11) as f64 / (1_u64 << 53) as f64) - 1.0
}
fn writer(path: impl AsRef<Path>) -> std::io::Result<BufWriter<File>> {
    Ok(BufWriter::new(File::create(path)?))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: particle_planar_selection FRESH_DIRECTORY".into());
    }
    let root = Path::new(&args[1]);
    fs::create_dir(root)?;
    let mut hydro = writer(root.join("hydro.csv"))?;
    writeln!(
        hydro,
        "method,n,h_over_dx,perturbed,requested_dt_s,status,steps,time_s,initial_acceleration_rms_m_s2,max_speed_m_s,density_relative_rms,pressure_relative_rms,mass_relative_defect,energy_change_j_m2,pressure_work_defect_j_m2,viscous_work_j_m2,energy_ledger_closure_j_m2,kick_drift_remainder_j_m2,density_kick_remainder_j_m2,eos_chain_remainder_j_m2,pair_momentum_defect_kg_m_s_m2,max_neighbors_1d"
    )?;
    for method in [Method::Dummy, Method::Gamma] {
        for n in [16, 24, 32] {
            for ratio in [1.5, 2.0, 2.5] {
                for perturbed in [false, true] {
                    for dt in [1e-4, 5e-5, 2.5e-5] {
                        let name =
                            format!("hydro-{}-{n}-{ratio}-{perturbed}-{dt}.csv", method.name());
                        let mut raw = writer(root.join(name))?;
                        fixtures::hydro(method, n, ratio, perturbed, dt, &mut hydro, &mut raw)?;
                        raw.flush()?;
                    }
                }
            }
        }
    }
    hydro.flush()?;
    println!("hydrostatic reduced screens measured (not production acceptance)");
    let mut shear = writer(root.join("shear.csv"))?;
    writeln!(
        shear,
        "method,case,n,h_over_dx,requested_dt_s,status,steps,actual_dt_s,contraction_limit_s,velocity_relative_rms,traction_relative_error,momentum_ledger_kg_m_s_m2,max_pair_momentum_defect_pa,wall_work_j_m2,viscous_fluid_work_j_m2,viscous_exchange_after_wall_work_j_m2,temporal_energy_j_m2,lower_traction_pa,upper_traction_pa,time_s,final_dt_s"
    )?;
    for method in [Method::Dummy, Method::Gamma] {
        for n in [16, 32, 64] {
            for ratio in [1.5, 2.0, 2.5] {
                for dt in [1e-4, 5e-5, 2.5e-5] {
                    for kind in [
                        fixtures::ShearKind::Poiseuille,
                        fixtures::ShearKind::Couette,
                        fixtures::ShearKind::Slip,
                    ] {
                        let label = match kind {
                            fixtures::ShearKind::Poiseuille => "poiseuille",
                            fixtures::ShearKind::Couette => "couette",
                            fixtures::ShearKind::Slip => "slip",
                        };
                        let name = format!("shear-{}-{n}-{ratio}-{dt}-{label}.csv", method.name());
                        let mut raw = writer(root.join(name))?;
                        fixtures::shear(method, n, ratio, dt, kind, &mut shear, &mut raw)?;
                        raw.flush()?;
                    }
                }
            }
        }
    }
    shear.flush()?;
    println!("native assembled viscous matrix and reduced shear screens measured");
    let mut clouds = writer(root.join("clouds.csv"))?;
    writeln!(
        clouds,
        "stencil,n,h_over_dx,perturbed,min_neighbors,max_neighbors,mean_neighbors,partition_rms_error,gradient_y_rms_error,laplacian_y_rms_error,decay_rate_s_inverse,decay_relative_error,momentum_rate_n,kinetic_power_w"
    )?;
    for stencil in [cloud::Stencil::Wendland, cloud::Stencil::Preview] {
        for n in [16, 24, 32] {
            for ratio in [1.5, 2.0, 2.5] {
                for perturbed in [false, true] {
                    let name = format!("cloud-{}-{n}-{ratio}-{perturbed}.csv", stencil.name());
                    let mut raw = writer(root.join(name))?;
                    cloud::run(n, ratio, perturbed, stencil, &mut clouds, &mut raw)?;
                    raw.flush()?;
                }
            }
        }
    }
    clouds.flush()?;
    let mut isolation = writer(root.join("isolation.csv"))?;
    isolation::run(&mut isolation)?;
    isolation.flush()?;
    // These APIs intentionally keep the missing scientific admission visible.
    let mut admission = writer(root.join("admission.txt"))?;
    writeln!(
        admission,
        "production dt: {:?}",
        eos::WATER.production_dt(1.0, 0.0)
    )?;
    writeln!(
        admission,
        "thin volumetric support: {:?}",
        isolation::Slab::new([1.0, 0.0, 0.0], -0.01, 0.01)
            .and_then(|s| s.admit_volumetric_support(0.1))
    )?;
    admission.flush()?;
    println!("3D cloud instantaneous operator screens measured; inspect retained failures");
    Ok(())
}
#[cfg(test)]
mod tests {
    #[test]
    fn perturbation_bound_and_reproducibility() {
        for i in 0..10000 {
            for d in 0..3 {
                let x = super::noise(i, d);
                assert!((-1.0..1.0).contains(&x));
                assert_eq!(x, super::noise(i, d));
            }
        }
    }
}
