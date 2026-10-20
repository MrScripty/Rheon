# G Source audit and corrections, 20 October 2026

Puma is the author credited throughout the existing research book. This appendix
records a documentation audit against remote main
`718469beedb9a949f4bb6f3299c7b880b4c47e47`, with separately identified open
research branches. It preserves historical results and failed criteria. It
does not change a solver, numerical threshold, proof, retained experiment or
active implementation plan.

## Identity and scope

The GitHub Pages API reports [the existing public site](https://mrscripty.github.io/Rheon/)
for `MrScripty/Rheon`, workflow deployment from main. Deployment `7116715012`
has a successful status dated 16 October 2026 and source
`d8771ad83c30644d29929c96534548ce238bb357`; its status gives that same URL.
This verifies the deployment target and recorded source, not a fresh HTTP
content comparison: direct site retrieval was unavailable in this audit.

The canonical inputs are `docs/research-book/chapters/*.md` and
`appendices/*.md`; `docs/education/build.py` discovers them and renders the
Pages reading edition, print HTML and Markdown downloads. The separate
`build_manuscript.py` builds the native-equation manuscript. The Pages workflow
is `.github/workflows/research-book-pages.yml`; the separately pinned reviewed
PR36 workflow must not be mistaken for a build of today's main. A new book or
guessed deployment is unnecessary.

Historical starting points are README-only `0cb4842`, original book commit
`4deee396`, first Rust carrier `6ba41c7`, first material solver `9593921`, and
later teaching/provenance revisions including `793b4d6`. Their Git history is
the original evidence; a corrected paragraph does not retroactively qualify it.
The earlier [general-fluid audit](../../audits/general-fluid-20261019/report.md)
is independently executed evidence at `bc2f472`, subsequently merged in PR119.
PR120 supplies a repair plan, not proof that its proposals already work.

## What caused the discrepancies?

No inspected core projection equation has a demonstrated pressure sign or
missing-density defect. Several later failures arise from boundary/model scope,
geometric predicates, algebraic conditioning or implementation state. The
research-error hypothesis is therefore **not established**.

Classification below distinguishes a false or ambiguous paper statement
(research/documentation error), applying a formula outside its assumptions
(misinterpretation), a defect in its discrete implementation, an unsupported
generalization from narrower evidence, and an unresolved causal question.
Some claims require more than one classification. A stated approximation is
not automatically an erroneous research result.

## Claim-to-equation-to-evidence audit

| ID / original claim or assumption | Source, implementation and retained evidence | Finding and correction |
| --- | --- | --- |
| G01 — Chapter 1's repository has no solver | True at `0cb4842`; current `src/lib.rs` exports grid, liquid and particle owners. `6ba41c7` adds carrier; `9593921` adds material preview. | **Stale documentation**, not an equation error. Label original baseline and current bounded capabilities separately. |
| G02 — Chapter 2 incompressibility and momentum | Mass product rule gives divergence zero when positive density is materially constant. Pressure acceleration is `-grad(p)/rho`; `src/forces.rs` distinguishes acceleration and force density. Chorin [S3] is a projection foundation, not a mesh/particle verification. | **No defect found in stated assumptions.** Constant viscosity and incompressibility permit the interior Laplacian simplification; boundary traction still needs the full stress. No evidence for a factor-density error. |
| G03 — Chapters 3–4 incidence signs and Poisson weights | `src/operator.rs::new` sets `A/(rho*d)`; pressure RHS is negative outward predicted flux / dt. Graph head-positive B gives `K=BSR^-1L^-1B^T`. | **No sign/unit defect found.** `Kp` and RHS are m³/s². Row volume division belongs in divergence, not an unqualified Euclidean-CG matrix on unequal volumes. |
| G04 — Pressure residual qualifies incompressibility | Chapters 3–4 derive `D(u+)=-dt V^-1(b-Kp)`; code recomputes full residual including the gauge and actual stored-field divergence. | **Conditional identity, not accuracy guarantee.** Tiny volumes amplify integrated residuals; f32 velocity storage adds error. Check full physical divergence and field accuracy independently. |
| G05 — CG repair implies wrong research Poisson formula | [Commit 34f6165](https://github.com/MrScripty/Rheon/commit/34f6165e6068997c3ef05b03d21241a360ccd84e) changes `true_report` scratch ownership and restart decisions; regression in `tests/pressure_contract.rs`. | **Implementation defect**, repaired without changing the continuum equation. Divergence-only rejection had disrupted the CG recurrence. Separate BLAS-sensitive historical CG reproduction is documented in the retained evidence dossier, not a new Poisson derivation. |
| G06 — Chapters 3, 8 and 20 positive cut weights imply physical consistency | `StaticObstaclePressure` uses shared box openings, nominal center distances and area-lumped mass. [S7 §2.1] explicitly allows approximated staggered volumes; Chapter 8 already supplies a skew-sample counterexample. | **Unsupported generalization** if extended to arbitrary cut centroids/meshes. Symmetry and a work identity do not establish geometric consistency or correct traction. Keep the normal-alignment and geometry hypotheses. |
| G07 — Conserved tracer/volume means conserved accurate liquid | `advection.rs` uses midpoint backtrace/interpolation; `liquid_volume.rs` instead shares signed donor volume transfers. `liquid_step.rs` ordinary mode retains an all-fluid carrier. | **Misinterpretation/model scope.** Appearance tracer is not liquid mass. Conservative represented fraction can diffuse; source/boundary ledgers and geometric interface error are separate. |
| G08 — Chapter 9 atmospheric pressure is a complete free-surface condition | Slab/column operators use half-cell/actual interface distance; `viscosity.rs` admits fully filled free-slip boxes. [S11 §§2–3] distinguishes coupled traction from the earlier pressure/viscosity split. | **Declared approximation; unsupported composition** if claimed general. With outward liquid normal, `p_liq-p_air=sigma*kappa+n^T tau_liq n`; `p=0` additionally neglects relevant normal viscous/capillary terms. Existing formula signs check. |
| G09 — Chapter 10 viscosity and energy | `viscosity.rs` uses normal strain weights 2V and engineering shear V, dynamic mu, nu=mu/rho, negative mass-weighted transpose; explicit guard `nu*dt*sum(1/h²)<=1/4`. Scalar diffusion's 1/6 is a different stencil. | **No viscosity-unit defect found.** Tensor dissipation, scalar positivity and full timestep stability are distinct. No general free-surface/mesh no-slip capability follows. |
| G10 — Chapter 11 generic SPH density is Rheon's mixed-material model | Standard `sum m_j W` [S13] differs from `rho0_i*sum(m_j/rho0_j)W` in `particle_fluid.rs`; density floor 0.05 and nonnegative linear liquid EOS are explicit. | **Interpretation hazard**, clarified by deriving the actual force below. Parcel mass is conserved while estimated volume/density can err. Artificial sound parameter is not physical water sound speed. |
| G11 — Spiky/poly6 difference proves original research wrong | [S13 §3.5] deliberately uses poly6 density and spiky pressure for repulsion. First retained material commit `9593921` already uses a poly6 energy-gradient pressure, with a matching independent derivative test. | **Method tradeoff, causal history unresolved.** A different force kernel is not the derivative of Rheon's chosen density energy. It does not invalidate [S13]'s different model. No earlier committed spiky Rheon particle version was found in this path. |
| G12 — Stable antisymmetric particle forces prove water accuracy | `particle_fluid.rs` has symmetric pressure/viscosity pairs, symplectic Euler and bounded retries. Audit records kernel density and viscosity moments, nonzero rest motion and failed pressure/density screens. | **Unsupported generalization.** Momentum cancellation does not imply consistency, hydrostatic balance, acoustic accuracy or a stable arbitrary neighbor graph. Missing support, EOS and initial equilibrium effects remain incompletely separated. |
| G13 — Mesh collision supplies an impermeable hydrodynamic wall | `mesh_collision.rs::resolve_segment` sweeps centers; `particle_fluid.rs` density/forces do not test mesh visibility. Static import `58cd23e` and rollback tests preserve geometric behavior. | **Boundary/model gap**, not an absent collision feature. Retained thin-barrier calculations find cross-wall interactions despite zero center crossings. Contact clearance is not a particle-radius/stress boundary. |
| G14 — Chapter 19 Reynolds wall-volume identity permits any motion | Geometric `dotV=integral us·nf` and source-free incompressibility together impose sealed total `dotV=0`. Moving cut cells need swept-volume balance. [S7 §3] explicitly notes incompatible scripted compression. | **Ambiguous research exposition**, corrected in place. A sealed inward piston cannot be repaired by silently projecting away the incompatible source. |
| G15 — Chapter 10 pressure load normal; Chapter 20 has no pressure consumer | Pressure load `-integral p ns` needs solid-outward ns. Main `obstacle_pressure.rs` stores and consumes `StaticObstacleGeometry`; PR45 aligned Stokes and PR47 diagnostics are merged. | **Documentation ambiguity/stale status**, corrected. Body pressure force, fluid traction and total viscous load must not share an unspecified normal. Historical checkpoints retain their source identities. |
| G16 — A wall adaptation passing work solves particle walls | PR121 `a64d4d3` fails both wall adaptations. PR123 `bc2573f` repairs gamma-adjoint work but retains uniform-pressure E0; exact partition retains other defects. PR124 `9d84f66` supplies connected material interfaces. | **Discrete consistency/representation findings**, not a generic source-paper error. The passing 1D material reference changes DOFs/family; no SPH/3D/timestep admission follows. |
| G17 — Positive floating aperture means a real open face | PR132 `e6525a1`, numerical source `0f4e7e0`, constructs exact dyadic intersections for the represented polygons and removes five positive artifacts with exact zero measure. | **Geometric implementation defect.** Exact topology and finite moments are separate checks. No aperture floor or retrospective threshold change. |
| G18 — A coupled residual proves a cut solution accurate | PR134 `f84f651` is tiny aligned algebra; PR135 `4f8bd22` derives restricted full-mass cut operators. PR139 `902f609`, numerical source `b0a9d52`, changes solver scaling/factorization/refinement, preserving CV1 operators. | **Floating conditioning/solver defect of the unscaled reference**, not a proved PDE error. Six square accuracy failures remain despite residual passing; CV2's bounded success has no generic cut/PDE/moving/production implication. |
| G19 — Native producer/integration tests validate physical water | PR131 iteration-count validation, PR141 `38c40ca` closed projection/cancellation stage, PR142 `4f2d553` producer/source admission; 228 focused tests at executed source `3b3b667`. | **Software qualification only.** Record known PR131 VTK installation and PR134 desktop CI failures separately; neither repairs nor refutes a mathematical benchmark. |
| G20 — CV3-A MMS has already validated the reference | PR141 admits the prospective static polygon contract; PR143 `bc82e92` prepares nine cases on PR139. Source/oracle/resource gates are pending at the inspected checkpoint. | **Unexecuted research**, not evidence of PDE convergence. Do not report the preparation PR or integration admission as numerical results. |

## Deriving the actual particle pressure force

This is an independent derivation for Rheon's smooth active-density branch,
not an attribution of a new formula to [S13]. Write reference density \(r_i\),
\(\rho_i=r_i\sum_j(m_j/r_j)W_{ij}\), and compression energy
\(U=\sum_i m_i e_i(\rho_i)\), with \(e_i'=p_i/\rho_i^2\).
Differentiation and the odd gradient of a symmetric kernel give

\[
 a_i^p=-\sum_{j\ne i}m_j
 \left(\frac{p_i r_i}{r_j\rho_i^2}
       +\frac{p_j r_j}{r_i\rho_j^2}\right)\nabla_i W_{ij}.
\]

This matches `particle_fluid.rs:656–665`. Multiplying by \(m_i\) produces
equal-and-opposite pair forces. \(m_j(p/\rho^2)\nabla W\) has units m/s²,
since a 3D kernel gradient has units m⁻⁴. Using another kernel gradient is a
different force model and breaks this particular energy derivative. It is not
by itself proof of instability or an error in every SPH formulation.

The density floor and liquid pressure clamp are piecewise model changes;
the smooth derivative argument must not cross their switches without analysis.
Liquid pressure is zero below reference density, including the density floor.
The gas branch uses a different EOS, and the current pair loop also skips nearly
coincident pairs. Those details prevent extrapolating the smooth calculation
into a global energy or near-coincidence robustness theorem. Poly6 pressure
repulsion also tends to zero at exact coincidence; [S13 §3.5] explains its
choice of another pressure kernel. Pairing and perturbed-support behavior need
independent tests.

## What the retained experiments establish

### Pressure work and a volume partition are independent checks

The [PR123 mathematics](https://github.com/MrScripty/Rheon/blob/bc2573f2ba8d51f9f473fb11285b0c26840e4f18/docs/plans/fluid-accuracy-composition/p1-adjoint/mathematics.md)
can be checked directly. For fixed positive masses and a positive differentiable
volume map \(V_i(q)\), let \(\rho_i=m_i/V_i\) and
\(U=\sum_i m_i e_i(\rho_i)\), with \(e_i'=p_i/\rho_i^2\). Then

\[
 dU=-\sum_i p_i\,dV_i,\qquad
 F_k=-\partial_{q_k}U=\sum_i p_i\partial_{q_k}V_i.
\]

At uniform pressure \(p_0\), the force is
\(F_k=p_0\partial_{q_k}\sum_iV_i\). Thus for interior particle variations
with fixed physical walls, exact pressure work does not imply constant-pressure
cancellation: the total-volume partition must also be position independent.
Moving walls instead have physical pressure reactions and boundary work; this
interior cancellation statement does not eliminate those loads. Gamma density
rates and forces must differentiate the same map. Calibrating density values
with frozen offsets does not calibrate its derivatives. PR123's offset planar
map is a new diagnostic adaptation, not a literal Mayrhofer implementation or a
refutation of that primary paper. PR124's successful connected-interface
construction is a different state family, bounded to the static n8 1D reference.

### Boundary pressure and manufactured-problem identity

CV1/CV2 use a distinct staggered-Q1/P0 restriction with full consistent mass,
symmetric stress and outward-fluid traction. Prescribed open traction fixes
absolute pressure; adding a closed-wall pressure gauge would change the problem.
Their rank/patch/work checks are not a uniform inf-sup theorem. The
[Larionov–Batty–Bridson supplement](https://cs.uwaterloo.ca/~c2batty/papers/Larionov2017/supplemental.pdf),
Section 1, explicitly gives nonzero pressure at the disk interface with zero
total traction. The proposed CV3-A prescribed-traction polygon is a different
problem and remains preparation-only at PR143. Neither source may be relabeled
as a Rheon moving-geometry or continuum-convergence result.

### Retained numerical results

These are reported historical results, not a new execution campaign here.
The [audit report and fixed protocol](../../audits/general-fluid-20261019/report.md)
retain the actual source, measurement definitions and failures:

| Experiment | Retained result | Permitted inference |
| --- | --- | --- |
| Water slab hydrostatics, 8/16/32 layers, both PCGs | max pressure error <=1.182015e-4 Pa; max speed <=6.86e-15 m/s | Correct bounded force/projection balance, including f32 predictor error; not arbitrary-geometry convergence |
| Smooth Neumann pressure MMS, 8/16/32/64 | RMS 15.65884/3.929765/0.9833875/0.2459061 Pa | Observed second-order spatial behavior in this interval family |
| 1 mm no-slip channel, 8/16/32 layers | relative RMS 2.139313%/0.534882%/0.133721% | Coarse fixed <2% target fails; refinement passes, no viscosity-unit repair inferred |
| Sine decay / MAC viscous vortex | time ratios .5082/.5041; spatial ratios .2518/.2479 | First-order time / about second-order space for those isolated models |
| Conservative translated fraction, n16/32/64 | initial .125 m³ conserved with boundary ledger; L1 shape error .0583744/.0422247/.0301952 m³ | Conservation with interface diffusion, not accurate shape from conservation alone |
| Particle water-rest initialization, 8/12/16 samples per side | final density RMS 18.7968%/14.7157%/12.5091%; substantial motion; nine physical screens fail | This initial state/model is not a qualified equilibrium. It does not prove instability of a correctly initialized hydrostatic state |
| Thin oblique particle barrier | zero detected center crossings; cross-barrier density contributions 33.3038/17.7712/10.2860 kg/m³ | Geometry contact works in these cases; neighborhoods are not hydraulically partitioned |
| PR139 CV2, 24 fixed planar solves | max velocity discrepancy 1.87e-12 against unchanged 1e-10 gate; six unscaled failures retained; mass condition about6.53e11 | Bounded numerical reference accuracy; no PDE convergence, generic cut stability, moving boundary or production-water accuracy |

The [PR139 qualification](https://github.com/MrScripty/Rheon/blob/902f6092de6044419d028f92dabd8e59ae7a87ba/docs/plans/fluid-accuracy-composition/q2-cut-solve/qualification.md)
distinguishes high-precision solution of exported rounded blocks from
independently integrated operators. R1's maximum normalized velocity error is
1.08e-16; R2's is 1.87e-12. The remaining representation gap is not hidden by
reporting R1 alone. Neither reference is a certified interval enclosure.

PR121's perturbed AHA adaptation reaches 0.167893 m/s and 19.0076% pressure RMS
against fixed 0.01 m/s / 2% gates; SA finest shear traction errors remain
29.33–35.87%. PR123's gamma-adjoint uniform-pressure force defects are
0.459820 ordered / 0.517953 perturbed. PR124's n8 connected-interface reference
passes its predeclared two-consecutive-step derivative rule while retaining eight
smallest-step Hessian failures. These campaigns are preserved, not relabeled
passing SPH or excused by visual quality.

## Corrections and outstanding claims

Dated edits label the README-only baseline, distinguish parcel mass/fraction
volume, specify the solid-outward load normal, restore sealed moving-boundary
compatibility, acknowledge bounded obstacle pressure consumption and update the
merged diagnostics/Stokes/Session checkpoints. Chapter 29 adds the researched
collision/boundary/coupling synthesis. Original proof hypotheses, companion
experiments and accepted/failing thresholds are unchanged.

Still unresolved are whether any earlier particle-bounce implementation existed
outside the retained history and what formula it used, the separate shares of
EOS/initialization/support error in the water-rest
screen, and general mesh-water, moving-volume, coupled traction and physical
validation. No publication should infer that native integration passes, exact
finite algebra or a small residual closes those questions. The new chapter's
verification ladder is prospective. Build/render/reviewer status belongs in the
task's separate audit report; the appendix does not predeclare those checks green.
