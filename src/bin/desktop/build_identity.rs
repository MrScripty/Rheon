//! Shared compiled declarations for native executors; never authentication or
//! complete compiler-read/build reproducibility evidence. Ordinary builds remain
//! explicitly unspecified unless their caller supplies valid declarations.
fn lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
pub fn source_revision() -> &'static str {
    option_env!("RHEON_SOURCE_REVISION")
        .filter(|value| lower_hex(value, 40))
        .unwrap_or("build revision unspecified")
}
pub fn material_sha256() -> Option<&'static str> {
    option_env!("RHEON_BUILD_MATERIAL_SHA256").filter(|value| lower_hex(value, 64))
}
pub fn executor_revision(owner: &str) -> String {
    format!(
        "rheon-{owner}/{}; source={}; tracked-material={}",
        env!("CARGO_PKG_VERSION"),
        source_revision(),
        material_sha256().unwrap_or("unspecified")
    )
}
