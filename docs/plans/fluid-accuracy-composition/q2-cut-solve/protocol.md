# CV2 prospective solver/reference protocol v1

Method selection: method.md, independently admitted QR variant before coding.
Original four CV1 source polygons, grids, all support/pressure rows, rho1 and
mu f64(.1) remain unchanged. Native in-process CV1 builder/traces/export reused
read-only; preserve all prior native-v1 evidence. No floors, clipping, gauges,
regularization or automatic fallback. Exactly one fresh release CPU campaign.
Source/protocol independent review and source/oracle freeze precede selected
contracts/campaign. No selected new numerical probe has run during discovery.

Fixed24solves: protocol's four fixtures × dt(.125,1)s × three scenarios:
- unforced: nodal u*=CV1 work-vector b (its exact dyadic polynomial at MAC nodes),
  g_traction=0, body force0;
- rigid: nodal u*=(.3-y,-.2+x)m/s, traction0;
- affine: nodal u*=CV1 affine(.25x+.75y+.1,-.2x-.25y-.3)m/s,
  physical σn, σ=-2I+mu(Agrad+Agrad^T), on original polygon boundary.
No RHS is manufactured as KKT action on a desired answer. Full boundary load is
integrated using CV1 physical trace, not from KU-B^Tp. Affine omissions/reversed
pressure are detectable via retained entrywise CV1 load checks.

Lref=Href=1m,Uref=1m/s; rho=1kg/m³. Pressure reference for every selected step is
Pref=rho*Uref*Lref/dt+mu*Uref/Lref, numerically1/dt+.1Pa; never measured from p.
All primitive f64 material, geometry, dt, gradient/c, actual rounded nodal u*
and generated σ coefficients are exported with the actual assembled A and f.
No confusion between Fraction(1,10) and Fraction.from_float(.1) is permitted.

TargetR1: high-precision solve of exact represented values of the actual native
exported full A, B and RHS f. The zero pressure block and constraints are exact;
no gauge. Includes native A/RHS construction rounding by definition.
TargetR2: independently reconstructed exact represented-source basis integrals,
using exact represented MU=.1f64, dt, SAME native rounded nodal u*, exact analytic
σ from represented MU/gradient and2Pa before multiplication/integration rounding.
Compute exact M/dt+K and M u*/dt+F before high-precision conversion. Native σ/load/
RHS rounding belongs to the R1/R2 perturbation gap. These are two discretizations
of arithmetic, not two physical models. Independently compare native coefficients
against BOTH targets; a precise solve of rounded blocks does not qualify R2.
Both high-precision targets solve at80 and120decimal digits and must agree
within1e-40 in Uref/Pref. Both highprecision direct scaled residuals must be<=1e-60. These are numerical
reference checks, not interval enclosures or rigorously certified forward-error
bounds. No separate approximate-inverse/interval proof is claimed.
Use independent LU (not native Cholesky/QR implementation). Exact CV1 support,
rank and fulloperator reconstruction remains required; native operator CSVs must
also match the original CV1 corpus byte-for-byte (same frozen implementation).

Native selected method B: diagonal velocity congruence and lowerCholesky,
scaled thinHouseholder QR of weighted B^T; no explicit Gram in the solve.
Exactly3 compensated original-full-target residual refinements. Export initial
and all3corrected coefficient/residual states; never choose an iteration after
observing reference errors. Unscaled partial-pivot KKT method A is a diagnostic
comparator only. All its results/refusals are retained; no fallback selection.
Use original full matrices for all residuals, energy/work and solution oracles.

Required finite fixed1e-10 gates for selected B, all24solves:
1 every velocity coefficient error/Uref and every pressure error/Pref vsR1 andR2;
  rigid/affine velocity and absolutepressure analytic targets also<=1e-10.
2 actual row momentum residual r=M(u-u*)/dt+Ku-B^Tp-F divided by
  sum_j|Mij|Uref/dt+sum_j|Kij|Uref+sum_c|Bci|Pref+|Fi| <=1e-10;
  cell continuity |Bu|/(sum_i|Bci|Uref)<=1e-10. Also independently compare actual
  closed-cell Cartesian+physical trace flux with Bu in h*Href*Uref scales.
3 E=.5u^T M u; Echar=.5sum_ijMij Uref²>0. Require raw
  raw=E-Eold+.5delta^TMdelta+dt*u^TKu-dt*u^TF and residual-corrected
  defect=raw-dt*p^TBu-dt*u^Tr, each ABSOLUTE VALUE/Echar<=1e-10.
  Record massskew=.5u^T(M-M^T)u*; also require the absolute defect after
  subtracting this explicit skew term/Echar<=1e-10 (no symmetrization). Plus
  dt*(sum|p_c(Bu)c|+sum|u_i r_i|)/Echar<=1e-10. Unforced/rigid energy
  <=Eold+1e-10Echar; dissipation>=-1e-10*mu*Uref²*Href.
4 physical boundary work direct trace integration vsu^TF /1W<=1e-10;
  physical linear/angular momentum change minusdt*actual boundary resultant
  <=1e-10 in rho*Lref³Uref and rho*Lref4Uref scales. Reconstruct these with
  independent physical basis integrals and original source edge loads.
5 all native and reference values finite, every original active/pressure row
  retained; independent exactrank full, factor identities/scales/source shape
  consistent within fixed1e-10 norms: ||L L^T-lowersymmetric(DAD)||inf/
  ||lowersymmetric(DAD)||inf, ||Q R-Z||inf/||Z||inf, ||Q^T Q-I||inf.
  Here Q=Z R^-1 reconstructed by triangular solves; no unstated scale floor.
  Also require relative∞ identities scaled_A=DAD, LY=DB^T and Z=YE
  (denominators respective exact right-side∞norms) and max-entry diagonal
  normalization |Dii²Aii-1|, |Ecc*||Ycol||2-1| <=1e-10. No residual-only accuracy claim.

Report separately originalM, A, KKT and equilibratedA/Z/Schur condition diagnostics,
iteration histories, baselineA/R1/R2 errors and R1-R2 gaps. No conditioning number
threshold is inferred from data. Preserve squarecondM6.5331614839e11 and minMii
1.7013962159e-14. Failure vsR2 is assembly/method sensitivity evidence even if
R1/residual pass. Stop admission/return to method selection with all failures.
No different scaling/QR/highprecision fallback or changed fixture after data.

Focused debug/release contracts: positive/singular/nonfinite Cholesky; QR with
negative valid diagonals, exact rankdeficiency refusal, orthogonality/factor
identity; coupled nonzero-g sign and rowshape validation; invaliddt; compensated
residual cancellation. StrictClippy/format/lockedreleasebuild. Source/binary/
command/outputs hash-bound. Independent always-on analyzer normal/-O/-OO with
small malformed/wrongsolution corpus whose guard reason identifies intended
failure; freeze before rawinspection/evaluation. No fullsuite for private-only
addition unless new production/dependency changes arise. No Lean proof reused
as a full-mass/IEEE solver claim. No PDE/hydrostatic/3D/production admission.
