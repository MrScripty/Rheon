# Independent production-provider qualification

Candidate: PR55 `3525c979e16629b295a6865e3abbb2a36803cc93`, isolated clean checkout `/workspace/rheon-production-validation`. It incorporates the AI verifier fixes at `89997c5`. No candidate source was edited. Rust 1.92.0, Linux x86_64 CPU, the previously SHA256-verified `/tmp/rheon-ai-independent/modal.bin`, and a fresh external Cargo target were used. This report qualifies a bounded discrete fixture, not continuum physics or performance.

## Executed acceptance

`cargo test --release --locked --no-default-features --lib --test provider_step_contract --test session_contract -- --include-ignored`, with `RHEON_AI_EXPORT` set, passed 29 tests: 17 unit, seven provider-step and five session contracts. The real-model test was executed, not ignored. The production example and library built successfully. Log: `/tmp/rheon-production-contracts.log`.

The external `tools/production_recovery_probe.rs` uses the candidate's fixture controls and public API. It additionally owns a heterogeneous initial fraction field `0.25 + 0.5*x/31`, with separate real models and workspaces for uninterrupted and rejection/retry trajectories. Across 24 accepted intervals:

- Exactly 23 intervals selected the learned provider; frame nine requested half dt and selected explicit Jacobi fallback. Accepted time was `0.3916666666666666` seconds.
- Late final-publication cancellation, expired deadline, NaN dt, and a post-pressure invalid liquid source preserved full dense3D publication and the complete previous provider report. Every recovered accepted frame matched uninterrupted output byte-for-byte; selected/attempted provider identity and rejection matched too.
- Published frame/generation/run identity and the exact accepted provider report matched the returned interval.
- A separate stricter Courant-budget case automatically shortened a supported `1/60` request to `0.0002282177322938192`. A recording wrapper verified that exact actual dt reached the real model's support check, which selected numerical fallback.

The independent NumPy checker reconstructs stored velocity advection, regional force, pressure correction, **advection-before-source** tracer updates and nonuniform donor-cell liquid transport. It also checks pressure gauge, selected provider, clocks, actual MAC divergence and sealed-volume conservation. It reuses the already reviewed NumPy interpolation helpers from the candidate's AI tooling, never its Rust numerical operators. Its tolerances are explicit engineering allowances for storage/operation roundoff, not formal error bounds.

Both normal and optimized Python accepted the real captures. Maximum velocity reconstruction error was `3.3881317890172014e-21`; tracer and liquid-fraction reconstruction errors were zero; actual divergence was at most `2.3283064365386963e-9`; volume error was at most `5.551115123125783e-17`. The nonuniform fraction changed by up to `5.816722498441784e-5`, so donor transport was nonvacuous. The test requires real artifacts and rejects five mutations in each Python mode: duplicate evolution, false time, false divergence, false selected provider and a constant pressure-gauge offset. Tests skip without explicit real-artifact paths and must not then be reported as qualification.

Source review found no production rollback or gate-bypass defect. Independent review of the external checker identified missing provider/gauge checks; both were added and adversarially tested before qualification. Original uniform captures and prior AI component evidence remain preserved separately.

## Reproduction

Set `CANDIDATE` to the clean exact PR55 checkout and `VALIDATION` to this harness checkout. Use an external `CARGO_TARGET_DIR`, the pinned toolchain and verified `RHEON_AI_EXPORT`.

```sh
cargo build --manifest-path "$CANDIDATE/Cargo.toml" --release --locked --no-default-features --lib
RHEON_CANDIDATE="$CANDIDATE" rustc --edition 2024 "$VALIDATION/tools/production_recovery_probe.rs" --extern rheon="$CARGO_TARGET_DIR/release/librheon.rlib" -L dependency="$CARGO_TARGET_DIR/release/deps" -o /tmp/production-recovery-probe
/tmp/production-recovery-probe /tmp/production-capture
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 python3 "$VALIDATION/tools/verify_production_transition.py" /tmp/production-capture "$CANDIDATE/tools"
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 RHEON_PRODUCTION_CAPTURE=/tmp/production-capture RHEON_CANDIDATE="$CANDIDATE" python3 -m unittest discover -s "$VALIDATION/tools" -p test_production_transition.py -v
OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 RHEON_PRODUCTION_CAPTURE=/tmp/production-capture RHEON_CANDIDATE="$CANDIDATE" python3 -O -m unittest discover -s "$VALIDATION/tools" -p test_production_transition.py -v
```

Local evidence: `/tmp/rheon-production-nonuniform/`, `/tmp/rheon-production-nonuniform.log`, `/tmp/rheon-production-nonuniform-transition.json`, and `/tmp/rheon-production-negative{,-optimized}.log`. Candidate build outputs are `/tmp/rheon-production-target`. The original uniform experiment remains under `/tmp/rheon-production-independent`.

## Hosted and claim boundaries

Exact-head native CPU AI, executable, rational and scoped metadata checks passed. Feature matrix terminal status is recorded in the PR follow-up. The research-book build failed in the aligned-strain publication step. Its hosted logs could not be downloaded (HTTP403); independent local execution of `aligned_strain_packet.source_core` reproduces `changed qualified native source Cargo.lock`. That file is unchanged from starting main `2209012`, supporting the documented pre-existing historical publication pin mismatch. The gate was not weakened or bypassed; the overall PR cannot be reported wholly green.

The reviewed Lean source/pin check passed locally; no fresh kernel build was performed. No new benchmark, full-frame speed claim, GPU, Godot, device graphics, ARM/WASM, free-surface/viscosity composition, or arbitrary CAD qualification was attempted. Provider-owned internal state is outside session rollback. Dense3D exports do not carry provider provenance, so the Rust probe checks it separately. Production integration closes the earlier missing full-step route within this supported sealed-box scope; pressure-stage timing still does not measure the whole frame.
