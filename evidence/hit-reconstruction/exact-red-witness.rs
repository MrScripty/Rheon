mod collision;
use collision::{SurfaceSettings, SurfaceStamp, TriangleSurface};
fn main() {
    let surface = TriangleSurface::new(
        SurfaceStamp { id: 1, version: 0 },
        vec![[0.0,0.0,1.0],[1.0,0.0,1.0],[0.0,1.0,1.0]],
        vec![[0,1,2]], SurfaceSettings::default(),
    ).unwrap();
    let hit = surface.first_hit([0.25,0.25,-1e20],[0.25,0.25,1e20],|_|false).unwrap().unwrap();
    let reconstructed_z: f64 = hit.barycentric.iter().map(|w|w*1.0).sum();
    println!("parameter={} reported_z={} reconstructed_z={}",hit.parameter,hit.position[2],reconstructed_z);
    assert_eq!(hit.position[2],reconstructed_z,"accepted off-facet hit");
}
