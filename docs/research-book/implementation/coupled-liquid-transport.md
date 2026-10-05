# Coherent carrier and represented-liquid transport publication

`LiquidTransportSimulation` owns the existing `Simulation` and
`LiquidVolumeState`, plus one explicitly capped accepted-pressure array. It
prepares the carrier's end velocity and gauge-fixed pressure, transports old
liquid fractions with that candidate velocity over the carrier's **actual** dt,
then publishes all three fields and both clocks after one final checkpoint.
The public accepted view is read-only; neither owner can be advanced separately
through this facade. It starts with the supplied liquid fractions and carrier
rest at t=0, including zero initial pressure. Pause rejects a complete coupled
step. This facade currently exposes no reset or arbitrary state import.

This closes the earlier volume-only transaction gap for a **transport-only**
coupled step on a fixed all-fluid Cartesian carrier box. The pressure domain
still includes every cell regardless of dry/mixed/full fraction. No liquid/air
pressure mask, interface traction, air extension, variable-density pressure
coefficient, mesh wall, viscosity, adhesion, capillarity or reconstructed surface
is supplied. Independent carrier and represented-liquid constant densities are
recorded explicitly; represented mass does not feed back into carrier momentum.
This is not a fully validated liquid simulator.

## Discrete equations and splitting

Carrier preparation uses the existing midpoint velocity advection, prescribed
forces, full ungauged pressure residual, actual stored-f32 divergence gate and
Courant gate. Closed stepping uses the original pressure workspace. Optional
prescribed-box stepping borrows the existing `BoxFluxStepWorkspace`, with
end-of-step normal boundary assignment and its explicit appearance-tracer
policy. Neither the pressure operator nor either preconditioner is changed.

For old represented amounts m_i=f_i V_i, reuse one signed face transfer

    T_e = dt A_e u_e^(candidate) f_e^(old upwind),
    m_i^(candidate) = m_i - sum_e B_ie T_e + dt s_i,
    f_i^(candidate) = m_i^(candidate) / V_i.

B is outward-positive at a face tail. Explicit inlet donors supply the outside
fraction only on inflow. Volume source s_i is in m³/s; smoke source concentration
and saturating tracer remain separate appearance data. Candidate liquid uses
the end velocity held throughout this interval; this is a declared first-order
splitting choice, not a temporal free-surface momentum integration proof.
Carrier dt may be smaller than requested. No hidden liquid substep or retry is
introduced; an independently smaller liquid Courant setting may reject the pair.

The conserved ledger remains

    V_after - V_before + V_outward - V_inward - V_source = error,
    M_after = rho_represented V_after.

Raw bounds, summed outward Courant, direct divergence, represented mass and the
existing compensated reduction budget must pass. Accepted carrier residuals
can still reject liquid near f=1; no clamp or tolerance relaxation conceals
that. Normal-or-exact-zero product/division guards are retained, with all their
conservative extreme-scale admission limitations. A small volume error does
not establish geometric interface accuracy.

## Publication and storage

The existing private candidate work is separated from its infallible owner
swap. Ordinary smoke and volume APIs still prepare and publish immediately with
their original callback sequences and arithmetic. The coupled path stages
both owners. `LiquidStepStage::Carrier(StepStage::BeforeCommit)` and
`Volume(VolumeStage::BeforeCommit)` denote owner readiness; the final
`LiquidStepStage::BeforeCommit` accepts the entire pair. No callback or fallible
numeric operation follows it. Pressure is copied from qualified scratch to the
accepted-pressure buffer and both candidate owners swap. Failed preparation,
cancellation, version/time/settings failures and paused stepping preserve the
accepted velocity/tracer/pressure/fraction bits, clocks and identities. Scratch
is disposable and may change. Retry overwrites it and reproduces fresh execution.

The facade retains 20 numerical Vec buffers: the existing 14 simulation arrays,
the existing five volume arrays, and one accepted f64 pressure array. For N
cells and F total component faces, nominal payload is

    (8 F + 56 N) + (8 F + 16 N) + 8 N = 16 F + 80 N bytes.

The facade's `SimulationConfig::memory_limit` now caps **all** these owned
capacities, including excess transferred initial Vec capacity. Allocation is
fallible and each owner receives a remaining budget. Step calls allocate no
heap buffers in the implementation; callbacks remain caller-owned behavior.
There are no cloned field snapshots or independent rollback authority. The
optional boundary workspace retains its existing separate cap and allocation
inventory, 4 F + 56 N nominal bytes. Reports expose both owner and workspace
bytes and their checked sum. Allocator overhead, RSS, caller source/force data,
PNG encoding workspace and raw export pixels are outside the owner limit.

## Numerical and proof qualification

Tests execute both original PCG identities and all signed axes against
hand-calculated pressure/volume transfer. Late liquid rejection after a second
pressure solve preserves nonzero accepted pressure. Every exercised callback
stage, including repeated slice/iteration occurrences and the final gate,
preserves both already accepted owners; retry matches uninterrupted bits.
Additional arithmetic fixtures independently target fraction division underflow,
Courant division overflow and subnormal divergence division. The original
Jacobi image/CSV fixtures are freshly replayed after this refactor.

The release `liquid_step` example transports a half-filled slab across a real
3D n×8×4 box with both pressure implementations. It exports per-step budgets,
first pressure, final pressure/fractions/face velocities and initial/first/final
PNG guidance. Independent checks recompute every-cell divergence, represented
volume, centroid and geometric overlap error, and compare fractions with a
binomial donor-cell oracle. Halving dt at fixed h may increase this stencil's
diffusion; the example records it alongside spatial refinement. The render is
the +Z fraction integral, opacity=1-exp(-8 integral f dz), with +Y upward. It is
an occupancy guidance map, not surface depth/normals or scattering. Pixel payload
has its own checked cap. Example IO and validation buffers are outside stepping
memory accounting. A second run checks deterministic output bytes.

No new Lean theorem is claimed. Historical `VolumeLedger.lean` supplies exact
finite shared-face/source/boundary algebra and **conditional** donor bounds;
`AffineProjection.lean` supplies the fixed prescribed-box exact algebra. Their
incidence, coefficients and solution hypotheses are supplied. They do not prove
this transaction, Rust indexing/assembly, IEEE arithmetic, iterative convergence,
geometry, continuum accuracy or performance. Existing reviewed proof sources,
axiom inventory, historical receipts, book chapter/PDF inputs and numerical
fixtures remain byte-preserved. See [executed evidence](../../../evidence/liquid-step/README.md).
