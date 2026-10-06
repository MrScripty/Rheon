# Conservative tangential column-profile remap prerequisite

The frozen flat-column shear source `40c19da6704237d83f3d67fe627f8a3196eea281` and evidence `aecac257660427c171fa07f0a03beecf770a9f59` remain unchanged. This additive successor implements `ColumnMomentumWorkspace`, an isolated overlap remap between two borrowed `ColumnSurfaceView`s from existing column owners. It accepts varying column heights and explicit admitted cap velocities. It publishes two caller-owned **cell-lattice tangential parcel profiles** after qualification. It does not publish carrier velocity, pressure, phase, geometry or time. The existing moving-surface viscosity refusal remains intact.

## Geometry and representation

For column c, take its existing footprint area A_c, constant density rho>0, normal spacing h and accepted height H_c=(full_c+fraction_c)h. The existing column constructor enforces bottom attachment, H_c>h/2, H_c<=(N-1/2)h, canonical full/partial/air fractions and neighbor height jumps <=h. Both borrowed views must match the workspace grid and axis, have equal volume identity and consecutive checked versions. These checks do not establish that a caller's stamps came from a particular physical interval.

Wet profile count is L_c=full_c+indicator(fraction_c>1/2). Partition the liquid into dual slabs

    D_(c,j) = [jh,(j+1)h]        for j<L_c-1,
    D_(c,L_c-1) = [(L_c-1)h,H_c].

Use **piecewise constant parcel velocities** U_(c,j) on those slabs. This interpretation has the same diagonal masses as the preceding shear prototype, m_(c,j)=rho A_c |D_(c,j)|, but differs from that prototype's piecewise linear trial interpolation for strain. A P1-to-parcel projection and a MAC-face mapping are unresolved. This remap alone does not justify composing these operators into a physical velocity step.

The two components follow ascending tangential axis order and use ordinary cell indexing. There is no normal component, normal momentum transport, lateral derivative, wall condition or traction operator. Different columns may have different profiles and heights; overlap is within each column. A_c is a **single column footprint**, not the entire lateral patch area used in the uniform shear benchmark.

## Explicit parcels and conservative algebra

Let old/new supports be [0,H^-_c] and [0,H^+_c]. Retained parcel weight from old node i to new node j is

    W_(j,i) = rho A_c |D^+_(c,j) intersect D^-_(c,i)|.

For growth, the admitted cap [H^-_c,H^+_c] has one explicit constant donor vector B_c, required from the caller. Its weight into node j is a_j=rho A_c |D^+_(c,j) intersect [H^-_c,H^+_c]|. For shrinkage, removed weight of old parcel i is r_i=rho A_c |D^-_(c,i) intersect [H^+_c,H^-_c]|, and its exported velocity is its original U_i. The implementation intersects intervals geometrically; it does not clamp phase or velocity. Shapes, dry zeros and every supplied donor value are checked.

In exact arithmetic, each target row partitions its mass, sum_i W_(j,i)+a_j=m^+_j, and each old column partitions retained plus removed mass. New velocity is the mass-weighted parcel average

    V_j = (sum_i W_(j,i) U_i + a_j B_c)/m^+_j.

An unmodified single-donor row copies its input exactly rather than introducing division error. Only the last dual slab differs from a full grid interval and it is at most 1.5h wide. Thus each new row intersects at most two old slabs. Large growth can add many new nodes, but rows above old support use only the explicit admitted parcel. The bounded implementation has linear grid cost and retains no dense transfer matrix.

Summing gives the extensive ledgers

    M^+ - M^- - M_in + M_out = 0,
    P^+ - P^- - P_in + P_out = 0.

Growth without a donor rejects. No routine infers donor momentum from phase fractions or creates a physical inlet across air. A **closed exchange** requires an external routing rule whose imported/exported mass and momentum agree. Arbitrary supplied donors instead represent explicit external exchange. The demonstration routes a shrinking column's single-velocity cap to its growing neighbor, preserving all three cap ledgers. A general shrinking cap can span multiple velocities, so one constant admitted donor would not in general preserve its kinetic energy; no general routing/closure claim is made.

For the declared lumped kinetic energy T=sum_j m_j |U_j|²/2, parcel averaging loses

    D_mix = 1/2 sum_(j,parcels k) w_(j,k) |U_k - V_j|² >= 0,
    T^+ - T^- - T_in + T_out = -D_mix.

This is numerical mixing loss. It is neither viscosity nor adhesion, and it is not automatically transferred to internal energy. Total thermodynamic energy is not implemented. Momentum-first remapping and the distinction between remapped momentum-derived kinetic energy and total energy are discussed in [Barlow et al., Sections 9 and 9.1.6, accepted primary manuscript](https://laro.lanl.gov/view/pdfCoverPage?download=true&filePid=13158174300003761&instCode=01LANL_INST). That work treats a broader compressible ALE setting; this column derivation and its qualification are specific to Rheon.

For actual stored f32 velocities Vhat_j=V_j+e_j, the report separately accounts for rounding momentum sum_j m_j e_j and work sum_j m_j e_j·(V_j+e_j/2). Thus the checked stored energy identity adds that rounding work to -D_mix. Positive donor weights yield componentwise convex bounds, checked for both proposed binary64 and stored f32 candidates.

## Acceptance, ownership and finite-scale limits

Compensated accumulation measures mass, momentum and energy. Production acceptance uses 64 binary64-epsilon magnitude budgets, finite values, row-mass partition, convex bounds and no energy increase after explicit cap exchange apart from bounded rounding work. It deliberately rejects nonzero subnormal intermediates, multiplication/division underflow to zero, nonfinite arithmetic and a nonzero proposed value converting to zero or subnormal f32. It applies no tolerance repair or clamp. Valid mathematical inputs may therefore reject outside the supported arithmetic scale, including cancellation near f32's smallest normal value.

The workspace owns exactly three cell-count binary64 vectors: staged target mass and two candidate profiles, nominal 24N bytes. Actual vector capacities are charged against its constructor cap. No remap-time heap arrays, snapshots or second accepted owner are created. Caller input/output storage and column-owner storage are separate explicit resources. `mass_scratch()` is working scratch from the last attempt and can be incomplete after rejection; it is not accepted state. All output writes occur after the final callback and all gates. Failure or cancellation leaves both supplied geometry views, both source profiles and both output buffers unchanged. Retry after dirty scratch is checked against a clean result.

## Qualification and observed behavior

Six native contract tests cover independent hand overlap weights and closed cap exchange on X/Y/Z, constant and identity profiles on both sides of center classification (including one wet node), large admitted support growth, every callback occurrence with retry equality, shape/stamp/field/density/scale/cap rejection, a late f32 subnormal candidate rejection, and an affine parcel-average refinement regression.

The native example records 13 cases / 127 remaps: three 32-remap closed exchanges, three eight-remap constant exchanges, three identity maps, and four affine refinement cases. Each case includes all before/after profiles, both geometries, explicit donors, complete native ledger reports and native initial/final grayscale PNGs. There is no physical clock in these sequences. The Y exchange starts at 6.328125 J and ends at approximately 0.945553943 J after 32 prescribed support swaps; momentum and mass close with measured rounding. Damping comes from remap mixing alone.

For affine U_0(y)=1+y, U_1=-U_0/2, old H=1 m, h=1/(n+1/4), new H=1+h/2 and admitted cap mean equal to the analytic affine mean, liquid-mass-weighted RMS errors for n=8,16,32,64 are approximately 0.00618558, 0.00226974, 0.000817905, 0.000291976 m/s. Errors are nonzero: piecewise constant old parcel means lose sub-parcel slope. Here only a bounded cap neighborhood changes, giving local O(h) error and observed weighted RMS O(h^(3/2)). This restricted manufactured sequence is not a general order theorem or accurate moving-surface simulation.

The Python gate independently intersects **all** old/new slabs, derives masses, stored f32 candidates, ledgers, mixing and rounding, checks exact carry, declared donors/units/stamps/memory, every interval and both native rasters. Twenty-two tampered fixtures are rejected under normal Python and `python -O`, including nonfinite records, balanced wrong updates, missing intervals/cases, altered geometry/donors, inflated budgets and pixels. A separate exact replay binds all frozen Jacobi and liquid examples, including the preceding shear packet. Historical book/PDF/proof/evidence inventories are preserved byte-for-byte, including the filled-box 2053-update f32 accuracy failure.

`ColumnMomentum.lean` proves twelve conditional exact-real identities: interval mass partition algebra, positive target mass, explicit-cap mass balance, weighted momentum, constant preservation, two-parcel mixing identity/nonnegativity and energy nonincrease, rounding momentum/work, closed-cap momentum, and convex bounds. Actual pinned Lean compilation and transitive namespace axiom audits pass, while injected `sorry` and extra-axiom variants compile but fail the audit. These statements do not prove the implementation's interval classification/partition, arbitrary-row assembly, foreign stamp provenance, cancellation atomicity, IEEE arithmetic, affine accuracy or convergence.

## Remaining moving-surface prerequisites

The carrier still uses full MAC face inertia and its existing ghost pressure crossing geometry. This remap uses liquid dual parcels on stepped column prisms. It does not supply a common MAC mass/divergence/gradient pair, a shared pressure/geometry accepted state, routed phase-to-momentum fluxes, normal momentum, variable density, varying-height symmetric strain or surface stress. Consequently `LiquidTransportSimulation::step_viscous` continues to refuse surface modes. Moving-surface viscosity, density laws, adhesion, capillarity and a fully validated liquid simulator remain separate goals.
