use rheon::{
    GridGeometry, PressureSettings, RotatingCylinderEmitter, Simulation, SimulationConfig,
};
fn sim() -> Simulation {
    Simulation::new(
        GridGeometry::new([24; 3], [1.0 / 24.0; 3], [0.0; 3]).unwrap(),
        SimulationConfig {
            density: 1.0,
            memory_limit: 4 * 1024 * 1024,
            pressure: PressureSettings {
                relative_residual: 1e-9,
                absolute_residual: 1e-12,
                divergence_limit: 1e-7,
                max_iterations: 2000,
            },
            actual_divergence_limit: 1e-5,
            max_courant: 1.0,
        },
    )
    .unwrap()
}
fn bits(sim: &Simulation) -> Vec<u32> {
    let s = sim.state();
    s.x.iter()
        .chain(s.y)
        .chain(s.z)
        .chain(s.tracer)
        .map(|v| v.to_bits())
        .collect()
}

fn main() {
    let mut output = std::fs::File::create(std::env::args().nth(1).unwrap()).unwrap();
    use std::io::Write;
    let mut s = sim();
    let e = RotatingCylinderEmitter::default();
    for _ in 0..12 {
        s.step_with_emitter(0.02, e, |_| false).unwrap();
        output.write_all(&s.state().time.to_le_bytes()).unwrap();
        for v in bits(&s) {
            output.write_all(&v.to_le_bytes()).unwrap();
        }
    }
    for preset in [rheon::FluidPreset::Water, rheon::FluidPreset::Air] {
        let config = rheon::ParticleFluidConfig {
            material: preset.material(),
            samples_per_axis: 16,
            max_particles: 256,
            memory_limit: 16 * 1024 * 1024,
            gravity: [0.0; 3],
            ..Default::default()
        };
        let mut p = rheon::ParticleFluidSimulation::new(s.grid().clone(), config).unwrap();
        let mut max_particles = 0;
        for _ in 0..4000 {
            let r = p.step(0.01, Some(e), || false).unwrap();
            output.write_all(&r.dt.to_le_bytes()).unwrap();
            output.write_all(&p.time().to_le_bytes()).unwrap();
            output
                .write_all(&(p.particles().len() as u64).to_le_bytes())
                .unwrap();
            max_particles = max_particles.max(p.particles().len());
            for particle in p.particles() {
                for value in particle.position.iter().chain(&particle.velocity) {
                    output.write_all(&value.to_le_bytes()).unwrap();
                }
            }
        }
        assert!(max_particles > 0, "baseline must cover particle births");
        eprintln!("{} max_particles={} time={}", preset.label(), max_particles, p.time());
    }
}
