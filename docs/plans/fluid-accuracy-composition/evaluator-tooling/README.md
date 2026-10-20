# Always-on evaluator input guards — tooling v2

Parent integration review reproduced optimization-sensitive acceptance of
incomplete input in the P1-adjoint/material evaluators, with related Q1/Q2
patterns. User authorizes this tooling repair, with frozen source/results retained
and NO numerical campaign rerun. Exact added write set: the four active analyze.py
files, byte-preserved originals in this frozen/ directory, and this tooling owner.
Isolated branch fix/numerical-evaluator-inputs-20261020 based on Q2/7d7306a.
No numerical Rust, dependencies, G1/C1, canonical lifecycle, old result/manifest,
frozen branches or scientific archives are changed. Standards pin0fc92a2 applies.

All15 original Python Assert predicates are promoted to always-on require calls:
7adjoint,4material,2Q1,2Q2. No scientific threshold, arithmetic, condition or output
schema changed. Invalid input exits nonzero with an intentional input-refused
diagnostic before emitting a report, including under -O/-OO. Original evaluators
are historical bytes only: their optimized bypass is intentionally reproduced
by tests, and they must not be used as active validation under optimization.

## Results and limits

- 129 subprocess checks,35 malformed fixtures. All15 original guard IDs exercised
  in normal/-O/-OO modes. This includes cardinality, same-total group reassignment,
  duplicate summary keys, wrong/reordered epsilons, nonfinite values and required
  positive site fields. Intentional refusal and empty stdout required.
- Real retained complete P1-adjoint/P1-material/Q2-v2 CSV reports are byte-identical
  to frozen normal-mode outputs in all three repaired modes.
- Real retained incomplete Q1 and empty Q2-v1 matrices remain nonzero refusals.
  Q1 has no complete retained matrix; the inner-group contract uses an explicitly
  SYNTHETIC cardinality-only fixture. Padded64 summary rows are not native
  measurements, scientific outputs, finest-order evidence or an accuracy claim.
- Four original optimized bypasses reproduced separately. Those outputs and all
  malformed fixtures are invalid tooling witnesses, never campaign evidence.
- Retained input CSV hashes unchanged. No native executable or campaign invoked.
  Independent full-AST reversal and frozen-byte/hash checks confirm unchanged
  predicates/formulas, beyond merely comparing the individual guard expressions.

This repairs the existing input guard contract; it does not add exhaustive CSV
schema, unique per-node identity, arbitrary domain or physical validation. Original
scientific failures and limited claims stand: stopped P1 selection, incomplete Q1
transport, finite-error Q2 geometry only, five topology artifacts before B/M/S/W.
See evaluator-hashes.json for separately versioned original/repaired identities.
Old scientific manifests identify their frozen source/copies, not these new tools.
No superseded report or archive hash is rewritten.

## Replay tooling checks

Use retained CSV roots and a fresh output directory:

```bash
python3 docs/plans/fluid-accuracy-composition/evaluator-tooling/test_contract.py \
  --adjoint /workspace/rheon-p1-adjoint-evidence/campaign-v1 \
  --material /workspace/rheon-p1-material-evidence/campaign-v1 \
  --q1 /workspace/rheon-q1-evidence/raw \
  --q2-v1 /workspace/rheon-q2-evidence/raw \
  --q2-v2 /workspace/rheon-q2-evidence-v2/raw \
  --output /tmp/fresh-evaluator-contract
python3 docs/plans/fluid-accuracy-composition/evaluator-tooling/check_guard_coverage.py \
  /tmp/fresh-evaluator-contract
```

This runs Python evaluators and synthetic input contracts only. It does not run a
native solver. Source/frozen copies, logs/fixtures/byte comparisons, input hashes,
all eight evaluator hashes, coverage and independent review are retained in the
separate tooling evidence archive. Existing numerical archives remain unchanged.
Parent/repository lead owns dependency/CI integration and merges.
