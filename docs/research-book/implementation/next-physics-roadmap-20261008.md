# Physics capability reconciliation, 2026-10-08

Inspected baseline: `ac8411670eddab3f4caf0fd73bfb785044b3925d`, tree
`021c92c5fbf65a7a178d5ef7b7992b7582cf884a`. This is a new design checkpoint
on `research/liquid-solid-boundary-next-design-20261008`, not a new native
qualification, solver implementation or publication approval. The older
[requirements map](requirements-roadmap.md) freezes an earlier PR24 state.

## Existing capability and the remaining requirements

| Requirement | Implemented bounded capability at the inspected baseline | Remaining work and evidence boundary |
| --- | --- | --- |
| Collision meshes | Immutable stamped triangle queries; single retained closed axis-aligned box fluid geometry; prescribed translation queries; isolated sphere/triangle impacts, Coulomb intervals and stationary single-facet sphere support. | Arbitrary embedded fluid topology, moving fluid boundaries, simultaneous rigid contact, settling and multiple supports remain absent. A hit normal is not fluid topology. Single-facet support is not a general resting-contact solver. |
| Forces | Explicit body acceleration/force density; fixed-slab forcing; retained P1 mesh pressure/traction reduction into force, torque and nodal loads; bounded spherical rigid impulse/motion. | Mesh traction is supplied by the caller. No matched map currently derives the solid wrench from the aligned fluid pressure/strain field. A net wrench alone does not determine P1 facet traction. |
| Liquid viscosity | Filled sealed MAC box; fixed reduced slab Navier/no-slip; owner-derived obstacle shear; reconstructed padded aligned-box symmetric-strain rows/actions. An opt-in aligned Stokes transaction already exists outside the production library. | The physical boundary-reaction map is missing. Arbitrary embedded/free-interface stress, spatially varying or nonlinear viscosity and a general advancing material solver remain separate contracts. The experiment is not a production completion claim. |
| Stickiness | Fixed-slab finite Navier coefficient describes mechanical slip friction. Dry Coulomb contact is separately implemented. | Adhesion/wetting needs wall and interface energies, contact-angle geometry, balanced capillarity and contact-line regularization. Increasing viscosity or slip friction supplies none of these. Analytic equilibrium cap examples do not implement relaxation. |
| Density | Constant carrier and represented density, stored inertia, explicit `mu/rho` scaling; bounded surface modes require represented/carrier equality. | Advected variable-density/two-phase inertia, pressure coefficients, momentum transport and material-interface qualification are absent. A new constant-material wrench does not close this requirement. |

The fixed cell-aligned free-surface/column transport modes also exist; they do
not supply the missing general free-interface stress or contact line. See
[liquid composition](coupled-liquid-transport.md), [cell-aligned surface](cell-aligned-free-surface.md)
and [column bridge](flat-column-mac-bridge.md) for their separate admissions.

The baseline stationary-support candidate has fresh source-bound root and
independent qualification receipts, including actual Rust/Fraction comparisons
and freshly compiled conditional Lean proofs. Its final durable source/evidence
capsule is Library `libfile_45324061d0f881919d778058198a49a0`, SHA-256
`60d9249e1717d2641ac778b3441dd3143fb99cb79175777b06ad55e1335eb8da`;
handoff Library `libfile_ed17293092948191885ac85b2c80ce1f`. Those receipts qualify
that frozen candidate and its documented checks. Inspection of the separate
Stokes experiment here does not freshly requalify it, nor do those receipts
qualify any proposed boundary lift. No erased unpublished source or old counts
are inherited as evidence for new work.

## Recommended next substantive feature

Derive and independently qualify an **immutable viscous solid-boundary wrench**
from the existing aligned symmetric-strain field. The domain remains one padded,
exactly grid-aligned stationary internal box, stationary no-slip solid, sealed
free-slip outer walls, fully wet fluid and constant positive density/viscosity.
This connects actual liquid stress work to collision-owned force and torque,
rather than adding another one-sphere special case. It reuses the existing
active face space, masses, row weights and geometry owner. The
[proposed contract](viscous-boundary-reaction-design.md) contains the derivation
and qualification gates; it is not an implemented operator.

The first result would be a stamped six-component wrench and matched finite
work diagnostics. Physical wall motion, time advancement, pressure coupling,
new contact solver and conversion into arbitrary triangle traction are outside
that result. Virtual twists identify forces and torques without prescribing a
time interval in which a rotating box stays grid-aligned.

## Pressure decision before a combined bridge

An independently derived counterexample blocks silently extending the current
pressure solve into a faithful solid-pressure load. For the unit center cube
`[1,2]^3` in the unit `0..3` grid, adjacent fluid cell centers lie at `z=1/2,5/2`.
Under `p=b+k*z`, adjacent-cell wall pressure gives `Fz=-2k`; the physical closed
box integral is `Fz=-k`. Constant-gauge cancellation and a matched transpose
work identity both pass. They do not detect this force bias.

| Owner choice | Consequence |
| --- | --- |
| Viscous wrench first; pressure held (recommended immediate scope) | Advances liquid-to-solid force/work without selecting a new pressure boundary model. |
| Adjacent-cell constant pressure trace | Preserves local geometric wall flux and admits a small discrete adjoint, but explicitly accepts affine-force bias `h/L` on a box of length `L` with normal cell width `h`. No exact buoyancy claim. |
| Affine-exact pressure trace with matched transpose, or expanded complementary solid/trace space | Requires a new boundary closure/support/metric decision. Affine interpolation changes the virtual flux distribution; the current one-cell padding does not guarantee a second outward normal sample. A new force interpolation cannot be combined with an unchanged wall-flux transpose by assertion. |

The owner must choose the pressure accuracy and closure policy before combined
pressure implementation. This decision need not select a whole new solver,
but any eventual advancing body/fluid system needs a separately reviewed
kinetic metric, boundary work and coupled acceptance contract.

## Sequence after the bounded wrench

1. Resolve pressure trace, flux closure and hydrostatic force/torque accuracy;
   qualify the combined immutable reaction independently.
2. Derive finite Navier slip against the matched bulk boundary work if desired;
   keep mechanical slip distinct from adhesion.
3. Choose either a sharp-interface contact-angle/slip law or a diffuse-interface
   wall-energy/mobility model before implementing wetting. The choice requires
   intended physical material and contact-line behavior; no arbitrary sticky
   force is selected here.
4. Derive material transport and matching variable-density/viscosity pressure,
   inertia and interphase momentum. General moving mesh topology and multiple
   rigid contacts remain separate major contracts.

This sequence does not resume PR25 case41/geometry-band campaigns, enlarge
resource limits or modify PR36. Publication, main merge, deployment and paid CI
remain held; the parent schedules CodeRabbit after a separately approved
publication candidate.

## Research and independent review

[Batty, Bertails and Bridson (2007)](https://www.cs.ubc.ca/~rbridson/docs/batty-siggraph2007-variationalcoupling.pdf),
sections 2–3, derive coupled pressure/rigid-body work using matched generalized
force and velocity maps. Their solid-volume metric cannot simply be imported
into Rheon's reduced active face space.

[Batty and Bridson (2008)](https://www.cs.ubc.ca/~rbridson/docs/batty-sca08-viscosity.pdf),
sections 4–5, motivate symmetric deformation, compatible stress samples and
volume-weighted dissipation. Rheon's reflected corner reconstruction and the
proposed lift are new derived choices.

[Qian, Wang and Sheng (2003)](https://sheng.people.ust.hk/wp-content/uploads/2017/08/Molecular-Scale-Contact-Line-Hydrodynamics-of-Immiscible-Flows.pdf),
section IV, provides a specific wall-energy/contact-line formulation beyond
ordinary viscous slip. It is research context, not a selected Rheon law.

The independent design review freezes the inspected baseline and records five
exact design counterexamples, source hashes, alternatives and proof/native
gates. It withholds combined implementation approval pending pressure closure.
Those calculations are research evidence, not comparisons with a nonexistent
new Rust operator. The original PDFs were inspected through the browser;
reviewer shell archival requests received proxy `403 Forbidden` and were not
bypassed. The source archives therefore contain citations and derived notes,
not those PDF bytes.
