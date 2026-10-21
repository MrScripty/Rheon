# Verification record

Implemented code checkpoint: `58cd23e906080a95d640314402b0afc2f3985e6b`, based on
main `2209012bc5f902693d0f695ec1c2d854126af317`. Later commits change only CI/evidence
records. [Draft PR56](https://github.com/MrScripty/Rheon/pull/56) is the integration
boundary; no merge or deployment. Date: 2026-10-13.

## Local acceptance

Debian 13 Linux x86_64, Rust/Cargo 1.92.0; Intel Xeon Platinum 8370C 2.80 GHz VM,
5 visible CPUs, 4-CPU cgroup quota, 16 GiB memory limit. Release numerical kernels
are CPU-only. No device/GPU performance qualification.

| Check | Result |
| --- | --- |
| `cargo test --locked --release --no-default-features` | 355 passed |
| `cargo test --locked --release --features desktop -- --test-threads=1` | 394 passed, including all 22 native tests |
| `cargo clippy --locked --no-default-features --all-targets -- -D warnings` | passed |
| `cargo clippy --locked --features desktop --all-targets -- -D warnings` | passed |
| `cargo fmt --all --check`, `git diff --check` | passed |
| Release native build | passed |

Included are 15 typed import contracts, 13 collision contracts and 10 particle
physics tests. Both liquid and gas have analytical plane/box center constraints,
normal removal/tangent transport, swept tunneling, oblique/grazing/edge/corner,
initial exclusion and attach/emission rollback checks. Import negatives target
specific diagnostics, not arbitrary earlier failures. Matrix/rotation hierarchy,
reflection and user placement have independently calculated coordinate checks.

Initial concurrent debug native checks hit the inherited carrier-only 10-second
live-frame gate. The complete release serial native suite passed, including that
same test; no timeout was relaxed. An intermediate release build overlapped
ongoing source/test edits and was discarded; final complete runs used frozen
source and passed.

Independent read-only review identified normal-clearance retreat, contact-band
endpoint and oblique-edge handling, huge-query precision cancellation, and FIFO
blocking. Each was fixed with a regression. Final direct Rust probes confirmed
geometry repairs; the reviewer reported no remaining blocking code finding.
Subsequent glTF raw-transform validation also rejects ambiguous matrix/TRS,
nonunit quaternions and local shear; its focused tests and Clippy passed.

## Native rendered evidence

A real eframe window ran on isolated Xvfb `:99`, 1250x820x24, with
`LIBGL_ALWAYS_SOFTWARE=1`, no pre-existing desktop session and no network display
listener. No sandbox weakening or altered production rendering path was used.
Portable Debian Xvfb/xdotool packages were extracted under `/workspace/gui-tools`.
The existing Mesa cache attempted a read-only home directory and disabled its
cache; native rendering still succeeded. Captures use ImageMagick from the actual
X window, with xdotool real pointer/keyboard input.

Untracked generated evidence lives in `.generated/collision-evidence/`:

- `native-import.png`: imported OBJ, actual 180-step water run and collision box.
- `native-air-run.png`: actual Air preset selected through UI and another
  completed 180-step run; stability-limited time differs from water.
- `native-glb-final.png`: GLB imported through the same path field, scale 0.25,
  exact transformed bounds and wireframe; prior run history cleared.
- `native-removed.png`: Remove mesh clears geometry and history.
- `manifest.json`: source/executable and artifact hashes; full test/lint logs.

These screenshots prove native interaction/rendering, not arbitrary geometry
physics. Amber wireframe is an explicitly labelled x-ray inspection overlay.
Fluid preview voxels can straddle a surface even when particle centers do not.
The numerical tests establish the separate, narrower collision claims.

## CPU measurements

`cargo run --locked --release --no-default-features --example collision_mesh -- --profile`
uses five repeats, deterministic query locations, serial numerical kernels and
analytical plane assertions. Numbers below are wall-clock observations on this
shared VM, not a budget or speedup claim. All workloads completed successfully.

| Triangles | Median build ms | 20,000 impact sweeps, median ms (min–max) | Retained mesh bytes |
| ---: | ---: | ---: | ---: |
| 2 | 0.0015 | 2.2495 (2.1731–4.5330) | 560 |
| 512 | 0.1180 | 8.0385 (7.9838–8.4465) | 125,720 |
| 8,192 | 3.2017 | 11.3670 (11.3029–11.4802) | 2,001,944 |
| 32,768 | 14.5058 | 18.3850 (16.2098–32.3809) | 8,001,560 |

The BVH therefore provides bounded spatial pruning on these distributed triangle
workloads; pathological traversal is still linear. No brute-force speedup ratio
is claimed. Retained bytes are Vec capacity payload, not RSS or parser peak.

For 1,728 particles and 40 accepted calls, median water baseline/barrier cost was
304.26/290.86 ms and air baseline/barrier 309.36/323.90 ms. Ranges overlap and
trajectories differ; this does not establish that collision is free or faster.
The 512-triangle barrier adds 125,720 retained bytes (820,928 to 946,648).
Both water workloads advance 0.04 s; air advances 0.003320553 s without and
0.003260322 s with the barrier. Sweeps above exercise impacts; the step workload
also includes ordinary noncontact work and is not an impact-heavy benchmark.

Separate actual imported-solid examples ran water and air for 120 accepted steps
with 1,656 retained particles after initial exclusion and zero centers inside the
solid. Water advanced 0.12 s, air 0.003640233 s. These are implemented-model
observations, not calibrated physical validation or equal-time performance tests.
Raw CSV and example outputs remain generated artifacts, not committed fields.

## Hosted CI and remaining integration blocker

The first Rust executable-contracts job passed, including its complete Python
harness path. Scoped/rational jobs passed. Feature-matrix jobs run full
core/default/desktop tests and strict lint; see the PR's live checks for exact
source-specific terminal states.

The new collision workflow passed its analytical contracts and actual imported
water/air/profile steps. Its first native startup failed because the Ubuntu
runner lacked `libxkbcommon-x11.so`; the workflow now installs
`libxkbcommon-x11-0` explicitly and prints native logs on startup failure. It
retains the real screenshot and CPU observations as GitHub artifacts.

The independently owned education/Pages workflow fails in
`docs/education/aligned_strain_packet.py:source_core` with
`changed qualified native source Cargo.lock`. Its preceding actual qualification
passed. [Original failing job](https://github.com/MrScripty/Rheon/actions/runs/38393443961/job/115282634059).
This is the source-binding repair explicitly owned by thread
`01a134ab-c34f-7168-abaf-0410102c702a`; this feature does not change that file,
weaken its gate or copy the owner's repair. Whole-repository CI acceptance stays
blocked until that owner integrates the repair. No main merge is requested.
