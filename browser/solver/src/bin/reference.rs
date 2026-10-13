fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reference = rheon_solver::reference()?;
    serde_json::to_writer_pretty(std::io::stdout().lock(), &reference)?;
    println!();
    Ok(())
}
