# Numerical research companion

Run experiments.py and depth_experiments.py with the pinned requirements. Then run make_figures.py. Results are small reference fixtures, not production fluid solvers or performance benchmarks.

experiments.py covers a gauge-fixed 8-cubed pressure graph, residual identity, periodic scalar transport, scalar implicit diffusion, exact rational flux cancellation and explicit layout arithmetic.

depth_experiments.py covers twelve mechanisms: orthogonal pressure gradients, interface-normal signs, full gauge residuals, affine boundary work, staggered affine sampling, rotating-field tracing, one-dimensional Galerkin multigrid, axis-aligned interface flux, sphere curvature, weighted viscous energy, frozen-position affine particle moments and SPH density gradients.

The binary32 comparison rounds binary64 pressure and replays algebra; it does not rerun the solver in binary32. PLIC, APIC, SPH and multigrid fixtures validate only the stated local constructions. Their complete three-dimensional fluid implementations remain future work.
