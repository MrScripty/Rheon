# Continuing complete-flow qualification

The benchmark/validation owner remains separate from the API, AI, physics and CAD implementations. Worker branches are tested in isolated detached worktrees. No branch composition or production activation is implied by an individual branch result.

## Published session flow: independent consumer checks

Source `f06a1270935b2ac285c68c0e594ab0742220913e` exports the documented `headless_session` trajectory: a 4×2×2 open unit box, carrier density 1000 kg/m³, represented density 800 kg/m³, uniform X boundary velocity 0.25 m/s, eight requested/accepted intervals of 0.0625 s, and an initial half-filled slab in X cell 1. This is a discrete fixture, not an engineering calibration or continuum convergence case.

`tools/verify_session_flow.py` independently checks the real exported constructor and all eight accepted frames:

- Exact frame/time sequencing and decimal-u64 carrier/liquid identities and versions.
- Staggered field shapes, finite fields and uniform corrected velocity.
- The first pressure projection has `dp/dx=-rho*u/dt`, giving pressure `-1000*i` Pa at X index i under the zero-cell gauge. Subsequent uniform intervals have zero projection pressure within the declared numerical tolerance.
- Donor transport has Courant 1/16: fraction at X index i after k steps is `(1/2)*choose(k,i-1)*(1/16)^(i-1)*(15/16)^(k-i+1)` where defined. Exported fractions match this independent solution.
- Divergence is recomputed from exported staggered faces. Liquid volume and represented mass are recomputed, and outflow is the previous final-X fraction divided by 64. This is open-boundary volume conservation, not constant retained volume.

Actual exported results: nine frames, accepted time 0.5 s, final volume `0.12365353541099466` m³, zero maximum fraction error, maximum recomputed divergence `4.547473724e-16` 1/s. Native output is `/tmp/rheon-session-frames.jsonl`. Adversarial tests reject missing frames, false times, rounded identities, wrong pressure/transport/velocity, false volume/divergence diagnostics, nonfinite fields and duplicate JSON keys. The tests require an explicit real export; without it they skip and must not be reported as qualified.

`tools/session_recovery_probe.rs` is an external Rust consumer test. After two nontrivial accepted frames, it exercises late precommit cancellation, expired deadline, invalid timestep and partial consumer-write failure. Every rejected operation must preserve the complete exported snapshot. All six subsequent accepted frames must exactly match an uninterrupted reference, including pressure, diagnostics, clocks and stamps. Rejections cannot consume step quota; quota exhaustion and stop preserve the final snapshot. This tests failure/recovery reproducibility, separate from the independent numerical oracle above.

Reproduce against an isolated **clean exact source checkout** with Rust 1.92.0; `VALIDATION` names this harness checkout, `CANDIDATE` names the exact API source, and `CARGO_TARGET_DIR` must be a fresh external build directory:

```sh
cargo build --manifest-path "$CANDIDATE/Cargo.toml" --locked --release --no-default-features --lib --example headless_session
"$CARGO_TARGET_DIR/release/examples/headless_session" > /tmp/session-frames.jsonl
python3 "$VALIDATION/tools/verify_session_flow.py" /tmp/session-frames.jsonl
RHEON_SESSION_FRAMES=/tmp/session-frames.jsonl python3 -m unittest discover -s "$VALIDATION/tools" -p test_session_flow.py -v
RHEON_SESSION_FRAMES=/tmp/session-frames.jsonl python3 -O -m unittest discover -s "$VALIDATION/tools" -p test_session_flow.py -v
RHEON_SESSION_FIXTURE="$CANDIDATE/examples/session_support/mod.rs" rustc --edition 2024 --test "$VALIDATION/tools/session_recovery_probe.rs" --extern rheon="$CARGO_TARGET_DIR/release/librheon.rlib" -L dependency="$CARGO_TARGET_DIR/release/deps" -o /tmp/session-recovery-probe
/tmp/session-recovery-probe
```

Normal and optimized real-artifact tests passed. The external recovery probe passed. No API-owner source or test was edited. The small export case is functional qualification, not a representative throughput benchmark.

## Combined trajectory/session acceptance still required

At the inspected published API/AI heads (`f06a127` and `b8ca87c`), the API owns box-flux liquid transport while the model owns sealed 32³ fixed-density/fixed-dt pressure proposals. These capabilities cannot be silently interchanged. When their owners publish trajectory integration, qualify the exact new sources using:

1. A nonzero complete trajectory with accepted time, fields, provider selection/rejection, corrected divergence and physical ledgers observed at every published frame. Compare equal accepted times and independent discrete/analytical references where applicable; agreement with another implementation alone is not physical accuracy.
2. Unsupported model domain, corrupt/failed inference and quality rejection with explicit numerical fallback, plus cancellation at late publication boundaries. Check that failures preserve the full accepted state and that retry matches an uninterrupted numerical reference. Do not count fallback as successful model qualification.
3. Source/weights/runtime provenance, bounded payload and process memory, multiple fresh repetitions and method-order rotation. Measure initialization, complete accepted step and consumer export separately, with hardware/thread/quota data. Pressure-stage or inference-only timings cannot stand in for frame cost.
4. Explicit scope for particle/material sessions, moving boundaries and CAD consumers. No GPU, rendered device frame, arbitrary geometry, continuum or trained-model generalization claim follows from current evidence.

The existing single-stage AI evidence and numerical/session evidence remain valid for their recorded commits; no combined integration is marked passed until it is published and exercised.

## Exact-head hosted status

On PR #46 head `60c15ade79cedc883f75c64bf48254d26b7a2655`, [cloud acceptance](https://github.com/MrScripty/Rheon/actions/runs/38382133689), [executable contracts](https://github.com/MrScripty/Rheon/actions/runs/38382133598/job/115244902502), and both metadata runs passed. The default/core-only/desktop feature jobs were still in progress at this checkpoint. Earlier successful runs are not substituted for these exact-head checks.

## Combined API stack: PR #53

The concrete combined source is `ec9d960c3717aa952a06f368e1a1881d38404029` from [PR #53](https://github.com/MrScripty/Rheon/pull/53), based on PR #48 and incorporating canonical inspection and pressure-provider contracts. It was fetched and checked out separately at `/workspace/rheon-api-combined-validation`; validation did not reconstruct or edit the implementation.

With the previously SHA256-verified real model export:

```sh
RHEON_AI_EXPORT=/tmp/rheon-ai-independent/modal.bin cargo test --locked --release --no-default-features --test session_contract --test material_session_contract --test pressure_session_contract --test particle_inspection_contract --test solver_provider_contract -- --include-ignored
cargo build --locked --release --no-default-features --lib --example headless_session --example material_session --example pressure_session
```

All selected contracts passed, including the real model through `PressureSession`, visible unsupported-domain fallback, and the deliberately retained high-frequency request whose fallback exhausts its original iteration budget without replacing publication. Material/session tests cover accepted controls, source identity, diagnostic evaluation time, cancellation/deadline preservation and actual shortened intervals. These are adapter/reference checks, not validation of real-material constitutive accuracy.

All three examples executed. Pressure selection reported `lab-modal-pressure-v1`, generation 1, residual L2 `6.10153409474846344e-12`, and no rejection. The inherited headless trajectory again passed the independent transport oracle, and the external complete recovery probe passed against this exact combined rlib. Logs are `/tmp/rheon-combined-{contracts,build,headless,pressure,recovery}.log`; output is `/tmp/rheon-combined-frames.jsonl`, `/tmp/rheon-combined-material.csv`, and `/tmp/rheon-combined-oracle.json`.

This stack correctly keeps three distinct capabilities: transport session, SPH/material session, and qualified pressure session. `PressureSession` is not full-step acceptance; the SPH EOS is not this grid pressure provider. Full-step AI selection through a production Simulation/Session remains unqualified.

## AI trajectory execution and confirmed receipt-rejection gaps

The later AI source `4e4383cc87d13054d8d1540e6caf8d4246db0536` adds `ai_pressure_trajectory`: independent evolving component loops for Jacobi, SGS, learned, injected quality fallback, shifted Jacobi and unsupported-origin fallback. It does not add a production simulation/session hook. Every mode owns its own evolving velocity/tracer state; references are compared offline, with no teacher-state feedback. Cancellation callbacks are always false, so this example does not qualify rejected-step rollback or cancellation/retry recovery.

Native release execution, strict example Clippy, independent field verification and nine actual-artifact tests in each of normal/optimized Python completed successfully. The experiment has two public lab seeds, 24 frames (0.4 simulated seconds), three rotated repetitions within one process plus a separate captured replay. Captured fields are independently checked for full pressure residual, corrected divergence, Courant, wall/gauge/range and numerical-reference errors. The timed repetitions do not retain their full fields.

Observed native process cost: 206.233245346 s wall, 206.205433 s CPU, peak RSS 14,208 KiB. Same AMD EPYC host, five-CPU affinity, four-CPU cgroup quota and 16 GiB memory limit as the baseline. Native execution is single-threaded (`--no-default-features`). Independent NumPy verification was subsequently run with `OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1`; no build/test ran concurrently with native timing.

Observed complete 24-step rollout medians (initialization plus numerical steps, **excluding field capture**): Jacobi 6009.166 ms, SGS 5209.929 ms, learned 1407.703 ms, injected fallback 1595.398 ms, shifted Jacobi 5869.476 ms, unsupported fallback 5932.789 ms. Model load was measured separately. Captured maximum actual divergence was `8.344650268554688e-7`; learned velocity/tracer/centered-pressure relative L2 errors against numerical references were `6.80e-8`, `2.43e-8`, and `7.91e-8`. These are bounded discrete-reference observations, not physical accuracy, untouched-test-set generalization, complete exported-frame timing or game-engine speed claims.

**Independent rejection qualification is not complete.** The added black-box `tools/probe_ai_trajectory_receipts.py` reproduced four gaps against these real artifacts, in both normal and optimized Python:

| Corruption | Candidate verifier result |
| --- | --- |
| All frame/run/main timing fields jointly zero | Accepted |
| Main process/model-load timers zero, rollout timers unchanged | Accepted |
| Every timed-frame residual set to `1e100` | Accepted |
| Frame 2 fields replaced by frame 1 for every seed/mode, with matching numerical diagnostics but unchanged time/frame identities | Accepted |

The last probe demonstrates missing transition validation: per-frame projection/reference agreement can survive duplicated evolution. Original learned seed-900 tracer integrals differ (`0.0003449856357520849` versus `0.0006897655723322139`), as do the frame hashes; the mutation is nonvacuous. It is a verifier defect, **not evidence that the original native evolution is wrong**. An independent reviewer reproduced all four gaps and confirmed the probe leaves originals untouched by replacing temporary hard links before writes.

Reproduce the refusal probes (exit 1 is the current expected failure and must not be relabeled success):

```sh
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 python3 tools/probe_ai_trajectory_receipts.py "$AI_CHECKOUT/tools/verify_ai_pressure_trajectory.py" /tmp/rheon-ai-trajectory-independent/trajectory
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 python3 -O tools/probe_ai_trajectory_receipts.py "$AI_CHECKOUT/tools/verify_ai_pressure_trajectory.py" /tmp/rheon-ai-trajectory-independent/trajectory
```

Preserved evidence: `/tmp/rheon-ai-trajectory-independent/receipt.json`, `gate-0.log` through `gate-3.log`, the full `trajectory/` field captures, and `/tmp/rheon-ai-trajectory-refusal-probes{,-optimized}.json`. The earlier pressure-only verifier also accepted zero corrected-call times and unbounded reported residuals; these were independently reproduced on its real artifacts. Tightening those owner-controlled verifiers is pending; validation did not edit them or weaken thresholds.

The execution environment remained available after the reported disconnect. Native outputs, original captures and local/remote branch identities were verified intact before the PR #53 checks were recorded. PR #46 head `b395132` has passing cloud, executable, metadata and core-only hosted checks; default/desktop were still running at this checkpoint.
