//! Q1 native grid-owner verification. Frozen failures are results, never relaxed criteria.
mod grid_verification {
    pub mod carrier;
    pub mod channel;
    pub mod sphere;
    pub mod transport;
    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    pub const RHO: f64 = 998.2;
    pub const MU: f64 = 0.001002;
    pub const NU: f64 = MU / RHO;
}
fn main() -> grid_verification::Result<()> {
    let out = std::env::args_os()
        .nth(1)
        .expect("fresh output directory required");
    std::fs::create_dir(&out)?;
    let root = std::path::Path::new(&out);
    grid_verification::channel::run(root)?;
    grid_verification::transport::run(root)?;
    Ok(())
}
