use rheon::*;
const CAP: [[f64; 2]; 3] = [[0., 1.], [0.5, 1.25], [1., 1.]];
fn geometry() -> FittedHeightGeometry<'static> {
    FittedHeightGeometry {
        cap: &CAP,
        bottom_x: &[0., 0.5, 1.],
        extrusion_width: 1.,
        density: 3.,
        dynamic_viscosity: 0.05,
    }
}
fn velocity() -> [[f64; 3]; 16] {
    let w = FittedHeightWorkspace::new(geometry(), Default::default(), |_| false).unwrap();
    let mut z = [0.; 34];
    for n in w.nodes() {
        let e = w.velocity_embedding(n.periodic_index, 0).unwrap();
        if e.weights[0] == 1.
            && let Some(j) = e.columns[0]
        {
            z[j] = n.position[1];
        }
    }
    let mut u = [[0.; 3]; 16];
    w.embed_velocity(&z, &mut u).unwrap();
    u
}
fn owner(settings: TranslatedViscousSettings) -> CoupledDiscreteFlow {
    CoupledDiscreteFlow::new(geometry(), &velocity(), Default::default(), settings, 81).unwrap()
}
#[derive(Debug, PartialEq, Eq)]
struct Snapshot {
    velocity: Vec<[u64; 3]>,
    positions: Vec<[u64; 2]>,
    mass: Vec<u64>,
    pressure: [u64; 16],
    time: u64,
    stamp: TranslatedViscousStamp,
}
fn snapshot(o: &CoupledDiscreteFlow) -> Snapshot {
    let s = o.state();
    Snapshot {
        velocity: s.velocity.iter().map(|v| v.map(f64::to_bits)).collect(),
        positions: s
            .geometry
            .nodes()
            .iter()
            .map(|n| n.position.map(f64::to_bits))
            .collect(),
        mass: s
            .geometry
            .nodal_mass()
            .iter()
            .map(|m| m.to_bits())
            .collect(),
        pressure: s.pressure_coefficients.map(f64::to_bits),
        time: s.time.to_bits(),
        stamp: s.stamp,
    }
}
#[test]
fn main_boundary_scan() {
 for h in [0.05,0.025,0.0015625] {
  for budget in [1,2,3,4,7] {
   let mut o=owner(TranslatedViscousSettings{max_iterations:budget,..Default::default()});
   let before=snapshot(&o);
   match o.step(h, |_|false) {
    Ok(r)=>println!("accepted h={h:?} budget={budget} iterations={} Newton_calls={} rate={:?} time={:?}",r.iterations,r.equation_evaluations,r.finite_momentum_rate_norm,o.state().time),
    Err(e)=>{assert_eq!(before,snapshot(&o));println!("refused h={h:?} budget={budget} error={e:?} accepted_state_preserved=true");}
   }
  }
 }
}
