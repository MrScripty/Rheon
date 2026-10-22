# P1 follow-on: work versus partition selection gate

Status: pre-experiment protocol v1. Base PR121/a64d4d3; original campaign source
39098e0 and all results remain immutable. Parent plan PR120/8422f77 retains all
production criteria. This is a disjoint static rejection gate, not a replacement
qualification campaign, a production method, or P2 initialization admission.

Exclusive writes: this directory, examples/particle_adjoint_selection.rs,
examples/particle_adjoint_selection/*, tests/particle_adjoint_selection_contract.rs.
No production, G1, shared module, root plan, proof, workflow or dependency writes.

## Question and candidates

Does correcting the pressure/density adjoint also eliminate the initial
pressure/E0 defect? Static results cannot decide the full PR121 trajectory failure. Compare `gamma-offset-adjoint-planar-v1` against an exact-partition
`clipped-voronoi-1d-reference-v1`. The latter is a causal reference, not a selected
water discretization. Reuse PR121's unchanged exact 3D Wendland planar marginal
and linear EOS. No viscosity or dynamics are added to this static probe.

Read mathematics.md for the complete map, derivatives, wall reaction and source
scope. Offsets are initialized once and frozen through every directional probe;
no density reset, renormalization, pressure-gradient substitution or damping.

## Frozen inputs

SI water rho0=998.2, c=32, g=9.80665. Closed domain [0,1] m, n=32,64,
h/dx=1.5 (R=3dx). Ordered y=(i+.5)dx and perturbed
 y=(i+.5+.1 sin((i+1)*2.399963229728653))dx.
The perturbation is deterministic and bounded by .1dx; it is a new static
probe, not a rerun or tuning of PR121's random campaign.

Uniform case: rho*=1.005rho0, g=0. Gamma-map mass=rho*dx, offset initialized
to rho*. Reference mass=rho*clipped Voronoi volume; this is an explicit distinct
reference quadrature, not a production variable-volume change.
Hydro case: top pressure=1000 Pa to keep all finite-difference states on the wet
EOS branch, rho(y)=rho0 A exp(g(1-y)/c²), A=1+1000/(rho0c²).
Gamma masses integrate that source profile over original equal-width parcels;
offsets initialize exact site density. Reference masses integrate the same
profile over the clipped cells; resulting density is the cell mean, without
site-value resets. Record both conventions and their effects.

At every case record every site's y,m,rho,p,V,F, equilibrium acceleration,
continuity rate, boundary derivative; signed offset and gamma minima for gamma.
Probe direction v_i=.01 sin((i+1)*1.4142135623730951) m/s.
Directional centered differences use displacement eps*v with eps=1e-5,1e-6,1e-7 s;
recompute density from the frozen map, never from a separate integration step.
Record U and total-volume gradients independently against analytic force/work.

## Fixed static criteria and stop rule

Fixed U_ref=.01 m/s. P_ref=1024*998.2*.005 Pa for uniform and
P_ref=1000+998.2*9.80665 Pa for hydro, independent of observed values.
Instantaneous work / (P_ref U_ref) and momentum+wall-reaction / P_ref <=1e-12.
Record their absolute SI residuals too.
Directional U-gradient and total-volume directional derivative agreement <=1e-6 at
at least two consecutive eps values, normalized respectively by P_ref U_ref and
U_ref. Density derivative error is normalized by rho0 U_ref/dx with the same <=1e-6
two-consecutive-eps criterion. Unavailable hydro/support diagnostics are NaN.
Uniform-pressure E0: max|F|/p* <=1e-10 (force per area has units Pa).
Report acceleration separately; no equivalence with a dx-based bound is assumed.
This algebra gate does not replace original trajectory energy or rest criteria.
All evaluated density states (including displaced probes) must stay on the wet
branch rho>rho0, asserted by the executable. Applicable maps must have positive
gamma, positive numerator/density/volume and ordered
interior sites throughout probes; fail closed otherwise.

Manufactured linear-pressure operator test p_i=1000+1000 y_i Pa uses the declared
geometry map, independent of EOS, and compares F_i with -1000 V_i. Record interior
and boundary relative error separately; do not discard boundary failures.
The common boundary band has distance<R=3dx from either wall; interior means
distance>=R. The separate alternating-pressure statistic excludes only the two
end sites, since that exact reference stencil property concerns nonend sites.
Record each wall load against continuum -p(0),+p(1), including absolute and
relative defects; global action/reaction cannot establish traction accuracy.
Report hydro initial max|F_i/m_i-g|/g and RMS, split boundary/interior, with no trajectory admission claim.
No hydro tolerance is invented from an initial-acceleration screen: any nonzero
residual is evidence for further selection, not an excuse to relax PR120 gates.
Reference alternating pressure p_i=1000+100(-1)^i must retain its interior zero
force. This is a pressure-to-force operator defect, not a completed EOS/Hessian
nullspace analysis; the unresolved acoustic gate blocks reference admission.

If the gamma repair fails E0, do not run another trajectory sweep. If the exact
partition reference passes E0 but fails boundary/hydro/nullspace gates, retain
those negatives and return to representation and boundary selection. No c/h/dt
sweep, smoothing, offset reset, mass adjustment or regularizer after seeing data.
Freeze source/protocol in a commit before numerical evaluation. Independent
review checks equations before freeze and recomputes results afterward.

## Qualification limits

No timestep is admitted. Work adjointness does not ensure a positive acoustic
Hessian: D²U=sum m[e'' Dρ Dρᵀ+e' D²ρ]. No native 3D viscosity, mesh geometry,
free-surface dynamics, GPU, desktop or physical-impact acceptance is inferred.
