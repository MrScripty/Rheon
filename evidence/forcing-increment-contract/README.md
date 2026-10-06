# Increment contract: research proposal and small exact examples

The [contract proposal](../../docs/research-book/implementation/forcing-increment-state-contract.md)
selects **ephemeral evaluator workspace** for a future research prototype,
with stored endpoints authoritative. Current native acceptance and operation
order remain the baseline. Even ephemeral precision can alter step acceptance
on initially identical rounded fields and hence change public trajectories.
Persistent compensation additionally changes restart/replay state; neither is
behavior-neutral. No production implementation is authorized by these examples.

Frozen base is `f0b81b4a5f76cb706e645fad4f38faf028c5727e`, tree
`59927a7fb02d949859b78f7e78865d1717f19870`. Its complete arithmetic diagnosis,
five refusals, 220 neighboring refusals, original temporal-band failures and
receipt correction remain byte-for-byte preserved. The independent review's
110-component/five-norm confirmation is a coordinator-supplied result; this
packet does not pretend to rerun or originate that review.

Run `python3 evidence/forcing-increment-contract/examples.py`, also with `-O`.
`results-normal.json` and `results-optimized.json` are actual outputs. Fraction
checks exact small algebra independently of binary64 operation order. There
are eight small examples and eleven required corruption rejections:

- Lost quarter-ulp increment: latent residual zero, stored-endpoint rate -1;
  a hybrid energy ledger loses work while the endpoint ledger balances.
- Identical binary inputs reordered: opposite acceptance outcomes at the
  original threshold. This is a scalar illustration, **not** a native repair.
- A moving two-column constraint: absolute and increment forms agree only
  when the accepted initial constraint defect is retained.
- An interpolated nodal row: the exact stable/direct and ledger defect formulas
  are nonzero before reduction roundoff.
- Two directions on one donor face: sum of transfers, not absolute net transfer,
  supplies the mixing identity on changing mass.
- Changed endpoint: pressure adjoint and positive strain must be recomputed.
- Rounded geometry: latent mass/operator assembly cannot be mixed with a
  different published domain.
- Retained low parts: later displayed values and restart continuation change
  when the tail is lost. This is no implemented Rheon checkpoint.

These examples are deliberately small analytic fixtures. They contain no
Rheon trajectory, accepted candidate, temporal experiment, native compensated
arithmetic, solver convergence/floor theorem or continuum validation. The
contract lists the discriminating native/full-equation, rollback, memory and
restart tests still needed before implementation. No Rust/Lean/workflow or
historical artifact bytes are changed. No broad native tests are rerun for this
research-only documentation packet.

`receipt.json` binds the completed artifacts and all frozen base tracked files.
Run `verify.py` with ordinary and optimized Python. Its explicit ValueError
checks do not disappear under optimization. The producer hashes completed
outputs only; no log or receipt hashes itself. Post-Git verification is recorded
separately without rewriting this packet.
