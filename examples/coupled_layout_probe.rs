use rheon::*;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let w = FittedHeightWorkspace::new(
        FittedHeightGeometry {
            cap: &[[0., 1.], [0.5, 1.25], [1., 1.]],
            bottom_x: &[0., 0.5, 1.],
            extrusion_width: 1.,
            density: 3.,
            dynamic_viscosity: 0.05,
        },
        Default::default(),
        |_| false,
    )?;
    for (i, n) in w.nodes().iter().enumerate() {
        println!(
            "raw={i} id={} xy={:?} x={:?} y={:?}",
            n.periodic_index,
            n.position,
            w.velocity_embedding(n.periodic_index, 0),
            w.velocity_embedding(n.periodic_index, 1)
        );
    }
    println!("pressure {:?}", w.pressure_columns());
    Ok(())
}
