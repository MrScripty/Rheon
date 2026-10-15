# Measured live-water pressure bundle handoff

The missing runtime input is the **existing frozen live-water CPU deployment**,
with checkpoint `weights.npz` SHA256
`aa82bd122fde9406d9e6b92114b3e6ba77281f5ee8b9c726aa168dacbd93a62d`.
Supply the measured bundle and its installed runtime, plus the retained parity
inputs below. Source code and historical receipts are available; the actual
deployment, weights and Torch runtime were not supplied for this implementation.
This document records requirements, not a new inference or qualification result.
The [M0 reconciliation](../plans/configurable-solver-components/reports/m0-reconciliation.md)
and pinned [lab acceptance record](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/live-ai-pressure-comparison/reports/acceptance.json)
identify this artifact. No retraining, substitute model, changed normalization,
weakened gate or fabricated successful prediction is part of the handoff.

## Required material and provenance

Use the lab at `a8426061380e82951b4dba1bbfa38ff93dc7ebb2` for the published
[exporter](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/deployment.py)
and [bridge](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/cpu_bridge.py).
The reported training-source revision is
`11408e93995a9f3c3f59e171ccd0889906531027`, distinct from that adapter/export
revision and from the Rheon execution revision. The retained checkpoint has
2,000 training updates. Preserve the actual 32-hex training Run ID from its
completed/drained owner record; it is not specified by this document.

| Supplied material | Required relationship |
| --- | --- |
| `deployment.json` | Schema `rheon-pressure-cpu-deployment-v2`, profile `sealed-water-seven-channel-float32`, bridge `cpu_bridge.py`; original descriptor SHA256 in retained evidence is `be4eef1f051206205598e6876c83edaf5246138e17613f1aa787f3df183a761f`. |
| `weights.npz` | Exact checkpoint SHA256 above; must match descriptor `checkpoint_sha256`, file inventory and final `checkpoint-2000.npz` entry in `history.json`. Do not rename another checkpoint into this identity. |
| `normalization.json` | Original train-only normalization; bytes/hash must match both `normalization_sha256` and the inventory. Supply the actual digest; no replacement scales are inferred here. |
| `history.json`, `receipt.json` | Original selected training artifacts, including final checkpoint tensor inventory, optimizer count, source closure and split identity. |
| `sources/` | Every source named by the measured `source-witness.json`, plus `sources/source-witness.json` and `sources/runtime-manifest.json`, copied and hashed by the exporter. Include the complete import closure rather than only `model.py`. |
| `cpu_bridge.py` | Exact measured/exported bridge bytes and length, inventoried with the deployment; host limit is 2 MiB. |
| `installed-runtime.json` and installed files | Exact registered runtime manifest, its complete measured file inventory and actual files at the recorded paths. The summary's `manifest_sha256` must match the installed manifest bytes. |
| Python interpreter and dependencies | Actual interpreter at descriptor `python`, matching descriptor `python_sha256` and runtime `python`; `site_packages`, NumPy, Torch and all measured runtime assets must resolve. Supply exact Python/NumPy/Torch versions from that runtime, platform/architecture and host-library assumptions. Numeric versions cannot be recovered from absent runtime bytes. |
| Parity inputs | Original training/validation packet (`dataset.npz`, `splits.json`) and retained CPU inference artifacts (`receipt.json`, `predictions.npz` with inventory) used by the lab verifier. A checkpoint alone cannot reproduce the retained parity check. |

The exporter requires the original project/source witness and registered runtime,
the completed/drained training `run.json`, selected artifacts and a fresh output
directory. It checks Run artifact SHA256/length, training source-witness closure,
final checkpoint identity and the registered runtime-manifest hash. The context
reservation must match the training receipt's hash **and length** for
`inputs/splits.json`; its frozen snapshot indices are `[15, 63]`, yielding
`context_policy = {mode: "native_prefix", minimum_native_steps: 15,
training_snapshot_indices: [15, 63]}`. Follow the pinned
[lab export instructions](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/README.md).
Do not alter historical artifacts or edit a descriptor to bless changed files.
Relocating/re-measuring a runtime changes its binding; retain the new descriptor
and complete provenance separately, without claiming the original descriptor hash.

## Loader and execution boundary

The existing [ModelProcess loader](../../src/bin/desktop/ai_comparison/model_process.rs)
reads at most 1 MiB of descriptor bytes. Legacy top-level records are closed; it
checks schema/profile, water/cubic declarations, positive training-update count,
hash/ID formats, bounded native-prefix policy and measured bridge/interpreter.
It does **not** independently verify every legacy inventory item before spawning.
The measured Python bridge performs full deployment/runtime checks before model
inference: safe deployment-relative paths, file SHA256/length, checkpoint and
normalization identity, registered runtime selection, interpreter identity and
runtime assets. `installed-runtime.json` is bounded to 16 MiB; runtime profile is
`managed_torch_python_v1`, at most 32,768 files and 6 GiB of inventoried bytes.
The bridge compares imported NumPy/Torch versions with the manifest. These checks
establish declared byte consistency, not authenticity or scientific correctness.

Startup uses the recorded interpreter directly with `-I -S -B`, a cleared
environment, `PATH=/usr/bin:/bin`, `CUDA_VISIBLE_DEVICES=''`,
`TULDOK_TORCH_DEVICE=cpu` and one OMP/MKL/OpenBLAS thread. The bridge adds only its
measured source and recorded site-packages paths to Python's import search path.
The ready handshake must bind checkpoint, training Run ID and minimum prefix,
with `device=cpu`, `model_dtype=float32`, seven channels,
`optimizer_updates=0` and `cuda_initialized=false`. Channels/weights remain
model-specific inside the package. `start_pinned` compares the complete inertly
resolved `ProviderIdentity` before child spawn; retain the descriptor hash as
part of every frozen rerun binding. Its pre-spawn legacy check does not replace
the bridge's full inventory checks.

This is trusted-local execution with host libraries and no concurrent mutation
through validation/use assumed. It is not sandboxing, package authentication,
descendant-process containment or a guaranteed hard Close deadline. Use the
existing single child/reader/writer/worker owners. Missing runtime inputs must
remain explicit unsupported/unavailable outcomes, never fixture substitution.

## Frozen scene, support and pressure meaning

Freeze and retain the entire actual [Scene](../../src/bin/desktop/ai_comparison/scene.rs)
and chosen diagnostic policy before execution. The existing default is seed
20261016, counts `[16,16,16]`, 16 cells/m, 64 steps, 15 warmup steps, force center
`[0.5,0.5,0.5]`, acceleration 1 m/s², radius fraction 0.25, swirl 0.7,
fluctuation 0.4, frequency 1.2 Hz and Jacobi PCG (`sgs=false`). This is a current
source default, not proof that it reconstructs every historical scene hash.

The existing scene fixes density 1000 kg/m³, dynamic viscosity 0.001 Pa·s,
dt=1/60 s, zero origin, cubic spacing, stationary sealed free-slip walls and no
mass source. Counts are independently 5–128 with product at most 65,536;
integer cells/m is 1–128 (`h=1/cells_per_metre`), steps at most 512 and retained
paired numeric frame budget 96 MiB. Rectangular extents with unequal counts are
permitted. The bridge accepts scalar h in `[1/128,1]`, the same rho/dt and bounded
counts. These are execution bounds: reported training coverage is only 5–16
cells/axis and 8 or 16 cells/m. Larger grids are extrapolation, not qualified
generalization. No obstacle, free-surface, variable-density, anisotropic-cell or
nonzero-origin support follows from tensor dimensions.

Perform **15 actual qualified native steps** with no AI call; first inference is
physical step 16 using solved step-15 pressure and the newly assembled residual
`b - A*p_previous`. Initial zero pressure is not solved history. IDs and declared
prefix counts alone cannot prove physical history. The legacy request has exactly
`id`, `counts_xyz`, `cell_size_m`, `density_kg_m3`, `dt_s`, `base_pressure` and
`native_input_residual`; it carries no origin/viscosity field. The caller owns
those physical facts. x-fast f64 arrays reshape to `[1,1,Z,Y,X]`; the measured
[model mapping](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/cuda-pressure-v1/model.py)
builds float32 residual, prior, physical XYZ, rho and dt channels. It applies the
frozen normalization, gauges/scales a correction and adds it to prior **once**.
The bridge returns full Pa pressure with cell-zero gauge; the host must not add
prior again. Zero residual returns prior exactly. Wire records are bounded to
8 MiB and returned pressure to 65,536 finite values.

## Qualification when the missing inputs arrive

1. Retain exact Rheon/lab revisions, original versus relocated descriptor identity,
   complete file/runtime measurements, versions, scene/policy and all parity-input
   hashes. Verify checkpoint identity before executing any supplied model code;
   do not install or train a replacement implicitly.
2. Run the pinned lab `verify_deployment.py` with `--deployment`, `--packet`,
   `--inference` and a fresh `--output`, then its boundary tests as documented in
   the [lab README](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/README.md).
   Require byte-exact retained CPU prediction parity, exact zero-residual bypass,
   CPU/no-CUDA/no-optimizer facts and early `unsupported_context`. Preserve actual
   exit status, stdout/stderr and report; parity is separate from pressure accuracy.
3. Run the existing real-deployment lifecycle test with fresh evidence storage:

   ```bash
   RHEON_AI_TEST_DEPLOYMENT=/absolute/measured/deployment.json \
   RHEON_AI_TEST_OUTPUT=/absolute/FRESH-lifecycle-evidence \
   cargo test --offline --locked --features desktop --bin rheon-ai-compare \
     real_cpu_deployment_cancellation_drains_child_and_allows_restart -- --ignored --nocapture
   ```

   The current [test](../../src/bin/desktop/ai_comparison/worker.rs) uses counts
   `[8,6,5]` with the default physical/forcing values, cancels after at least two
   predictions following warmup, checks retained cancellation and child drain,
   then restarts to 16 steps with exactly one AI prediction and a distinct run.
   This legacy continuation lifecycle test does not certify model accuracy or
   teacher/stop-policy behavior.
4. Qualify the integrated typed teacher and independent-stop policies through
   the existing Engine/worker, preserving equal-request native-reference binding,
   actual history/transition, explicit no-candidate reasons and cold SQLite
   roundtrip. Teacher history must remain native even with biased output;
   independent stop must retain first rejection without advancing it. Preserve
   raw legacy JSON and distinguish these new policy results from legacy continued
   diagnostic trajectories. A storage fixture or protocol pass cannot establish
   measured-model parity or independent physics truth.
5. If native-window operation is claimed, separately observe the real executable
   and this measured package through start, warmup, prediction, cancel, restart
   and close. Record the exact platform and evidence; stdlib/Xvfb package fixtures
   and headless UI tests do not establish this measured-bundle result.

The retained lab result is **workflow verified, model accuracy failed**: the
64-step warm-start run has 49 AI checks and zero passes; the first supported
relative residual is about 0.0372545 against 1e-6. Preserve this adverse evidence.
Cold-start runs are unsupported testing setups, and native-prefix frames are not
AI passes. New qualification must report observed success, failure or unavailable
inputs honestly; it cannot promise an accuracy pass for the frozen checkpoint.

## Separate trajectory-v2 CI artifact

[AI-pressure CI](../../.github/workflows/ai-pressure.yml) fetches
`experiments/trajectory-v2/model.pt` at lab revision
`e8a90bfef6b2955c502008ccc62443e29dd9f2ec`, SHA256
`25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9`.
Its native modal-gain export is `RHEONAI1`, with a fixed 32³ unit domain and
density 1.25 kg/m³; see the [qualified component contract](../implementation/ai-provider/README.md).
That real-checkpoint CI fixture uses the native Rust forward pass and independent
NumPy replay. It is neither the seven-channel viscous-water network nor its
normalization, interpreter/Torch deployment, bridge, prefix or desktop workflow.
A green trajectory-v2 job therefore does not supply or qualify the missing
`aa82…a62d` measured live-water bundle.
