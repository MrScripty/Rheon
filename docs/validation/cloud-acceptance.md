# Cloud correctness and CPU performance acceptance

## Goal and milestones

1. Reuse reviewed native reference/refinement/conservation cases without new physical claims.
   Acceptance: selected release contract targets pass unchanged, and reviewed Lean source/pin inventory passes.
2. Measure paired adaptive versus fixed-policy CPU execution across thread counts.
   Acceptance: at least three fresh process repetitions per pair; every accepted trajectory has identical
   field fingerprints and accepted time across policies, repetitions and threads for the same solver/size;
   every reported divergence is at most `1e-5`. Solver-to-solver bit equality is not required.
3. Retain replayable evidence and independent review, commit a dedicated branch and open a draft PR.
   Acceptance: exact commands, source/binary identity, raw logs, failures, hardware constraints and timing
   ranges survive in a machine-readable receipt. No merge or deployment.

Current base: `2209012`. No AGENTS.md, repository .agents/skills, coding-standards files, Standards MCP
or persistent native goal tool were available in this environment. Existing pinned tools, strict lints,
explicit Python checks, generated-evidence hygiene and established proof claim boundaries are followed.

## Run

```sh
export CARGO_TARGET_DIR=/tmp/rheon-cloud-target
python3 -m unittest discover -s tools -p test_cloud_acceptance.py -v
python3 -O -m unittest discover -s tools -p test_cloud_acceptance.py -v
python3 tools/cloud_acceptance.py /tmp/rheon-cloud-result --sizes 16 32 --threads 1 2 4 --steps 12 --repeats 3
```

The checkout must be clean and committed. Output must be fresh and outside the checkout. The driver builds the locked release example itself.
Linux `wait4` reports CPU time and this child's peak RSS in KiB, not a cumulative children high-water mark.
The receipt records affinity, CPU description, cgroup CPU/memory limits and before/after system load.
Runs are serial, alternate policy order and include cold scheduler/SGS calibration. Repetitions are fresh
trajectories from rest, not successive warm windows of one simulation. Thread oversubscription is permitted
and recorded, not interpreted as dedicated core availability. Shared-cloud timing is descriptive;
there is no flaky absolute speed threshold or assertion that adaptive must win.

`cloud_frame_lab` takes size, threads, steps, policy and registered pressure solver ID. `fixed` means
existing unbound hardware/workload-based batch sizing, not one globally constant chunk size. `adaptive`
binds the existing `CpuScheduler` to every worker. SGS's internal tile calibration remains enabled in both;
this experiment isolates linear batch feedback, not every adaptive mechanism. Numerical kernels are unchanged.
The runner covers both current registered pressure implementations. New provider implementations need
explicit registration in the matrix and model-specific correctness/acceptance cases before qualification.

CPU frame cost includes a full accepted simulation step and copying all velocity/tracer fields into an
owned reusable capture buffer, consuming every frame with a checksum. It includes scheduler warmup, but
excludes pool/simulation creation (reported process wall time includes these). Capture allocation is outside
the frame timer; its retained payload is reported separately from simulation payload and peak process RSS.
This is a bounded CPU consumer workload, not the desktop or a game engine's complete frame. Rendering,
GPU transfer, network/API serialization, actual CAD consumers, real-device frame budgets and AI inference
are unmeasured. No GPU or real-time claim follows from these results.

## Correctness and extension contracts

The release suite includes whole targets so added tests automatically join these gates:

- `pressure_contract`, `implementation_contract`: hand reference, manufactured anisotropic pressure,
  full residual/gauge acceptance, both registered methods and failure contracts.
- `parallel_contract`: replay and parallel execution contracts.
- `liquid_volume_contract`: closed-volume conservation, exact binomial transport reference and spatial
  refinement; this does not claim semi-Lagrangian tracer mass conservation.
- `viscosity_contract`: bounded shear reference, energy and fixed-grid time refinement.
- `dense3d_sequence_contract`, `dense3d_controls_contract`: current shared export/control schema and lifecycle.

Physics/API/AI owners can pass repeatable `--extra-contract TARGET` arguments once their test target exists
on the tested branch. Never treat a missing future branch or absent provider fixture as a passing gate.
The benchmark record schema is `rheon-cloud-acceptance-v1`; its acceptance parser rejects missing/extra
fields, nonfinite metrics, incomplete trajectories, excessive divergence and incomplete frame accounting.
Replay fingerprint is a regression diagnostic (64-bit collision risk), not a mathematical proof; native
contract tests provide separate field/report assertions.

`proofs/scripts/check_sources.py` verifies exact reviewed Lean source inventory and pins only. This run
does not rebuild Lean or rerun the kernel axiom audit. Existing exact finite-real theorems do not certify
Rust/IEEE refinement, geometry assembly, solver convergence, continuum correctness or performance.
See `proofs/README.md` for the actual theorem hypotheses and historical source-bound qualification.
