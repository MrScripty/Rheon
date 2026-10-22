# Prospective aligned coupled reference v1

No selected tests/native computation before independent source/protocol review and
freeze. Two exactly aligned fixtures reuse existing StaticObstacleGeometry and
AlignedStrain with sealed outer normal/free-slip tangential walls and stationary
no-slip interior box. Unit grid3³ h1, origin0, obstacle indices[1,2]³; anisotropic
grid3x4x3 h(.5,.75,1.25), origin(-.5,.25,-1), obstacle indexlo(1,1,1),hi(2,3,2).
Both are fully aligned resolved geometry, not a cut-cell or free-surface extension.
Each has rho/mu=(1,.1) or(998.206,.001002), dt=.125 or1: eight cases. The water
scale checks dimensions/numerics only; no experiment or physical qualification.

At each existing active face, actual positive M=rho*A*d (kg). Integrated outward
B (m²) has +A at the negative cell and -A at the positive cell. Existing strain
rows S have inverse-length coefficients; geometry weights are2V for normal rows
and fluid-sector V for shear. W=mu*geometryweight; K=SᵀWS (kg/s). All positive
mass and nonnegative finite weights must remain; no floor/merging. A=M/dt+K.
Assemble and solve ONE KKT system [A,-Bᵀ;-B,0] with rhs[M*u*/dt,0]. Prescribed
boundary velocities are homogeneous and already eliminated by existing rows;
there is no undisclosed nonzero boundary lift. Keep old split comparator unchanged.

Analytic input at actual active face position (SI numerical coordinates x/Lref,
etc, Lref=1m,Uref=1m/s), axis index d=0,1,2:
u*=Uref[sin(.7x/Lref-.4y/Lref+.3z/Lref+.2d)
          +.25cos(x/Lref+.6y/Lref-.2z/Lref+.4d)].
No manufactured RHS generated from the assembled matrix/desired solution, no
forcing; initial divergence need not vanish. Pressure is Pa. Pressure reference
Pref=rho*Uref*Lref/dt+mu*Uref/Lref. Gauge one first pressure cell per actual
connected component; omit only those pressure columns/constraint rows, reconstruct
all pressures with gauge0 and check ALL fluid-cell continuity afterward. Verify
incidence graph components and B column cancellation independently; homogeneous
flux RHS is component-compatible. Nonhomogeneous compatibility/lifts are outside
this reference. Direct partial-pivoted elimination bounded to256unknowns, exact
zero/nonfinite pivot refusal; no arbitrary small-pivot floor or production backend.

Mandatory fixed gates before admission:
- Independent analytic aligned geometry, active-face IDs/positions/areas/distances,
  mass, component/gauges, every strain row/term/weight and B/K/A/coupled block agree
  within1e-12 scaled by max(1,actual magnitude), signs/IDs/counts exactly. S rows
  independently reconstructed from aligned finite differences/halfwidth obstacle
  traces and per-quadrant geometry, never accepted solely by reassembling exports.
- Independent direct solution of independently assembled blocks: velocity error
  /Uref <=1e-10, pressure error/Pref <=1e-10. Report condition numbers; rank of B
  is nc-components, positive M, nonnegative W, symmetric K and KKT verified.
- Full momentum r=M(u-u*)/dt+Ku-Bᵀp normalized per face by
  M_i*Uref/dt+sum_j|Kij|Uref+sum_c|Bci|Pref <=1e-10. Full cell continuity
  |Bu|/(sum_j|Bcj|Uref)<=1e-10, also report physical flux and divergence /V.
- E=.5uᵀMu, Echar=.5sum(M)*Uref²; unforced backward-Euler balance
  E-Eold+.5||u-u*||²_M+dt*uᵀKu=dt*pᵀBu+dt*rᵀu.
  Both raw unforced balance/Echar and defect-corrected balance/Echar <=1e-10;
  dt*(sum|p_c(Bu)c|+sum|r_i u_i|)/Echar <=1e-10. Dissipation >=0 and energy
  <=Eold+1e-10Echar. Independently reconstruct all terms, pressure work and bound.
- Every selected value finite. No criterion relaxation after observation. Retain
  any refusal/failure; stop dependent progression. No continuum MMS orders inferred.

Focused debug/release contracts cover pivoting/singular/nonfinite refusal and
coupled block dimension, gauge count, invaliddt and exact component flux
cancellation. Strict Clippy; ONE actual release CPU campaign with fresh root,
source/binary/commands/outputs/hashes retained. Independent oracle source frozen
before raw evaluation; eight-case structure/evaluator gates always-on underPython
normal/-O/-OO. Full-suite rerun unnecessary for a private reference-only addition.
Existing plan StokesWork theorem is exact-real correspondence guidance with fixed
M/B/S/W, stationary homogeneous boundary and actual residual; do not claim new
kernel proof or IEEE/PDE/physical result. Cut/free-surface operators, disk/annulus
MMS, oblique hydrostatics and production backend remain pending and gate progress.

Prefreeze rank clarification: compute components from independently reconstructed positive-area face incidence. The weighted graph-incidence theorem fixes rank(B)=nc-components exactly; no observed SVD tolerance selects rank. Floating condition numbers are diagnostics only. Native tiny dimension refusal precedes allocating dense blocks.
