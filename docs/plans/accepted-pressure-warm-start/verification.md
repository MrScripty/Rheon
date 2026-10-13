# Verification and measurement gates

Before performance measurement, test explicit borrowed seeds on independent hand
systems: exact solution/zero iterations, nontrivial approximate guess, wrong gauge,
wrong length/nonfinite seed, incompatible full RHS, zero RHS, current-RHS stopping
threshold, iteration failure, scaled divergence overflow and cancellation/retry.
Confirm default cold reports/fields exactly match the frozen base.

Owner tests: first warm request starts cold; next native accepted step uses exactly
its predecessor's accepted pressure; changed source/dt retains eligibility; new
owner with same caller IDs/generation cannot share provenance; successful unsupported
route invalidates; failed unsupported route preserves. Fail/cancel at all downstream
carrier/volume/reconstruction/final checkpoints; compare accepted field bits,
pressure, seed metadata, snapshot, clock, quota and deterministic retry with a clean
control. Test source/transport failures and deadline handling. Verify no new seed
array for LiquidTransport and tight memory-limit behavior. Standalone remains cold.

Provider tests: preferred successes, rejected output and numerical fallback retain
cold initialization and existing replay; provider publication invalidates native
history. PressureSession's qualified-pressure stage remains separate. New API
capabilities/version and report fields must state dense3D history limitations.

Controlled performance: isolated source/target roots, release locked builds and
frozen hashes. Pre-record workloads and sample counts before running. Compare cold
reference, candidate cold opt-out and candidate AcceptedPreviousV1. Equal requested
inputs/horizons/current tolerances; report accepted dt/time, residual/divergence,
conservation/energy/refinement errors against independent oracles. Warm/cold physical
arrays may differ within established numerical accuracy; do not enforce bit identity
between policies or invent looser accuracy allowances. Same-policy replay is exact.

Primary sustained cases: 16³/32³/64³ and anisotropic grids, 1/2/4 workers where usable,
cold startup and sustained fixed horizons, steady and changing sources/timesteps.
Adverse cases: abrupt/reversed/removed source, reset/new owner, rest/zero RHS, route
transitions. Negative controls: opt-out CPU frames and representative particle counts.
Include seed preparation/copying, initial true residual, all failed attempts, full
accepted steps/capture, iterations, managed memory and per-run distributions.
No simultaneous builds/tests during timing. Alternate balanced randomized pairs;
retain every trial and order. Same-binary controls can characterize noise but cannot
be used to dismiss a repeatable adverse workload.

Admission requires a repeatable whole-step gain beyond measured variability with
unchanged accuracy/failure guarantees and non-adverse representative controls.
Report all tradeoffs; fewer iterations is insufficient. Reject/revert unclear or
harmful candidate; do not tune layout or selectively extend trials for significance.
No device/GPU guarantee follows from cloud CPU timings.

Before draft publication: independent source/results review, full scoped numerical,
surface/provider/API/rollback suites, default/core/work-observation regression,
format and strict feature lints, then exact-head CI. Existing book gate untouched;
known independent failures are reported, never suppressed.

## Corrected milestone3569d3b local results

Rust1.92 locked release, fresh target `/workspace/scratch/warm-corrected-target`:
`cargo test --release --locked`373passed/0failed/3ignored;
`--no-default-features`366passed/0failed/3ignored;
`--features work-observation`383passed/0failed/3ignored.
`cargo clippy --locked --all-targets --features work-observation -- -D warnings`
and `cargo fmt --all -- --check` passed. No production changes after this freeze.
The three ignored tests require the pinned actual model; earlier milestone ran
those explicitly, and corrected-head model checks remain in the final gates.


## Final local gates and retained evidence

The corrected production source remains exactly at `3569d3b`; later changes are
evidence helpers and documentation. Strict default, core-only and desktop all-target
Clippy checks pass, in addition to the earlier work-observation check. All 18 actual
pinned-model contract tests pass (pressure-session 4, provider-step 8, solver-provider
6). The fresh corrected CLI passes the Python executable suite: 119 tests, 21 existing
conditional skips, zero failures. Four evidence-helper negative tests pass in both
normal Python and `-O` mode. Formatting and diff whitespace checks pass.

All corrected matrix archives unpack with matching hashes. Reanalysis under Python
`-O` exactly reproduces the qualified 40-case/240-triplet summary. Rejected evidence
still returns nonzero with 15 violation checks across five lost-horizon triplets.
The fixed 228-run confirmation cohort completes with exact fields and horizons.
Independent review recommends scoped admission with the particle timing uncertainty
recorded in results.md. Exact-head CI links are recorded on the draft PR; no book
qualification gate is weakened or deployment invoked.
