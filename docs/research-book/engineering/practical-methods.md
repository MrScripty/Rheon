# Practical mechanisms: research, derivations and acceptance scope

This continuation completes the finite teaching packages specified in the
[engineering plan](../../education/ENGINEERING_COMPLETION_PLAN.md). Its eight
browser mechanisms are authored analytical/algebraic references. None modifies
a production solver, imports source code from a paper or claims a measured
backend speedup. The 248 admitted states are independently evaluated by
[the rational/struct oracle](../../education/practical_oracle.py), which also
rejects deliberately corrupted physical quantities and acceptance decisions.

## Wall stress and loads

For Newtonian incompressible flow use total stress σ = −pI + 2μS and traction
σn with the declared outward fluid normal. At a stationary no-slip wall, work
vanishes although traction can be nonzero. At a moving wall it need not vanish;
body and boundary work must accompany internal dissipation. The steady unit-gap
profile in chapters 2 and 19 derives all forces and powers analytically and
renders their fixed-scale velocity/traction arrows. Constant pressure shifts
cancel on a geometrically closed surface whose area normals sum to zero; an
open patch's pressure load requires its reference and exterior loading.

[Batty and Bridson's primary Stokes paper](https://arxiv.org/pdf/1010.2832v2),
section 3, equations 3.1–3.5, was read for symmetric deviatoric stress,
no-slip velocity and total traction at a free surface. Its method is a
staggered embedded-boundary discretization; it does not supply a collocated
Rheon boundary implementation. The chosen wall profile is our independently
integrated continuum oracle. General outlet/backflow, moving geometry and FSI
need their own explicit boundary/data/energy contracts. The official Re20
benchmark is particularly useful because its velocity-gradient outlet
operator differs from a zero full-symmetric-traction substitution; see the
[benchmark research record](constitutive-validation-research.md).

## Sparse iteration and CPU claims

[Shewchuk's primary guide](https://www.cs.cmu.edu/~quake-papers/painless-conjugate-gradient.pdf),
sections 5.2–5.3, was read for Jacobi splitting and the iteration matrix's
spectral-radius criterion. Positive definiteness alone does not ensure
Jacobi convergence. Our two-unknown Dirichlet matrix has an exact positive
energy and independently derived smooth/alternating error factors. The browser
executes each iteration and recomputes the full residual, while the Python
oracle evaluates closed-form mode powers rather than duplicating the loop.
Lean states matrix positivity, both mode identities and the strict interval
0 < α < 4/3. No general CG, multigrid or production-convergence theorem follows.

CPU performance needs a frozen workload, actual timing, quality/error matching,
paired controls, warmup, environment and variability. Main's merged PR58 retains
measurements and rejects the investigated pressure fusion after adverse controls;
no speedup from that rejected experiment is claimed. PR59 warm-start performance
acceptance remains separate. Chapter 13's exact byte-count fixture includes
values, CSR indices and work vectors, but cannot supply measured peak memory or
cache behavior. Chapter 15's executed reduction makes a precision issue visible
without rewriting another worker's numerical implementation.

## Transfer and finite arithmetic

The [APIC authors' publication summary](https://disneyanimation.com/publications/the-affine-particle-in-cell-method/)
was read for PIC information loss and affine-transfer motivation. The associated
[technical report](https://media.disneyanimation.com/uploads/production/publication_asset/105/asset/apic-tec.pdf)
was retrieved but its available text extraction did not expose a usable PIC/FLIP
section; no full-report reading or APIC proof is claimed here. The one-sample
PIC/FLIP fixture is fully defined in chapter 11, including unequal masses,
prescribed grid increment and blend fraction. It demonstrates linear momentum
and loss/preservation of unresolved velocity variance at fixed positions.
It does not establish angular momentum, SPH neighbor validity or dynamic stability.

[The normative ECMAScript Math.fround algorithm](https://tc39.es/ecma262/multipage/numbers-and-dates.html#sec-math.fround)
was read: the conversion uses binary32 ties-to-even and returns the represented
value in Number format. The browser explicitly rounds every addition. Python
checks its twelve traces by struct conversion. Lean separately proves only the
exact-real order identity. No equivalence between that theorem and floating-point
execution is asserted. This separation is essential for parallel CPU reductions.

## Specified turbulence, verification and qualified proposals

The [constitutive research record](constitutive-validation-research.md) identifies
primary model sections, precise SST-2003 coefficients, pressure convention,
limited production and the corrected omega-source convention. Chapter 22's
first probe uses supplied positive omega, nonnegative k, strain and F2. It
checks an algebraic constitutive response, including zero k. It neither computes
wall distance/blending nor integrates turbulent transport. The specific stress
closure and its model-validation requirements remain explicit.

[NASA's spatial convergence procedure](https://www.grc.nasa.gov/www/wind/valid/tutorial/spatconv.html)
was read for Richardson/GCI assumptions, safety factors and the asymptotic
comparison. Our actual midpoint quadrature makes a known quadratic error law
observable, then adds a constant bias that persists under refinement. The
closed-form independent oracle verifies the sampled sum and extrapolation.
Lean proves extrapolation identities conditional on the quadratic error model;
it does not establish that arbitrary CFD data have that model or that GCI is
a probabilistic physical-uncertainty bound. Benchmark geometry, BC, force and
pressure definitions are pinned separately; no Rheon benchmark result is invented.

The two-cell candidate gate mirrors the essential mathematical distinction
between a gauge offset and a pressure-jump error. It explicitly checks finite
and unit contracts before its full residual tolerance and exact fallback.
Its proposals are synthetic. A production AI provider additionally requires
source/model/schema provenance, compatibility, bounded resources, mutation and
fallback contracts, actual inference testing and matched-quality timing.
Passing this educational gate establishes neither learned accuracy nor speed.

## Ledger and limits

Implemented: eight recomputed 3D mechanisms with fixed declared scales and real
controls. Configured: relevant scroll-linked examples in all 25 chapters and six
appendices. Checked locally: 248 finite states, invalid-control refusals and eight
oracle mutations under normal/optimized Python. Proposed Lean: 23 statements
with explicit assumptions; kernel qualification pending. Hosted browser and
full publication qualification pending. Human screenshot review remains unavailable
where supported artifact downloads return HTTP403; no denial is bypassed.

The finite research/book work can be reviewed independently of a full engineering
backend. Continuum convergence, turbulent-flow validation, production collocated
assembly, general mesh stress loads, moving geometry and performance acceptance
remain distinct development/scientific obligations. Placement counts never
replace those obligations.
