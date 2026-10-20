# G1 implementation qualification protocol

Authority: `plan.md` G1 admission and the fixed acceptance in `verification.md`. This protocol was frozen before native workflow and overhead evaluation. No numerical formula, stage order, supported domain, pressure-v1 meaning, audit criterion or old archive meaning may change. Retain failed evaluations.

Source base: plan120 `8422f77` on main `bc2f472`. Use Rust 1.92.0, existing binary64 pressure/fractions and f32 carrier. The material preset does not confer physical validation. Native fixtures use the existing Session, Worker, process supervisor, immutable configured intent and deterministic classical candidates.

## Contract checks

Run focused workflow and additive SQLite tests, then affected existing pressure, Session, recording/runtime and desktop worker/producer/archive/selection tests. Cover viscosity off, applied zero and nonzero; shortened actual interval; accepted, rejected, support-refused and failed providers; native fallback; cancellation before provider, after return, after qualification, during native fallback and before commit. Include accepted generation zero and attempted generation one.

Assert immutable accepted state and no accepted row for failed intervals. Preserve explicit accepted but unrecorded tails. Mutate source, interval, native input, request controls, descriptor digest, ordered nodes, runtime outcome and publication association. Unknown versions/kinds, oversized inputs and corrupt controls must reject. Known missing or legacy workflow evidence remains unavailable.

## Native workflow

Use the normal native App with isolated temporary project, settings and output roots. Prepare saved intent and actual fixture-provider archives through the existing Worker in the test-only fixture preparation function. Use an isolated Xorg dummy display with software graphics and XTEST input, without an ambient user session.

Through normal controls:

1. Open a saved configured run set and rerun from zero.
2. Open the live workflow. Select nodes by pointer and keyboard with visible focus.
3. Inspect actual interval, provider/fallback, candidate/accepted residuals, limits and units; change the selected accepted interval and open retained evidence.
4. Cancel and close. Remove the provider executable, then cold reopen the saved run and inspect the same retained descriptor/receipts.
5. Exit and reap owned application/provider processes.

Focused real-egui tests also cover generation-zero terminal inspection, invalid/missing/legacy states, source changes, pan/zoom/reset, text navigation, focus and close/reopen. Startup and screenshots alone cannot pass the workflow gate. Keep screenshots, interaction/assertion logs and archives outside Git; commit only compact source/binary hash manifests.

## Fixed capture plus cached projection budget

Workload: sealed 16³ grid over 1 m³, density 998.2 kg/m³, represented fraction 0.5, force acceleration [0.001, 0, 0] m/s², requested dt 0.001 s, Jacobi PCG v1, no viscosity/provider. Pressure limits: relative 1e-10, absolute 1e-12 m³/s², predicted divergence 1e-8 s⁻¹, actual divergence 1e-5 s⁻¹, maximum 2,000 iterations.

Baseline: existing `Session::step_with_pressure_provider(None)`. Candidate: `step_with_configured_pressure_observed(None)` plus the actual validated cached `WorkflowInspector::set_attempt` projection. Disk I/O and rasterization are outside this metric; bounded selection and cached-paint tests cover them separately.

Use optimized release, the same CPU/process, five warmup intervals, then exactly thirty paired intervals with alternating first path. Record each pair in nanoseconds. Compute the median of the paired `(candidate / baseline) - 1` values. Outside the timers, assert identical accepted state, pressure, fractions and time after every pair. The fixed target is at most 5% median added capture-plus-projection time. Report environment, variability and raw pairs. If variability dominates, classify the result unresolved. Do not change physics, workload or threshold to obtain a pass.

Reproduction:

```
cargo test --release --locked --features desktop --bin rheon-desktop \
  paired_capture_and_cached_projection_overhead_protocol -- --ignored --nocapture
```

The descriptor must remain at most 1 MiB and node summaries at most 256 per interval; the existing 16 KiB persisted metadata limit also applies. Formatting and strict Clippy cover library/desktop features. Independent architecture, persistence and native UI reviews must dispose material findings. C1 remains dependency-gated until G1 qualification.
