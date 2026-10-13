# Verification record

Initial runnable checkpoint, 2026-10-13; final acceptance is still in progress.
Base main: `2209012bc5f902693d0f695ec1c2d854126af317`.
Environment: Debian 13 Linux x86_64, Rust/Cargo 1.92.0; Intel Xeon Platinum 8370C
2.80 GHz VM, 5 visible CPUs and 4-CPU cgroup quota, 16 GiB memory limit.
No discrete GPU/device performance evidence is claimed.

## Focused checks and independent review

- 15 import contracts passed, including typed negative outcomes, resource limits,
  URI/symlink/FIFO rejection, GLB and analytical transform/placement cases.
- 13 collision contracts and 10 particle physics tests passed, including water
  and air plane/solid nonpenetration, normal removal/tangent continuation,
  tunneling, grazing, edge/corner contact, admission failures and state rollback.
- New native egui import/reject/remove/repeat and actual worker geometry/history
  lifecycle tests passed. Desktop compile and strict binary Clippy passed.
- Independent read-only review found grazing/clearance/edge cancellation and
  extreme-coordinate issues plus FIFO blocking. Each was repaired with a
  regression. Final direct Rust probes confirmed the geometric fixes; the
  reviewer reported no remaining blocking finding. Later glTF raw transform
  validation adds matrix/TRS ambiguity, quaternion and shear rejection.

Full release feature suites, final lint, native rendered evidence and CPU
measurements are being collected. One inherited carrier-only live-frame test
 timed out at its 10-second gate in debug under concurrent builds; it has no
collision mesh. The release serial native suite is the pending discriminator;
no timeout was relaxed or test removed.

## Reproduction and evidence boundaries

Run the commands in [collision mesh contract](../../../collision-meshes.md).
The focused GitHub workflow retains CPU profiles, imported water/air execution,
and a real native software-rendered screenshot. Existing Rust CI covers the full
default/core-only/desktop test and lint matrix. Screenshot evidence demonstrates
native rendering of actual model/run state; analytical tests establish the
separate limited collision response claims.

CPU profiling uses five repeated release samples for 20,000 swept queries at
2/512/8,192/32,768 triangles, plus 1,728-particle water/air 40-step workloads with
and without a 512-triangle barrier. Sweeps exercise impacts; stepping includes
ordinary noncontact work and does not isolate impact-heavy solver cost. Accepted
time is recorded because gas stability control can shorten dt. No speedup,
constant-time worst case, calibrated physics, real-time budget or GPU claim.

Generated logs, CSV and screenshots are retained outside Git as required by
`docs/REPOSITORY_OUTPUTS.md`; input meshes/buffers are authored synthetic fixtures.
