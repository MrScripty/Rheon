# Resumable third-velocity implementation checkpoint

The service meter reported 99% used; shell/tools remained usable. This snapshot
is **qualification in progress**, not a fully frozen final feature qualification.
The existing preflight process must not be duplicated or interrupted. Poll unified
session **77194**, then inspect the live untracked
`evidence/extruded-third-velocity/native-preflight/receipt.json`. This directory
contains immutable copies of its status/logs at this boundary. The source commit
and exact hashes are in `receipt.json`.

Source implements `CoupledDiscreteFlow::new_extruded` and `step_extruded` on the
same two-column graph: x and z periods one; z-invariant fields; constant rho=3,
mu=.05; impermeable/free-slip bottom and material cap with weak natural traction.
It carries all three components, not general 3D motion. Owner/candidate scalar
buffers are reused; the additional fixed scratch reservation gives 470,992 bytes
on this target, versus the unchanged zero-third 450,320 bytes.

The preliminary release route passed the seven full nonzero/constant/atomic
contracts and then the eighth actual large-field third failure test. The current
exact-source release route includes all eight plus old coupled/translated tests.
Its live receipt determines which final preflight commands have completed.
Preliminary strict normal replay of the first capture passed all 268 publications,
34 momentum equations, separate/total work, eight independently regenerated DAE
references and twelve corruptions. The corrected reader evaluates nonnegative
physical shear squares; its first failed matrix-quadratic constant-field reader
and diagnostic (-1.25e-19 loss) remain in `native-trials/`. The current mapped
capture adds actual periodic IDs and every other publication field is identical.
This is not a final optimized-mode replay or full Rust feature matrix claim.

Next, source `/workspace/rheon-setup/env.sh` (one Cargo build job). Finish/poll
77194 first. Run full Rust tests for default, `--no-default-features`, and
`--features desktop`, recording actual exits/logs. Current preflight runs fmt,
release contracts and all three strict all-target Clippy modes. Do not rerun a
completed check unless a change/failure justifies it.

Compile final default/no-default `extruded_third` exports and compare their bytes;
also rerun `coupled_discrete_identity` and compare to the frozen oracle capture.
Run current `replay.py` in normal and optimized Python with
`OPENBLAS_NUM_THREADS=1`, input `native-trials/mapped-native.jsonl` or the fresh
final export, and `--negative-self-test`. Both runs must regenerate references,
recompute all errors/ratios and retain unchanged work/GCL/momentum gates.
The old first export lacks the new mandatory periodic ID field and is only a
preserved preliminary trial, not final replay input.

Create actual-state PNG/GIF and an interactive *recorded-state viewer* using the
exported node IDs and velocities; it must describe this same doubly periodic,
z-invariant model. Do not substitute a separate simulator. Update a new native
chapter; the derivation chapter and all earlier book/packet files are frozen.
Finally freeze source and evidence separately with complete hashes, exact trees
and historical preservation. Native production source changed legitimately;
validate old oracle/equation receipts in their frozen worktrees, not against the
new live production files. Original Jacobi fixtures, prior negative evidence and
`.00078125` host Newton refusal remain untouched. No new Lean theorem.

Already pushed preserved milestones on this branch: composition
`acebeaa388a9971e6e5fa9677d58aa15d7a677f3` / evidence
`84f92259294fbf21df1990d3795500bd6df76211`; third derivation source
`76a6807ea696843395429554cec1bea4cf9963a7` / evidence
`cac2c7b11da5f0bdb0c9f352135ce9d58bc29da2`.
Parent owns reviews, PRs and merges. No PR was requested or created.
