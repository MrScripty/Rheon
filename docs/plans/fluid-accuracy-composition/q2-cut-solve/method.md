# CV2 proposed algebraic method and primary reasoning

The continuum/basis operators stay CV1. For dt>0 use
A=M/dt+K, C=[A,-B^T;-B,0], f=M u*/dt+F; solve A u-B^T p=f, B u=g=0.
Exact M SPD and full rowrank B imply an invertible saddle problem. Pressure is
absolute on the prescribed traction boundary. No closed-box gauge is introduced.

Proposed selected bounded method B (binary64):
Dii=1/sqrt(Aii); lower Cholesky of DAD gives L.
Y=L^-1 D B^T, Ecc=1/||Y_col_c||_2;
thin Householder QR of Z=Y E gives upper R. H=Y^T Y is diagnostic only,
never formed or factored to compute the solution. These are invertible coordinate/factor operations,
not a mass/volume floor or modified boundary condition. Solve
v=L^-1 D f; p=E R^-1 R^-T E(g-Y^T v);
u=D L^-T(v+Yp). Thus the pressure sign follows the original saddle equations.
Three fixed refinement corrections use the original full A/B and RHS, with
compensated original residuals, the same factors and g_delta=-B u.

Native M/K may have tiny rounding asymmetry. Lower triangles define the factor
preconditioner; the ORIGINAL full exported A remains the solve target/residual.
Do not silently symmetrize it or claim its exact equivalence to the ideal SPD
matrix. All factor entries/scales and iteration0..3 states are exported. Refuse
nonpositive/nonfinite Cholesky pivots/scales, zero/nonfinite QR pivots or arithmetic; no absolute tiny-pivot cutoff,
regularization, support removal, field clipping or success based on residual only.

For exact ideal blocks, full A/Schur Riesz congruence produces [I,-Q^T;-Q,0]
with QQ^T=I. Its spectrum is1 and(1±sqrt5)/2; this is a finite-dimensional exact
algebra statement, not an IEEE condition guarantee or cut-uniform PDE theorem.
The observed unscaled mass condition remains recorded; report distinct A/KKT,
scaled A/Schur and factor diagnostics. Back-transformation can amplify errors.

Primary sources read:
- Netlib LAPACK DPOSVX, https://www.netlib.org/lapack/double/dposvx.f:
  SPD diagonal equilibration, Cholesky, refinement and original-coordinate output.
- Netlib LAPACK Users' Guide, https://www.netlib.org/lapack/lug/node80.html and
  https://www.netlib.org/lapack/lug/node81.html: forward error vs backward error,
  componentwise sensitivity and the scope of equilibrated condition estimates.
- Southworth/Oliver2020, https://arxiv.org/pdf/2001.00711, propositions1.1–1.3:
  exact Schur factorization/invertibility and exact-arithmetic block-preconditioner
  results. No claim those iteration bounds apply to this floating factor code.
No author source code is copied. Native factors use basic separately written
finite dense Cholesky/Householder QR/triangular solves, no new dependency or external backend.

Mandatory independent comparisons distinguish:
1 actual native solution vs high-precision solution of EXPORTED rounded A/B/f;
2 native solution vs high-precision INDEPENDENT exact represented-source M/B/K/F;
3 difference between those references (assembly/load/RHS perturbation);
4 analytic rigid/affine patch and physical work/flux/momentum conditions.
All required solution gates retain1e-10; changing scale/basis does not excuse a
failed original-coordinate criterion. Negative findings return to method selection.

Before implementation, independent review admitted the QR version. A formed
Schur normal matrix would square Y conditioning; QR supplies its triangular
factor without that formation. Netlib DGELS primary source
https://www.netlib.org/lapack/double/dgels.f documents QR/LQ-based fullrank solves.
QR signs are arbitrary: negative R diagonals are valid, zero/nonfinite refuse.
For a KKT residual correction [r_u;r_p], this method takes g=-r_p.
