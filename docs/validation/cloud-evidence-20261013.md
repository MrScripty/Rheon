# Qualification evidence: 2026-10-13

Clean source: `1f79a4dac4eca66de4c08c65824dca61a977dd03`; base `2209012bc5f902693d0f695ec1c2d854126af317`.
Binary SHA-256: `3b146a23503fd7fcd380e0d41ef69891e649ef89b947c99b9e97967196cc3616`.
Local receipt SHA-256: `554682d76e6ff0cb7ccca5bbfd9c54c4aa72b68f62bd38bbd94ece4dd0da5c2f`.

Raw local receipt and gate/sample logs: `/tmp/rheon-cloud-final/`. Generated evidence is intentionally outside Git.
[Hosted cloud workflow passed](https://github.com/MrScripty/Rheon/actions/runs/38380775812) on the PR merge checkout, with the raw artifact retained for 14 days. The workflow reports successful upload; direct artifact download from this environment returned HTTP 403. Local measurements below are independently available.

## Hardware and scope

Linux x86_64, AMD EPYC 9V74 80-Core Processor; affinity CPUs `0,1,2,3,4`; cgroup quota `400000/100000` (four CPU equivalents); 16 GiB cgroup memory limit. Rust 1.92.0, locked release build with `parallel`. Shared host; no exclusive-host claim.

Both current pressure methods, sizes 16³/32³, 1/2/4 Rayon workers, fixed/adaptive linear batch policy, three independent process repetitions and twelve accepted steps each: **72 process trials**. Policy order alternates by repetition. Fixed means existing unbound workload sizing. SGS tile calibration remains enabled for both policies.

CPU frame timing includes simulation step plus owned field copy/checksum, with reusable capture allocation outside the timer. Process timing and per-child peak RSS also include startup. This does not measure rendering, GPU transfer, network serialization or a complete device/game-engine frame.

## Results

All 72 trials passed replay/time and divergence acceptance. Accepted time: `0.23999999999999996` s. Maximum divergence: `1.0058283805847168e-07` 1/s. Peak process RSS across all trials: **14,720 KiB**.

| Grid | Workers | Solver | Fixed CPU frame ms, median [min–max] | Adaptive CPU frame ms, median [min–max] | Fixed/adaptive median |
| --- | ---: | --- | ---: | ---: | ---: |
| 16³ | 1 | jacobi-pcg-v1 | 17.562 [16.592–18.660] | 17.818 [17.652–17.963] | 0.986× |
| 16³ | 1 | sgs-pcg-v1 | 14.353 [14.291–14.922] | 15.163 [14.566–15.384] | 0.947× |
| 16³ | 2 | jacobi-pcg-v1 | 17.557 [16.894–18.474] | 18.045 [17.220–19.886] | 0.973× |
| 16³ | 2 | sgs-pcg-v1 | 14.803 [14.565–15.276] | 16.023 [15.049–16.072] | 0.924× |
| 16³ | 4 | jacobi-pcg-v1 | 17.167 [16.262–18.056] | 18.252 [17.188–18.268] | 0.941× |
| 16³ | 4 | sgs-pcg-v1 | 14.606 [14.393–16.109] | 14.977 [14.665–16.021] | 0.975× |
| 32³ | 1 | jacobi-pcg-v1 | 247.788 [246.233–263.833] | 235.881 [234.971–259.738] | 1.050× |
| 32³ | 1 | sgs-pcg-v1 | 167.075 [166.672–171.413] | 171.266 [167.319–173.306] | 0.976× |
| 32³ | 2 | jacobi-pcg-v1 | 215.256 [184.960–217.347] | 196.192 [181.725–210.849] | 1.097× |
| 32³ | 2 | sgs-pcg-v1 | 118.175 [108.268–119.851] | 113.054 [100.501–130.867] | 1.045× |
| 32³ | 4 | jacobi-pcg-v1 | 208.215 [199.246–212.661] | 194.465 [189.863–217.391] | 1.071× |
| 32³ | 4 | sgs-pcg-v1 | 116.624 [112.923–118.120] | 122.808 [104.660–131.457] | 0.950× |

Simulation/capture retained payloads are 333,824/68,608 bytes at 16³ and 2,646,016/536,576 bytes at 32³. These payloads are not whole-process memory caps.

The earlier full run from `bf46cd8` used the same binary hash. Some apparent gains reversed: 32³ SGS with two workers changed from 0.859× to 1.045×, and four workers from 1.102× to 0.950×. These short shared-cloud measurements do not establish a universal adaptive speedup or a stable performance regression threshold. Deterministic correctness and measurement validity are regression gates; timing is observational.

## Exact checks

- `cargo fmt --all --check`: passed.
- `cargo clippy --locked --features parallel --all-targets -- -D warnings`: passed.
- `cargo test --locked --no-default-features`: complete suite passed, including the 412-second material regression group.
- `python3 -m unittest discover -s tools -p test_cloud_acceptance.py -v`: four passed.
- `python3 -O -m unittest discover -s tools -p test_cloud_acceptance.py -v`: four passed.
- `python3 tools/cloud_acceptance.py /tmp/rheon-cloud-final --sizes 16 32 --threads 1 2 4 --steps 12 --repeats 3`: passed.
- The driver runs 58 release tests across pressure, implementation, parallel, liquid-volume, viscosity and dense3d sequence/control targets, with `--nocapture`; reviewed Lean source/pin inventory gate passed.

The retained slab oracle gives L1 volume errors 0.09375, 0.068359375 and 0.04909515380859375 at 16/32/64 cells with Courant 0.5; maximum volume-balance error is zero in these fixtures. At 64 cells, Courant 0.25 gives 0.06039030345194791: smaller timesteps do not remove spatial transport diffusion. Existing tests preserve their independent oracle and refinement requirements.

## Independent review and limits

An independent reviewer checked the benchmark, harness, negative tests, docs and workflow. Findings fixed: dirty-source reproducibility, duplicate/invalid measurements, CI grids below the parallel threshold, descendant timeout cleanup and workflow context placement. Final review reports no blocking findings. The hosted workflow context error was fixed before its successful run; no numerical test was weakened.

Lean source identity is checked, but no fresh Lean kernel or axiom audit was run here. Exact finite-real results do not prove Rust/IEEE correspondence or continuum validity. GPU, ARM/non-Linux devices and rendered game-engine budgets remain unqualified. AI pressure-stage checks are separately bounded below. The benchmark covers grid pressure solvers, not particle-frame throughput.

## Independent worker-branch checks

These checks use isolated detached worktrees; none of the worker implementations were merged into this branch. They qualify the exact commits below, not a combined integration branch.

- Physics [PR #45](https://github.com/MrScripty/Rheon/pull/45), `bbc6089888e257962b968e70ed84ec61b7668d5b`: `python3 tools/qualify_aligned_stokes.py /tmp/rheon-physics-independent` passed from a clean checkout. Its source-bound receipt records 57 Rust tests, strict Clippy/formatting, 374 rational primitive cases (366 accepted, eight explicit refusals), and 18 Python tests each in normal/optimized modes. No fresh Lean kernel run. The experiment is bounded aligned-obstacle decay, not arbitrary CAD flow or a continuum refinement qualification.
- Inspection [PR #47](https://github.com/MrScripty/Rheon/pull/47), `1923d5bd3442a5369878a1a317b52f4a89026d5b`: `cargo test --locked --no-default-features --test particle_inspection_contract` passed (one test). Log: `/tmp/rheon-inspection-independent.log`. This checks emitted identity and accepted-generation coherence, not UI rendering.
- API [PR #48](https://github.com/MrScripty/Rheon/pull/48), `f06a1270935b2ac285c68c0e594ab0742220913e`: `cargo test --locked --no-default-features --test session_contract` passed (five tests). Log: `/tmp/rheon-api-independent.log`. These cover accepted state lifecycle, export/quota and rollback; no network transport or game-engine budget claim.

After integration, append `--extra-contract aligned_stokes_contract --extra-contract particle_inspection_contract --extra-contract session_contract` to the driver when those targets exist. The physics rational oracle remains an additional required gate; native target success alone does not replace it.

### Actual AI pressure checkpoint

AI [PR #49](https://github.com/MrScripty/Rheon/pull/49), `b8ca87cab7f74f64c1686714a33e843fbb178c3d`, was checked in a detached worktree. The pinned upstream checkpoint at `e8a90bfef6b2955c502008ccc62443e29dd9f2ec` has SHA-256 `25533a2fbaa5ab0f5fb574dc6307a2b8d6a686deb7db867d0d03338d4d70d5a9`; exporter identity verification passed and produced SHA-256 `902ad8d77b16ae42f22f90bf71c22c19ae2e3cbe0cc0f4cc3d770930d4049aea`.

Exact independent commands, with pinned Rust and NumPy 2.3.5:

```sh
python3 tools/export_ai_pressure.py /tmp/rheon-ai-independent/model.pt /tmp/rheon-ai-independent/modal.bin
RHEON_AI_EXPORT=/tmp/rheon-ai-independent/modal.bin cargo test --locked --no-default-features --test solver_provider_contract -- --include-ignored
cargo run --locked --release --no-default-features --example ai_pressure -- /tmp/rheon-ai-independent/modal.bin /tmp/rheon-ai-independent/run
python3 tools/verify_ai_pressure.py /tmp/rheon-ai-independent/model.pt /tmp/rheon-ai-independent/run
RHEON_AI_CHECKPOINT=/tmp/rheon-ai-independent/model.pt RHEON_AI_RUN=/tmp/rheon-ai-independent/run python3 -m unittest discover -s tools -p test_ai_pressure_tools.py -v
RHEON_AI_CHECKPOINT=/tmp/rheon-ai-independent/model.pt RHEON_AI_RUN=/tmp/rheon-ai-independent/run python3 -O -m unittest discover -s tools -p test_ai_pressure_tools.py -v
```

All six Rust contracts passed, including the otherwise ignored real-checkpoint test. Five Python artifact/replay/refusal tests passed in each mode. The real release example completed 36 calls (four deterministic MAC fields, three rotated repetitions, three methods); the independent replay measured native/checkpoint relative L2 `1.0654486812809561e-15`, full-operator relative residual `3.1293922175596754e-9`, and predicted divergence `6.608867712346959e-10`.

Single-thread corrected-call medians were 187.045 ms Jacobi, 150.472 ms SGS and 11.674 ms learned. This is a bounded 32³ pressure-stage comparison, with actual corrected divergence checks, not full-trajectory or game-engine speedup evidence. No comparison to an analytic spectral inverse was performed. Raw logs and fields are in `/tmp/rheon-ai-independent/`. The real weights and independent replay remain required alongside `--extra-contract solver_provider_contract`; default test discovery alone skips the actual-checkpoint gate.
