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

Tests include both signs of U, rotated wall frames, uniform wall/fluid translation, curved-wall interpolation and zero/coefficient limits. A stationary relative state must dissipate zero. Refinement should approach the analytic Couette profile and traction. The finite proof does not establish those geometric limits, and this edition supplies only the affine oracle rather than a production moving-wall implementation.

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
