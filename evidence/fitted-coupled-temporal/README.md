# Coupled quadratic temporal cell: mass/momentum pass, work rejection

Source and evidence checkpoints are separate on `research/coupled-moving-divergence`.
The companion book is
`docs/research-book/implementation/fitted-coupled-temporal-prototype.md`.
It records the proposed system/gates, independent review, exact tangent
compatibility and the subsequent physical temporal experiment.

The six-coordinate chart follows the reviewer construction: cap `(ux0,ux1,uy0)`
and selectors `z[0],z[1],z[4]`. Six chart accelerations and 16 pressure coefficients
solve all 22 integrated conservative momentum equations on a quadratic cap path.
All 24 strong divergence/acceleration equations, actual cap motion and physical
shared fluxes are checked. **All three candidate cells are rejected by the fixed
weighted-work gate. Zero advancing simulation states are published.**

* `model.py`: common fitted construction, actual geometry derivatives, full xy
  mass/strain/divergence/pressure, vector donor convection and coupled chart.
* `exact.py`: two exact tangent systems, actual GCL/work/momentum identities and
  a projection-invisible incompatible full component.
* `cell.py`, `study.py`: bounded nonlinear solve and three rejected cell replays;
  actual positive/negative face transfers and separate residual-work measurement.
* `certificate.py`: symbolic full-row identities and an exact rational interval
  geometry/rank certificate for the `.05` real path. Maximum 1,024 boxes; 512 used.
* `independent.py`: independently assembled rational operators/trajectory and
  80-digit summation of actual weighted residual work and energy defect.
* `test_reference.py`: thirteen contracts, executing in normal/optimized Python.
* `verify_cell.py`: actual replay with eight corruption controls per cell per mode,
  including endpoint-velocity momentum routing and zero-balance circulation.
* `qualify.py`: actual executions, modes/replays, versions and source hashes.
* `render.py`: actual recorded quadratic candidates, clearly marked rejected.
* `trials/`: preserved failed/aborted certificate attempts, the original resource
  budget failure and initial implementation errors. No successful claim uses them.
* `preservation.json`, `unchanged-production.json`: full historical bytes and all
  prior Rust/Cargo/test/example source bytes unchanged.
* `receipt.json`, `verify.py`: complete source/packet/root binding; nested receipts
  included. Use `verify.py --evidence-commit HEAD --negative-self-test`.

The certificate covers the real binary-coefficient `.05` path. It does not prove
IEEE execution or a mesh-family/inf-sup theorem. Shorter-cell ratios measure
work failure, not accepted temporal accuracy. There is no new Lean or Rust
advancing solver. The general public refusal and every previous numerical/proof
limitation remain unchanged.
