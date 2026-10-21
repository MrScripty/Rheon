# Execution ledger

2026-10-13: selected next slice under continuing user authorization. Stacked
base bbc6089888e257962b968e70ed84ec61b7668d5b, main ancestor 2209012. Inspected
existing sphere/rigid candidates and StaticObstacleShear. Independent reviewer
confirmed the BE energy and equal/opposite impulse derivation before coding.
No borrowed candidate receipt is a new qualification.

2026-10-13: implemented reduced prescribed/dynamic plate coupling with no
production API changes. The solve reuses existing shear geometry/conductances;
new workspace stores four node-sized vectors and publishes both caller fields
and report only after every gate. An initial free-slip test exposed roundoff
in a disconnected uniform component; exact per-component nullspace preservation
fixed it without snapping small differences. Reduced the transient test dt from
h² to h²/8 (same T=0.125 s) to separate temporal error sufficiently to meet the
original fine-grid error target; did not relax that target.

Local acceptance commands/results:

- cargo test --locked --no-default-features --test plate_coupling_contract
  --test obstacle_flow_contract --test static_obstacle_contract -- --nocapture:
  37 passed. New suite has 7 model/transaction tests and 4 inherited interval
  tests; inherited obstacle/shear/geometry suites have 26 tests.
- cargo test --locked --test plate_coupling_contract --example plate_coupling:
  11 + 4 passed (example repeats interval primitives).
- cargo fmt --all --check: passed.
- cargo clippy --locked --no-default-features --example plate_coupling
  --test plate_coupling_contract -- -D warnings: passed.
- cargo clippy --locked --all-targets -- -D warnings: passed.
- python3 [and python3 -O] tools/check_plate_coupling.py --native
  /workspace/.rheon-tools/target/debug/examples/plate_coupling --output NEW_EXTERNAL
  --allow-dirty: 18 real native steps independently checked against rational
  dense solve; 11 corruptions rejected each mode. Final committed runs omit
  --allow-dirty and record exact head/source/binary hashes outside Git.

Continuum fixture: mu=rho=1, H=1, T=0.125 s,
u(y,0)=y+0.2*sin(pi*y), fixed plate speeds (0,1). RMS centroid errors:
N=8 dt=0.001953125: 0.0011362806949240976 m/s;
N=16 dt=0.00048828125: 0.00028526293107859464 m/s;
N=32 dt=0.0001220703125: 0.00007139048628485824 m/s.
Both error ratios exceed 3 and fine error is below 1e-4 m/s. This finite
resolved-mode refinement is not a general convergence theorem/calibration.

Independent review confirmed assembly, exact nullspaces, finite-mass impulse,
work, atomicity and payload accounting. Reviewer ran all 11 Rust tests and
independently verified 18 native steps/11 hostile records. Review found that
wide reported defect intervals could pass the initial oracle; requiring the
actual interval endpoints to satisfy the reported budget, and requiring the
zero-work energy-change upper endpoint nonpositive, closed the gap. No remaining
blocking review findings.

PR45 exact-head closure: bbc6089888e257962b968e70ed84ec61b7668d5b has passing
hosted core/default/desktop, executable aggregate, native/rational, Lean and
dense3D checks. Book build alone fails the frozen Cargo.lock admission already
reproduced on unchanged main; deployment is skipped. No gate was weakened.

Retained resources: /workspace/Rheon-physics at bbc6089 is protected by
origin/feature/bounded-obstacle-stokes-20261013 for parent PR45 review.
/tmp/rheon-physics-main-check at 2209012 is protected by origin/main and retained
for book-blocker reproduction. /workspace/Rheon-plate is retained-protected by
its dedicated remote branch after push for stacked PR review. This worker owns
only these task-created worktrees; parent disposition after integration/rejection
triggers safe cleanup. No owner/unknown checkout is eligible for this cleanup.
