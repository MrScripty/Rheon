# AI comparison integration handoff

Publication branch: `integration/ai-pressure-comparison-20261015` in
`MrScripty/Rheon`. Base: main `43ed26b645dd42de906d0fac7d1ec7d5ab6e351a`.
The original comparison/lifecycle commit
`66e50388ec7a788dbda0ad1227e330996edf506e` is an ancestor, unchanged. It was
previously local, not rebased or replaced. The branch also includes
`fe9d58648f15a5aefd9e33fe47d4e6bc5f855b82`, the native training viewer and shared
voxel renderer. The handoff adds documentation, small receipts and stronger
lifecycle assertions; production comparison behavior is unchanged.

Matching already-published lab branch: **`feat/sealed-water-pressure-training`**,
commit **`a8426061380e82951b4dba1bbfa38ff93dc7ebb2`** in
`MrScripty/rheon-surrogate-lab`. The adapter/export implementation entered at
`d6a92920bd5d9bbcc693709182d9cf35f8a910db` and is present at that matching tip.
The deployment's model-training source is
`11408e93995a9f3c3f59e171ccd0889906531027`; those are distinct provenance roles.

## Source map

| Responsibility | Source |
|---|---|
| UI start/restart, cancel/close, retained timeline | [ai_comparison/mod.rs](../src/bin/desktop/ai_comparison/mod.rs) |
| Joinable worker, two tracks, terminal publication, lifecycle test | [worker.rs](../src/bin/desktop/ai_comparison/worker.rs) |
| Child startup, bounded protocol, deadlines, kill/wait/readers | [model_process.rs](../src/bin/desktop/ai_comparison/model_process.rs) |
| Native prefix, previous-pressure ownership, independent evolution, gates | [engine.rs](../src/bin/desktop/ai_comparison/engine.rs) |
| Physical constants, shape/spacing/force budgets | [scene.rs](../src/bin/desktop/ai_comparison/scene.rs) |
| Per-run lease, scene/model/frame/completion records | [record.rs](../src/bin/desktop/ai_comparison/record.rs) |
| Signed pressure differences and legend | [view.rs](../src/bin/desktop/ai_comparison/view.rs) |
| Standalone CLI and non-GUI verification | [rheon-ai-compare.rs](../src/bin/rheon-ai-compare.rs) |

Lab sources below are pinned to the matching published commit:

- [Deployment export and training-context binding](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/deployment.py).
- [CPU model bridge and request/reply validation](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/cpu_bridge.py).
- [Seven-channel preprocessing and output reconstruction](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/cuda-pressure-v1/model.py).
- [Water model wrapper](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/sealed-water-v1/model.py),
  [deployment instructions](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/experiments/live-water-comparison-v1/README.md),
  [retained acceptance](https://github.com/MrScripty/rheon-surrogate-lab/blob/a8426061380e82951b4dba1bbfa38ff93dc7ebb2/docs/plans/live-ai-pressure-comparison/reports/acceptance.json).

## Worker and child lifetime

`Worker::start` validates the scene and creates fresh shared history, cancellation
flag, child slot and one joinable thread. Numerical work runs in an owned
one-thread Rayon pool. `Record::begin` takes a project-directory file lease and
creates a fresh run directory. Restart creates a new simulation and child; it
does not resume the cancelled pressure/velocity history.

`ModelProcess::start` bounds and decodes the descriptor, checks its profile and
context policy, hashes the interpreter/bridge, then launches the measured Python
directly with `-I -S -B`. It clears the inherited environment and sets only the
declared PATH, CPU selection and single-thread BLAS/OpenMP variables. Stdin,
stdout and stderr are piped. The child slot is installed with a second
cancellation check to handle cancellation racing startup.

Owned output/error threads drain the pipes. Stdout uses closed typed replies,
finite pressure values, request correlation and an 8 MiB record bound; its queue
holds two replies. Stderr exceeding 64 KiB kills the child. Startup accepts
loading records then requires a matching checkpoint/Run identity, seven float32
channels, CPU device, zero optimizer updates and no CUDA initialization. Startup
reply deadline: 90 seconds. Prediction reply deadline: 30 seconds; cancellation
is polled during receives. Request writes use ordinary blocking pipe I/O.

Cancellation sets the shared flag and invokes `Child::kill`. On Linux this kills
the immediate child with SIGKILL. Numerical callbacks also observe cancellation.
`ModelProcess::drop` stops delivery, closes stdin, takes the child, calls
kill/wait, then joins both readers. Successful execution explicitly drops the
model before publishing completion; error unwinding drops it before publishing
failure/cancellation. Worker completion therefore follows the normal child and
reader teardown path. `join_finished` joins the thread and diagnoses panic;
`Worker::drop` cancels, joins and makes a final child kill/wait attempt.

The UI defers window close during active work: it cancels, waits for the worker
to finish/join, then permits close. Starting again replaces the completed worker
with a fresh one. Retained immutable frames survive cancellation and can be
scrubbed. `latest.json` and `activity.json` are atomic observations; this worker
does not claim Tuldok managed-Run custody.

Limits relevant to a generic provider architecture: this is **one trusted local
Python child**, not a process-group/subreaper or plugin sandbox. Descendants are
not tracked or killed as a tree. Kill/wait and join cleanup errors are currently
best-effort rather than a structured OS custody receipt; poisoned locks/panic
have weaker reporting. Blocking writes, file hashing and reader joins have no
independent hard teardown watchdog. The real test covers the measured child and
normal cancellation/restart, not arbitrary hostile descendants, every startup
failure, or every shutdown race. Preserve ownership while designing a broader
provider lifetime instead of inferring generic containment from this test.

## Deployment and model adapter

Descriptor schema: `rheon-pressure-cpu-deployment-v2`; profile:
`sealed-water-seven-channel-float32`. Export consumes completed/drained training
artifacts and verifies their hashes, source witness, final declared checkpoint,
normalization and registered runtime. It copies the measured inference sources
and inventories them in the deployment. Context is derived from the exact
hash-bound training split artifact, not a historical constant replaced in code.
Runtime/interpreter/source/weights are measured at loading. Host OS libraries and
assets remaining unchanged during execution are explicit trusted-host assumptions;
there is no sandbox or cross-machine runtime portability qualification.

The deployed baseline has 2,000 training updates and checkpoint hash
`aa82bd122fde9406d9e6b92114b3e6ba77281f5ee8b9c726aa168dacbd93a62d`.
The later local lab branch `experiment/water-learning-sequence-v1` at
`4402db4915ecd1f5267d217317c011595ee7ab3e` contains newer training results; its
selected weights have **not** replaced this comparison deployment.

Each request contains `id`, `counts_xyz`, scalar `cell_size_m`, `density_kg_m3`,
`dt_s`, `base_pressure` and `native_input_residual`. Arrays are full-grid,
X-fastest, finite float64 values. They reshape to `[1,1,Z,Y,X]` on CPU. The
residual is `b - A(p_previous)` from the AI track's actual prepared velocity;
it is not a reward model or the previous frame's residual. Pressure is in Pa;
this native integrated pressure-operator residual is in m³/s².

The network input is float32 `[1,7,Z,Y,X]`, ordered:

| Channel | Preprocessing |
|---|---|
| native residual | Divide by train-only residual RMS scale |
| previous pressure | Divide by train-only pressure RMS scale |
| X, Y, Z | Physical cell centers `(index + 0.5) * h`, in metres with unit scale 1 m |
| density | Broadcast scalar divided by train-only density scale |
| timestep | Broadcast scalar divided by train-only timestep scale |

There are no voxel-size channels. Scalar `h` generates coordinates and defines
the physical operator. Normalization is loaded from the hash-bound training
artifact, never recomputed from the evaluation trajectory. The seven-channel
GroupNorm network has 371,729 parameters and returns one same-grid correction.
Raw float32 output is converted to float64, multiplied by the pressure scale,
and has its cell-zero value subtracted. Returned absolute pressure is previous
pressure plus that gauge-fixed correction. All-zero residual bypasses the network
and returns the preceding pressure exactly. The bridge replies with absolute
pressure; a provider must not add previous pressure a second time.

The bridge is stateless across requests apart from resident weights. There is no
recurrent hidden state or stack of previous frames. Rust owns pressure, velocity
and tracer histories. The original native snapshot indices `[15,63]` yield a
15-step minimum native prefix. Physical steps 1–15 clone the qualified native
state to both tracks without inference; the first AI request is step **16**, with
step-15 solved pressure. The worker enforces the minimum and the bridge rejects
early IDs as `unsupported_context`. An ID alone does not prove actual history:
the bridge trusts the Rust caller to provide the corresponding state.

After handoff the tracks evolve independently. Native reference steps must
qualify; raw AI equation/divergence failures remain diagnostic frames, with no
native fallback substituted in the AI track. Hard finiteness/gauge/Courant/wall
or flux failures stop the run. The signed pane compares evolving trajectories;
after the first handoff it is not an isolated same-RHS pressure-solver benchmark.
The core accepted `Simulation`/provider contract remains unchanged. A future
fallback/acceptance policy needs an explicit owner and must retain the AI verdict.

## Enforced physical profile and coverage

The reference uses `JacobiPcgV1` by default; `Scene.sgs` selects
`SymmetricGaussSeidelPcgV1`. These are native PCG preconditioner choices, separate
from the raw learned pressure proposal.

| Setting | Current enforcement | Coverage/assumption |
|---|---|---|
| Density | Constant 1000 kg/m³ in native engine/request; descriptor and bridge checked | Other positive densities supported by lower model helpers are not enabled here |
| Dynamic viscosity | Constant 0.001 Pa·s in native viscosity stage; Rust checks deployment profile | Not a bridge request feature; cannot validate arbitrary viscosity from that message |
| Timestep | Constant 1/60 s in engine; bridge checks exact value | No variable/adaptive timestep integration |
| Grid | Each axis 5–128; product ≤65,536 | Training coverage 5–16 per axis; permitted larger sizes are extrapolation |
| Spacing | `h = 1/cells_per_metre`, integer cells/m 1–128, spacing `[h,h,h]` | Bridge accepts finite scalar h in [1/128,1]; training used 8 or 16 cells/m |
| Domain size | Counts times h per axis | Rectangular domains allowed; voxels remain cubic |
| Coordinate origin | `[0,0,0]` by constructor; coordinate channels use that origin | No origin field in bridge protocol; translating a generic grid needs an adapter change |
| Boundary | Stationary sealed box, zero normal wall velocity, free-slip; checked after correction | Homogeneous sealed pressure operator; no inlet/outlet, mass emitter, obstacle or free-surface model |
| Material/forcing | Filled uniform-density water; bounded localized directional/swirl/fluctuating acceleration | Prescribed fluctuations do not prove resolved turbulence |
| Pressure gauge | Cell (0,0,0) exactly zero, input/output checked | Comparisons require the same gauge |

Scene steps: 1–512, with `warmup_steps < steps` and checkpoint minimum enforced.
Retained numeric payload is bounded by `(steps+1)*cells*24 <= 96 MiB`; this is
not a total process-RSS bound. Native iteration limit is 3,000, reference relative
residual target 1e-8, model target 1e-6 and absolute residual floor 1e-12.
The native solver divergence setting and comparator predicted-divergence limit
are 1e-7 s⁻¹; the comparator separately allows actual corrected-float32 divergence
up to 1e-5 s⁻¹. Courant must not exceed 0.5. Wall-normal velocity is exactly zero;
net flux is bounded by max(1e-12, 1e-8 × sum(abs(divergence)) × cell volume). These runtime constraints do not establish model
accuracy: the retained supported 64-step run passed **0/49 AI checks**.

## Reproduce and inspect lifecycle evidence

Original evidence from `66e50388...` is retained unchanged in
[the original test log](ai-pressure-comparison-tests/lifecycle-original.log.json).
The publication rerun strengthens assertions for the restart's AI phase and Linux
immediate-child disappearance. Its [log](ai-pressure-comparison-tests/lifecycle-publication.log.json)
and [small receipt](ai-pressure-comparison-tests/integration-receipt.json)
retain counts, source hashes, command and environment, without weights or fields.
Log JSON envelopes preserve exact original UTF-8 text and its SHA-256, including
trailing empty lines.

```bash
# LAB and RHEON name your checkouts; deployment is a separately exported artifact.
RHEON_AI_TEST_DEPLOYMENT="$LAB/runs/live-ai-comparison/deployment-2/deployment.json" \
RHEON_AI_TEST_OUTPUT="$LAB/runs/live-ai-comparison/FRESH-lifecycle" \
CARGO_TARGET_DIR="$LAB/runs/live-integration/native-target" \
cargo +1.92.0 test --manifest-path "$RHEON/Cargo.toml" --offline --locked \
  --features desktop --bin rheon-ai-compare \
  real_cpu_deployment_cancellation_drains_child_and_allows_restart -- --ignored --nocapture
```

Run with a fresh output path. Measured environment: Linux Mint 22.2 x86_64,
kernel 7.0.0-31-generic, Rust 1.92.0; installed Pumas Python 3.14.7, Torch
2.14.0+cu132 and NumPy 2.5.3, explicitly CPU-only inference. The CUDA-capable
package does not mean this test uses GPU inference. Child environment includes
`CUDA_VISIBLE_DEVICES=''`, `TULDOK_TORCH_DEVICE=cpu`, and BLAS/OpenMP thread counts 1.

Cancellation begins after at least two AI predictions (physical step ≥17),
preserves at least 18 frames including initial state, publishes `cancelled`,
joins the worker and empties its child slot. Restart in the same leased project
directory creates a distinct run, completes 16 steps and retains 17 frames.
**Exactly one restarted step performs AI inference:** step 16, after 15 native
steps. Both child PIDs are absent from Linux `/proc` after joining. This covers
basic post-warm-up inference/restart, not a long restarted AI rollout or accuracy.

## Coordination and retained resources

This is a shared publication branch for Jeremy's configurable-solver integration;
preserve its commits and use forward commits for follow-up. No merge into main
or rewrite is performed. The local worktree under the lab's
`runs/water-learning-sequence-v1/Rheon` is retained while it serves native viewer
and project source references. Retire it after accepted integration and consumer
migration, with the published ref protecting its head.

At publication, the original Rheon checkout on local `main` has unrelated dirty
recording/particle/simulation/frontend changes: notably `src/simulation.rs`,
`src/recording.rs`, `src/particle_fluid.rs`, `tests/recording_contract.rs`, desktop
`history.rs`, `inspector.rs`, `archive_view.rs`, `particle_view.rs`, `viewport.rs`,
`rheon-desktop.rs`, and docs. The older serving checkout also has dirty
`desktop/live_project.rs` and its README. Those changes are not included or owned
by this handoff; coordinate with their owner before overlapping. Use a clean
checkout of the publication branch. The lab's newer water-learning experiment
and selected candidate are separate from the frozen comparison adapter.
