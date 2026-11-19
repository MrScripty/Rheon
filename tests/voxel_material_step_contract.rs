//! Bounded real coupled-MAC fixtures; no native/persistence/performance claim.
use rheon::voxel_material::{
    MaterialPair, VoxelMaterialConfig, VoxelMaterialError, VoxelMaterialSimulation,
    VoxelMaterialStage,
};
use rheon::{Axis, GridGeometry, PressureError, PressureSettings};

const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];
const LIMIT: usize = 64 * 1024 * 1024;
fn config(gravity: [f64; 3]) -> VoxelMaterialConfig {
    VoxelMaterialConfig {
        gravity,
        memory_limit: LIMIT,
        pressure: PressureSettings {
            relative_residual: 1e-12,
            absolute_residual: 1e-12,
            divergence_limit: 1e-8,
            max_iterations: 3000,
        },
    }
}
fn vertical(n: u64) -> (GridGeometry, Vec<f64>) {
    let g = GridGeometry::new([n; 3], [1.0 / n as f64; 3], [0.0; 3]).unwrap();
    let f = (0..g.cell_len())
        .map(|i| {
            if i % (n as usize) < (n / 2) as usize {
                1.0
            } else {
                0.0
            }
        })
        .collect();
    (g, f)
}
fn bits(s: &VoxelMaterialSimulation) -> Vec<u64> {
    let state = s.state();
    let mut values = state
        .liquid_fraction()
        .iter()
        .map(|f| f.to_bits())
        .collect::<Vec<_>>();
    for a in AXES {
        values.extend(state.velocity(a).iter().map(|u| u.to_bits()));
    }
    values.push(u64::from(state.pressure().is_some()));
    if let Some(p) = state.pressure() {
        values.extend(p.iter().map(|p| p.to_bits()));
    }
    values.extend([state.time().to_bits(), state.generation()]);
    if let Some(r) = s.last_report() {
        values.extend([
            r.generation,
            r.time.to_bits(),
            r.dt.to_bits(),
            r.phase_masses[0].to_bits(),
            r.phase_masses[1].to_bits(),
            r.cumulative_phase_volume_budgets[0].to_bits(),
            r.cumulative_phase_volume_budgets[1].to_bits(),
        ]);
    }
    values
}

#[test]
fn one_phase_limits_and_arbitrary_material_zero_gravity_remain_at_rest() {
    let g = GridGeometry::new([4; 3], [0.25; 3], [0.0; 3]).unwrap();
    for f in [
        vec![0.0; g.cell_len()],
        vec![1.0; g.cell_len()],
        (0..g.cell_len()).map(|i| (i % 3) as f64 * 0.5).collect(),
    ] {
        let mut sim = VoxelMaterialSimulation::new(
            g.clone(),
            MaterialPair::water_air(),
            &f,
            config([0.0; 3]),
        )
        .unwrap();
        assert!(sim.state().pressure().is_none());
        for _ in 0..3 {
            let r = sim.step(0.01, |_| false).unwrap();
            assert_eq!(r.projection.actual_divergence_max, 0.0);
            assert_eq!(r.viscosity.dissipation, 0.0);
        }
        assert_eq!(sim.state().liquid_fraction(), f);
        for a in AXES {
            assert!(sim.state().velocity(a).iter().all(|u| *u == 0.0));
        }
        assert!(sim.state().pressure().unwrap().iter().all(|p| *p == 0.0));
        if f.iter().all(|f| *f == 0.0) {
            assert!(sim.last_report().unwrap().phase_masses[1] > 0.0);
            assert_eq!(sim.last_report().unwrap().phase_masses[0], 0.0);
        }
    }
}

#[test]
fn full_viscous_gravity_projection_preserves_hydrostatic_layers_and_pressure() {
    let g = GridGeometry::new([4; 3], [0.2, 0.3, 0.4], [-1.0, 0.7, -0.3]).unwrap();
    let material = MaterialPair::water_air();
    assert_ne!(material.liquid().viscosity(), material.gas().viscosity());
    assert!(material.gas().viscosity() > 0.0);
    for d in 0..3 {
        let f = (0..g.cell_len())
            .map(|i| {
                let p = [i % 4, (i / 4) % 4, i / 16];
                if p[d] < 2 { 1.0 } else { 0.0 }
            })
            .collect::<Vec<_>>();
        let mut gravity = [0.0; 3];
        gravity[d] = -9.81;
        let mut sim =
            VoxelMaterialSimulation::new(g.clone(), material, &f, config(gravity)).unwrap();
        for _ in 0..3 {
            let r = sim.step(0.002, |_| false).unwrap();
            assert!(r.projection.actual_divergence_max <= 1e-8);
            for a in AXES {
                assert!(sim.state().velocity(a).iter().all(|u| u.abs() < 3e-10));
            }
            let p = sim.state().pressure().unwrap();
            for k in 0..4 {
                for j in 0..4 {
                    for i in 0..4 {
                        let q = [i, j, k];
                        if q[d] == 3 {
                            continue;
                        }
                        let mut hi = q;
                        hi[d] += 1;
                        let rho = |q: [usize; 3]| {
                            if q[d] < 2 {
                                material.liquid().density()
                            } else {
                                material.gas().density()
                            }
                        };
                        let expected = (rho(q) + rho(hi)) * 0.5 * gravity[d] * g.spacing()[d];
                        let actual =
                            p[hi[0] + 4 * (hi[1] + 4 * hi[2])] - p[q[0] + 4 * (q[1] + 4 * q[2])];
                        assert!(
                            (actual - expected).abs() < 2e-6,
                            "axis {d} at {q:?}: {actual} vs {expected}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn off_midplane_hydrostatic_layers_preserve_rest_on_a_second_full_step_geometry() {
    let n = 6usize;
    let split = 2usize;
    let spacing = [0.125, 0.2, 0.3];
    let grid = GridGeometry::new([6; 3], spacing, [0.5, -0.25, 1.0]).unwrap();
    let materials = MaterialPair::water_air();
    let cell = |p: [usize; 3]| p[0] + n * (p[1] + n * p[2]);
    for axis in 0..3 {
        let fraction = (0..n * n * n)
            .map(|index| {
                let p = [index % n, index / n % n, index / (n * n)];
                if p[axis] < split { 1.0 } else { 0.0 }
            })
            .collect::<Vec<_>>();
        let mut gravity = [0.0; 3];
        gravity[axis] = -9.81;
        let mut sim =
            VoxelMaterialSimulation::new(grid.clone(), materials, &fraction, config(gravity))
                .unwrap();
        for _ in 0..2 {
            let report = sim.step(0.002, |_| false).unwrap();
            assert!(report.projection.actual_divergence_max <= 1e-8);
            for component in AXES {
                assert!(
                    sim.state()
                        .velocity(component)
                        .iter()
                        .all(|u| u.abs() < 3e-10)
                );
            }
            let pressure = sim.state().pressure().unwrap();
            // Independent Cartesian half-column integration, including the
            // off-midplane interface and the first/last interior wall edges.
            let density = |p: [usize; 3]| {
                if p[axis] < split {
                    materials.liquid().density()
                } else {
                    materials.gas().density()
                }
            };
            for k in 0..n {
                for j in 0..n {
                    for i in 0..n {
                        let lower = [i, j, k];
                        if lower[axis] == n - 1 {
                            continue;
                        }
                        let mut upper = lower;
                        upper[axis] += 1;
                        let expected =
                            0.5 * (density(lower) + density(upper)) * gravity[axis] * spacing[axis];
                        let actual = pressure[cell(upper)] - pressure[cell(lower)];
                        assert!(
                            (actual - expected).abs() < 2e-6,
                            "axis {axis}, lower {lower:?}: {actual} vs {expected}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn water_air_moves_and_conserves_each_frozen_phase_in_closed_box() {
    let (g, f) = vertical(8);
    let mut sim = VoxelMaterialSimulation::new(
        g.clone(),
        MaterialPair::water_air(),
        &f,
        config([0.0, -9.81, 0.0]),
    )
    .unwrap();
    let v = g.cell_volume();
    let initial = [
        f.iter().sum::<f64>() * v,
        f.iter().map(|f| 1.0 - f).sum::<f64>() * v,
    ];
    let center = |f: &[f64]| {
        f.iter()
            .enumerate()
            .map(|(i, f)| f * ((i / 8) % 8) as f64)
            .sum::<f64>()
            / f.iter().sum::<f64>()
    };
    let initial_center = center(&f);
    let mut dissipated = false;
    for _ in 0..20 {
        let r = sim.step(0.002, |_| false).unwrap();
        assert!(r.dt > 0.0 && r.dt <= 0.002 && r.attempts <= 8);
        assert!(r.transport.volume_courant <= 0.5 && r.transport.mass_courant <= 0.5);
        assert!(r.projection.actual_divergence_max <= 1e-8);
        assert!(
            sim.state()
                .liquid_fraction()
                .iter()
                .all(|f| f.is_finite() && (0.0..=1.0).contains(f))
        );
        let actual = [
            sim.state().liquid_fraction().iter().sum::<f64>() * v,
            sim.state()
                .liquid_fraction()
                .iter()
                .map(|f| 1.0 - f)
                .sum::<f64>()
                * v,
        ];
        for phase in 0..2 {
            let independent_reduction =
                2.0 * (g.cell_len() + 4) as f64 * f64::EPSILON * (initial[phase] + actual[phase]);
            assert!(
                (actual[phase] - initial[phase]).abs()
                    <= r.cumulative_phase_volume_budgets[phase] + independent_reduction
            );
            assert!(r.cumulative_phase_volume_budgets[phase] / initial[phase] < 1e-8);
        }
        assert!(r.viscosity.identity_error <= r.viscosity.arithmetic_budget);
        dissipated |= r.viscosity.dissipation > 0.0;
    }
    assert!(dissipated);
    assert!(center(sim.state().liquid_fraction()) < initial_center);
    assert!(
        AXES.iter()
            .any(|&a| sim.state().velocity(a).iter().any(|u| u.abs() > 1e-8))
    );
    assert_ne!(sim.state().liquid_fraction(), f);
}

#[test]
fn every_cancel_stage_preserves_nonzero_accepted_state_and_reproducible_retry() {
    let (g, f) = vertical(4);
    let c = config([0.0, -9.81, 0.0]);
    let mut reference =
        VoxelMaterialSimulation::new(g.clone(), MaterialPair::water_air(), &f, c).unwrap();
    for _ in 0..3 {
        reference.step(0.002, |_| false).unwrap();
    }
    let expected = bits(&reference);
    for stage in [
        VoxelMaterialStage::BeforeTransport,
        VoxelMaterialStage::Transport,
        VoxelMaterialStage::Viscosity,
        VoxelMaterialStage::Gravity,
        VoxelMaterialStage::Projection,
        VoxelMaterialStage::BeforeCommit,
    ] {
        let mut sim =
            VoxelMaterialSimulation::new(g.clone(), MaterialPair::water_air(), &f, c).unwrap();
        sim.step(0.002, |_| false).unwrap();
        sim.step(0.002, |_| false).unwrap();
        let before = bits(&sim);
        let mut reached = false;
        let result = sim.step(0.002, |s| {
            reached |= s == stage;
            s == stage
        });
        assert!(reached, "{stage:?}");
        assert!(
            matches!(result, Err(VoxelMaterialError::Cancelled)),
            "{stage:?}"
        );
        assert_eq!(bits(&sim), before, "{stage:?}");
        sim.step(0.002, |_| false).unwrap();
        assert_eq!(bits(&sim), expected, "{stage:?}");
    }
}

#[test]
fn bounded_step_shortening_and_targeted_failures_are_transactional() {
    let (g, f) = vertical(4);
    let c = config([0.0, -9.81, 0.0]);
    let mut sim =
        VoxelMaterialSimulation::new(g.clone(), MaterialPair::water_air(), &f, c).unwrap();
    let r = sim.step(1.0, |_| false).unwrap();
    assert!(r.dt < 1.0 && r.attempts > 1 && r.attempts <= 8);
    let before = bits(&sim);
    assert!(matches!(
        sim.step(f64::NAN, |_| false),
        Err(VoxelMaterialError::InvalidTimeStep)
    ));
    assert_eq!(before, bits(&sim));
    let total = sim.allocated_bytes();
    let mut tight = c;
    tight.memory_limit = total - 1;
    assert!(matches!(
        VoxelMaterialSimulation::new(g.clone(), MaterialPair::water_air(), &f, tight),
        Err(VoxelMaterialError::BufferLimit { .. })
    ));
    let mut limited = c;
    limited.pressure.max_iterations = 1;
    let mut failing =
        VoxelMaterialSimulation::new(g, MaterialPair::water_air(), &f, limited).unwrap();
    let initial = bits(&failing);
    assert!(matches!(
        failing.step(0.002, |_| false),
        Err(VoxelMaterialError::Pressure(
            PressureError::IterationLimit { iterations: 1, .. }
        ))
    ));
    assert_eq!(initial, bits(&failing));
}
