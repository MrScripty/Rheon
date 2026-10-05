# Static passive-tracer barrier interaction

Chapters 6, 17 and 20 require component-aware transport and sampling before a
complete irregular-boundary pressure method. Chapter 20's thin-shell discussion
also distinguishes path visibility from endpoint classification: a surface can
block interpolation communication without defining an enclosed volume. This
milestone connects the reviewed static query to the existing owned simulation's
passive concentration stage, using an explicitly bounded appearance policy.

## Scope, units and stage order

`Simulation::step_with_tracer_barrier` accepts an optional borrowed static
`TriangleSurface` alongside the existing requested interval, smoke source and
external forces. The surface has world coordinates in metres; velocity is m/s,
interval duration is seconds, and the scalar is passive concentration. In the
simulation it remains in [0,1]; a source adds concentration units/second with the
original saturation. The low-level sampler/transport accepts any finite f32 donor
values. Geometry's id/version is caller-owned and reported on accepted results.

Velocity advection, legacy/external forces, fixed-box pressure projection and its
physical divergence/Courant gates precede tracer transport exactly as before.
Tracer sources follow the barrier stage. The barrier is an appearance transport
constraint: velocity samples may cross it, pressure still has no internal wall
flux or cut geometry, and cells are not classified as solid/fluid. It is not an
impermeable fluid obstacle, no-slip condition, liquid interface or moving wall.
The admitted surface may be an open two-sided sheet, without inventing an inside.
Moving intervals cannot be passed to this static API.

## Checked trace legs and donor communication

Using the accepted-old scalar and candidate-projected frozen velocity, form the
same clamped midpoint locations as the legacy kernel:

\[
m=\operatorname{boxClamp}(x-\tfrac12\Delta t\,u(x)),\qquad
 d=\operatorname{boxClamp}(x-\Delta t\,u(m)).
\]

Test the two constructed segments x→m and m→d in order. Any computed contact,
including an endpoint contact, reverts the scalar to its old arrival-cell value.
A blocked first leg stops before sampling u(m). This is a declared reversion
policy; no wall pushout, post-contact continuation or ODE accuracy at contact is
claimed. A zero-length leg has no crossing. Static ambiguity rejects transport,
even if a previously tested facet had a definite candidate. The trace is a
piecewise linear geometric model, not a certified curved characteristic.

A clear trace is insufficient: its departure interpolation stencil may still
straddle a thin sheet. For the original eight cell-lattice corners, let wᵢ be the
nonnegative trilinear weight and χᵢ the direct d→donor visibility bit. Only
positive-weight corners are queried. Coincident departure/donor positions have
no crossing segment and retain their donor. Define

\[
 W=\sum_i\chi_i w_i,\qquad
 \widehat w_i=\chi_iw_i/W,\qquad q_d=\sum_i\widehat w_iq_i.
\]

Require W>0. Blocked donors have zero coefficient; renormalization retains
partition of unity and constant fields. Dropping corners without renormalization
would lose this property. All-positive-donor visibility uses the original sum
arithmetic; the final result is clamped to the min/max of positive visible donors
in both cases. A fixture demonstrated a one-ulp violation from zero-weight
extreme values relaxing the legacy sampler's wider clamp; the new sampler's
final visible-range clamp corrects it. Original samplers are unchanged.

No visible donor returns `NoVisibleDonor`; outside-box sampler positions and
nonfinite fields/positions/timesteps have explicit errors. Within the physical
box, the original clamped sample-support extension is retained. A point on the
surface can have every noncoincident donor blocked and fail. There is no epsilon
pushout or invented solid classification to repair it.

## Publication and resource ownership

Low-level output is caller-owned scratch and may be partially written on error
or cancellation. Inputs and geometry remain immutable. The simulation uses its
existing candidate arrays and commits fields/time/generation together only after
all usual source, range, diagnostic and cancellation gates. A barrier error or
late cancellation preserves every accepted bit; a retry overwrites all scratch.
Reports count reverted traces, samples with blocked donors and blocked positive
corners and include the surface stamp. No diagnostics are published on failure.
`None` retains the original tracer kernel, callback sequence and results.

Queries poll cancellation per candidate donor and reference facet. No persistent
array or geometry copy is added; surface-capacity accounting remains separately
owned from `Simulation::allocated_bytes`. This linear cell/facet scan is not a
performance guarantee. Bounds and constant preservation do not imply scalar
mass conservation, wall work, swept-volume conservation or physical calibration.

## Exact-real and executable acceptance

`evidence/tracer-barriers/VisibleWeights.lean` defines masked weights, positive
total normalization and the accepted reversion value. Eight theorems establish
nonnegative masked/normalized coefficients, zero blocked coefficients,
partition of unity, visible-donor bounds, constant preservation and bounds for
reversion/acceptance. Original weights, visibility bits, a positive retained total
and supplied visible-donor/arrival bounds are explicit hypotheses. Geometry
predicate correctness, IEEE refinement, shared face fluxes and complete fluid
conservation remain unproved. The namespace axiom audit and actual negative
admitted/custom-axiom probes accompany the module; the historical inventory/PDF
qualification is preserved without relabeling.

Twelve Rust fixture methods cover all three sheet orientations, signed/3D donor
ranges, constants, zero-weight extrema, first/second trace contacts, stencil
leakage despite a clear trace, unobstructed legacy agreement, no-donor/ambiguity,
input admission, low-level partial scratch and actual simulation cancellation,
error/retry, optional-path equivalence, force/pressure/memory agreement and both
pressure implementations. The runnable example compares eight accepted steps
per implementation with/without the static tracer sheet:

```sh
cargo run --locked --release --no-default-features --example tracer_barrier
```

This addendum is outside the frozen book PDF inputs. Irregular-boundary velocity
sampling and pressure/topology assembly need their own compatible flux/adjoint
contracts before this can be presented as a fluid collision boundary.
