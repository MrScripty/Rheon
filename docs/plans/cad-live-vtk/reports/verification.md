# CAD live view / VTK verification

Scope composes PR50 obstacle lab, PR47 inspection and PR53 `ec9d960` (which
contains PR48 transport API and PR49 provider contracts). Transport directly
consumes SessionSnapshot; the existing material worker retains its canonical PR47
capture. The VTK material adapter directly consumes PR53 MaterialSnapshot.
No competing snapshot schema or physics kernel was introduced.

Native X11/llvmpipe checks at 1250×820 and 900×650: unavailable constructor pressure;
accepted transport step changes fraction/probe; cell picking; physical aspect
ratio; particle speed legend, vectors and click selection (particle85); scrollable
dock; VTI/VTP save; timeline scrub250 retains labeled latest generation1024.
Both UI exports opened in VTK9.3.1:64 grid cells and87 points. Initial camera was
pulled back to prevent clipping. Expanded export controls reduce plot height;
collapsing them restores space. No floating inspector remains.

Independent review of production source and screenshots found no blocking stage,
cache, lifecycle or overlap issue. Findings addressed: empty-state export,
Drop-safe dock test construction and direct SliceData native-field regression.
Root reviewed the delegated encoder and exercised the actual independent reader.

Tests and local evidence:
- `cargo test --locked --features desktop --bin rheon-desktop -- --test-threads=1`:
  31 passed, including dock geometry, egui slice input, native publication refresh,
  lifecycle/cancellation, particle cache and export failure behavior.
- `cargo test --locked --no-default-features --test vtk_export_contract`:5 passed.
- `cargo run --locked --no-default-features --example vtk_export_fixture -- /tmp/cad-final-vtk`
  and `python3 tools/test_vtk_roundtrip.py /tmp/cad-final-vtk`:all seven files passed
  actual VTK9.3.1 precision, geometry, stages, associations and identity checks.
- `cargo clippy --locked --features desktop --all-targets -- -D warnings`:passed.
- `/tmp/cad-live-inspector-narrow.png`, `/tmp/cad-live-accepted.png`,
  `/tmp/cad-live-narrow.png`:native screenshots independently inspected.

Hardware boundaries: software GL only. Native Windows/macOS, physical touchpad,
HiDPI, discrete GPU, Godot/Tuldok integration, and full-step learned-solver coupling
are untested. Exports are synchronous; particle export holds the capture mutex
while writing for consistency and may pause capture/UI. No real-time promise.
Large-volume rendering/export throughput remains a later milestone.
