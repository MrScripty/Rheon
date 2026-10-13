//! Bounded numeric consumer; JSONL frames on stdout, diagnostics on stderr.
mod session_support;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut session, boundary) = session_support::fixture([4, 2, 2], 0);
    let mut output = std::io::stdout().lock();
    session.snapshot().write_dense3d(&mut output)?;
    for _ in 0..8 {
        session.step(session_support::inputs(), boundary, None, |_| false)?;
        session.snapshot().write_dense3d(&mut output)?;
    }
    eprintln!(
        "accepted time={} s; array payload={} bytes",
        session.snapshot().state().carrier.time,
        session.allocated_array_bytes()
    );
    session.stop();
    Ok(())
}
