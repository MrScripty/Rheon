//! Engine-neutral fixed-update host. Run one bounded illustrative frame batch.
mod session_support;
use std::time::{Duration, Instant};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut session, boundary) = session_support::fixture([4, 2, 2], 0);
    let fixed_dt = 0.0625;
    let mut accumulator = 2.0 * fixed_dt;
    let deadline = Instant::now() + Duration::from_secs(1);
    // Both catch-up count and accumulated lag are capped by the host.
    for _ in 0..2 {
        if accumulator < fixed_dt {
            break;
        }
        let report = match session.step(session_support::inputs(), boundary, Some(deadline), |_| {
            false
        }) {
            Ok(report) => report,
            Err(rheon::SessionError::Deadline) => break,
            Err(error) => return Err(error.into()),
        };
        accumulator -= report.carrier.step.dt; // actual accepted dt, never requested dt
    }
    accumulator = accumulator.min(2.0 * fixed_dt);
    let snapshot = session.snapshot();
    println!(
        "render frame={} time={} s; retained lag={} s",
        snapshot.frame(),
        snapshot.state().carrier.time,
        accumulator
    );
    // Read face velocities via snapshot.state().carrier.{x,y,z}; copy/interpolate
    // into engine-owned display buffers before releasing this immutable borrow.
    Ok(())
}
