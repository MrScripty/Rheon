# Qualified main / liquid-volume composition

Qualified source `f4af9b47223f4f6928f40f7f3b5283e6023bf620`, tree
`a209d4b8739f3bd353c4c685ac408c70aeae184c`. Ordered parents are PR16 main
`4be8f9dffa179a22c7ef127ca2948f18d0c245a3` (tree
`fdaee0494a05b985c8a71f80eb901caf580da1da`) and accepted scale-guard candidate
`7985ca568d4c921fab782d22db9c05729329112e` (tree
`b7d4bbda61fc9c4df79c949e9d12e389518a3869`, parent `e6243021792ae894f7eab29e34ca64278159dbcb`).
No rebase, cherry-pick, force push or rewritten historical receipt. Automatic
merge had no conflicts. `src/simulation.rs`, `src/pressure.rs` and
`src/operator.rs` are identical to PR16 main; shared divergence error conversion
is retained. Incoming code differs only by rustfmt's one-line fixture wrapping.

Rust/Cargo 1.92.0, LLVM 21.1.3, locked dependencies, one build job, default
Cargo profiles. Actual default/core/desktop test counts are 138/133/144; focused
liquid tests: 18. Formatting and all-target Clippy with warnings denied pass in
all three feature configurations. `commands.json` retains the initial formatting
failure; `commands-green.json` records all successful native checks. The repair
rejects nonzero subnormal products/divisions, nonzero underflow to zero and
nonfinite arithmetic; accepted state is preserved. This remains a conservative
scale admission policy, not a certified IEEE error bound.

Fresh release build replayed all three original Jacobi fixtures. PNG/CSV bytes
match exactly, and stable original manifest fields match. Whole manifests contain
elapsed timing and do not match byte-for-byte. Existing exact proof-source and
PDF freshness checks pass. No new Lean theorem or Rust refinement proof is claimed.
`preservation.json` checks 1238 main and 1318 candidate frozen evidence/proof/book
chapter/PDF blobs. Historical candidate packet verifiers still describe their
historical source and must be run there; no receipt is rebound to this repair.

This is the tested composition checkpoint before atomic carrier/liquid transport
work. Existing numerical/proof/real-time/free-surface limitations remain intact.
Parent owns reviews, PRs, merge decisions and PR16 CI monitoring.
