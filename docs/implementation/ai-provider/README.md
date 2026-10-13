# Qualified AI pressure components

`rheon::solver_provider` adds opt-in, programmatic pressure-stage selection.
`ProviderWorkspace::solve(request, settings, Some(&mut provider), cancelled)` tries
the provider, independently qualifies its candidate, and uses the chosen Jacobi
or SGS reference when the provider declines or fails. `None` selects that numerical
reference directly. Existing simulation, CLI and desktop defaults are unchanged.
The API owner can expose these types through a separate application contract.

`PressureRequest` describes a full sealed-box integrated RHS, grid, density and dt.
Rheon constructs the operator itself; the API cannot accidentally pass a
free-surface/obstacle operator under a sealed-box label. Provider traits expose
capability checking plus descriptive version/stage/tensor shape, x-fastest layout,
f64 dtype, SI units and model provenance. New general or specialized physics
stages need new typed requests and physical acceptance gates; a metadata stage
label is not executable permission to solve arbitrary physics.

`ProviderReport` identifies selected and attempted providers, model source/hash,
rejection reason, recomputed candidate diagnostics (including rejected quality),
accepted numerical diagnostics, and total/inference/qualification/fallback wall
times. `total` includes input checks, inference, output checks and any fallback.
Construction, file reads, application state transfer and f32 velocity correction
are explicitly outside it. Use `examples/ai_pressure.rs` for those added scopes.

## Real model and limits

The native CPU backend executes the existing lab's `ModalPressure` forward pass:
three cosine transforms, multiplication by 32,767 **learned** gains, three inverse
transforms and subtraction of the zero-cell gauge. It does not calculate inverse
eigenvalues, train, use reference pressure, or call a numerical solver inside
inference. The trained basis/gains come from the actual frozen checkpoint.

Source: [rheon-surrogate-lab](https://github.com/MrScripty/rheon-surrogate-lab/tree/e8a90bfef6b2955c502008ccc62443e29dd9f2ec).
Checkpoint: `experiments/trajectory-v2/model.pt`, SHA256
`25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9`.
Export SHA256: `902ad8d77b16ae42f22f90bf71c22c19ae2e3cbe0cc0f4cc3d770930d4049aea`.
The upstream Apache-2.0 license applies to this translated forward algorithm;
author credit: Puma / MrScripty. No upstream weights or simulation outputs are
copied into this repository. The exporter requires the exact checkpoint SHA256,
extracts known tensor entries without executing pickle, and refuses overwriting
an existing output. It needs only Python's standard library.

The native loader checks layout, finite positive gains and the exact export's
FNV-1a checksum for accidental corruption. FNV is **not authentication**. The
SHA256-verified exporter and trusted source are required; the loader's reported
SHA256 identifies the expected artifact, not a cryptographic runtime attestation.
The tensor contract is fixed: little-endian f64, header `RHEONAI1`, 1,024 basis
entries in mode/cell order, then 32,767 flattened z/y/x nonconstant gains.

Supported model domain is exactly a 32³ unit box at origin zero, spacing 1/32 m,
density 1.25 kg/m³, dt 1/60 s, impermeable walls and zero-cell pressure gauge.
Changed dimensions/spacing/origin/density/dt decline before model invocation and
fall back to the matching numerical sealed-box solve. Obstacles, free surfaces,
moving boundaries, variable density, viscosity and general physics are outside
this provider contract. No geometry or general fluid learning claim is made.
The lab's recorded approximately 8× CPU rollout result is historical evidence
for its own hardware, threads and trajectories, not a universal claim.

## Acceptance and ownership

Input shape, finite values, sealed RHS compatibility and finite nonnegative
settings are checked before calling a provider. Every output cell must be written,
finite and gauge-zero. Rheon recomputes the full ungauged `b - Ap`, including cell
zero, using the same residual and predicted-divergence thresholds as PCG. Failed
inference, unsupported cases and rejected predictions retain a reason and use a
fresh zero-start numerical reference; no rejected pressure enters the fallback.
Cancellation aborts without fallback. In-process providers must cooperate with
cancellation; panics and hung external implementations are not isolated here.

The workspace owns candidate arrays; any error invalidates `pressure()`.
The caller must only publish after `Ok` **and** independently check the actual
corrected f32 velocity divergence. The example does this against 1e-5 s⁻¹. The
pressure API alone does not claim transactional full-step or transport acceptance.
The workspace enforces its Vec payload cap (eight cell arrays, including PCG).
The fixed model allocates 270,328 bytes of tensors and 524,288 bytes of scratch;
no field allocation occurs inside native inference. Allocator overhead, process
RSS, application fields and external provider allocations are outside these caps.

## Reproduce with actual weights

Use Rust 1.92.0 and Python 3 with NumPy (qualification replay only). Clone the lab
next to Rheon and pin its source commit above. No GPU or PyTorch is needed.

```sh
mkdir -p .generated/ai-provider
python3 tools/export_ai_pressure.py \
  ../rheon-surrogate-lab/experiments/trajectory-v2/model.pt \
  .generated/ai-provider/modal.bin
cargo run --locked --release --no-default-features --example ai_pressure -- \
  .generated/ai-provider/modal.bin .generated/ai-provider/run
python3 tools/verify_ai_pressure.py \
  ../rheon-surrogate-lab/experiments/trajectory-v2/model.pt \
  .generated/ai-provider/run
RHEON_AI_EXPORT="$PWD/.generated/ai-provider/modal.bin" \
  cargo test --locked --no-default-features --test solver_provider_contract -- --include-ignored
```

The example refuses an existing output directory. It tests four deterministic
broadband MAC fields, three repetitions and rotating method order: 36 real
invocations. Each compares pressure to Jacobi and checks full residual and actual
corrected divergence. Model fallback is a qualification failure in this example.
It saves a native pressure/RHS pair for an independent NumPy checkpoint replay,
full-operator residual verification, and timing/receipt validation. These fields
are generated artifacts, not committed baseline fixtures. This is a pressure
comparison, not a full trajectory, rendering, or real-time benchmark. Native
kernels run single-threaded in the documented no-default-features command.

Unit tests use explicitly named test doubles to test failure handling, never as
AI evidence. The real-checkpoint test is explicitly ignored without an exported
artifact and is run using `--include-ignored` in the qualification command.
The dedicated `ai-pressure.yml` CI job downloads the immutable checkpoint, verifies
its SHA256, runs the real-checkpoint test and independent replay, and tests receipt
rejection. It gates correctness, not wall-time speedups.
Missing weights prevent real-model qualification; they are never replaced with
mock predictions. Numerical-only selection remains usable without weights.

## Review and remaining qualification boundaries

Independent review compared transform direction, gain ordering and gauge to the
upstream implementation and ran the actual-checkpoint contracts and NumPy replay.
It found no blocking numerical defect. Its real-artifact coverage findings were
addressed, including partial-transform cancellation and corrupt valid exports.

GPU, other inference runtimes, ARM, WebAssembly, application wire protocols,
desktop selection, longer trajectories, and scenes beyond the bounded kernel
rollout below are not qualified by this milestone. No speedup over an analytic spectral inverse
is established. Qualification is agreement with this discretized reference and
these input fields, not a general physical accuracy guarantee.


## Bounded independently evolved trajectories

`examples/ai_pressure_trajectory.rs` runs the actual Rheon velocity advection,
lab forcing, pressure provider, f32 velocity correction and passive-tracer
advection in an explicit component loop. Each backend owns and evolves its own
accepted state; no numerical-reference state enters a learned rollout. Candidate
velocity and tracer publish only after pressure, actual divergence, Courant and
finite/range gates pass. This qualifies the component loop. The existing
`Simulation` and desktop still need an API-owner provider-selection hook.

The scenario is adapted with attribution from the pinned lab probe: 32³ sealed
unit box, density 1.25 kg/m³, dt 1/60 s, prescribed three-component localized and
broadband forcing, and a passive concentration. Concentration is bounded but its
semi-Lagrangian transport is not a conservative liquid-mass method. The freeze
in `plan.md` preceded execution: 24 frames (0.4 simulated seconds), two already
public lab seeds (900/901), three rotated timing repetitions, then a separate
capture replay. This is not an untouched test set or a broad generalization test.

Six modes run separately: Jacobi, SGS, learned, learned with an explicitly
injected output error on frame 9, shifted-origin Jacobi, and unsupported-origin
fallback. The error fixture first executes the real model, then perturbs one
pressure value to exercise the quality gate. The trace must show Jacobi fallback
on frame 9 and real learned inference again afterward. It is labeled fault
injection, never counted as an accurate AI prediction. The shifted-origin case
uses the same local forcing coordinates at origin [1,1,1]; every request must be
declined before inference and match its corresponding shifted numerical
reference bit for bit.

```sh
cargo run --locked --release --no-default-features --example ai_pressure_trajectory -- \
  .generated/ai-provider/modal.bin .generated/ai-provider/trajectory
python3 tools/verify_ai_pressure_trajectory.py .generated/ai-provider/trajectory
RHEON_AI_TRAJECTORY="$PWD/.generated/ai-provider/trajectory" \
  python3 -m unittest discover -s tools -p test_ai_pressure_trajectory.py -v
```

The complete manifest is written last. A failed/incomplete rollout cannot pass
the verifier. There are 864 timed frames and 288 captured replay frames. Each
captured binary stores full ungauged RHS and pressure (little-endian f64), then
x/y/z MAC velocity and tracer (little-endian f32); x is fastest. The independent
NumPy verifier reassembles the full pressure operator, actual divergence,
Courant, wall/gauge/range checks and integral ledgers on every captured frame.
It compares learned and injected-fallback velocity, tracer and centered-pressure
relative L2 errors to both numerical rollouts at every frame (limit 1e-5).
It also rejects missing/duplicate frames, misreported selections, corrupted
fields, false diagnostic receipts, and underreported whole-rollout timing.

Reported resident rollout wall time includes state/workspace construction and
all 24 complete steps (forcing, advection, inference, pressure qualification,
fallback if needed, correction, tracer transport and all acceptance gates).
Frame CSV serialization follows the measured loop. Model file reading/loading
is separately measured once. Captured replay runs include field-output overhead
and are excluded from timing medians. `main_body_including_capture_ms` includes
all runs and replay output within Rust main; OS process startup/teardown and the
independent Python verifier are outside that timer. OS caches are uncontrolled.
No speedup threshold gates qualification. CI runs this exact bounded experiment
plus adversarial verifier tests against the immutable real checkpoint.

This does not qualify GPU execution, other model versions/operators, free liquid
surfaces, viscosity, obstacles, moving boundaries, application wire protocols or
long-term stability. Numerical agreement is with this discretized reference, not
a general physical accuracy guarantee or superiority over an analytic inverse.
