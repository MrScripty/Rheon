# 14 Guidance as a geometric measurement

Guidance rendering is a measurement layer with a camera model, clipping, sampling and units. It must preserve the spatial information actually simulated. A downstream model may change appearance; Rheon should export reproducible geometric evidence.

## Camera coordinates

For camera coordinates \((X,Y,Z)\) with positive Z forward,

\[
u=f_xX/Z+c_x,\qquad v=f_yY/Z+c_y.
\]

Axial depth \(Z\) differs from radial range \(\sqrt{X^2+Y^2+Z^2}\) off axis. Name the representation, handedness, image origin, clipping and pixel-center convention. Mirroring or shifts may be camera-contract defects rather than fluid errors.

Linearized projection gives

\[
\delta u\approx(f_x/Z)\delta X-(f_xX/Z^2)\delta Z.
\]

The same world-space error moves more pixels near the camera, and depth errors shift off-axis silhouettes. Evaluate actual camera distributions, not just a world-space norm.

## Surface and volume outputs

Liquid can export silhouette, depth and normals in a declared frame. Smoke may integrate extinction. Piecewise constant extinction along a ray gives

\[
T=\exp\left(-\sum_i\sigma_i\Delta s_i\right),
\qquad \alpha=1-T.
\]

Increasing samples without adjusting segment length changes opacity incorrectly. Concentration-to-extinction is a rendering model, not conservation. A diffuse volume has no unique surface; first-hit depth needs a named threshold or accumulated-opacity rule.

Graphics depth buffers may be nonlinear in distance. Exporting raw values as metre depth is incorrect without conversion. Validate planes at known distances and off-axis spheres. Include validity masks rather than ambiguous zero depth.

## Metrics and integration

Silhouette IoU is \(|S\cap T|/|S\cup T|\), with an explicit both-empty case. It can hide a displaced thin filament in a large body. Add boundary-distance percentiles and maximum displacement. Compare depth on mutually valid pixels and report missing surfaces separately. Normals use angular error.

Rest scenes under a fixed camera expose flicker from grid crossings, mesh ambiguity and thresholds. Filtering can reduce flicker while shifting edges, which must remain in the error budget.

Export camera calibration, world scale, dimensions, channel semantics, masks, physical time, state ID and renderer version. Resizing requires updated intrinsics or a stated resampling transform. A depth image without its mapping is ambiguous even when attractive.

The first renderer should be simple and deterministic. Its acceptance is geometric agreement with known primitives and state, not photorealism. Integrated diffusion experiments need separately pinned models and seeds.

## Read the field before interpreting the picture

The sticky example beside this chapter is the existing **24³ carrier-flow
lesson**: it displays a computed tracer and instantaneous velocity from that
bounded grid model. It is not the native material/SPH preview, a CAD flow
solution, or a pressure-load measurement. The additions below explain how to
ask better questions of each model; they add no new live material or CAD solver.

Start with a field's meaning. A passive tracer marks where transported material
has been represented; opacity is a display rule applied to that marker. Neither
is automatically mass density. A velocity arrow represents a vector, while its
length may be fixed or proportional to speed. An instantaneous streamline is
tangent to one frozen velocity field; it does not record a parcel's past motion.
The [engineering-visualization guide](../implementation/cad-visualization-contracts.md)
develops sampling, sections, pressure, loads and export from these distinctions.

Rheon has two separate data paths. The native material preview exposes particle
position, velocity, mass, temperature and material ID. Its per-particle density
and pressure are private, and the reported maximum pressure is evaluated before
the position and wall-temperature update; accepted speed describes a later
stage. A pressure maximum cannot become a synchronized pressure slice merely by
coloring accepted particle positions. The carrier path instead has staggered
face velocities and a pressure correction. Its correction is not particle
pressure, and its aligned face flux is not SPH mass crossing. See the
[material contract](../implementation/material-preview-contracts.md) and
[source-anchored inventory](../implementation/cad-visualization-contracts.md#rheons-present-starting-point-two-distinct-data-paths).

For every probe, ask: **what quantity, where, in which units, at what physical
time and evaluation stage, and from which data?** Mark it native, derived,
prescribed or unavailable. Show the sampling rule, validity and pressure
reference when applicable. An unavailable value must remain distinguishable from
zero. A color range belongs to the measurement: lock it across compared runs,
and show numerical values alongside color.

## Predict a measurement before adding a view

Consider a unit box with constant density. Prescribe three fields: rest,
uniform velocity \((1,0,0)\,\mathrm{m/s}\), and shear
\((y\,\mathrm{s}^{-1},0,0)\) with \(y\) measured in metres. A density-only image
is identical in all three cases. Through the unit face normal to positive x,

\[
Q=\int_A\mathbf u\cdot\mathbf n\,dA
\]

is respectively 0, 1 and 0.5 m³/s. Reversing the normal reverses the signs.
These are prescribed analytic fields, not three native simulations satisfying
the same box-wall conditions. This small example teaches why a slice, arrows
and an oriented section answer questions that opacity cannot.

The seven [lesson specifications](../companion/cad-visualization-lessons.json)
are a proposed progression, **not seven implemented browser controls**. The
[analytic companion](../companion/cad_visualization_fixtures.py) checks selected
mathematical reference calculations; it does not call Rheon or validate its
physical models. Detailed controls, assumptions and failure probes are in
[the worked lessons](../implementation/cad-visualization-contracts.md#seven-interactive-worked-lessons).

| Question | Prediction to check | Limit to keep visible |
|---|---|---|
| Can constant density hide motion? | Unit-section flux is 0, 1 or 0.5 m³/s. | A prescribed field need not meet the solver's walls. |
| What should a planar channel measure? | The no-slip laminar profile has \(U_{\max}/U_{\mathrm{mean}}=1.5\). | Infinite-width or periodic-span assumptions exclude influential sidewalls; the SPH box preview is not this test. |
| Does vorticity identify a vortex? | Simple shear has vorticity but zero rotation-minus-strain Q; rigid rotation differs. | Q conventions differ when divergence is nonzero. |
| Does a streamline show particle history? | For \(\mathbf u=(1,t,0)\) in the stated SI scales at 2 s, streamline, pathline and streakline have different shapes. | Quantitative time snapshots and stable particle IDs cannot be recovered from preview bytes. |
| Can color hide a tenfold change? | At \(x=0.5\) m, the two prescribed speeds are 0.5 and 5 m/s. | Per-frame autoscaling can make both pictures look alike. |
| Does refinement establish accuracy? | The manufactured quantity \(1+2h^2\) has observed order two and extrapolate one. | This is not an error estimate for Rheon or evidence of physical validation. |
| Why does a particle cloud look less dense at its edge? | The specified poly6 lattice gives about 1009.775 kg/m³ at its center and 817.000 kg/m³ at an outer-face particle. | Finite support and reconstruction affect the image; normalizing a display attribute does not repair density physics. |

## Keep geometry and engineering claims separate

A CAD design surface, the boundary used by the solver and a display mesh may
all differ. A smooth surface colored from nearby fluid samples is not yet a
conservative force integral. Pressure loads require a declared pressure
reference, consistently oriented numerical boundary and supported stress
model. Uniform pressure cancels on a closed surface with consistent normals;
it generally does not cancel on an open patch. Wall shear additionally requires
a qualified viscous wall treatment and wall gradient. Box-contact impulses are
a different diagnostic, not a wall-shear map. These distinctions follow the
separate [pressure, viscous and moment conventions](https://doc.openfoam.com/2306/tools/post-processing/function-objects/forces/forces/)
used in established analysis tools.

The rotating emitter is a prescribed source. Its pressure-head speed, midpoint
orientation and fixed-volume parcel credits do not provide a nozzle collision
solid or simulated nozzle walls. The source definitions and accepted publication
are anchored in [emitter geometry](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/emitter.rs#L34),
[profile](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/emitter.rs#L78),
[particle admission](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L387)
and [transactional publication](https://github.com/MrScripty/Rheon/blob/2209012bc5f902693d0f695ec1c2d854126af317/src/particle_fluid.rs#L698).

The first useful native extension is an unquantized, stage-labelled particle
inspector and export, followed by reconstruction with visible support and
invalid regions. Add histories only after retaining quantitative state, and
loads only after establishing their geometry and stress contract. An analytic
check verifies a calculation; a relevant refinement study estimates numerical
error; matched experiments address physical validation. The
[source bibliography](../sources/cad-visualization-bibliography.md) and
[implementation order](../sources/cad-visualization-implementation-plan.md)
preserve the evidence and prerequisites behind that sequence.

## Draft checkpoint: inspect a retained obstacle projection

Unmerged [PR50](https://github.com/MrScripty/Rheon/pull/50), inspected at
`3f49a158a0f1bec93f47edf6cc42ad86242e4aa5`, adds a native
[obstacle projection inspector](https://github.com/MrScripty/Rheon/blob/3f49a158a0f1bec93f47edf6cc42ad86242e4aa5/src/bin/desktop/engineering.rs).
Its 12×8×6 grid has 0.125 m spacing and a fixed aligned obstacle. One accepted
pressure projection over 0.05 s supplies a retained result; this is not an
advancing wake or a viscous wall-load computation. Its
[scope and validation record](https://github.com/MrScripty/Rheon/blob/3f49a158a0f1bec93f47edf6cc42ad86242e4aa5/docs/implementation/cad-obstacle-lab.md)
records the fixture, gauge, rendering choices and remaining integration work.

A practical inspection sequence is to probe a fluid cell, rotate the camera,
and revisit the same cell: presentation changes must not change its value.
Then move the slice. Scalar colors retain a full-state valid-cell range, so a
slice change cannot silently rescale a small feature into a large one. Pressure
means gauge-fixed correction in Pa; speed is reconstructed from adjacent face
velocities, not a native face flux. Solid cells report unavailable, not zero.
Velocity arrows encode velocity times 0.1 s, with physical aspect ratio and
camera foreshortening; they are neither normalized directions nor trajectories.

The lab's snapshot and camera are separate from live appearance history.
Scrubbing an SPH appearance frame cannot retime the retained projection or
recover quantitative particle fields. The draft uses CPU-drawn slices and wire
overlays, and requires full-fluid or solid cells; it supplies no arbitrary CAD
import, cut-cell reconstruction, calibrated pressure/viscous load, VTK exchange
or GPU-performance result. Its reported local PR47 combination remains a tested
draft integration, not a merge or shared accepted API. The existing browser
chapter still presents its separate bounded carrier-flow lesson.
