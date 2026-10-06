# Nonuniform xy pressure and the unresolved material-path gate

Source and evidence are frozen separately on `research/nonuniform-pressure-path-gate`.
See `docs/research-book/implementation/fitted-nonuniform-pressure-path-gate.md` for
the proposed acceptance test, exact counterexample and required coupled equations.

This packet qualifies a solved nonzero pressure on the smallest supported fitted
mesh and proves that a subsequent straight material cap path violates local
incompressibility and finite GCL. **Zero advancing steps are accepted.** It adds
a bounded native diagnostic example; production APIs/owners/arithmetic, all
existing tests and frozen evidence remain unchanged. It is not another enabled
restricted flow, nor a validated general free-surface simulator.

* `reference.py`: exact rational geometry, full xy saddle solve, pressure/strain
  work, exact nonzero geometry derivative and local Reynolds defect.
* `numerical.py`: physical positive/negative face integrals on the rejected path;
  16/32-point measured refinement, finite local GCL failure and its quadratic trend.
* `test_reference.py`: eleven actual contracts in normal and optimized Python.
* `verify_native.py`: independent exact rebuild of debug/release data, eight real
  corruptions per invocation, including a zero-balance face circulation.
* `qualify.py`: actual reference/native executions, complete input hashes and
  exact mode/profile replay; `qualify_rust.py`: actual Rust feature matrix.
* `render.py`: actual native solved pressure and rejected path, labelled accordingly.
* `trials/`: unrefined pressure-coordinate failure and strict decimal/float path
  failure. Their gates remain unchanged in the qualified result.
* `preservation.json`: complete historical SHA-256/Git-blob inventory, including
  all previous Jacobi fixtures, numerical failures and conditional proof evidence.
* `receipt.json`: exact source binding and every packet file except itself;
  nested receipts are included. `verify.py --evidence-commit HEAD
  --negative-self-test` also checks the root bytes and frozen Git inventory.

There is no new Lean theorem. The differentiated constraint/DAE derivation is
mathematical research, not a verified nonlinear temporal integrator or IEEE proof.
Whole-interval rank, traction accuracy, arbitrary geometry, constrained positivity,
variable density, adhesion, capillarity and reconstruction remain unqualified.
