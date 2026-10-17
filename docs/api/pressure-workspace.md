# Pressure workspace initial estimate

`PressureWorkspace::solve_rhs_with_initial_pressure(operator, rhs,
initial_pressure, dt, settings, cancelled)` solves a sealed-domain pressure
system from an explicit full pressure estimate. It uses the same matrix,
preconditioner, allocation and stopping owners as `solve_rhs`.

Both inputs must match the operator's grid and contain only finite values. The
estimate must have cell-zero pressure equal to zero; the method never shifts
its gauge. A wrong gauge returns `InvalidInitialPressureGauge`. Free-surface
operators return `UnsupportedInitialPressureDomain`; ordinary `solve_rhs`
continues to support their existing contract.

The initial residual is the full `rhs - A * initial_pressure`, including the
gauge row. Relative tolerance uses the original RHS norm. The sealed RHS must
pass the existing compatibility budget and is never mean-shifted. The estimate
is copied exactly once and Krylov scratch is reset before iteration. An exact
estimate can pass at zero iterations. A zero estimate preserves ordinary solve
pressure and report bits for both Jacobi and symmetric Gauss-Seidel PCG.

Cancellation, iteration limits and numerical failures retain their existing
typed outcomes. Pressure scratch is an accepted solve only after success; a
successful report still precedes the separate actual corrected-f32-velocity
divergence gate. No field allocation is added to a solve.

The implementation incorporates the sealed initial-pressure contract previously
qualified by the lab's retained patch `b851b38d2d03280220bc4648cd4d721b2e796786fb18d8188dc335336fe2a998`.
Its eight contract fixtures are retained in `tests/initial_pressure_contract.rs`,
with additional explicit nonfinite/wrong-shape RHS and single-cell cases. The
current library owns the method; consumers call it directly.
