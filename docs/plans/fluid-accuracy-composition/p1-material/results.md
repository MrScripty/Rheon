# Coupled gate closes in1D; use the grid route for production work

**The one n8 material-map reference meets its frozen native necessary gates.**
Independent source-integral and field review also passes; the disposition is
recorded in review.md.
This is a successful bounded reference construction, not an SPH repair, production
water acceptance or a reason to start a third moving-mesh solver family.

The source/protocol/evaluator froze at3b609ee443a1a704f6aeeb4b836fcf30f7f4d248
before any tests or measurements. Exactly8 states,72 node records,64 element
records,24 fixed-end and24 translation probes completed in one native CPU run.
No resolution/support/timestep sweep or trajectory run occurred. Both previous
negative campaigns, their criteria and archives remain byte unchanged.

| Fixed native check | Actual result | Disposition |
|---|---:|---|
| Volume telescope error | 0 | Pass |
| Mass assembly defect / M_ref | <=2.28e-16 | Pass1e-12 |
| Mass first-moment alias / (M_ref L_ref) | <=1.14e-16 | Pass1e-12 |
| Instantaneous work / fixed power scale | <=2.70e-16 | Pass1e-12 |
| Global pressure/body-force balance / P_ref | <=1.84e-16 | Pass1e-12 |
| Base uniform interior force / P_ref | <=2.28e-14 | Pass1e-10 |
| Base hydro interior force / nodal weight | <=2.68e-13 | Pass1e-10 |
| Base endpoint load defect / P_ref | <=2.28e-14 | Pass1e-10 |
| Affine pressure plus physical endpoint lift / (1000Pa) | <=1.57e-16 | Pass1e-12 |
| Edge versus assembled-H quadratic / fixed energy scale | <=2.79e-18 | Pass1e-12 |
| Fixed-end Cholesky pivot minimum | >=8.706e6 Pa/m | Positive on all8 states |
| Scaled-H absolute-row frequency² bound | 262144–292638.962 s^-2 | Finite positive; no nonlinear dt admission |
| Alternating CELL pressure interior force | exactly20Pa | Retains a response, unlike the prior central stencil |
| Wrong equal-half-mass gravity control | up to6.89394e-5 of nodal weight | Detects discarded first moments |
| Wrong pressure-only wall control | up to.0673944 of P_ref | Detects missing total endpoint reaction |

Hydro equilibrium reactions are -9936.983974Pa below and +100Pa above for both
ordered and perturbed reference partitions. These come from the total pressure
and source-gravity derivative, rather than postprocessed load assignment. The
ordered wrong-gravity control is already1.19421e-7 of weight; the perturbed
control increases it to6.89394e-5, both visible against the1e-10 equilibrium gate.
The controls exceed their predeclared1e-8 discriminator threshold.

All states and displaced evaluations passed J>0 and source-extrema wet/1%-envelope
checks. The maximum reported density compression among the8 state records is
.97215848%; extrema for displaced probes are asserted, not individually recorded.
This closed100Pa-top fixture still does NOT replace the plan's free-surface
hydrostatic/rest trajectory or3D wall acceptance matrix.

## Retained finite-difference limitation

Every one of the16 case/direction groups passes at the TWO consecutive predeclared
steps1e-5s and1e-6s. The8 fixed-end1e-7s Hessian differences FAIL their individual
1e-6 normalized criterion; all8 translation differences at that step pass. Thus
40/48 individual probe points pass, while all16 groups meet the frozen rule.
No point was dropped, threshold changed, source adjusted or campaign repeated.

| eps(s) | Max normalized potential-derivative error | Max normalized Hessian/force difference |
|---|---:|---:|
| 1e-5 | 4.22e-10 | 6.48e-9 |
| 1e-6 | 4.31e-9 | 5.52e-8 |
| 1e-7 | 5.42e-8 | 1.39735e-6 |

Error grows as displacement shrinks, so the smallest native difference is not
accepted as a more precise derivative. Independent80-digit reconstruction gives ideal-increment Hessian difference
error<=1.99e-15 at1e-7s, while f64-rounded coordinate increments alone yield
1.25892e-6–1.36499e-6 across all8 cases, versus native1.21609e-6–1.39735e-6.
Coordinate rounding amplified by stiffness thus reproduces the failure scale;
this explanation does not turn the failed native points into passes. The source
series truncation and native integration rounding are separately checked: maximal
normalized mass/first-moment discrepancies are2.52e-16/1.58e-16 against the frozen
1e-12 thresholds. See review.md for independent source/field evidence. This source
and positive assembly identity do not justify pretending every finite difference
passed, or claiming a formal IEEE error theorem.

## What changed the result

PR123 established that exact work without an invariant volume partition leaves
E0, while an exact Voronoi partition without consistent first moments/gravity
leaves irregular-site hydro and boundary errors. Here one material map supplies
volume, density, mass moment, gravity and endpoint derivatives. Source integration
preserves hydrostatic weak balance on either reference partition; no site/cellmean
EOS reset, density offset or corrective pressure multiplier is used. A cell
interface incidence supplies the stiffness, avoiding the1D even/odd displacement
null of a central-site stencil.

The cost is a different representation: fixed connected material interfaces,
reference source moments, a deliberately lumped kinetic metric and held boundary
DOFs. This is not a selected SPH method. No viscosity,3D determinant/mesh validity,
remapping, general solid geometry, free-surface stress or nonlinear trajectory
was tested. Stationary endpoint reactions do not qualify moving walls.

**Recommendation: close this bounded P1 production-selection attempt and
prioritize existing-grid Q1 verification → Q2 shared geometry/coupled Stokes.**
The [route decision](route-decision.md) compares concrete owners and unresolved
obligations. This recommendation stands even though the reference gate succeeds.
No further kernel/material-family campaign, Q implementation, canonical-plan
migration, merge or G1 edit is included. Repository lead owns the next admission.
