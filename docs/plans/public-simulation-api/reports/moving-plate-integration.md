# Moving-plate API integration milestone

Reviewed source: [PR #52](https://github.com/MrScripty/Rheon/pull/52), commit
`fd281ebf51d63b5406f86a74cdc67873ea9a0174`, stacked on PR #45. The authority is its
[consumer contract](https://github.com/MrScripty/Rheon/blob/fd281ebf51d63b5406f86a74cdc67873ea9a0174/docs/plans/moving-plate-coupling/reports/contract.md)
and `experiments/plate_coupling.rs`, including `Plate`, `Workspace`, `Report`,
`Stage`, `Error` and the reused outward interval backend. Reviewed read-only;
this dependency has not been merged into PR #53 or main.

## Decision and scope

Track M5 as a separate integration milestone. The existing production sessions own
all-fluid transport, SPH particles, or qualified sealed-box pressure. None owns the
layered tangential velocities, immutable shear closure and two finite/prescribed
plate modes of this experiment. The source deliberately remains outside `src/lib.rs`
as a path-import experiment and has no time/generation owner. Adding a capability
flag or borrowing the word "pressure" cannot establish compatible executable behavior.
M1–M3 therefore do not advertise moving-plate support.

API owner designs the future adapter; physics owner retains numerical and interval
acceptance. This is an explicit scope separation, not a dependency on parent-designed
contracts. Admit M5 as its own coherent implementation/review slice when exposing the
experimental kernel through a supported library boundary is selected. No numerical
promotion or general-geometry promise follows merely from a successful wrapper.

## Proposed adapter obligations (not implemented API)

A separately named `PlateSession` would borrow an immutable `StaticObstacleShear`
for its lifetime, own the experiment workspace and accepted fluid/wall speed arrays,
and reuse `SessionIdentity` for run/source identity. A borrow avoids self-referential
geometry/workspace ownership. Construction fixes the tangent axis and two `Plate`
modes; a dynamic plate requires a positive mass for the represented periodic section.
Initial fluid layer and wall speeds belong to the session after construction.

Per-step controls would contain positive dt in seconds and prescribed wall speeds in
m/s only for prescribed modes. Dynamic wall speeds evolve from accepted state, so a
prescribed-speed override for a dynamic mode must be rejected. Configuration/mass,
tangent/normal axis and geometry changes require a new session. The existing kernel
must retain sole responsibility for admitted shapes, finite-normal-or-zero scalar
validation, solve/interval gates and cancellation outcomes.

Preflight finite advancing next time and checked next generation **before** invoking
the transactional kernel. It already commits caller speed arrays and its last report
together after its final gates. The adapter's remaining publication bookkeeping must
be infallible, so an overflowing clock cannot leave speeds committed without their
identity. Rejection/cancellation must preserve arrays, report, time, generation and
accepted controls. Deadline cancellation remains cooperative at existing `Stage`
checkpoints; no hard timing or callback/kernel preemption is promised.

A borrowed accepted snapshot would expose fixed axes/modes, layer fluid speeds, two
wall speeds, accepted time/generation/controls and the existing `Report`. Preserve
`Interval` enclosures and units without reducing them to a falsely exact scalar:
fluid boundary impulses are N s, energies/work/dissipation are J, speeds m/s, mass kg.
Plate impulse has the opposite sign to the corresponding impulse onto fluid.
Prescribed actuator work is external; dynamic exchange is internal. A borrowed view
is immutable for its lifetime; it does not promise cold replay or a numerical archive.

The combined memory promise must include actual session-owned speed-array capacities
and the kernel's combined payload (borrowed shear storage plus workspace arrays).
Subtract session arrays before passing the remainder to the kernel, and count the
borrowed shear once. Geometry, caller-retained copies, allocator metadata, stack and
process RSS remain outside this explicit payload cap. No unbounded history is added.

## Physical and platform capability boundary

Only the admitted cross-section-spanning lower box and connected fluid extrusion,
including the represented partial first layer, are supported. Tangent differs from
normal. Tangential directions are periodic/invariant; sliding leaves normal geometry
unchanged. Lower-wall closure is the existing no-slip/Navier shear closure; upper
wall is no-slip. This is tangential planar viscous coupling, not arbitrary CAD motion.

Normal pressure, loads/torque, rotation, contact, wall position, finite edges/end
walls, fluid advection, free interface, variable material, GPU and engineering
accuracy are unavailable. Do not add zero-valued pressure/load fields that look like
computed results. Requests outside the selected model need typed unsupported/invalid
outcomes. The interval backend assumes Linux x86_64 and default round-to-nearest
binary64 controls; no other platform claim is inherited from the host API.

## Acceptance gates and next implementation slice

Allowed future writes: a plate-session adapter and additive exports, adapter tests
and example, and this plan/contract. Moving the kernel or selecting an experimental
library module needs an explicit coordinated boundary decision with the physics owner;
reuse its exact implementation and interval types, never fork the algorithm.

Required automated contract/integration evidence on the admitted Linux x86_64 host:

1. Real prescribed/dynamic/mixed calls match the existing kernel's speeds and report.
2. Source/run IDs, stage, controls, time and generation correspond to that exact
   accepted interval; rejection at every callback preserves their complete tuple.
3. Invalid/overflowing clock and quota/budget requests reject before numerical mutation.
   Changed prescribed controls do not overwrite or relabel earlier accepted data.
4. Snapshots retain original impulse signs and interval endpoints; unavailable physical
   quantities remain unavailable. Direct unsupported-axis/shape/platform cases reject.
5. Existing `plate_coupling_contract` and independent dense-rational checker still pass
   using clean-source/native-binary identity. Wrapper tests do not replace that physics
   evidence, qualify other platforms or establish real-time behavior.

CAD remains the renderer/VTK owner. Consume these raw layered values only through an
explicit future adapter; do not portray them as a general 3D velocity field or attach
uncomputed normal pressure. M5 remains Deferred until this separate exposure slice is
admitted. The next implementation action is the library-boundary decision plus the
bounded owner above, not a geometry/physics generalization.
