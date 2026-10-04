# Accepted result-validation integration

Expected prior PR3 head: `0de6a857adb17a6ee0312e889aff50e2cd7aa6dd`.
Parent independent review accepted validation head
`416744700a45a8e74de27ca4e25cd00b0931926c` and successor terminal-state repair
`8bc0f54604d98d838b3a1ab2f6fcdfff9af2ac51`. Both are preserved as ancestors;
the lineage is already a fast-forward, with no merge or cherry-pick required.
This integration adds evidence only to the accepted source snapshot.

The harness pins/checks the complete matched workload, qualifies every accepted
diagnostic step and PNG container, and publishes only after qualification.
Publication failure enters failed state before cleanup/persistence, clears result,
retains primary/cleanup/persistence errors and stays failed on repeated polls.
Failure to remove comparison.json or persist status.json can leave stale files;
the job's terminal state/error is authoritative. No crash-durable storage claim
is made. Cancellation and optional timeout semantics remain intact.

## Combined local checks

Official Rust 1.92.0 / LLVM 21.1.3, Linux x86_64, one Cargo build job:

- 49 default, 44 core-only and 55 desktop feature tests pass.
- Strict Clippy in all three configurations and formatting pass.
- Locked release CLI build and all 20 Python harness tests pass.
- Real-child combined-failure and repeated-poll regressions pass alongside
  cancellation, TERM/KILL/reap, timeout, workload, diagnostic and image checks.
- Three original Jacobi demo PNG/CSV pairs replay byte-for-byte; all original
  non-timing manifest fields match. Outputs/receipt are under replay/.
- All 48 preserved benchmark packets pass the final validator. No new benchmark
  timing runs or broader scientific/cross-platform guarantees are claimed.

Compiler facts, raw gate logs, source hashes and counts are retained in this
directory. verify.py checks the accepted ancestry, current source identities,
protected numerical/proof/book/evidence paths, gate logs, original replay bytes
and retained packets. Logs remove trailing whitespace and surplus terminal blank
lines only for Git checks.

## Binary and historical-evidence limits

The release rebuild produced SHA256
`3f11c4061a0f41c9ea118e3c90b2fc50a989d58d1e6ac606a00fe1519a10b988`.
The old measurements remain bound to their measured binary SHA256
`e10000f4fb61ba09765d7e9cbc76cac0463550cc7aecbcd72a66d458e0a1a584`.
No bit-reproducible executable claim is made. The source-bound historical
verifier correctly rejected the rebuilt executable; its failed attempt is
preserved in historical-binary-binding-attempt.log. No historical receipt or
source binding was loosened or rewritten. Current receipt binds the rebuilt
release separately, and its deterministic original PNG/CSV outputs match.

Release profile is Cargo's default optimized profile; compiler.txt,
release-build.log and the receipt's build environment retain its inputs.
Historical 16³/32³/64³ standard/tight measurements keep their original source,
compiler/profile, shared-host hardware and contention receipts. Requalification
checks data, not a new speed measurement. No controlled speedup, sustained
real-time acceptance, physical ground-truth error or new native-window test is
implied by this integration.

The process deadline remains optional and disabled by default (run_timeout=None).
When enabled it is polling-driven TERM/KILL/reap, not a hard OS deadline, and
does not impose a default timeout on the production CLI or native desktop.

## Reproduction and hosted gate

From the repository root after selecting the pinned Rust toolchain:

```sh
cargo fmt --all --check
cargo test --locked
cargo test --locked --no-default-features
cargo test --locked --features desktop
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --no-default-features --all-targets -- -D warnings
cargo clippy --locked --features desktop --all-targets -- -D warnings
cargo build --locked --release --bin rheon
python3 -m unittest discover -s tools -p 'test_*.py' -v
python3 evidence/result-integration-qualification/verify.py
```

Replay uses the unchanged archival replay.py with a fresh destination. The
verification command reads retained evidence and does not rerun timing workloads.
Actual hosted CI must qualify the published integration head; historical green
runs are not relabeled as its CI. No PR merge, external review request, thread
resolution, credential/permission expansion or network-policy change is made.

## Completion and remaining decisions

Admitted fixed-box, selectable-pressure/native comparison, cancellation/result
ownership, matched benchmark, optional-deadline and complete-result qualification
milestones are delivered. PR1's separately accepted review repair has actual
hosted Lean/mathlib and numerical gates; it remains a distinct source lineage.
No remaining unambiguous implementation task is specified in these milestones.

Parent coordinates stack review, thread disposition and integration. The open
low-resolution real-time objective needs an explicit grid, physical horizon,
accuracy setting, frame-time budget and target hardware before profiling or a
new solver approach has an acceptance criterion. Additional native-platform or
accessibility qualification needs a named target platform. A newly rendered
book edition needs a separate artifact/render task; preserved historical book
artifacts are not silently replaced. No speculative features fill those decisions.

## Successor replay provenance checks

The repaired verifier retains the fixed historical accepted-source/log gates.
Run the candidate's verify.py by absolute path from historical integration checkout
`2d36096f17c494b25c46e64335521cea5e19db5d`, not the current combined source.
Use `--receipt-output FRESH_JSON`; the default exclusively creates
successor-verification.json and never rewrites the original receipt.

Retained replay run.json fields must match the named original fixture, excluding
only measured_step_seconds and allowing the documented newer schema additions.
Original PNG/CSV byte checks remain. The old replay receipts have no producing
executable hash: their recorded rebuild identity is a historical reported build
fact, not verified producing-binary provenance. The successor historical-only
check reports that limit and leaves current_release_sha256/build_info null.
Merely supplying a --binary is rejected; no current executable is retrospectively
qualified against someone else's retained outputs.

A distinct fresh helper executes the actual release binary and records its SHA,
build information, driver hash, commands, every produced run.json/PNG/CSV hash and
fixture hashes. It checks executable hashes around each run and before publishing,
then verifies the whole fresh packet before creating its receipt. This is local
provenance, not signed attestation, a proof against concurrent swap-and-restore,
bit reproducibility or a binary-to-source theorem. From the current candidate root:

```sh
python3 evidence/result-integration-qualification/fresh_replay.py produce target/release/rheon FRESH_REPLAY_DIR
python3 -O evidence/result-integration-qualification/fresh_replay.py check target/release/rheon FRESH_REPLAY_DIR --receipt-output FRESH_CHECK_JSON
```

Historical integration verify.py may additionally receive both --fresh-replay DIR
and --binary EXECUTABLE to check this independently labeled fresh packet. Accepted
historical source hashes remain historical; no binary-to-that-source assertion is
inferred. Old list-form replay receipts cannot qualify as fresh provenance. After
any rebuild changes the executable hash, produce another fresh replay rather than
changing a prior receipt's hash. Incidental replay step timings make no performance
claim and never replace old benchmark measurements.
