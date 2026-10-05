//! Orthographic opacity guidance export along +Z. This is a concentration
//! integral, not physical light scattering, depth, normals, or liquid rendering.
use crate::{GridGeometry, LiquidTransportSimulation, Simulation};
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
    let (width, height) = png_dimensions(simulation.grid())?;
    let pixels = guidance_pixels(simulation, pixel_limit)?;
    write_pixels_png(width, height, output, pixels)
}

/// +Z projection of represented liquid fraction, with +Y upward and the same
/// fixed 8/m appearance coefficient as smoke guidance. This is an occupancy
/// integral guidance map, not reconstructed surface depth/normals or scattering.
/// Pixel and PNG workspace accounting follows write_guidance_png.
pub fn write_liquid_guidance_png(
    simulation: &LiquidTransportSimulation,
    output: impl Write,
    pixel_limit: usize,
) -> Result<usize, ExportError> {
    let (width, height) = png_dimensions(simulation.grid())?;
    let pixels = liquid_guidance_pixels(simulation, pixel_limit)?;
    write_pixels_png(width, height, output, pixels)
}

fn png_dimensions(grid: &GridGeometry) -> Result<(u32, u32), ExportError> {
    let [nx, ny, _] = grid.counts();
    let width = u32::try_from(nx).map_err(|_| ExportError::ImageSize)?;
    let height = u32::try_from(ny).map_err(|_| ExportError::ImageSize)?;
    Ok((width, height))
}

fn write_pixels_png(
    width: u32,
    height: u32,
    output: impl Write,
    pixels: Vec<u8>,
) -> Result<usize, ExportError> {
    let bytes = pixels.capacity();
    let mut encoder = png::Encoder::new(output, width, height);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&pixels)?;
    writer.finish()?;
    Ok(bytes)
}

/// Raw +Z opacity projection, with +Y upward. Shared by PNG export and desktop
/// display so comparison views cannot silently use a different render model.
pub fn guidance_pixels(
    simulation: &Simulation,
    pixel_limit: usize,
) -> Result<Vec<u8>, ExportError> {
    let tracer = simulation.state().tracer;
    projection_pixels(simulation.grid(), pixel_limit, |cell| {
        f64::from(tracer[cell])
    })
}

/// Raw +Z represented-fraction guidance pixels from one accepted state.
pub fn liquid_guidance_pixels(
    simulation: &LiquidTransportSimulation,
    pixel_limit: usize,
) -> Result<Vec<u8>, ExportError> {
    let fraction = simulation.state().liquid.fraction;
    projection_pixels(simulation.grid(), pixel_limit, |cell| fraction[cell])
}

fn projection_pixels(
    grid: &GridGeometry,
    pixel_limit: usize,
    sample: impl Fn(usize) -> f64,
) -> Result<Vec<u8>, ExportError> {
    let [nx, ny, nz] = grid.counts();
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
    for j in 0..ny {
        for i in 0..nx {
            let column = (0..nz)
                .map(|k| {
                    sample(grid.cell_index([i, j, k]).expect("bounded cell")) * grid.spacing()[2]
                })
                .sum::<f64>();
            let opacity = -(-8.0 * column).exp_m1();
            pixels[i + nx * (ny - 1 - j)] = (255.0 * opacity.clamp(0.0, 1.0)).round() as u8;
        }
    }
    Ok(pixels)
}
