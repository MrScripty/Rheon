# Numerical research companion

Run experiments.py and depth_experiments.py with the pinned requirements. Then run make_figures.py. Results are small reference fixtures, not production fluid solvers or performance benchmarks.

experiments.py covers a gauge-fixed 8-cubed pressure graph, residual identity, periodic scalar transport, scalar implicit diffusion, exact rational flux cancellation and explicit layout arithmetic.

depth_experiments.py covers twelve mechanisms: orthogonal pressure gradients, interface-normal signs, full gauge residuals, affine boundary work, staggered affine sampling, rotating-field tracing, one-dimensional Galerkin multigrid, axis-aligned interface flux, sphere curvature, weighted viscous energy, frozen-position affine particle moments and SPH density gradients.

The binary32 comparison rounds binary64 pressure and replays algebra; it does not rerun the solver in binary32. PLIC, APIC, SPH and multigrid fixtures validate only the stated local constructions. Their complete three-dimensional fluid implementations remain future work.

## Committed-evidence check

CI preserves both committed JSON files and runs `test_verify_results.py` against
them before executing the fixtures. It then runs `verify_results.py --baseline-dir PATH`. The latter
compares the regenerated files against those preserved baselines. Container
shape, keys, booleans, integers and strings must match exactly. Finite floats
permit relative error 1e-8 or absolute error 1e-12; this allows small platform
rounding differences, not changes to a fixture's independently enforced physical
or algebraic acceptance thresholds. Only the top-level `environment` metadata
contents are excluded. NaN and infinities in numerical evidence are rejected.
Changes to the experiment or intentionally updated evidence require review of
both source and baseline. This gate detects stale or independently altered
published evidence; it does not establish correctness of the experiments.

### Explicit CG reproduction profile

The original JSON files remain historical observations and must not be rewritten
to match a different CPU backend. The strict default comparison is unchanged.
OpenBLAS CPU dispatch changes reduction rounding inside SciPy CG; on this fixture
that changes the intermediate residual trajectory enough to fail a pointwise
comparison, while the converged pressure agrees to about 3.13e-13 across the
tested kernels. A global tolerance increase is not used.

CI now explicitly selects `OPENBLAS_CORETYPE=Haswell` and
`OPENBLAS_NUM_THREADS=1` and passes `--cg-profile haswell-openblas-0.3.30` to the
comparator. These are process/job-local reference-experiment controls, not
production solver changes. The profile qualifies Linux x86_64, Python 3.12,
NumPy 2.3.5, SciPy 1.17.0, and both wheel BLAS libraries at OpenBLAS 0.3.30 using
single-threaded Haswell dispatch. Hardware must support that kernel. The pinned
threadpoolctl 3.6.0 dependency verifies the loaded libraries rather than trusting
the environment variable alone. Different or missing runtime identities fail.

`reproduction/haswell-openblas-0.3.30.json` contains a separately recorded CG
history, not a replacement for `results.json`. Before using it, the comparator
requires exact SHA256 matches for both preserved historical files and both
experiment sources. Only `projection.histories.cg` selects the qualified replay
history; the iteration count, every other numerical field, shapes and keys keep
their original checks and tolerances. Changed sources or historical evidence
require explicit profile review and requalification. There is no automatic
fallback to whichever profile happens to pass, and CI never regenerates profiles.

See [the diagnosis and retained experiments](../evidence/cg-reproduction/README.md)
for the exact failed CI value, CPU-kernel isolation, direct-solve comparison,
negative tests, and limits. Backend selection is documented in
[OpenBLAS usage](https://github.com/OpenMathLib/OpenBLAS/blob/develop/USAGE.md)
and [NumPy troubleshooting](https://numpy.org/doc/stable/user/troubleshooting-importerror.html).
