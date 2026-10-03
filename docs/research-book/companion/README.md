# Numerical research companion

Run experiments.py and depth_experiments.py with the pinned requirements. Then run make_figures.py. Results are small reference fixtures, not production fluid solvers or performance benchmarks.

experiments.py covers a gauge-fixed 8-cubed pressure graph, residual identity, periodic scalar transport, scalar implicit diffusion, exact rational flux cancellation and explicit layout arithmetic.

depth_experiments.py covers twelve mechanisms: orthogonal pressure gradients, interface-normal signs, full gauge residuals, affine boundary work, staggered affine sampling, rotating-field tracing, one-dimensional Galerkin multigrid, axis-aligned interface flux, sphere curvature, weighted viscous energy, frozen-position affine particle moments and SPH density gradients.

The binary32 comparison rounds binary64 pressure and replays algebra; it does not rerun the solver in binary32. PLIC, APIC, SPH and multigrid fixtures validate only the stated local constructions. Their complete three-dimensional fluid implementations remain future work.

## Committed-evidence check

CI preserves both committed JSON files before executing the fixtures, then runs
`test_verify_results.py` and `verify_results.py --baseline-dir PATH`. The latter
compares the regenerated files against those preserved baselines. Container
shape, keys, booleans, integers and strings must match exactly. Finite floats
permit relative error 1e-8 or absolute error 1e-12; this allows small platform
rounding differences, not changes to a fixture's independently enforced physical
or algebraic acceptance thresholds. Only the top-level `environment` metadata
contents are excluded. NaN and infinities in numerical evidence are rejected.
Changes to the experiment or intentionally updated evidence require review of
both source and baseline. This gate detects stale or independently altered
published evidence; it does not establish correctness of the experiments.
