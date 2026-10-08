# 24 Slip and wall friction

Impermeability sets relative normal velocity. A tangential boundary law must be selected separately. No-slip sets tangential fluid velocity equal to the wall. Free-slip sets tangential traction to zero. Navier slip relates traction and relative tangential speed. None of these selects a wetting angle.

## Dissipative Navier law

With a normal/sign convention chosen so traction on the fluid opposes slip, let \(t_t=-\beta(u-u_s)_t\), where \(\beta\geq0\) has units Pa s/m. Relative power is

\[
t_t\cdot(u-u_s)_t=-\beta|(u-u_s)_t|^2\leq0.
\]

`slip_power_nonpositive` proves the scalar component version, and component sums extend it to a tangent plane. The inequality concerns relative dissipation. A moving wall can still increase fluid kinetic energy through external work. It does not prove a spatially assembled slip discretization has this sign.

For positive viscosity, slip length \(\ell_s=\mu/\beta\). The limiting cases must be represented explicitly: \(\ell_s=0\) is no-slip, while zero \(\beta\) is free-slip and corresponds to unbounded slip length. Avoid dividing by zero or approximating both limits with the same finite parameter. When \(\mu=0\), this relation cannot define the same viscous boundary model.

## Couette reference

A lower stationary wall at y=0 and upper wall moving at U at y=H give steady affine speed

\[
u(y)=U\frac{y+\ell_s}{H+\ell_s}
\]

when only the lower wall has Navier slip and the upper wall has no-slip. Then \(u(0)=\ell_s u'(0)\), \(u(H)=U\), and shear stress is \(\mu U/(H+\ell_s)\). Increasing slip raises lower-wall speed and reduces shear at fixed U. It does not change equilibrium contact angle.

The reference generator verifies both boundary equations and exports profiles for slip lengths 0, 0.1 and 0.5 m in a unit-height slab. The browser uses actual profile samples for its arrows and shows the shear and dissipated power. These large illustrative slip lengths clarify the mathematics; they are not fitted values for water on a named material.

## Friction is a model choice

Dry Coulomb friction bounds a tangential force using normal contact force. Applying it to a liquid-solid wall is a separate constitutive assumption, not the default Newtonian no-slip boundary law. A penalty damping term likewise has its own units and finite-step dissipation. Label it as a numerical/constitutive choice if used. A single “stickiness” parameter cannot consistently identify all of these mechanisms.

Cox's primary contact-line analysis explains why a microscopic mobility mechanism matters [E9]. A macroscopic slip length selected for a resolved Couette profile need not serve as the microscopic regularization of a contact-line model. Using it for both requires an explicit scale argument and validation.

## Proposed state and tests

`TangentialLaw` distinguishes no-slip, free-slip and finite Navier coefficient. `WettingLaw` owns angle/energies separately. Boundary elimination must include known moving-wall terms; diagnostics retain relative dissipation and external wall power independently.

The broader test plan includes both signs of U, rotated wall frames, uniform wall/fluid translation, curved-wall interpolation and zero/coefficient limits. A stationary relative state must dissipate zero. Refinement should approach the analytic Couette profile and traction. The finite proof does not establish those geometric limits; the native operations below qualify only the fixed flat-slab slice.

## Both walls slipping: a distinct oracle

The native [flat-slab wall-friction update](../implementation/column-wall-friction.md)
now advances actual Rust velocities with finite nonnegative wall coefficients,
positive density and viscosity, separate relative dissipation and actuator work.
Its reproducible `column_wall_shear` laboratory compares the existing finite
endpoint basis with the symmetric two-wall oracle below. This bounded model
does not add wetting, contact lines or arbitrary collision-mesh fluid boundaries.

The browser lab uses lower-wall slip only. With equal finite slip lengths ℓ at both stationary-lower/moving-upper walls, the separate exact profile is

\[
u(y)=U\frac{y+\ell}{H+2\ell},\qquad
\tau=\mu\frac{U}{H+2\ell}.
\]

The two relative wall speeds have magnitude Uℓ/(H+2ℓ), and actuator power partitions as

\[
\tau U=\mu\left(\frac{U}{H+2\ell}\right)^2H
+2\frac{\mu}{\ell}\left(\frac{U\ell}{H+2\ell}\right)^2.
\]

Handle ℓ=0 as a no-slip constraint instead of dividing by zero. This formula is an independent analytical fixture; it must not replace the one-wall denominator in the existing browser data. The original comparison figure below uses symmetric two-wall slip and displays bulk/wall power separately.

![Exact fully developed Couette profiles with symmetric Navier slip; wall input partitions into bulk and wall dissipation.](figures/expansion/couette-slip-and-power.svg)

## Exact no-slip in the retained flat-slab basis

The [connected native progression](../implementation/native-wall-force-sequence.md)
starts with both walls at finite Navier friction, adds compatible exact no-slip,
then adds stationary-wall body-force-driven Poiseuille startup. These are three
distinct reference problems. The original one-wall 3D analytic lab remains an
equilibrium record, while native case/time controls select saved Rust profiles;
none is a live general fluid solver.

The [compatible endpoint no-slip operation](../implementation/column-no-slip.md)
now distinguishes an exact prescribed tangential constraint from finite Navier
friction. It requires the incoming trace to match the fixed wall speed. Rather
than introducing a large beta, it eliminates the endpoint increment and reports
the reaction impulse J=-dt F_unconstrained. Delivered wall work is w dot J and
can have either sign. Relative reaction work J dot (u-w) vanishes, while bulk
viscosity continues to dissipate energy. Wetting and adhesion remain separate.

The native `column_no_slip` lab provides case selection and five actual stored
snapshots for two-wall no-slip at 8/16 layers and lower-Navier/upper-no-slip at
16 layers. Its mixed continuum reference is the original one-wall oracle above.
It includes compatible initial energy, reaction impulses, signed actuator work,
bulk and finite-slip dissipation, and explicit/f32 rounding corrections.

The existing basis extends wet-center endpoint values constantly to each wall.
Thus the wall trace is constrained exactly, but comparison at wet centers still
has spatial endpoint error. The discrete equilibrium for two no-slip walls is
u_j=U j/(L-1); for lower slip it is
u_j=U(jh+ell)/((L-1)h+ell). Distance to these equilibria also includes remaining
physical relaxation and numerical transient/rounding effects. A decreasing
endpoint deviation under the bounded refinement is not temporal order.

Four new Lean statements prove compatible trace preservation, zero relative
reaction work, momentum balance and the constrained finite work identity under
explicit step/assembled-power assumptions. They do not certify stencil assembly,
Rust floating point or general wall geometry. The native operation refuses
prescribed speeds incompatible with the incoming trace and two no-slip walls
sharing one wet node, where separate reactions would be nonunique. It retains
no boundary-speed history and supplies no time-varying wall-speed evolution model.
