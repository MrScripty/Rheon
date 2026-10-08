# Native finite wall friction in a fixed flat slab

This implements the wall-traction slice of Chapters 19 and 24. The existing
explicit body-force API and collision triangle queries retain their own scopes.
The new operation advances liquid shear velocities, rather than changing their
visualization. It reuses the fixed flat-column mass/basis from
[column shear quadrature](column-shear-quadrature.md).

## API and physical units

`ColumnShearWorkspace::update_with_walls` consumes two `ColumnShearWall` values
in lower/upper order. Velocity is a world-space tangential vector in m/s; normal
motion is rejected. Friction beta is finite and nonnegative in Pa s/m. Zero beta
is free slip. Positive beta requires positive dynamic viscosity mu in Pa s;
positive density rho in kg/m³ weights inertia. For mu,beta > 0 the continuum
slip length is mu/beta. Exact no-slip is not represented by a large artificial
coefficient; it remains unsupported by this operation. Wetting and adhesion
are separate, unimplemented mechanisms, not synonyms for this wall friction.

The supplied column geometry stays fixed. Its cap becomes a confining wall for
this call; this is not a moving free-surface or a sealed MAC carrier update.
Lateral directions are periodic, normal velocity is zero, and tangential
velocity must be constant across each layer. The inherited varying-height and
unsupported-field refusals remain. Output is copied only after all gates.

## Equation, momentum and energy

Let A be slab area, m_j=rho A omega_j the existing liquid-only lumped mass, and
k=mu A/h the interior edge stiffness. With endpoint trace u_b and prescribed
wall velocity w_b, the force on the liquid is

    F_b = -beta_b A (u_b - w_b).
    M delta = dt (-K u + sum_b endpoint_b F_b),  v = u + delta.

The existing constant endpoint extensions make the wall trace equal to the
first/last wet-center value. This deliberately retains the existing basis; it
has a finite spatial endpoint error and is not an exact continuum wall trace
reconstruction. If one wet node remains, both walls act on that same mass.

The bulk dissipative power is P_bulk=uᵀKu; relative wall power loss is
P_wall=sum_b beta_b A |u_b-w_b|². Actuator power delivered to the liquid is
W=sum_b w_b dot F_b, which can have either sign. Exact-real algebra gives

    T(v)-T(u) + dt(P_bulk+P_wall) - dt W - deltaᵀM delta/2 = 0.
    momentum(v)-momentum(u) = dt sum_b F_b.

Rust accounts separately for actual stored-f32 rounding work and momentum.
`ColumnWallShearReport` exposes each wall force, bulk/relative-wall dissipation
and actuator work. Its nested shear report includes total dissipation and wall
impulse in the balance checks. Moving walls may add energy; this is not hidden
behind a damping parameter. Common wall/fluid translation generates no drag.

The existing 64-epsilon identity budgets are retained with explicit wall-work
and impulse magnitudes added to their scale. The explicit admission number is
dt max_j((adjacent edge weights + incident beta A)/m_j), required <=1. Active
wall velocities extend the convex bound. There are no automatic substeps or
coefficient adjustments. No new workspace vectors or per-step allocations are
introduced; the existing five normal-count arrays are reused.

`proofs/Rheon/WallFriction.lean` proves exact wall-force/work decomposition,
translation invariance, nonnegative relative dissipation, and the finite
explicit work identity under declared step and power hypotheses. It does not
prove Rust/IEEE arithmetic, stencil assembly, spatial convergence or wetting.

## Reproducible native laboratory

Build and run from the repository, selecting a fresh directory outside Git:

```sh
cargo build --release --no-default-features --example column_wall_shear
target/release/examples/column_wall_shear /tmp/rheon-native-wall-lab
```

The lab performs exactly six cases of 8192 steps over 96 seconds, with a unit
height/area slab, lower wall speed zero and upper speed 1 m/s. Baselines use
8 and 16 layers, rho=3, mu=0.15 and beta=0.3 on both walls. Four additional
16-layer cases double density, double viscosity, halve friction and double
friction independently. Four transient snapshots and all parameter units are
exported. HTML shows actual Rust profiles alongside a labeled steady reference;
it does not pretend the browser runs a liquid solver.

For symmetric slip length ell=mu/beta the continuum steady reference is
u(y)=(y+ell)/(H+2ell). The discrete fixed-basis equilibrium at node j is
u_j=(j h+ell)/((L-1)h+2ell). Distance from the latter checks closeness to the
known discrete equilibrium and combines unfinished physical relaxation,
explicit-step transient error and stored-f32 rounding; it does not isolate
integration error or establish temporal order. The difference between the two
steady references exposes the endpoint spatial error. The fixed engineering
acceptance criteria require decreasing continuum max error at 8→16 layers,
finest error <0.01 m/s, and both discrete-equilibrium errors <5e-4 m/s. The
other parameter cases are reported as finite transients, not automatically
advertised as equilibrium or continuum convergence.

Open generated `index.html`, with the adjacent CSV/HTML files kept together.
All generated lab outputs, logs, plots and compiled proof products belong
outside Git. Existing failed numerical assertions, E2 evidence and pressure
selection refusal remain preserved in their separate frozen branches. This
milestone does not qualify variable density, general 3D strain, collision-mesh
fluid topology, interface evolution, contact-angle physics or material fits.

## Primary-source scope

Qian, Wang and Sheng, *Physical Review E* 68, 016306 (2003),
[original author-hosted paper](https://sheng.people.ust.hk/wp-content/uploads/2017/08/Molecular-Scale-Contact-Line-Hydrodynamics-of-Immiscible-Flows.pdf),
describes ordinary Navier slip away from a contact line and gives slip length
as viscosity divided by the wall coefficient. Its generalized contact-line
condition additionally includes uncompensated Young stress. This operation
implements only the single-phase linear wall-traction law, not that generalized
condition or the paper's diffuse-interface model.

The original [Huh–Scriven publisher abstract](https://www.sciencedirect.com/science/article/abs/pii/0021979771901883)
(1971, DOI 10.1016/0021-9797(71)90188-3) reports the moving-contact-line stress
and dissipation singularity under adherence. The original
[Cox publisher abstract](https://www.cambridge.org/core/journals/journal-of-fluid-mechanics/article/abs/dynamics-of-the-spreading-of-liquids-on-a-solid-surface-part-1-viscous-flow/97CAB1BF3439F4B1AA429FFA37C80C42)
(1986, DOI 10.1017/S0022112086000332) treats small-capillary-number spreading
with microscopic slip or another local mechanism. Those two abstract-level
checks motivate the contact-line exclusions; no full-text formula from either
is claimed to be implemented here.
