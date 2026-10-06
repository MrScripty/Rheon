# External forcing in the moving viscous extrusion

This successor preserves third-component review checkpoint `35b00247` on its
own branch. The implementation roadmap's transport/force dependency and
Chapter 19 motivate the next missing capability: prescribed body-force impulse
and signed force work in the genuinely advancing fitted viscous-liquid owner.
The fixed-box `Simulation::step_with_forces` already exists; its face masses and
split force-stage work cannot be substituted for this changing liquid domain.

## Domain and admission before implementation

Use exactly the reviewed domain `x in R/Z, 0<y<H(x,t), z in R/Z`, both periods
one, with all fields independent of z and full velocity `(u,v,w)`. Retain the
two-column Powell–Sabin space, density three, dynamic viscosity `.05`, height
sum `2.25`, impermeable stationary bottom with natural tangential traction and
material cap with weak natural atmospheric-relative traction. There are no
extrusion end walls, new contact line or new surface reconstruction. Gravity
acts on this same domain; it does not silently supply a different wall model.

Accept at most sixteen caller-owned, spatially uniform `BodyForce` descriptions
for one interval. Reuse the existing `ForceUnits` distinction: acceleration is
m/s² and force density is N/m³, divided by the fixed density three. Reject any
region, unsupported count, nonfinite/subnormal conversion or arithmetic failure.
Sum the vectors in slice order with checked arithmetic; retain only three
bounded scalars for the call. These forces add no liquid volume and do not alter
material density. Localized regions require a separately derived quadrature.

The flat layer at rest under gravity was proposed as the first analytic
benchmark. An actual preliminary probe at heights `(1.125,1.125)` fails the old
`full constant-rank chart residual` gate. That refusal is retained. This
successor will not relax the chart or claim an implemented flat-tank benchmark.
Its advancing qualification uses the two existing admitted nonflat fixtures.
The flat-state/pressure-space admission gap remains separate work.

## Conservative equations and force work

For the common prescribed acceleration a, let

    f_xy(q) = Rxy^T diag(m(q)) a_xy,
    f_w(q)  = Rw^T diag(m(q)) a_z.

The semidiscrete momentum equations acquire these sources:

    Mxy zdot + Rxy^T(mdot Uxy+c_xy) + Kxy z + B^T p = f_xy,
    Mw xidot + Rw^T(mdot w+c_w) + Kw xi = f_w.

All original moving divergence, differentiated divergence and cap kinematics
remain. The algebraic pressure is freshly solved under the requested interval
force. A force can change the pressure when the interval starts without an
independently evolved pressure state. Constructor pressure describes the
unforced accepted initial velocity; it is not a forced right-limit pressure.

Choose a first-order accepted-geometry body-force load, while retaining endpoint
strain/pressure and donor momentum:

    r_xy = Rxy^T(m1 U1-m0 U0+c_F,xy) + h(Kxy1 z1+B1^T Pi-f_xy0),
    Aw xi1 = Rw^T diag(m0) w0 + h f_w0.

`Aw` and the actual integrated donor mass fluxes are unchanged. This is old-domain
force quadrature, not exact body-force integration along the geometry path.
In the bounded-real small-step limit it approximates the declared semidiscrete
body force to first order. The force-density conversion uses the same liquid
density as masses; the multiplier Pi is not a pressure impulse.

Dotting the actual finite residual with endpoint velocity gives

    E1-E0 + D_BE+D_mix+D_mu + W_pressure+W_GCL - W_force - W_residual = 0,
    W_force = h sum_i m0_i U1_i dot a.

Use separately reported xy/third work, both third shears, and their full-vector
sum. Work is signed. Energy growth supplied by the force is allowed only through
this budget; old-speed power and an isolated explicit-kick work identity are
incorrect replacements for this coupled finite rule. The 128-epsilon work
factor is unchanged; its dimensional scale includes `abs(W_force)`. This derives
the necessary force term rather than enlarging an arbitrary tolerance.

Constants are admitted in x and z, so their exact momentum impulse is
`h M_liquid a_x` and `h M_liquid a_z`; total mass is still `3.375`. Vertical
momentum includes the bottom's reaction and is not claimed conserved. The third
constant-field test must evolve `w0+h a_z`, rather than retain unforced w0.
The accepted-mass load is deliberate: GCL gives `Aw 1=Rw^T m0`. Thus constant
third acceleration and the unchanged endpoint donor scheme reproduce this
evolution exactly in real arithmetic. Substituting `h Rw^T m1 a_z` produces a
nonconstant O(h²) error when masses change. It is a distinct first-order source
rule; its failure of this constant-acceleration experiment must be recorded,
not hidden by a larger constant-field tolerance.
All direct/stable momentum, true residual, path/GCL, quadrature and bounded
Newton gates remain. Arbitrarily tiny IEEE steps remain unqualified.

## Next acceptance experiment and publication

Before native implementation, independently assemble force sources and the
combined DAE; verify signed work, exact constant-mode impulse and both shears on
the original initial and pressure-state fixtures. Retain the actual flat-chart
refusal. Exercise nonzero force in x/y/z, force reversal and acceleration versus
force-density equivalence at rho=3. Dropping `mdot U`, a source component or
`W_force` must fail the equations/work experiment.

The native successor should opt into an explicitly bounded additional scratch
reservation on the existing `CoupledDiscreteFlow`; it must not create another
accepted owner or force a post-publication velocity kick. Force conversion,
candidate nonlinear/third solves, all ledgers and reports finish before the
existing common publication barrier. Check every force/candidate cancellation
stage and actual late failures from accepted nonzero forced states, including
the pressure-state fixture, and compare successful retry bits with the baseline.
Empty forces must retain the old numerical publications and rejection policy.

Export actual velocity, pressure, geometry, mass, time and owner identity at
five common-endpoint intervals. Regenerate the forced DAE reference, recompute
errors/ratios, and show actual accepted-state renders with force metadata and
the complete force-work budget. This is the next experiment, not a passed
production claim in this proposal.

No new Lean theorem is claimed here. The existing exact force-work and finite
conservation algebra motivates the identities but does not prove this moving
IEEE implementation. Collision-mesh cut geometry, adhesion/wetting, variable
density/viscosity, capillarity, support changes and general three-dimensional
spatial variation remain missing capabilities with separate acceptance work.
