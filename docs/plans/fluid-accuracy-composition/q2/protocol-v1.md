Q2 first bounded reference/geometry implementation proposal (no selected test runs yet)

Trigger: Q1 b9a9d4c native CPU campaign stopped on fixed sphere depth exhaustion;
all 12 completed 16/32 transport trajectories diffuse strongly, 3 Full controls
refuse. Do not promote transport; use only dependency-ready geometry prerequisite.

Exclusive proposed files: examples/grid_geometry_reference.rs,
examples/grid_geometry_reference/*, tests/grid_geometry_reference_contract.rs,
docs/plans/fluid-accuracy-composition/q2/*. No existing production edits, G1,
root canonical lifecycle, proof, dependencies or lib exports. GridGeometry reused.

Private CPU reference version1, not an alternative production family or solver:
convex CCW polygon clipped to existing GridGeometry's Cartesian cells (one metre
extrusion; planar XY velocities only). One source shape supplies cell volume /
first and second moments, unique X/Y open-face segments and first moments,
clipped staggered-face dual volumes, normal-strain cell volumes, shear-strain
vertex dual volumes and physical-boundary oriented segments / first moments.
Keep every computed positive cut, no flooring/merging/sliver deletion.
Native clipping is not a certified topology oracle for arbitrary underresolved
continuum cuts; only the fixed source cases and retained independent errors are
qualified. Moment accumulation uses a translated local origin; nonempty cuts with
three or more represented vertices and zero area, negative fan orientation or
nonfinite moments are refused as unresolved, never converted into empty fluid. Record zero
open faces whose dual volume is positive without choosing active velocity DOFs;
that extension/assembly decision belongs to later B/M/S operator admission.

Method: rectangle half-plane polygon clipping, exact-real polynomial polygon
moments from Green's formula in native f64; clipped line intervals independently
computed from the original source convex halfplanes for each unique grid face.
Shared grid-face flux records consumed with opposite cell signs. Physical segments
come from parameter-clipped original source edges with retained IDs and are separate, including nonzero prescribed
traction. No pressure gauge or pressure=0 free boundary is introduced.

Cases: supplemental disk R=.75, origin0, domain[-1,1]^2, n32/64/128, inscribed
regular polygon with8n segments. Exact disk geometry is a separate reference;
polygon approximation is method geometry error, not quadrature uncertainty.
Rotated .3m square at0/17/31 degrees, center(.537,.479), domain[0,1]^2,
n16/32/64, 1m extrusion. A fluid convex polygon is a geometry primitive only;
it does not represent the plan's fluid exterior of an obstacle or full hydro run.

Predeclared prerequisite gates (not replacements for canonical Q2 criteria):
finite nonnegative volumes/quadrature weights; all per-cell closed-boundary vector
and first-moment identities normalized by h and h^2 respectively <=1e-10;
global analytic rotated-source area/first/second moment relative error <=1e-10;
unique-face intervals agree with independently clipped cell-face edges <=1e-10h;
global sum dual volumes for each velocity/stress association equals source volume
within1e-10 relative (grid domain contains source).

CPU flux checks use u=(x^3,y^3): integrate unique grid-face flux and physical
segments with 2-point Gauss; per-cell flux vs3*(moment_xx+moment_yy) <=1e-10
normalized by h^2 times fixed source scale. Also quadratic divergence-free
u=(x^2+2xy,-2xy-y^2) checks geometry/flux closure, not incompressible solve.
Stress check uses same divergence-free u, mu=.1 or .001002 and nonzero absolute
p=1+mu*x-3mu*y. sigma=-pI+2muD(u), div sigma=(mu,mu). Native integrated traction
from shared boundary moments equals mu*V per component, normalized by fixed
nonzero stress scale times h^2, <=1e-10. Retain scalar pressure and full traction;
negative control setting p=0 must produce the analytically predicted defect.
Disk geometry area / radial second-moment / cubic-flux / linear-stress force error
<=1e-4 finest and pairwise orders1.8..2.2; this tests analytic curved-shape
approximation convergence, not MMS velocity/pressure/stress accuracy.

Require independent read-only discretization review before source freeze, then
focused debug/release tests, Clippy and ONE actual release CPU campaign. Independent
vertical-slice/positive integration per-cell oracle against polygon coordinates
and source analytic disk/rotated source moments; tolerance1e-10 normalized h^2;
all native raw table counts, signs and flux/stress ledgers reconstructed.
No full-suite rerun after reference-only addition unless new production changes.
Full coupled disk/annulus MMS, water hydrostatic obstacle, physical traction,
B/M/S/W dense oracle/component/gauge/solver selection remain pending. No Lean
claim; existing plan StokesWork exact-real proof remains relevant only after actual
coupled assembly exists. Parent owns canonical Q2 lifecycle reconciliation.

Prefreeze clarifications from independent mathematical review:
- All polynomial formulas use numerical coordinates x/Lref,y/Lref, Lref=1m,
  Uref=1m/s, extrusion Href=1m. Functions return SI velocity; cubic coefficients
  are Uref/Lref^3 and quadratic coefficients Uref/Lref^2. Pressure is
  p0 + mu*Uref/Lref*(x/Lref-3y/Lref), p0=1Pa. tau_ref=1Pa is a fixed nonzero
  normalization reference; global nonzero force scale mu*Uref/Lref^2*Vsource.
- Areas times Href are represented volumes. Oriented segment measures times Href
  are physical areas; stress integrals are forces. Normal closure scales h*Href,
  first/flux closure h^2*Href with Lref/Uref factors numerically1 in SI;
  traction closure tau_ref*h^2. These geometry budgets are NOT volume-normalized
  incompressibility or full physical-traction criteria.
- Global first and second moment errors use A_exact*Lref and A_exact*Lref^2,
  respectively, including zero disk moments. No division by exact-zero moments.
- Retain full source vertices and original source-edge IDs for each physical
  segment. Unique grid-face intervals are computed independently by source
  halfplanes; physical pieces use source-edge parameter clipping, not reclassified
  cut-cell polygon edges. All staggered dual boxes are width h centred at their
  association with one-metre extrusion; they may overhang the containing grid.
- Finite source edges exactly coincident with a primal grid plane are explicitly
  unsupported by reference v1 and refused before geometry publication. A focused
  negative contract covers this ownership ambiguity; there is no double counting.
  Point tangencies have zero measure. Main shifted square/disk cases have no
  finite coincident source edge. A later coincident-boundary support policy needs
  separate admission; this reference does not establish it.
- p=0 control must reproduce wrong-minus-correct total force
  (mu,-3mu)*Vsource (SI coefficients above), not merely produce a different value.
  Disk flux/stress convergence largely measures the same polygon-moment geometry
  approximation, explicitly not independent PDE-error evidence.

Final prefreeze source clarifications: source vertices must be globally unique,
not just adjacent-distinct; doubled winding is refused by a negative contract.
X/Y face arrays both use native GridGeometry indexing (x fastest); shear vertices
are x fastest with counts(n+1,n+1,1). Export all global mxx/mxy/myy and compare each
against the analytic source at the fixed nonzero moment scale, independently of
radial moment convergence. Radial convergence is also evaluated explicitly.
