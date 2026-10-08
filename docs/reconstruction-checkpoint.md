# Aligned-strain reconstruction checkpoint

This is a new reconstruction from public PR28 commit
`b61ae293224ff8262e99984a1b8fa8b2135b8c83`, on branch
`reconstruct/aligned-strain-pr28-20261008`. It is not recovered source.
The erased unpublished commits, Lean sources, oracle, evidence hashes and pass
counts are unavailable and confer no qualification on this work.

The authorized scope is one retained exactly grid-aligned internal box with at
least one fluid-cell padding, stationary no-slip obstacle and sealed free-slip
outer walls. The pressure face space and stored mass `rho*A*d` are preserved.
The reconstruction derives six strain families, explicit fluid-sector edge
quadrature, trace elimination at represented distances, a finite-row work
identity and a conditional exact-real stability theorem before implementation.
The Rust API is restricted to immutable bounded rows, actions and diagnostics.
Its nearest-rounded stability estimate is unenclosed and cannot authorize a
step. There is no advancement or pressure coupling API.

The held PR25 case41 and geometry-band campaigns remain outside this branch.
No main merge, deployment or broad PDE/refinement campaign is authorized.
An independent review is required before an ordinary draft is published.
The root instance is responsible for scheduling CodeRabbit after publication.

Generated records, receipts and tool logs live outside Git in
`/workspace/rheon-reconstruction-evidence`. Source and checkpoint commits are
saved durably on the named branch. Checkpoints are incomplete until fresh
qualification and independent review are recorded here; existing PR28 results
remain historical evidence for PR28 alone.

## Initial state

- Exact PR28 base fetched and checked out on 2026-10-08.
- No `AGENTS.md` found in the workspace; repository source/audit instructions
  read. No unavailable scope/memory holds are bypassed.
- Regular `gpt-6.1-sol` high-effort research/Lean, Rust and independent rational
  oracle instances active. Implementation waits for the derived stencil.
- GitHub connector authenticated as `MrScripty`. Shell `gh` token is invalid;
  connector Git object writes are the available authorized publication path.
- Fresh unchanged-PR28 scoped baseline passed 16 obstacle-flow and 10
  static-geometry contracts. These are new baseline results, not qualification
  of the reconstruction.

## Reconstructed source checkpoint

The original viscosity and variational-pressure papers were inspected; the
derivation records primary citations and labels the aligned sector/corner
stencil as a new choice. Represented center/edge distances determine actual
fluid-quadrant weights. Outer engineering-shear samples vanish analytically
under the sealed free-slip trace model; every included internal fluid-sector
row, including an empty coefficient row, is retained.

Immutable Rust actions and force/work diagnostics, an independent Fraction
oracle, and conditional Lean proof sources have been newly written. The first
focused Rust contracts and actual-native comparisons pass provisionally.
The rational central unit cube independently yields 48 active faces, 210
rows, 6 zero rows and B=17; actual native unit and anisotropic/nonmidpoint
fixtures agree with the oracle, with deliberate-corruption rejection checks.
The frozen native checkpoint at tree
`d5b700e586cd9e03ef771e058bf38a5c9de5904b` passed the clean-source native
qualification runner. It checked five actual native configurations, all
declared corruption probes, Rust contracts/Clippy/formatting, and normal and
optimized Python oracle/gate tests. Its receipt explicitly says
`lean_checked=false`; it does not qualify later proof edits.

Independent review found that an absolute tolerance floor accepted sign
corruption at extreme scales. The oracle now enforces positivity/signs and
relative primitive comparisons; cancellation scales use independent sums of
absolute contributions in the corresponding physical units. The exact reported
counterexamples and additional scale corruptions reject. Source binding now
includes untracked intermediate sources and rechecks path set, Git status,
HEAD and bytes. Fresh final clean-source receipts and exact-source independent
review remain pending.

Lean 4.19.0 and exact manifest dependencies are installed. The official mathlib
cache returned HTTP 403; no cache was obtained. The pinned dependencies are
compiled from source. The new module kernel-compiles; the allowed-axiom audit
and both normal/optimized rejection probes pass. The finite maximum supplies
the coefficient bound, including zero rows. Euler decrease requires positive
masses, nonnegative fixed weights, time and viscosity and `dt*mu*B <= 2`.
Composition additionally requires exact mass matching and an exact full
pressure solve. Assembly, IEEE refinement and global/continuum convergence
remain outside these statements. Final proof inventory/source qualification
is being frozen. No draft, main merge or deployment has occurred.
