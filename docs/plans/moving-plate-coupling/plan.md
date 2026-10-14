# Moving-plate shear and finite-mass coupling

Status: Accepted for the finite M2 local acceptance claims. Operation: verify,
authorized by the continuing broader-physics request. Current phase: complete.
Next slice: none. Hosted evidence remains separately identified by exact head. Owner/integration owner: broader-physics worker; public
API integration remains with API task 01a134aa-6179-75ab-b738-c1db75dbf00d (PR48,
initial f06a127). Parent coordinates shared contracts.

## Objective and acceptance

Advance the stationary milestone with observable fluid response to a moving
solid boundary and equal/opposite finite-mass solid feedback in a bounded
reduced mode. Reuse StaticObstacleShear's immutable owner, actual layer volumes,
centers and conductances. Implement prescribed tangential plates (Couette) and
unforced dynamic scalar plate masses in one backward-Euler transaction.

Acceptance status: local integration/contract gates passed; see ledger. Required automated integration/contract evidence on
Linux x86_64 and Rust 1.92.0: exact one-layer prescribed and two-dynamic-plate
controls; actual per-node momentum equations; combined energy/work identity;
equal/opposite boundary impulse; zero-slip-difference rest; free-slip decoupling;
nonuniform cut-layer and all-axis mapping; continuum linear Couette stationary
profile; independent rational dense solve over exported real native records;
rejection/cancellation preserving caller velocities and accepted report. These
are simulated physical models, not experimental calibration. Supporting gates:
formatting and strict Clippy, focused inherited geometry/shear regressions, and
independent review. Hosted checks must identify their exact commit.

## Decisions, scope and derivation

The fluid normal geometry is fixed. One common tangent axis differs from the
normal. Both tangent directions are invariant/periodic; plates translate only
tangentially, so the represented normal volumes/openings do not change. There
are no edges/contacts/torque/normal motion or independent plate position/clock.
A dynamic plate's positive mass is for this modeled periodic cross-section.
Prescribed plates have externally imposed speeds and no invented finite inertia.

At every fluid layer use m=rho*V and the stored geometric conductances k.
Each fluid-fluid edge contributes equal/opposite k*(v_neighbor-v). Boundary
fluid impulse is J=dt*k*(w_new-v_new). A dynamic plate satisfies M*(w_new-w_old)
=-J; a prescribed plate supplies actuator impulse/work, J and J*w respectively.
The extended chain M+dt*K is symmetric positive definite after eliminating
prescribed endpoints. Solve it with bounded preallocated Thomas storage, then
check actual stored output independently of the solve:

    E_new-E_old + E_increment + dt*sum_edges k*(relative_new)^2 = W_prescribed
    P_new-P_old = sum_prescribed J

Outward intervals enclose each momentum equation and ledger; relative acceptance
allowances have no unit-sized floor. Failure/uncertainty never publishes. The
existing outward arithmetic premise remains Linux x86_64 default scalar binary64;
no new theorem of machine arithmetic or continuum convergence is claimed.

Alternatives: directly merge the existing sphere/contact/rigid stack (does not
supply moving fluid wall work); arbitrary embedded normal motion (requires new
swept-volume, topology and biased pressure-load decisions); chosen reduced
plate model (supported by existing Newtonian shear closure, reversible and
independently testable). No user material/contact-law choice is guessed.

## Milestone, write set and gates

M2 (Accepted): experiments/plate_coupling.rs, examples/plate_coupling.rs,
tests/plate_coupling_contract.rs, tools/check_plate_coupling.py,
.github/workflows/plate-coupling.yml, this plan directory. No src/lib.rs, public
session API, renderer, book-owned files or Cargo dependency changes.
Gate: all objective acceptance evidence above plus review; then dedicated push
and draft PR stacked on feature/bounded-obstacle-stokes-20261013 at bbc6089.
The inherited PR45 files are unchanged, with explicit range review before push.

Retain /workspace/Rheon-plate and its branch for parent integration review;
owner is this worker, synchronization is explicit-only with the stacked base,
retirement is after parent disposition. Commits remain protected by the remote
feature ref. Original /workspace/Rheon and PR45 checkout remain untouched.

## Composed-design review: applicable

1. Geometry/closure ownership stays in StaticObstacleShear; this module owns
   coupled dynamics, acceptance and scratch; example and tests own fixtures.
2. Fluid/plate speeds must publish together. Borrowed geometry and mode identity
   cannot change during a step. No renderer, file or public session interleaving.
3. Callers know a borrowed shear owner, tangent, plate modes/masses, their full
   velocity state, dt, cap and cancellation; they need not assemble/solve forces.
4. Changing geometry law touches its existing owner; changing coupling touches
   this workspace/oracle; future API serialization is separately owned and not
   prebuilt here. Changing a fixture affects example and independent controls.
5. Public owner getters carry stable physical coefficients. No private fields,
   hidden mutation order or versions are accessed. The interval module is reused.
6. The opt-in module can be compiled/tested or removed without activating it in
   production. One coupled acceptance transaction cannot be split into two owners.
7. Deleting the workspace would scatter solve/acceptance/publication logic into
   callers. Deleting the independent oracle loses a distinct solve/work check;
   it is fixture-sized, not a new general evidence framework.
8. Necessary complexity is a tridiagonal solve plus coupled work accounting;
   keep it in one module with no provider registry/adapters/new dependencies.

## Standards and dependencies

Reviewed source standards MrScripty/Coding-Standards at 188beda1, Core/Router,
Planning/Discovery/Development Proportionality, Implementation/Verification,
Architecture/Contracts, Commit/Documentation, Library and Rust/API guidance.
No application-purpose Standards Engine tools are exposed; source review is not
Engine approval. Relevant .agents skill in that repository is authoring-only,
not applicable to application edits. Workspace/Rheon has no AGENTS.md/skills.
Existing repository output and pinned-toolchain rules continue to apply.

## Blockers and re-plan triggers

No implementation blocker. General pressure-to-body coupling remains blocked by
known affine pressure-load bias; excluded here. Re-plan for shared API edits,
normal/moving-topology geometry, external force/torque, contact, new material
laws, or any failed physical identity that invalidates the selected model.

See [ledger](execution-ledger.md), [issues](issues.md) and
[consumer contract](reports/contract.md). No ADR or separate adapter is needed.
