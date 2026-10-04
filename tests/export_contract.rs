#![cfg(feature = "png-export")]
use rheon::{
    ExportError, GridGeometry, PressureSettings, Simulation, SimulationConfig, write_guidance_png,
};
#[test]
fn raw_export_limit_rejects_before_writing() {
    let g = GridGeometry::new([2; 3], [1.0; 3], [0.0; 3]).unwrap();
    let c = SimulationConfig {
        density: 1.0,
        memory_limit: 4096,
        pressure: PressureSettings {
            relative_residual: 1e-8,
            absolute_residual: 1e-12,
            divergence_limit: 1e-8,
            max_iterations: 100,
        },
        actual_divergence_limit: 1e-5,
        max_courant: 1.0,
    };
    let s = Simulation::new(g, c).unwrap();
    let mut bytes = Vec::new();
    assert!(matches!(
        write_guidance_png(&s, &mut bytes, 3),
        Err(ExportError::PixelBudget { .. })
    ));
    assert!(bytes.is_empty());
    assert_eq!(write_guidance_png(&s, &mut bytes, 4).unwrap(), 4);
    let mut reader = png::Decoder::new(std::io::Cursor::new(bytes))
        .read_info()
        .unwrap();
    let mut data = [255_u8; 4];
    reader.next_frame(&mut data).unwrap();
    assert_eq!(data, [0; 4]);
    assert_eq!(data.as_slice(), rheon::guidance_pixels(&s, 4).unwrap());
}
