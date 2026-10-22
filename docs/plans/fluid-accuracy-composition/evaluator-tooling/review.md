# Independent evaluator tooling review

Disposition: accepted as a tooling-only repair of the existing optimization-sensitive input guards. No remaining blocker within this repair's declared scope. This does not grant new scientific qualification or production admission.

Reviewed active source in `/workspace/Rheon-evaluator` at `7ec6a749b746af5a3ee5c903807c48cdd016d4da`, with original evaluator bytes bound to parent base `7d7306a`. Reviewer made no source edits and executed no evaluator, solver, native test, or scientific campaign. Independent scripts parse source, hash files and inspect previously saved subprocess logs only. Review files are confined to this evidence subtree.

`check_ast.py` independently verifies all eight manifest hashes and that each frozen file is byte-identical to `git show 7d7306a:<original path>`. It reverses only each promoted `require(condition, message)` expression to its original `Assert` node and removes only the new helper. The entire resulting program AST equals the original program AST for all four evaluators. There are 15 original guards (7 adjoint, 4 material, 2 Q1, 2 Q2), and no active `Assert` nodes. This establishes preservation of predicates, numerical formulas, thresholds, control structure and output schema beyond checking guard expressions alone.

`check_results.py` independently inspects the expanded retained run at `/workspace/rheon-evaluator-evidence-expanded`. All 129 saved subprocess records have matching stdout hashes. All 35 malformed fixtures intentionally refuse in normal, `-O` and `-OO` modes with nonzero exit status, the explicit input-refused diagnostic and empty stdout. Together with the two retained incomplete matrices, this comprises 111 independently checked intentional refusals. All 15 distinct original guard IDs are actually reached in each of the three modes; no guard's coverage is inferred merely from another earlier cardinality failure.

Complete retained P1-adjoint, P1-material and Q2-v2 outputs match their frozen normal-mode JSON byte-for-byte in each repaired mode. All four recorded original `-O` bypasses exit successfully with parseable JSON, preserving the vulnerability witnesses. The retained incomplete Q1 and header-only Q2-v1 matrices refuse under the original normal evaluator and all three repaired modes. All 13 retained input CSV hashes still match. The synthetic Q1 group fixture only supplies cardinality to reach an inner guard; its copied coarse rows are not native finest-resolution measurements or scientific evidence.

The fixture set covers empty/truncated matrices, duplicate summary keys, same-total reassigned nested groups, wrong and reordered epsilon lists, nonfinite derivative/probe values, and zero/negative required site fields. This is an adequate discriminator for the declared assertion-to-always-on repair. It is not an exhaustive CSV schema, unique node identity or arbitrary physical-domain validator; those unchanged original limitations must not be presented as fixed.

The new README accurately limits the result. Archived original evaluator bytes intentionally retain their historical assertions and must remain historical witnesses. Scientific campaign archives, negative findings and prior scope limits remain authoritative: P1 promotion stopped, Q1 transport incomplete, and Q2 finite-error geometry integration alone leaves five native aperture topology artifacts unresolved before active topology/DOF and coupled assembly. No new numerical method or acceptance threshold is supported by this tooling review.

Replay the independent checks without executing an evaluator:

```bash
python check_ast.py /workspace/Rheon-evaluator /tmp/fresh-independent-ast
python check_results.py /workspace/Rheon-evaluator /workspace/rheon-evaluator-evidence-expanded /tmp/fresh-independent-results
```

Scripts, JSON summaries and captured logs are retained alongside this review. `SHA256SUMS` binds the final independent deliverables.
