//! Orthographic opacity guidance export along +Z. This is a concentration
//! integral, not physical light scattering, depth, normals, or liquid rendering.
use crate::Simulation;
use std::{fmt, io::Write};
#[derive(Debug)]
pub enum ExportError {
    ImageSize,
    PixelBudget { required: usize, limit: usize },
    AllocationFailed,
    Encoding(png::EncodingError),
}
impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "guidance export failed: {self:?}")
    }
}
impl std::error::Error for ExportError {}
impl From<png::EncodingError> for ExportError {
    fn from(e: png::EncodingError) -> Self {
        Self::Encoding(e)
    }
}
/// One grayscale byte per XY ray. Pixel payload is capped separately from the
/// simulation budget. PNG compression workspace, output buffering and allocator
/// overhead are not included in either retained simulation array accounting or
/// this raw-pixel limit; no whole-process memory-cap claim is made.
pub fn write_guidance_png(
    simulation: &Simulation,
    output: impl Write,
    pixel_limit: usize,
) -> Result<usize, ExportError> {
    let grid = simulation.grid();
    let [nx, ny, nz] = grid.counts();
    let width = u32::try_from(nx).map_err(|_| ExportError::ImageSize)?;
    let height = u32::try_from(ny).map_err(|_| ExportError::ImageSize)?;
    let len = nx.checked_mul(ny).ok_or(ExportError::ImageSize)?;
    if len > pixel_limit {
        return Err(ExportError::PixelBudget {
            required: len,
            limit: pixel_limit,
        });
    }
    let mut pixels = Vec::new();
    pixels
        .try_reserve_exact(len)
        .map_err(|_| ExportError::AllocationFailed)?;
    if pixels.capacity() > pixel_limit {
        return Err(ExportError::PixelBudget {
            required: pixels.capacity(),
            limit: pixel_limit,
        });
    }
    pixels.resize(len, 0_u8);
    let tracer = simulation.state().tracer;
    for j in 0..ny {
        for i in 0..nx {
            let column = (0..nz)
                .map(|k| {
                    f64::from(tracer[grid.cell_index([i, j, k]).expect("bounded cell")])
                        * grid.spacing()[2]
                })
                .sum::<f64>();
            let opacity = -(-8.0 * column).exp_m1();
            pixels[i + nx * (ny - 1 - j)] = (255.0 * opacity.clamp(0.0, 1.0)).round() as u8;
        }
    }
    let bytes = pixels.capacity();
    let mut encoder = png::Encoder::new(output, width, height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    writer.finish()?;
    Ok(bytes)
}
