# CV1 prospective deciding protocol, reference version1

Scope: operator/physical-trace builder only. No native coupled timestep/pressure
solve. Freeze method/protocol after independent review, native source after
source review, BEFORE selected tests/campaign. One fresh native CPU campaign.
Retain all refusals/failures; return to method selection without changing budgets.
Earlier Q1/Q2/P1 evidence and exactsource qualification remain immutable.

Fixtures (rho1kg/m³,mu.1Pa*s,Href1m,Lref1m,Uref1m/s):
- rectangle (.17,.23),(.83,.23),(.83,.79),(.17,.79); domain[0,1]²,n4.
- old shifted square side.3,center(.537,.479),17degrees; domain[0,1]²,n8.
- inscribed64edge diskR.75,center0, vertices .75(cos(2pi*k/64),sin(...));
  domain[-1,1]²,n8. Polygon approximation distinct from ideal disk geometry.
- CCWcornertriangle(.81,.99),(.99,.81),(.99,.99); domain[-1,3]²,n4.
  Its positive liquid lies in one primal[0,1]²cell with unsupported old left/bottom
  hduals. New basis support and fullmass must retain required coefficients.
No feature/volume floor. Source must fit BOTH staggered nodal lattice hulls;
reject missing halo and finite sourceedges coincident with primalgrid planes.

Reuse nativeGridGeometry IDs and immutablePolygon admission. Exact BigRational
clipping to every common halfcell box; positive rational fan triangles retained.
Empty/zero-measure rational cuts contribute nothing; unrepresentable positive
triangle/area or nonfinite arithmetic refuses, never becomes empty fluid.
Positive3x3Gauss-Legendre tensor quadrature after Duffy triangle map: nodes on[0,1]
(.5*(1±sqrt(3/5)),.5), weights(5/18,4/9,5/18). Physical Duffy map x=a+r*((1-s)*(b-a)+s*(c-a)), r,s in[0,1]; Jacobian=positivecross*r.
Mass has totaldegree4; Duffy maps each coordinate degree≤1in each parameter,
so mapped integrand degree≤5perparameter after Jacobian, quadrature exact in reals.
Each velocitycomponent's Q1knot rectangle and every pressurecell line belong to
common partition. Export source, actual piece bounds, rational clipped vertices/
areas, triangle and quadrature fields, basis indices/values/derivatives, all active
associations, matrices and trace ledgers. Old hdual volume is observation only.

For physical sourceedges and Cartesian gridface apertures, exact intersections
and split segments at EVERY common halfcell knot line before2pointGauss. On each
piece basis trace is degree≤2; flux and constant-stress nodal load exact in reals.
Every positive primalcell has a full basis-level physical divergence map from
its signed unique Cartesianface fluxes PLUS physicalsource boundary. Compare all
entries against B, not just a divergence-free solution or global cancellation.

Required gates (all finite; fixed1e-10 unless stated):
1. Independent Fraction from exact represented input floats certifies ALL common
   piece and active-basis positivity/indexing; zero mismatches. All positive
   quadratureweights and active Mdiagonal>0; native/independent actual geometry
   source area/first/second moments match scaled by h²,Lref at1e-10. No tangent
   artifact or source-coordinate change hidden by a finite tolerance.
2. Partitionofunity and x/y affine reproduction in both componentbases at ALL
   volume/trace quadrature points <=1e-10 in Lref scales. Negative halo/coincidence/
   orientation and unrepresentable-positive contracts reach exact refusals.
3. Independent exact polynomial integration of M/B/K and physical source/cartesian
   trace coefficients over rational clipped regions, not reused Gauss code. Errors
   normalized by fixed rho*Lref³, Lref² and mu*Lref respectively <=1e-10. All nodal
   constant-stress trace maps match volume gradient maps / (Lref*Href)<=1e-10.
   Everycell B entry matches signed physical trace / (h*Href)<=1e-10.
4. Independent exact rational rank(B)=ALL positivepressurecells for this traction
   corpus. No graph-gauge assumption/SVDtolerance. Independent fullM Cholesky
   positive, finite; report M and B condition numbers diagnostically,
   no rank/threshold selected after data. Conditioning failure stops a native
   step admission; no solve or diagonal-lumped substitute in CV1.
5. Exact representable constanttranslations and rotation(-y,x) have B/Kactions
   <=1e-10 in fixed scales. Fullmass translation/angularmomentum maps agree with
   independent basis integrals, not clipped-dual lumping. Existing aligned M/K
   differ by declared quadrature, but affine strain/continuum units match.
6. Affineu=(.25x+.75y+.1, -.2x-.25y-.3)m/s using numericalx/Lref,y/Lref,
   p0=2Pa, sigma=-p0I+mu(A+Aᵀ). Independently integrate F_i=∫Γφ_i·sigma*n from
   original physicalsourceedges. EVERY nodal KU-Bᵀp0=F <=1e-10 in1Pa*Lref*Href.
   Also p-only load -p0∫φ·n must equal -p0Bᵀ1 entrywise; wrong omittedpressure
   load must differ at some node and satisfy F_omit-F_full=+p0Bᵀ1. Global force/
   torque/power can cancel even for wrong pressure; totals alone are invalid.
   A coefficient projected onto Bᵀ1 may check2Pa, explicitly not PDEpressure
   recovery. No F constructed as matrixaction on a desired solution.
7. FullM polarization and pressure/viscous work use frozen nodal coefficient
   vectors (numerical x/Lref,y/Lref; multiply byUref):
   a_x=1/4+x/2-y/8+xy/4; a_y=-1/8+x/4+y/2-xy/8;
   b_x=-1/4+x²/8+y/4; b_y=1/2-x/8+y²/4.
   Pressurevector q_c=(3/4+x_c/8-y_c/4)Pa at originalprimalcellcenters.
   Compare polarization E(a)-E(b)+.5||a-b||²_M-aᵀM(a-b),
   aᵀK b versus independently integrated2muD(ua):D(ub), and
   qᵀB a versus(Bᵀq)ᵀa plus independently integrated q*div(ua).
   All discrepancies <=1e-10 in fixed Eref=rho*Lref³Uref²,
   Dref=mu*Uref²*Href,Wref=1Pa*Lref²*Uref scales respectively;
   Qref=Lref²Uref,Fref=1Pa*Lref². This is operator identity, not a timestep energy
   acceptance. No new Lean/native/IEEE claim inferred from abstract algebra.

Focused debug/release contracts: source/hull/coincidence/orientation refusal,
exact tangent vs positive cut and collapsed-positive refusal; polynomial/basis
partition/derivative and triangle quadrature moments; physical trace pressure
sign. StrictClippy/lockednativebuild. ONEreleasecampaignfreshroot and always-on
analyzer normal/-O/-OO. Independent oracle sources hashed before raw evaluation;
fullrank and polynomial/load/flux reconstruction required, source/binary/commands/
raw/hash reports retained and saved privately. No fullsuite repetition for private
reference unless new production/dependency changes occur. Exactsharedwriteset:
new examples/grid_cut_variational_reference.rs plusprivate modules under
examples/grid_cut_variational_reference/, new testadapter, new q2-cut-method/*;
read-only reuse earlier geometry/exact modules, no edits to those source owners.

Planar boundary convention: this is the two-dimensional Stokes problem measured per Href=1m depth. GridGeometry countsZ=1 is indexing/unit bookkeeping; there is no dynamic uz and no physical free end-cap boundary or three-dimensional closed-volume claim. Γ contains only polygon edges times Href. Do not infer unprescribed 3D normal end-cap traction from these planar tests.
