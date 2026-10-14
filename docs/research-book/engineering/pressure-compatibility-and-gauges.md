# Pressure compatibility and gauge handling

Authored finite teaching derivation, 2026-10-13. Based on PR60 `900e30b`;
see [the bounded plan](../../education/PRESSURE_GAUGE_PLAN.md). This is not a
production Poisson solver, a new numerical algorithm or a CFD benchmark.

## Start from the boundary and the face

Two disconnected rectangular regions each contain two cells sharing one face.
Every cell has cross-section A=1 m². Its width in metres therefore equals its
volume in m³. Choose V=(1,1,1,1) or (1,3,2,2). The centre distance in each pair
is d=(V_L+V_R)/(2A). Density is 1 kg/m³ and dt=1 s. The internal face weight is
w=A/(rho d), numerically 1 for uniform geometry and 1/2 for unequal geometry.
The scene uses 0.3 display units per metre and genuinely changes cell widths.

Orient internal volume flux from left to right. Outward incidence is C=(1,-1),
the negative of the book's B convention. Let Q_i be prescribed external outward
volume flux. The pressure correction has homogeneous normal derivative at the
external boundary; the prescribed normal velocity is not a prescribed pressure.
With zero initial internal flux, the corrected internal flux is

    q = -dt w (p_R-p_L),
    Kp = (w(p_L-p_R), w(p_R-p_L)),
    b = -Q/dt,       r = b-Kp,
    divergence_i = (Cq+Q)_i/V_i = -dt r_i/V_i.

Here Kp and b have units m³/s², r is an integrated residual, p is pressure in
Pa, q is m³/s, and divergence is 1/s. Since A=1 m² the numerical internal
velocity equals q, but the display labels its arrows as velocity. The lesson
solves this two-by-two algebra directly; it does not run CG or advance a fluid.

## Compatibility is separate for each disconnected component

The rows of each K block sum to zero. Therefore b_L+b_R=0 is necessary. With
w>0 it is also sufficient for this pair: set p_L=0 and p_R=b_R/w. The other
row then holds. The full kernel consists of one constant per pair because
w(p_L-p_R)=0 implies p_L=p_R. There are **two independent constants**, not one.
The Lean slice proves the cancellation, necessity and explicit compatible
solution; it does not formalize arbitrary-graph connectivity or kernel rank.

Use compatible b=(1,-1,2,-2). In unequal geometry, fixed-left pressures are
(0,-2,0,-4) Pa, jumps (-2,-4) Pa, and internal corrections (1,2) m³/s. Both
full residual and corrected divergence are zero. The supplied external flux
is Q=(-1,1,-2,2), so each region has balanced inflow and outflow.

Now use b=(1,0,-1,0). Its global sum is zero but region sums are (1,-1).
No pressure can solve the original system. Dropping each left row and fixing
its pressure to zero yields p=(0,0,0,0). The retained rows have zero residual;
the full residual is (1,0,-1,0). Unequal volumes give actual divergence
(-1,0,1/2,0) /s. A reduced-row norm would incorrectly accept this candidate.
The rendered status therefore remains **rejected**.

For A_op=V^-1 K, a left null vector is V per component, while the right null
vector is constant. Compatibility for normalized data f is sum(V_i f_i)=0,
not the unweighted sum of f. The symmetric integrated K instead uses the
ordinary sum of b. Mixing these tests changes the problem on unequal volumes.

## Three representatives of the same compatible pressure

Fixing one cell per component is an algebraic gauge when a compatible singular
system has already been specified. It is not a physical Dirichlet boundary.
Fixing just one cell in the whole disconnected mesh leaves another constant
undetermined. The following choices all preserve within-component differences:

| Unequal geometry, compatible b | Region A pressures | Region B pressures |
| --- | --- | --- |
| Fixed left cell | (0,-2) | (0,-4) |
| Arithmetic mean zero | (1,-1) | (2,-2) |
| Volume mean zero | (3/2,-1/2) | (2,-2) |

Arithmetic mean subtracts (p_L+p_R)/2; volume mean subtracts
(V_L p_L+V_R p_R)/(V_L+V_R). These select different representatives when
volumes differ. The former is orthogonal to constants in Euclidean coordinates;
the latter is orthogonal in the volume-weighted inner product. Minimum-norm
claims require naming that inner product. Neither changes Kp, face flux or r.

The two offset controls add c_A and c_B after choosing a representative. Their
nonzero settings deliberately leave its zero-gauge constraint while preserving
the physical face differences. A pressure difference **between disconnected
regions** is not determined without additional reference data. Prescribed
pressure, traction or a coupled free surface can remove gauge freedom; those
conditions are outside this all-Neumann lesson.

## Projecting b is an explicit source modification

The optional comparison subtracts half the component sum from each b entry.
For b=(1,0,-1,0), it gives b_hat=(1/2,-1/2,-1/2,1/2). In unequal geometry,
fixed-left p=(0,-1,0,1), flux=(1/2,-1/2), and the residual against b_hat is zero.
Against the **original** b, r=(1/2,1/2,-1/2,-1/2), and actual original-boundary
divergence is (-1/2,-1/6,1/4,1/4) /s. Original acceptance remains false.

This is Euclidean projection of the integrated b, not a uniquely physical repair
and not a volume-weighted projection of normalized f. It would imply changing
boundary/source data. The GUI reports the source delta and still computes its
acceptance and boundary work using the original Q. Production code needs an
explicit modelling decision before changing incompatible data.

## A local pressure error is not a gauge

Add epsilon=1/4 Pa only to region A's right cell after constructing a candidate.
For compatible unequal geometry, p=(0,-7/4,0,-4), internal flux=(7/8,2),
r=(1/8,-1/8,0,0) and divergence=(-1/8,1/24,0,0) /s. The volume-weighted total
still vanishes, but local divergence does not. This perturbation changes a
within-component pressure difference and must be rejected. It is not a global
pressure offset or a floating-point-error simulation.

## Keep boundary work when shifting the gauge

The exact outward-incidence identity for one pair is

    p_L(q+Q_L)+p_R(-q+Q_R) = -(p_R-p_L)q + p_L Q_L+p_R Q_R.

Both sides are watts. Under a common pressure shift c, the interior term stays
fixed and boundary work changes by c(Q_L+Q_R). The left pairing changes by the
same amount; the identity remains valid even for incompatible data. Balanced
external flux makes each change zero. A global sum of Q=0 cannot remove the
effect of independently shifting two incompatible components.

For compatible unequal geometry without perturbation, interior work is 10 W
and boundary work -10 W. For the incompatible fixed-left candidate, shifts
(c_A,c_B)=(2,-2) produce boundary work -4 W and pairing -4 W, with zero interior
work. This diagnoses inconsistent prescribed flux; it is not a physical
prediction that absolute pressure becomes observable in a valid sealed flow.
The minus-pressure traction convention gives physical pressure-force power
with the corresponding opposite sign; the GUI labels the algebraic pairing.

## Primary-source research and applicability

PETSc's developer-authored [MatSetNullSpace notes](https://petsc.org/release/manualpages/Mat/MatSetNullSpace/)
explicitly distinguish compatible singular systems from inconsistent ones,
identify the transpose nullspace in the solvability condition, and discuss
minimum-norm representatives. [MatNullSpaceRemove](https://petsc.org/release/manualpages/Mat/MatNullSpaceRemove/)
describes removing specified nullspace components. These support the distinction
between gauge selection and changing an incompatible right-hand side; no PETSc
code, dependency or convergence result is incorporated here. Read 2026-10-13,
release documentation identifying v3.26.0.

Gebhardt and Romero, [arXiv:2603.12959v1](https://arxiv.org/pdf/2603.12959v1),
sections 1–2 (pages 1–3), treats solutions modulo a finite-dimensional kernel
and a selected orthogonal representative under stated Hilbert-space assumptions.
Its original-problem load annihilates the kernel. This is useful research context
for separating compatibility from representative choice. Its regularization and
finite-element convergence analysis is not implemented, independently certified
or inherited by this finite-volume example. The finite formulas above are our
independent derivations, not a reproduction of that proposed method.

Bochev and Lehoucq, [SIAM Review 47(1), 50–66](https://doi.org/10.1137/S0036144503426074),
was verified through the publisher's abstract and metadata. Full text requires
access; no full-paper reading is claimed. It remains a deeper variational-reading
item. No access control was bypassed.

## Proof, implementation and validation boundary

[PressureGauge.lean](../../../proofs/Rheon/PressureGauge.lean) proves 15 finite
exact-real contracts. Division lemmas assume nonzero weight, volume or volume
sum; the physical control domain strengthens these to positivity. Fixed-cell
sufficiency explicitly assumes compatibility. The gauge-offset identity holds
without that assumption and therefore cannot be mistaken for solvability.
No arbitrary mesh nullspace characterization, PDE accuracy, iterative-solver
convergence, JavaScript/Rust equivalence, IEEE correctness or runtime guarantee
is proved. Current source-bound kernel status is in
[native-sequence.json](../../education/native-sequence.json).

There are 288 finite control states, checked against independent rational
closed forms and deliberate false-acceptance mutations. Browser controls,
actual cell/bar/arrow geometry, PDF/link checks and human screenshot review are
separate evidence layers. Existing dependency locks and native admission inputs
remain unchanged. No third-party implementation, data, figure or new package is
imported; the code is independently authored within Rheon's Apache-2.0 project.
