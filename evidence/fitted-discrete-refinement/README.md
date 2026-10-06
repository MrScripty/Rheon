# Coupled discrete-work refinement successor

Frozen base source/evidence `31f8614f/aa89d1af` and all earlier failures remain
unchanged. This successor keeps the same explicit first-order endpoint-donor
momentum equation, actual path-integrated face mass and separate BE/mixing/
viscous work. It repairs the Newton floor by eliminating seven exact known
velocity coordinates before a fifteen-row divergence solve; stable inertia alone
failed and its trace/source are retained. All full 24 velocity/acceleration rows,
material cap, pressure, direct/stable momentum, mass and work gates stay intact.

`diagnose.py` records the frozen refusal and each attempted repair incrementally.
`rate.py` derives an exact quadratic momentum-flux difference coefficient.
`cells.py` retains the coarse pressure-state rate failure and tests the same band
on finer intervals plus convergence to that exact coefficient. `trajectory.py`
records five complete repeated cases on each of two initial fields, with bounded
incremental output and actual nonzero-state cancellation/failure/continuation.
`verify_cell.py` and `verify_trajectory.py` replay actual operators and include
meaningful corruptions. `independent_work.py` reconstructs exact rational
operators and sums at 80 digits; no NumPy force/work arrays are reused.
`test_reference.py` has 23 contracts including the exact frozen refused input.

The book `docs/research-book/implementation/fitted-discrete-work-refinement.md`
records the arithmetic/consistency derivation and selected native owner/workspace
slice before its implementation. No new Rust public step or Lean theorem is
claimed by this research packet. Continuous and exact integrated varying-donor
momentum remain distinct equations with their original failed gates. Native
memory/owner/rollback, continuum spatial/traction accuracy, positivity, density,
adhesion/capillarity and reconstruction still need separate qualification.

Complete file/source/historical binding includes nested qualification receipts.
Verify with `verify.py --evidence-commit HEAD --negative-self-test`. Failed and
interrupted trials are included; only the root receipt excludes itself.

`certificate.py` certifies two first coarse real polynomial paths with 512 exact
interval boxes each; repeated-path/IEEE coverage is not implied. `render.py`
plots actual temporal errors, separate losses and allowance ratios and renders
32 actual accepted finest endpoints per field. All attempted implementation
failures remain in `trials/` alongside successful runs.
