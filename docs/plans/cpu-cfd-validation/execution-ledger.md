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
