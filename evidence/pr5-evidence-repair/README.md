# PR5 evidence-integrity successor

Branch `repair/pr5-evidence-integrity` starts at accepted integration
`94cdc967ad0b91ee03b272509695b4d7da24c209`, tree
`d8b4a04402553350441618e6d4f48a906f6efdb7`. No solver, CG, desktop, research,
book/proof, original measurement/fixture or previously accepted PR4 gate changes
are included. PR2 retarget remains held; immutable review PR4/5 are untouched.

All three reviewed verifier files were byte-identical between immutable review
head `2d36096f17c494b25c46e64335521cea5e19db5d` and accepted 94cdc967 before
this repair. Their existing fixed historical source/data scopes remain fixed.
They are not rebased to treat current combined source as measured historical
source. New outputs use exclusive successor receipts, preserving old receipts.

## Findings and dispositions

- [4178807103](https://github.com/MrScripty/Rheon/pull/5#discussion_r4178807103):
  the terminal verifier retains tracked Git diff checks and rejects all untracked
  protected additions, including ignored files, before receipt/PASS. No ignore
  patterns exempt generated protected-path files. Use a clean historical
  checkout for complete inventory qualification; build caches under proofs/.lake
  legitimately violate that inventory claim.
- [4178807105](https://github.com/MrScripty/Rheon/pull/5#discussion_r4178807105):
  integration replay checks every original manifest field except
  measured_step_seconds, alongside existing PNG/CSV byte comparisons. Original
  fixture inventory is explicit; documented newer schema additions are allowed.
- [4178807108](https://github.com/MrScripty/Rheon/pull/5#discussion_r4178807108):
  old replay list receipts lack a producing-binary hash. Historical-only successor
  checks therefore report no current executable qualification and leave current
  binary fields null. An executable alone cannot qualify those old outputs.
  A separate fresh helper really runs the named binary, checks its hash around
  execution, validates original manifests/bytes, and records producer identity,
  build info, driver/fixture/produced-artifact hashes and commands before receipt
  publication. Fresh qualification requires that exact producer hash; the old
  e100/3f11 identities are never substituted or retroactively manufactured.
- [4178807111](https://github.com/MrScripty/Rheon/pull/5#discussion_r4178807111):
  each implementation's samples/median/min/max must match its named historical
  case-receipt entry and existing local-sample calculations. Method inventories,
  finite values and integer/float types are checked. All former result-verifier
  assertions are explicit failures; their original conditions are preserved.

No finding is intentionally left unresolved. Fresh replay is local provenance,
not signed attestation, atomic execution-file immutability, a binary/source proof
or bit-reproducible-build guarantee. Hash checks around each run do not prove
against a concurrent swap-and-restore. No new performance claim is made.

## Actual qualification

All 13 end-to-end regression methods pass under normal Python, -O and
PYTHONOPTIMIZE=1. The tests clone real historical Git/evidence into disposable
checkouts, mutate real retained packets and invoke actual verifier CLIs. Failed
checks must exit unsuccessfully, emit no PASS and create no success receipt.

They cover tracked/untracked/ignored protected changes; non-timing replay manifest
corruption; PNG/CSV bytes; named fixture inventory; old replay provenance rejection;
an appended-trailer ELF with unchanged --build-info but a different SHA; fresh
run/byte/provenance mutations; coordinated doubled durations and matching local
summaries; individual receipt timing fields/types; receipt-method reorder acceptance;
and optimized source/control/finite checks. Only the documented old timing field
may differ in historical manifest comparison. Fresh produced run.json hashes also
protect that incidental timing field against post-production edits.

All six result cases (48 real retained packets) and fixed historical integration
source/log/packet/replay audits pass under all three modes. Historical integration
receipts explicitly claim no current binary qualification. Fresh actual release
replay produced all three original demo manifest/PNG/CSV matches; its stored
producer packet qualifies under all three modes. All fixed historical verifiers
reject the current combined checkout before receipt/PASS.

The accepted PR4 14-method gate suite and 20 existing harness tests pass again.
A locked release CLI build succeeds with pinned Rust 1.92.0. Numerical/Rust sources
are byte-identical to accepted 94cdc967, so Rust feature tests, Clippy and Lean are
not repeated or relabeled as fresh tests for this evidence-only change.

The original terminal verifier pins binary e10000f4..., unavailable in the inspected
release targets/dependencies. Its real Git inventory guard passes on clean 8bc0f54
and its CLI rejects tracked/untracked/ignored mutations in all modes before the
binary gate. A full fresh pass of that exact historical binary gate is not claimed
or weakened. Original successful historical receipts/logs remain untouched.

## Reproduction

From the current candidate root with its release binary available:

```sh
cargo build --locked --release --bin rheon
python3 evidence/result-integration-qualification/fresh_replay.py produce target/release/rheon FRESH_REPLAY_DIR
RHEON_FRESH_REPLAY=FRESH_REPLAY_DIR python3 -m unittest discover -s evidence/pr5-evidence-repair -p test_integrity.py -v
RHEON_FRESH_REPLAY=FRESH_REPLAY_DIR python3 -O -m unittest discover -s evidence/pr5-evidence-repair -p test_integrity.py -v
RHEON_FRESH_REPLAY=FRESH_REPLAY_DIR PYTHONOPTIMIZE=1 python3 -m unittest discover -s evidence/pr5-evidence-repair -p test_integrity.py -v
python3 -O evidence/result-integration-qualification/fresh_replay.py check target/release/rheon FRESH_REPLAY_DIR --receipt-output FRESH_CHECK_JSON
```

Use absolute FRESH_REPLAY_DIR paths for test overrides. RHEON_BINARY may point to
an external Cargo target executable; otherwise tests use target/release/rheon.
The stored fresh-replay packet is the actual cloud run. If a rebuild changes the
binary hash, generate another fresh packet and select it with the test override;
do not alter the stored receipt to match a different executable. Incidental replay
step timings are not benchmark replacements or performance evidence.

Invoke the candidate's historical verifier scripts by absolute path, from these
separate clean historical checkouts, with --receipt-output FRESH_JSON:

- result: 416744700a45a8e74de27ca4e25cd00b0931926c
- terminal: 8bc0f54604d98d838b3a1ab2f6fcdfff9af2ac51 (original e100 executable required for full gate)
- integration: 2d36096f17c494b25c46e64335521cea5e19db5d

The integration verifier optionally accepts both --fresh-replay DIR and --binary
EXECUTABLE for the separately labeled fresh packet; that does not assert the
executable was compiled from its fixed historical accepted-source tree.

Independent review and coordinated integration remain pending. No PR advancement,
main merge, immutable review-branch update, PR2 retarget, review-thread resolution,
bot request or credential/network-policy expansion was performed.
