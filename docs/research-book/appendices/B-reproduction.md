# B Reproduction and evidence

Source includes chapter Markdown, appendices, document and figure builders, experiments, covers and Lean. Word equations are native and editable; PDF is the rendered reading copy. Cover fluid imagery is illustrative, not simulated evidence.

## Numerical environment

Executed fixtures use Python 3.12.14, NumPy 2.3.5 and SciPy 1.17.0 on Linux x86_64. Figures use Matplotlib 3.10.8. Requirements pin these packages. The deterministic seed is 20261003. The script writes results.json including full residual histories.

From this book directory, retrieve immutable historical observations into ignored
storage before generating new ignored outputs. The qualified Linux x86_64 replay uses Haswell dispatch on hardware
that supports that kernel, with both wheel BLAS libraries at OpenBLAS 0.3.30.
These process-local settings reproduce the separately recorded CG profile, not
the historical CG trajectory byte-for-byte; historical JSON stays unchanged.

~~~sh
python3 -m pip install -r companion/requirements.txt
baseline=$(python3 companion/historical_baselines.py)
OPENBLAS_CORETYPE=Haswell OPENBLAS_NUM_THREADS=1 python3 companion/experiments.py
OPENBLAS_CORETYPE=Haswell OPENBLAS_NUM_THREADS=1 python3 companion/depth_experiments.py
OPENBLAS_CORETYPE=Haswell OPENBLAS_NUM_THREADS=1 python3 companion/verify_results.py \
    --baseline-dir "$baseline" --cg-profile haswell-openblas-0.3.30
python3 companion/make_figures.py
~~~

No result files or recorded CG histories are tracked in the source checkout.
Verification checks historical/source hashes and the loaded
runtime, substitutes only the explicitly qualified CG history, and retains the
original comparison tolerances for all fields. See the companion README for
profile limits. Assertions cover graph projection, rational conservation,
periodic transport bounds, diffusion decay and memory arithmetic. They do not
qualify production runtime or a GPU backend. Run experiment assertions with
normal Python, since `-O` disables those scientific experiment assertions.

## Lean environment

The toolchain is leanprover/lean4:v4.19.0. Mathlib is pinned to c44e0c8ee63ca166450922a373c7409c5d26b00b and transitive revisions are locked. The workflow uses a pinned checkout action and official versioned elan, read-only permissions and no retained checkout credentials.

~~~
cd proofs
python3 scripts/check_sources.py
lake exe cache get
lake build
lake env lean AxiomAudit.lean
python3 scripts/test_audit.py
~~~

Inventory digests are reviewed, not regenerated in CI. Changed statements require mathematical review, inventory updates and full qualification. A digest establishes source identity, not truth.

The first successful qualification is [workflow run 37141752646](https://github.com/MrScripty/Rheon/actions/runs/37141752646), associated with source head ecf97d3a943ebe30d20acc1cf01b94dc08ff96a2. The runner checked the pull-request merge ref against then-current main. The retained log records actual checkout and all proof/audit stages. Later proof changes require new qualification.

## Boundaries and rendering

Proofs establish properties of Lean definitions. Fixtures establish small reference calculations. Rust architecture, liquids and performance remain implementation work. No production throughput, latency or GPU-memory benchmark was executed.

Rebuild and render after layout edits, inspect every page, and check native equation structure separately. Export success alone does not establish readable notation or clean pagination.

## Extended mechanism fixtures

Run python companion/depth_experiments.py for twelve restricted fixtures. They cover staggered affine sampling, rotating-field backtracing, a two-level Dirichlet multigrid mechanism, axis-aligned interface flux, sphere curvature, weighted viscous energy, frozen-position affine particle transfer and SPH constraint gradients.

The affine sampler errors are at most \(2.23\times10^{-16}\), while the intentionally wrong x-face offset creates a -0.07 bias. Over one unit of rotation time, midpoint tracing error decreases from 0.00166645 at 10 steps to 0.0000260416 at 80; Euler decreases from 0.0511230 to 0.00626930.

The 31-unknown Dirichlet multigrid fixture reduces residual from 1.13620 to 0.00873160 in one cycle and near floating precision in eight. Its exact coarse-space correction discrepancy is \(3.17\times10^{-15}\). This nodal one-dimensional mechanism fixture is not the proposed cell-centered three-dimensional fluid hierarchy.

For a radius-0.5 sphere, curvature errors decrease from 0.0392195 at h=0.1 to 0.000624805 at h=0.0125. The weighted viscous fixture decreases energy from 7 to 0.941037. Its tiny computed negative minimum eigenvalue, about \(-2.49\times10^{-16}\), is numerical roundoff around the exact constant nullspace, not a proof of negative dissipation.

The 27-node quadratic particle support reproduces its affine matrix to \(3.85\times10^{-16}\) and represented angular momentum to \(1.16\times10^{-16}\). Removing one support node and renormalizing leaves a first-moment defect of 0.205778. The SPH analytical constraint gradient differs from finite differences by at most \(1.12\times10^{-10}\). These results validate the specified local algebra and derivatives, not a full APIC, SPH or PBF simulation.

The boundary review fixtures distinguish linear from affine divergence: omitting pressure boundary work creates a 0.21 identity defect. A skew pressure connection gives normal gradient 3.2 instead of 2 despite positive geometry. An offset fraction step confirms opposite fraction-gradient and level-set-gradient signs. A compatible chain has retained residual maximum 0.001 but full maximum 0.003, so a 0.0015 threshold must reject it despite the reduced solve criterion.
