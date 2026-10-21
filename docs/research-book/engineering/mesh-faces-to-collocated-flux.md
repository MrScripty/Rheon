# Mesh faces to collocated flux: three finite research controls

Research stage based on PR51 `33094fb`, 2026-10-13. These are authored
pedagogical derivations of established mathematics, not a new CFD algorithm.
They prepare an engineering backend without implementing or qualifying one.
The plan and acceptance gates are in [ENGINEERING_PLAN](../../education/ENGINEERING_PLAN.md).

## One orientation and one shared transfer

Give every internal face an owner and neighbour. A positive integrated volume
flux phi leaves its owner and enters its neighbour. Its incidence column is
(+1,-1) in the outward-flux convention; its column sum is zero. This incidence
is C=-B relative to Chapters 3–4; the existing B sign convention is unchanged. With constant
rho=1 kg/m³, the numerical mass flux has the same value in kg/s. For a transported
momentum component v_f, both cells must use the **same** phi v_f. Equal and
opposite contributions then cancel. Two independently reconstructed transfers
need not cancel, even if each cell appears locally plausible.

The finite update m_L' = m_L - dt phi, m_R' = m_R + dt phi conserves the sum in
exact arithmetic. This says nothing about positivity without a timestep bound,
accuracy of phi, variable-density momentum, or whether a mesh reader assembled
all intended faces. Reversing a face orientation must reverse its flux and swap
its incidence signs, leaving the physical update invariant.

## Two cells: derive the missing boundary term

Both cells have volume 1 m³, centre separation 1 m and external face area 1 m².
The shared opening has area A in {1/4,1/2,1} m². Its prescribed velocity is u in
{-1,0,1} m/s. Left external outward flux is q in {-1/2,0,1/2} m³/s; right is -q.
The pressure test pairs are (1,3), (3,1), (1,1) Pa. The 81 states are instantaneous
operator probes, not solutions of momentum equations or moving-wall updates.

Write phi=A u. The cell divergences are (phi+q,-phi-q) divided by the unit volumes.
Their weighted sum vanishes. For arbitrary real pressure test values:

    pL (phi+q) + pR (-phi-q) = -(pR-pL) phi + (pL-pR) q.

Every term is power in watts. In metric notation Gp=(pR-pL)/L, the interior
term is -(Gp) u A L; the prescribed-flux contribution is p^T Q.
This is the affine counterpart of the homogeneous adjoint identity already in
Chapter 8, with the sign convention now completely specified.

At A=1/2, u=1, q=1/2 and p=(1,3), divergences are (1,-1) /s. The pairing is
-2 W: interior -1 W plus boundary -1 W. Dropping the boundary term predicts the
wrong value. At q=-1/2 both divergences vanish, but interior -1 W and boundary
+1 W still cancel. Divergence-free does not mean each work contribution vanishes.
The rendered opening is geometric; velocity arrows have a fixed velocity scale.
Pressure values are probes, never labelled a converged pressure solution.

## Collocated checkerboard: expose the unresolved mode

Take four unit-spaced cells with periodic indices. The planar strip is extruded
for inspection; its ends are identified, not walls. Use **kinematic** pressure
pi=p/rho, with alternating values (a,-a,a,-a) m²/s². The central cell gradient
(pi[i+1]-pi[i-1])/(2h) vanishes for every cell, including when a is nonzero.
A cell-velocity response u_i=-r g_i with constant positive response r in seconds
therefore vanishes, as does naive averaging to faces.

For this restricted constant-diagonal, zero-source model, separate the face
pressure difference from the interpolation of cell gradients:

    u_f = average(u_i,u_j) + r average(g_i,g_j) - r (pi_j-pi_i)/h.

The first two terms cancel in this model. At h=1, a=1 and r=1 the four oriented
face responses are (2,-2,2,-2) m/s and their cell divergences (4,-4,4,-4) /s.
Thus the compact pressure response detects the mode missed by the central cell
gradient. Constant pressure gives zero in both constructions. The periodic
sum of divergence remains zero; that global cancellation alone would not detect
the local defect. For arbitrary four pressure values, the compact operator has
quadratic form r times the sum of the four squared neighbour differences.

This derivation illustrates the motivation for momentum-weighted face
interpolation. It is **not a complete Rhie–Chow implementation**: no general
momentum off-diagonal source, variable coefficient interpolation, relaxation,
transient correction, boundary treatment or nonorthogonal reconstruction is
provided. Those choices must be derived together before claiming a SIMPLE or
PISO/PIMPLE solver. The step from a cell momentum equation to face flux and then
continuity is explained in Greenshields and Weller's developer-authored
[section 5.10](https://doc.cfd.direct/notes/cfd-general-principles/pressure-velocity-coupling).
The original Rhie–Chow publisher text was inaccessible during the preceding
assessment; no original-paper reading or copied implementation is asserted.

## Symmetric strain: include the boundary supplying the work

On a centred unit cube, prescribe U=L x, independent of z, with constant
mu>=0 and zero pressure. For the planar gradient L=((a,b,0),(c,d,0),(0,0,0)),
D=(L+L^T)/2 and tau=2 mu D. Its integrated dissipation is

    P = 2 mu a² + 2 mu d² + mu (b+c)².

It is nonnegative. Since tau is constant, div(tau)=0: positive dissipation is not
a claim of a nonzero local viscous force or an advancing decay. At each face
centre, t=tau n and U=L x. Face area is one; summing t dot U over all six faces
is exact here because the integrand on each planar face is affine. This sum
is tau:L=P. Boundary work balances the constitutive dissipation.

| Field at rate s | Planar gradient | P in a unit cube |
| --- | --- | --- |
| Rigid rotation U=(-s y,s x,0) | a=d=0, b=-s, c=s | 0 |
| Simple shear U=(s y,0,0) | a=c=d=0, b=s | mu s² |
| Planar extension U=(s x,-s y,0) | a=s, d=-s, b=c=0 | 4 mu s² |

The comparison mu L:L is 2 mu s², mu s² and 2 mu s² respectively.
It must not replace 2 mu D:D while discarding the different boundary terms.
All these affine fields have zero component Laplacian; this example does not
claim that a scalar Laplacian alone produces rotational decay. No-slip cube
walls would be incompatible with these imposed fields. The browser shows
velocity and traction with separately labelled fixed scales, not equal units.

Batty and Bridson, [arXiv:1010.2832v2](https://arxiv.org/pdf/1010.2832v2), section 3,
equations (3.1)–(3.5), supplies the full symmetric constitutive stress and total
traction context. Sections 2–3 distinguish coupled stress/boundary treatment
from simplifying the interior equation. Their staggered embedded-boundary
method and convergence evidence are not inherited by this collocated research.
The cube calculations above are independently derived constitutive controls.

## Proof and execution claims

[EngineeringFlux.lean](../../../proofs/Rheon/EngineeringFlux.lean) states shared
face cancellation, two-cell conservation, boundary work and its counterexample,
the checkerboard witness/response, periodic pressure energy and planar
constitutive power. Nonnegative power explicitly assumes mu>=0. Unit geometry
and definitions are stated above; geometric integration and parser correctness
are not formalized. The checkerboard blindness identity is algebraic even at
h=0 under Lean's total division, but the physical lesson admits only h=1.
There is no theorem of general collocated stability, convergence, PDE accuracy,
Rust/JS equivalence, IEEE correctness, native execution or performance.

Current kernel status is in [native-sequence.json](../../education/native-sequence.json).
The browser reference has 81+18+27=126 bounded states. Independent rational
checks and actual browser glyph checks are distinct from kernel qualification
and from human screenshot review. Unsupported controls refuse; no clamping
manufactures admissibility.

## Source, dependency and data review

No new dependency, mesh corpus, external code or figure is incorporated.
The authored equations, geometry and browser/Lean code use the existing locked
Three.js viewer and pinned Lean/mathlib. Cargo and npm locks are unchanged.
[The source-output policy](../../REPOSITORY_OUTPUTS.md) keeps generated captures
and numerical receipts outside Git. Existing package versions/lock identities
are recorded in [dependency audit](dependency-audit.json); this review covers a
zero-dependency-change educational slice, not a blanket approval of future
parsers, AMG libraries, mesh data or model weights.

Gmsh is excluded: no binary, SDK, wrapper, linking, bundling, generated corpus or
copied implementation. Its [official licensing statement](https://gmsh.info/#Licensing)
is GPL; a file format specification is not permission to import GPL code.
The original Rhie–Chow, Patankar–Spalding and Issa full papers remain explicit
reading prerequisites for a production coupling design. Scientific attribution
here does not import upstream software. This note reports no original algorithm
novelty or engineering benchmark acceptance.
