# Execution ledger

2026-10-14: Inspected clean saved checkout; refreshed remote refs without reset,
stash or rewriting. Created isolated validation worktree from main 8baf459.
Read current public CPU sources, existing pressure/viscosity tests and milestone
limits. No local AGENTS.md/.agents skills found. Read current Coding-Standards.
User task admits the scoped plan and implementation. Primary-site research
found NASA TMR's move to tmbwg.github.io; source rights remain asset-specific.

Plan written before implementation. Acceptance pending; no solver-core edits.

Elementary implementation executed 12 public-CPU refinement points, strict
Clippy/format, one Rust diagnostic test, 12 Python acceptance/oracle tests and
18 existing pressure/viscosity contract tests. Development/precommit receipts
are external at /tmp/rheon-cpu-cfd-development and /tmp/rheon-cpu-cfd-precommit;
they identify dirty source hashes and are not clean release checkpoints.
Both pressure methods show spatial orders 1.902/1.976, viscosity spatial
1.975/1.985 and temporal 1.003/1.002. Actual projected divergence <=4.41e-12;
viscous <=7.75e-6; wall-normal/flux/energy-growth maxima zero.
The first new-resolution build hit a transient dependency-download 403 before
execution; seeding the standalone lock from current root versions and rerunning
resolved it. Public API compile errors were repaired in harness only. A Python
mock inadvertently intercepted platform discovery; isolating platform metadata
in the mock repaired the test. Subsequent named checks passed.

Parent then supplied reviewed solver plan PR72 at 67baa105 and assigned the two
independent verifier tools. Read S1-A through S1-G and separate educational
reference/oracles. Replanned the next slice to native S1 validation; no solver
core, public exports, or canonical solver plan edits. Format and exact acoustic
cell-average initialization proposed through parent; native evidence pending.

Committed/pushed elementary slice as ef4f8e4a59d119533f0e41cf89fa11f3f94ab759;
draft PR73 opened. Re-executed the real release suite from that clean commit at
/tmp/rheon-cpu-cfd-ef4f8e4-clean: all 12 points passed, CSV hash recorded in the
authored validation-results report. No generated logs/frames committed.

Implemented the two delegated S1 tools in disjoint scope. Exact conservative
Sod integration splits at waves and derives reference pressure after averaging U;
linear conservative acoustic averages use analytic sinc factors. Observed Sod
root bracket 5.55e-17, sampled acoustic nonlinear residual 3.97e-11 with doubling
ratio 4.0000063. All 19 verifier tests passed normally and -O on NumPy 2.3.5.
Synthetic reader/policy probes remain explicitly separate from real-native
acceptance. Corrected one negative test's expected diagnostic to the actual
intended unresolved-second-mode boundary; no gate was relaxed. Cargo executable
selection follows reported compiler-artifact identity, not a guessed path.

Native producer format/clean implementation commit has not been supplied/admitted;
A6 remains blocked pending parent coordination. Installed required NumPy wheel
license text identifies GPL-with-GCC-exception and LGPL bundled libraries;
recorded parent policy disposition need without changing/installing runtimes or
claiming a GPL-free environment. Original worktree/core sources remain untouched.

Parent subsequently admitted the export format/conservative initialization at
4621d69 and relayed it to the native owner. Parent permits the existing NumPy as
a separately installed validation tool; no runtime distribution is proposed or
authorized and component licensing remains disclosed. Inspected PR73 hosted
push/PR runs at the exact 4621d69 head: both dense3D workflows passed, neither
executes the new S1 verifier. CodeRabbit skipped review because the PR is draft.
Launched an explicitly requested read-only independent verifier review.

Review found capture defects: empty sandbox markers rejected allowed output,
and ignored build.rs could affect a clean-commit build without source binding.
Repaired the full input boundary using a tracked-only Git archive, complete
exported-source/archive hashes and unsupported Cargo configuration/compiler
override rejection. Original ignored work/markers are preserved. A tiny real
native protocol test verifies exclusion without CFD fields or S1 acceptance.
Independent follow-up confirmed both original closures and 23 normal/-O passes,
then found CARGO_BUILD_RUSTFLAGS bypass. Added its guard and exact-diagnostic
tests; final checks/review pending. No old receipt was refreshed. The admitted
first-mode-only speed scope and second-mode counterexample are now explicit;
no threshold/native format change occurred.

Final independent focused review confirmed the build-specific flags repair;
all findings resolved. Final 23 tests passed normally and under -O, including
the tiny native capture-protocol probe; py_compile and diff checks passed.
This is capture mechanics evidence only. Actual S1 source pin/numerical capture
remains pending; no numerical receipt/source admission was refreshed.

Parent supplied native 1b62ae3 and its reported numerical failures. Created
read-only retained-protected native input worktree/ref and executed reviewed
tracked-only origin capture from clean verifier aea43a5. Original failure and
origin pin 7b53940f...e5c4dbec remain unchanged externally. Diagnosed avoidable
Sod t=0 quadrature rounding; direct analytic constant-state integration makes
all native initial differences zero without widening 16-epsilon allowance.
All 24 tool tests passed normally and under -O. Independent reviewer confirmed
repair and reproduced acoustic diagnosis with a separate scalar Fourier model.
Actual acoustic order 0.5614384 is reproduced as 0.5614385 by the independent
linearized LLF symbol. Documented causal cell-average/resolution evidence and a
prospective 16/24/32 heuristic requiring parent admission. No threshold, native
fixture, solver or exporter edit; no acceptance/source admission refreshed.

Committed/pushed the narrow oracle repair and causal report as
6454acf79279330eb01aaee568f1d347688c84f5. Fresh tracked-only capture of native
1b62ae3 built and ran, then failed exactly acoustic/order-below-0.6. Origin
e7c098d13eb19fcc54abb37436c0f0e0ffc7e851cd30487e0a590a712c1866a7 and
failed assessment are retained in /tmp/rheon-s1-1b62ae3-6454acf-sod-fixed.
Read-only externally pinned verify repeated that failure. Original aea43a5
Sod failure remains untouched. Both dense3D hosted workflows passed at 6454acf;
they do not execute S1. No open bounded review findings. Plan is Blocked pending
parent prospective replan admission; no numerical acceptance or release tag.
