# M2 kernel and evidence admission

M1 is locally committed at `638008d4f06e885b681428a9d67bd5c4c3e0b01e`.
M2 owns only the approved coupled full-box numerical model; it adds no native
mode, brush, recorder, material pair selection or external dependency. The
initial implementation is serial CPU, using the existing pressure owner.

Write set: new `src/voxel_material/{transport,viscosity,step}.rs`, the existing
module/state/boundary README and topic status/evidence. Private kernel oracles
live in their module's `#[cfg(test)]` sections so they do not require public
state-mutator/test APIs. Public complete-step checks live in
`tests/voxel_material_step_contract.rs`. This refines the plan's proposed three
integration-test files: private component oracles stay at their actual owner,
and the integration test exercises the complete public simulation API. No
existing source owner beyond M1's module is changed in this slice.

## Concrete state and arithmetic choices

One owner holds accepted and disposable state, the M1 Jacobi projection,
transport Q/H/J and two cell scratch fields, and two stress face scratch fields.
Minimum aggregate f64 payload is `8*(14*N + 9*sum_face_lengths)` bytes;
constructors recheck actual Vec capacities and pass remaining caps to each
owner. Caller input, allocator/process overhead and future native adapters are
separate. There are at most eight dt-reducing attempts from the same accepted
state for the volume/dual-mass Courant or viscous bound. Other failures return
directly. A final callback precedes an infallible state/report/budget swap.

The tree processes descending x-fastest cell index. Its parent lowers X,
otherwise Y, otherwise Z. The correction is applied once to the oriented
parent→child face; the actual stored correction is propagated to the parent's
residual. The full residual is independently recomputed afterward. Original
all-cell divergence is checked before closure. Correction/root budgets use the
admitted tree bound plus gamma_(16N) times original absolute flux/residual scale;
the maximum tree depth is nx+ny+nz−3. This is not exact binary64 closure.

Stored F uses the approved incoming constant-preserving formula, with incoming
and outgoing volume Courant each≤1/2. H is the same old donor F times Q; J is
rho_g Q+(rho_l−rho_g)H. Cell continuity must match dt*rho_old*d_i, and dual
continuity must match its two half-cell defects. Momentum uses that same dual
J and end-candidate mass, with outgoing mass Courant≤1/2. No clipping,
rebinarization, mean-shifting, ambient refill or mass repair exists.

All nonzero intermediate arithmetic must remain normal; underflow outside that
policy refuses explicitly. Upward-rounded gamma bounds use machine epsilon,
which conservatively exceeds binary64 unit roundoff. Per-cell fraction
evaluation has at most six incoming terms with coefficient(dt, absQ, V), donor
difference, product and accumulation, plus final addition: fewer than64 rounded
operations along that local expression. Its absolute scale includes F_old and
the incoming |donor|+|F_old| terms. **Gas=1−F inherits this absolute F error**;
a small gas fraction cannot substitute its own small relative scale. Gas adds
the complement-expression allowance. Reductions add gamma_(N+4) on direct
before/after phase volumes. Each phase's sum of induced dt*phase_old*|d_i| must
itself fit the arithmetic allowance; the admitted per-step total is accumulated
from the original frozen reference, never reset each step.

Cell/dual mass checks use separately computed old/new masses and actual shared
flux sums. The local128/64 operation envelopes bound fraction/coefficient/flux
and half-cell reductions respectively, with density-scaled absolute mass/flux
magnitudes. They are not pressure-tolerance allowances. Review must check these
expression paths; a failed deciding fixture cannot be repaired by tuning the
envelopes to observed output.

Variable stress is K=Sᵀ W S, with normal strains at cells and interior shear
edges; free-slip outer shear traction is zero. Cell mu is volume weighted;
shear mu is the declared harmonic mean of four cells, evaluated with scaling
by their minimum to avoid reciprocal overflow (zero if any is zero). Fixed
normal faces have no dynamic mass or stress update. Actual weighted rank-one
strain contributions yield an upward Gershgorin bound for M⁻¹/² K M⁻¹/²;
dt*bound≤1. Work and the explicit update energy identity are checked separately.
The energy/work reduction dependency paths are bounded by128N operations;
weights/masses are the same stored-coefficient expressions in each view, not a
claim of exact continuum-interface traction. Viscosity acts before gravity;
gravity immediately precedes the variable-density projection.

Independent review R1 corrected the work scale before M2 admission. For each
represented strain row c and positive stored weight W, accumulate upward
T=sum_i |c_i*u_i| and B=sum_rows W*T². There are at most four entries per row,
fewer than6N rows and fewer than9N dynamic face entries (counts≥4). The strain
multiply/add dependency, rank-one force products/assembly, dissipation products
and final dot reductions fit the retained128N envelope. The exact dot of the
represented row is bounded by T; stored-strain error has absolute scale T,
so its contribution to work-versus-stored-dissipation error is covered by B,
including nearly rigid cancellation. The separate upward final-dot scale
C=sum_faces |u_i*(Ku)_i| covers the final stored-force products/reduction.
The work budget is gamma_(128N)*(B+C+D), where D is stored nonnegative
dissipation. Upward accumulation bounds the positive scale computations;
unsupported overflow/subnormal intermediates still refuse. A small final
signed work or small strain energy alone is not used as the error scale.
This is a forward absolute expression bound for represented coefficients,
not a guarantee that nearly rigid high-speed states have small relative error.

The direct explicit energy-expression arithmetic allowance retains its original
gamma_(128N)*(E_before+E_after+dt*D+E_update). Add the separately upward dt times
the work budget when substituting D for u.Ku in that identity. This separates
direct energy arithmetic from strain/force work error and leaves spectral
stability, phase-volume/mass and pressure gates unchanged. The new predeclared
cancellation fixture uses a two-entry row c=(-1,1), W=0.3, velocities2^40 and
2^40+2^-10: its dyadic difference is known independently; its absolute rank-one
scale bounds the cancellation which the old signed-work budget omits. A
four-cubed anisotropic near-rigid viscous island additionally traverses the
actual assembled stress/energy gates. It is a component fixture, not an authored
velocity or supported initial condition. Neither original numerical assertion
nor the128N operation envelope was tuned to observed output.

## Deciding tests selected before execution

- Private phase/momentum fixtures: dyadic closed circulation at density
  ratios1/2/10/100/1000 and Water/Air, independent conservative donor sums,
  strict bounds, matched dual continuity and a local constant-velocity patch.
- Phase-only interface reference: frozen analytic divergence-free closed flow,
  box in its uniform interior, speed4m/s, grids8/16/32, translation time1/16 and dt≤0.02h.
  Exact per-cell translated-box intersection supplies L1 error; require a
  decreasing trend, without claiming a formal convergence order for a jump.
- Private stress fixture: independently assembled dense strain form, variable
  mu/rho, work/energy and upper spectral bound; zero mu is exact identity and
  a high-viscosity layer changes the independently expected stress. Targeted
  dt violation returns its own stability failure.
- Public coupled runs: all-Air/all-Water limits and arbitrary zero-gravity
  rest; aligned layers on X/Y/Z under gravity with actual unequal Water/Air
  viscosities, checking pressure differences (2e-6Pa) and rest velocity
  (3e-10m/s), including near walls. The wrong gravity→viscosity order must fail.
- Eight-cubed Water/Air vertical interface,20 steps at requested dt0.002:
  strict fractions, genuine nonzero material motion, decreasing liquid center
  of mass, original-reference phase balances and small derived cumulative
  volume budgets (relative≤1e-8), actual stored divergence≤1e-8. Report actual
  step shortening; this is a bounded numerical fixture, not a benchmark.
- Typed invalid input, actual pressure iteration exhaustion, memory admission,
  every cancellation stage after nonzero accepted state, unchanged accepted
  F/u/p/time/generation/report on rejection, and deterministic reconstruct/run
  comparison. Fresh-process persistence/native cold rerun remains M4/M5.

Independent review and actual tests decide M2. Ordinary mixed-interface or
Water/Air failure invokes the plan's stop/replan condition. No native exposure
or feature acceptance follows from compiling these kernels.
