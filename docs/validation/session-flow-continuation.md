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
