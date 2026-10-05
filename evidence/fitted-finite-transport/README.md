# Actual finite relative-transport research checkpoint

No production Rust, API or advancing refusal changes in this checkpoint. The
source/evidence identities and complete preserved history are bound in receipt.json.

This extends the existing translating varying-cap flow by fixing the bottom's
tangential mesh coordinates. It derives actual finite shared transfers from the
same differentiated periodic Powell–Sabin geometry. Nodal masses change; full
momentum is carried with these transfers and endpoint viscosity in one constrained
research solve. Pressure remains analytic zero, with no new pressure-solver claim.

`reference.py` proves symbolic whole-interval GCL, positive masses/microareas and
absence of poles/flux reversals on cap offset [0,1/8]; integrates 76 physical face
functions, including 56 logarithmic primitives. `numerical.py` independently
assembles and solves the dense composed equation and compares repeated time steps
with a tolerance-refined, nonautonomous ODE reference. These are **Python research
runs**, not a native advancing liquid implementation or continuum accuracy study.

Run `qualify.py NEW_DIRECTORY` to execute geometry, numerical studies and nine
unit tests in normal/optimized Python. Output directories are never overwritten.
Run `verify.py --negative-self-test --evidence-commit HEAD` (also with `python -O`)
to bind exact Git source, nested receipts and every preserved historical file.

Important failures remain data in the packet:

- Endpoint-frozen flux passes node GCL (about 2.08e-17) yet misses actual physical
  face transfer by up to 0.001636186128. Face circulation cannot be detected from
  mass marginals. The initial failed GCL-only discriminator traceback is retained.
- Constrained pure transport has 93 negative coefficient-map entries and changes
  one [0,1] velocity fixture to [-0.0120824975,0.5107670997]. No constrained convex
  velocity bound or positivity theorem is claimed. Momentum/work remain valid;
  liquid masses are positive independently from the actual geometry.
- The first successful ODE trial used SciPy's reported adjustment of too-small
  rtol. Its warning/raw output is retained. Final runs explicitly set rtol=3e-14,
  atol=2e-14 for the tighter reference and compare against rtol/atol=2e-13.

Pressure rank 32 is measured by exact elimination at three specified geometries
only. The general pressure-coupled step, uniform inf-sup, nonuniform xy motion,
continuum spatial convergence and other liquid features remain unqualified. No
prior Lean claims or frozen evidence are rewritten.

The next native implementation needs bounded physical quadrature, one reusable
candidate geometry workspace, a nonsymmetric composed solve, true residual/work
and actual face-provenance gates, and a common final publication barrier. CG is
not valid for the nonsymmetric transport-plus-viscosity operator.
