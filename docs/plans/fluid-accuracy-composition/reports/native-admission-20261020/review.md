# Independent complete-source admission mechanism review — 2026-10-20

Decision: the existing explicit version-three admission refresh is appropriate. No version-four inventory or selector/qualifier/oracle change is needed for this extension. The provisional closure is verified as a review input. Final source admission and execution remain deferred until the assigned iteration-count and evaluator repairs are coordinated and independently reviewed. This review neither activates the provisional map nor qualifies the candidate.

## Exact subject and independent checks

Reviewed private candidate `6137e46ccfcc576b239dbe34f6700b84f109de38`, tree `8fa48084cd9e8af2dde077812fbc5d967efbd8f5`. It was clean at the initial observation. Input `provisional-complete-closure.json` SHA256 is `ea0ea4b43b222b20b760d4ac7fc1af6c93e7a5b4467f08b9e0be8c663d873a2a`; canonical sorted compact source-map SHA256 is `de1f61379e0f5ad7dd3ff04180edee4591993fcbb958b805b51ce2df1f6c6e70`.

I independently reconstructed the unchanged selector from its source and filesystem: exactly 135 paths, 11 added, 16 changed, 108 unchanged, zero removed versus the admitted 124. Every mapped digest matches both the candidate's committed Git blob and observed file bytes. All 124 prior hashes also match committed `bc2f472787060d9a69cb619ed2c2fea10fd754b6` blobs. The JSON receipt lists all paths and critical pins; source-map.json retains the reviewed map. These are observed bytes, not an instruction to copy them into the active inventory.

All eleven reviewed mechanism/history files remain byte-identical to that prior main, including the three inventories, admission review, selector/packet code, checker, qualifier, oracle and their tests. The active old v3 inventory remains SHA256 `75415a192381d0457785526120c120f7ced9217169afbc64f2511e834c0ff20f`; the admission review remains `5836c3fa71bf20b72dbde79a2f5d27dd281ffc3930e0af7ad81f9039fd3edd42`. No admission input was changed by this reviewer.

During receipt preparation the owner's worktree advanced to clean `64f49a92b9bd3e8097e584713e557e0d6498c32d`, tree `0f90cad33429249cabd654551bb4833c626c482d`. All 135 selected inputs and eleven mechanism/history files still matched the reviewed candidate. This later observation is recorded separately, not substituted for the exact 6137 subject or for future execution evidence.

## Existing owned mechanism and history

`NATIVE_ADMISSION_INTEGRATION_REVIEW.md:85–89` explicitly preserves earlier v3 review as history and refreshes the same integration epoch while retaining schema/base/predecessor semantics. Lines119–123 authorize fresh qualification only, bind the review and complete inventory committed together, and require future wiring to receive its own explicit review and fresh receipt. This is the smallest established mechanism for the present extension.

Append a bounded current review without rewriting earlier decisions. Explicitly update the complete v3 source map and its review SHA together after final coordinated source review. Preserve:

- Schema `rheon-reviewed-aligned-strain-native-sources-v3` and base `21d4122c3d0edb2ca5c3ce089175c70035d737a3`.
- Frozen v1 SHA256 `84c036618b32de66623d8ded24ae8cf95194457e36b9a130087a9c2bf3bd3f6a` and v2 SHA256 `6ada2133501aab449ed7ee73f802d8a4593db50c857f0a8f8ce2af4507721d6d`.
- Immutable primary `f75cd66c6dc4167ad1ec3febb72e6fb2aa6b4d0b`, tree `bda8ebca0a226e10f03f975ffcf9701df4326095`, all reviewed-base ancestry checks, and existing receipt/packet schemas.
- The conservative all-src/.cargo/toolchain/build-and-qualification-input selection; no assumed numerical subset or test-module exclusion.

All five relevant primary/v1/v2/v3/qualifier bases are candidate ancestors. A new epoch would require a reason to change these trust semantics; none is needed here. Historical v3 bytes remain available at their committed subjects and in retained evidence. Never imply their old pass receipts qualify the refreshed map.

`aligned_strain_packet.py:78–138` selects the newest present inventory, refuses changed predecessors, enforces exact identity/review binding/ancestry/complete path set/current digests, and does not fall back on malformed newest records. `trusted_primary:181–218` additionally binds the native receipt to the committed source path set, actual committed source and all four admission-file blobs, and executable digest. Keep these checks intact. The early admission check is source identity, not a failed or passed numerical comparison.

## Final procedure after the owners' fixes

First obtain the coordinated final heads, review their exact source and evaluator deltas, rederive the complete map, and commit the explicitly approved inventory plus appended review. The counts may remain135, but neither their count nor these hashes should be assumed unchanged. Require clean committed inputs and preserve failed/intermediate receipts separately.

With explicit Rust/Cargo/Python environment, exclusive accurately disclosed target, `CARGO_INCREMENTAL=0`, fresh external output directories, and no `--allow-dirty`:

```sh
python3 docs/education/check_native_admission.py
python3 tools/qualify_aligned_strain.py /tmp/FINAL-qualified
python3 docs/education/aligned_strain_packet.py \
  --output /tmp/FINAL-packet \
  --executable "$CARGO_TARGET_DIR/debug/examples/aligned_strain" \
  --primary-qualification /tmp/FINAL-qualified/qualification.json
RHEON_ALIGNED_PACKET=/tmp/FINAL-packet python3 -m unittest discover -s docs/education -p test_aligned_strain_packet.py -v
RHEON_ALIGNED_PACKET=/tmp/FINAL-packet python3 -O -m unittest discover -s docs/education -p test_aligned_strain_packet.py -v
```

Clear ambient PYTHONOPTIMIZE for the normal invocation; PYTHONDONTWRITEBYTECODE avoids source-tree cache writes. Require 18 qualifier commands: fmt, locked Rust contracts/build/strict Clippy, five qualification-guard tests normal and optimized, five actual native specimen executions each with independent rational comparison and adversarial negatives, and ten oracle tests in both modes with the actual native TSV supplied. No Lean claim without the separate executed Lean commands.

Require all26 packet contracts in normal mode and all26 under `-O`, using the actual fresh packet rather than accepting packet-dependent skips. Existing negative controls cover malformed/stale newest inventory without fallback; changed/missing/added source/build inputs; changed frozen predecessors and review path/SHA; committed receipt/source/admission mismatch; dirty state and changed source/tree/binary; output reuse/Git-owned output; rehashed corrupt native/packet data; removed rows, mass, reflected-corner and basis corruption; extra artifact/receipt fields and unauthorized stepping scope. Preserve explicit runtime refusal checks under optimization. No new permanent gate machinery is necessary merely to refresh approved hashes.

Audit exact final head/tree, before/after tracked bytes, command exits, artifacts and executable hashes. Fresh receipt status must be passed/clean. The packet must reopen and verify its exact four-file roster and newly generated native data. Bind any final claimed subject explicitly; later source/admission edits require a new review and fresh execution rather than receipt relabeling.

## Scope and remaining blockers

This is mechanism/identity/history review, not a second behavioral review of all27 changed/added modules. The independent source owner review and focused G1/C1-A/C1-B1/Q1 evidence remain necessary. In particular the 18-command qualifier does not execute every new module's unit tests: run and retain the composed focused contract matrix separately, plus applicable native/browser and hosted publication gates. Do not equate an all-source identity map with all-source scientific qualification, learned-model availability, a full fluid accuracy result, or new GUI evidence.

The known iteration-count consistency and evaluator repairs are assigned elsewhere and pending for the final subject. No builds, simulations, models, qualification suites, publication, ref changes or merge retries were performed here. No additional admission-mechanism blocker was found.
