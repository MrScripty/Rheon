# Isolated follow-on proof proposals

These sources are deliberately outside `Rheon.lean` and the historical
`source-inventory.json`. They do not extend the root's old qualification receipt.
Use the root `lean-toolchain` and pinned `lake-manifest.json` dependencies.
From `proofs/`, after dependencies are available:

```sh
lake build Mathlib.Data.Real.Basic Mathlib.Algebra.BigOperators.Ring.Finset Mathlib.Algebra.Order.BigOperators.Group.Finset Mathlib.Data.Fintype.Fin Mathlib.Tactic.Linarith Mathlib.Tactic.Ring
python3 FollowOn/check.py ../.generated/fluid-plan-proofs
```

Publication models checked immutable candidates and checked-predicate preservation;
it explicitly demonstrates that this does not imply arbitrary accuracy.
StokesWork derives the shared-operator backward-Euler energy identity including
momentum defect, then exact constrained nonincrease under stated sign assumptions.
It assumes the coordinate equations/constraint; it does not prove existence,
convergence, boundary assembly, physical validity, IEEE execution or Rust refinement.

See the plan's `reports/qualification.md` for exact actual commands, source hashes,
compiler, dependency qualification and current results. Before production use,
review source/axiom inventories and deliberately integrate only selected statements
into the normal root proof gate. Do not regenerate historical inventories blindly.
