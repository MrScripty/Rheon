# Divergence-only CG restart repair

Review [5405152446 / finding 4176859712](https://github.com/MrScripty/Rheon/pull/2#discussion_r4176859712)
is valid against accepted PR2 combined head
`da56ed2213326d53e8d62741ce074061820f05a5`. The shared `true_report` and
`solve_loaded` functions are byte-identical in accepted PR3
`2d36096f17c494b25c46e64335521cea5e19db5d`; their hashes are retained in
`source-equivalence-and-replay.json`. PR3 adds Jacobi/SGS preconditioner selection
around this shared loop.

PR2 functional repair is `f2c9e98777085443d502f63a0985302fbbc13dcc`, tree
`1f0e5ca1823ac915887bb8a119599f3c25e826e7`, on
`repair/pressure-divergence-restart`. Its unchanged cherry-pick onto PR3 is
`34f6165e6068997c3ef05b03d21241a360ccd84e`, tree
`4648fba4995ed839fa3957b2c90045fba8da8d3f`, on
`qualification/pr3-pressure-divergence-restart`. Only src/pressure.rs and the
focused pressure regression change in these functional commits. This evidence
commit records those source revisions, not a new algorithm revision.

## Mechanism and mathematical limits

Let T=max(absolute tolerance, relative tolerance times RHS L2 norm). The existing
linear candidate uses the gauge-eliminated recursive residual norm <= T.
Acceptance additionally requires the recomputed **full** residual, including
the gauge row, to satisfy L2 <= T and

```text
max(abs(b - Kp)) * dt / cell_volume <= divergence_limit.
```

Previously every rejected candidate replaced the recursive residual and reset
the direction to the preconditioned residual. Once the linear threshold was
satisfied but divergence remained too large, repeated candidates could therefore
reduce PCG to successive preconditioned steepest-descent steps. Neither a small
recursive residual nor a passed linear threshold establishes the divergence
condition.

The repair separates diagnostics from replacement. Full residual diagnostics
use the existing product scratch buffer and leave the recurrence intact. When
the full linear residual rejects a recursive linear candidate, the original
replacement/restart behavior remains. A divergence-only rejection instead
continues PCG. Checks still occur at every linear candidate and budget exhaustion;
successful reports still use the full true residual. T, the divergence limit,
iteration budget, cancellation checks and six-array allocation are unchanged.
No adaptive threshold factor, new configuration, solver method or extra buffer
is introduced. The existing actual-f32-divergence gate remains separate.

## Bounded reproductions and numerical changes

`probe.rs` and `probe-pr3.rs` specify a 5x4x3 grid, spacings 0.2/0.25/0.3,
density 1.3, manufactured pressure ((17*i) mod 31)/31, dt 0.02,
absolute residual 1e-12, divergence limit 1e-9 and iteration budget 120.

| Relative tolerance | Jacobi before / after | SGS before / after |
| --- | --- | --- |
| 1e-3 | IterationLimit(120) / success(35) | IterationLimit(120) / success(16) |
| 1e-6 | success(48) / success(35) | success(20) / success(16) |
| 1e-11 | success(39) / success(39) | success(18) / success(18) |

At 1e-3 the original PR2 trace records 103 divergence-only rejections/restarts,
iterations 18 through 120. `before-trace.log` was obtained by adding the single
diagnostic print in `instrumentation.patch` to the exact accepted source; that
print is absent from production. The new regression independently assembles
the closed-box stencil, checks both limits on the full residual, pressure/gauge
recovery and unchanged array bytes. It fails at the old 120-iteration limit and
passes the repair. Its failed baseline run is retained, including an isolated
target-directory repeat.

Pressure bits and report values change in the divergence-only cases, including
the 1e-6 cases that previously succeeded. These changes are the consequence of
preserving conjugate directions and stopping when both existing gates pass;
they are not a claim of a uniformly better numerical answer. Both methods'
1e-11 pressure arrays and reports remain byte-identical in the paired probe.
The bit comparisons are recorded in `receipt.json`.

All original demo-16, demo-plume and demo-64 opacity.png and steps.csv fixtures
remain byte-identical after **both** repaired release binaries. PR2 run.json
differs only in measured timing. Original fixtures and historical receipts are
not rewritten, and existing implementations/IDs are retained.

## Qualification and reproduction

Rust 1.92.0 (ded5c06cf21d2b93bffd5d884aa6e96934ee4234), LLVM 21.1.3,
x86_64-unknown-linux-gnu; default Cargo test/dev and optimized release profiles,
one Cargo build job, locked/offline dependencies. Shared Xeon Platinum 8370C
cloud VM, five visible CPUs, four-CPU cgroup quota, 16 GiB cgroup memory limit.
These are iteration/regression observations, not timing benchmarks or a
speedup/real-time claim. CPU access is not exclusive.

- PR2: 43 default and 39 core-only Rust tests; both strict all-target Clippy
  configurations and formatting pass.
- PR3 port: 50 default, 45 core-only and 56 desktop Rust tests; all three strict
  all-target Clippy configurations and formatting pass. Desktop GUI launch and
  visual/accessibility behavior were not qualified here.
- PR3: all 20 Python comparison/result-ownership tests pass against the repaired
  release CLI, including cancellation, timeouts and publication failures.
- PR2 proof-source gates pass under normal/-O Python. Proofs, inventory,
  dependencies, manuscript and export source bytes are preserved. No new Lean
  build or physical-model proof is claimed.

Commands and source/log hashes are in `receipt.json`. Use distinct Cargo target
directories per checkout. One initial matrix attempt raced with a baseline
build in a shared target and ran an overwritten test executable; the failed
log is retained and excluded from qualification. The isolated rerun passes.
An initial Python attempt lacked the tests' expected target/release/rheon path;
that failed log is also retained. A link to the private PR3 target directory
provided the existing expected path, without changing tests or product code.

To reproduce the focused test in the appropriate checkout:

```sh
CARGO_TARGET_DIR=/tmp/rheon-pr2-repair-target cargo test --locked --offline --test pressure_contract tighter_divergence_gate_keeps_conjugate_gradient_progress -- --nocapture
```

For paired diagnostics, use disposable checkouts of the named base/repair
commits, copy the appropriate probe into tests/pressure_restart_probe.rs, then
run cargo test --locked --offline --test pressure_restart_probe -- --nocapture
with a separate target directory for each checkout. The PR3 probe additionally
records pressure bits for both methods; its generated test file was removed
before the full candidate matrix.

Independent parent review remains pending. Neither accepted PR branch is
advanced. No PR update, external review request, thread resolution or merge.
No PNG flush change, new physics, paid service, credential/permission expansion
or network-policy change was made.
