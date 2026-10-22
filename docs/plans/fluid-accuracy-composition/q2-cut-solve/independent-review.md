# CV2 independent deciding review

Disposition: **PASS for the fixed24 frozen-domain, planar CV1 coupled numerical
reference steps.** Both original-coordinate reference comparisons, analytic
patches, physical work/flux/momenta, factors and evaluator-custody gates pass at
the unchanged prospective1e-10 budget. This accepts the bounded CV2 information;
it does not admit a production solver/provider, PDE convergence, cut-uniform
pressure stability, changing free surface, solid boundary lifting, full3D water
or physical impact. No essential user/physical input is missing for this corpus.

Native numerical source is
`b0a9d52cc266cb651c5e67c07ec5f4f82419db55`; copied release binary SHA256 is
`c1f115f54e2867a2429b6d04305240ab242a1745b4c249e1e18c5512d3babfde`.
The ONE owner-executed native campaign exited0. The independent oracle was
prospectively frozen as SHA256
`5382622591188efb6d9148077f8940d4269ac7717424631423b18169b47e0705` before
new raw inspection/evaluation. The reviewer executed no native program, selected
pre-freeze numerical probe, fallback solve or candidate source edit. Read-only
source review caught/fixed, through the owner before freeze, nonfinite norm
masking, late-refusal state loss and premature operator-summary pass status.

The selected method was fixed before data: diagonal velocity congruence,
lower-Cholesky approximate preconditioner, scaled thin Householder-QR pressure
factor, followed by exactly3 compensated original-full-target residual
corrections. Tiny native rounding asymmetry is retained in the actual target;
lower triangles define only approximate factors. No Gram normal matrix is used
for the pressure solve, no pressure gauge is removed and no positive basis
restriction is dropped. Original unscaled partial-pivot elimination is retained
only as a diagnostic comparator, never an automatic fallback.

Independent Fraction source clipping and analytic polynomial/simplex integration
reconstruct all640 common regions,196 active velocity restrictions and66
positive pressure rows. Exact B rank remains16/16,13/13,36/36,1/1 in protocol
order. All68 active restrictions with zero old h-width dual area remain present.
Source, geometry, basis/quadrature, moment/trace and operator CSVs match the
previous65-file CV1 corpus byte-for-byte. No volume/mass floor or hidden source
perturbation was introduced. All8 factor blocks and7860 saved baseline/selected
coefficient-state records were checked, including initial and corrections1..3.
All24 scenarios have complete source/input/factor/state/metric custody.

The independent implementation imports no candidate code and uses neither its
Cholesky nor its QR for numerical truth. Two targets use mpmath1.3.0 LU at80 and
120decimal digits, reusing each factor for its3 scenario RHS:

- R1 binds the actual represented exported full A/B/RHS values exactly, isolating
  solver error for the rounded native system.
- R2 independently integrates the represented source/basis with the exact ratio
  of native mu0.1, the same native rounded nodal u*, exact represented gradient
  primitives and pressure2Pa before traction products/load/RHS rounding. It
  exposes operator/load/RHS arithmetic sensitivity.

Fresh160digit direct residual checks and80/120 agreement are numerical oracle
quality tests, not interval enclosures or rigorous IEEE forward-error certificates.
Maximum normalized agreement is2.6333569884e-78 (required<=1e-40). Maximum scaled
direct residual is1.1702305555e-80 at80digits and1.4118015997e-120 at120digits
(required<=1e-60). No inverse/interval certificate or additional native higher-
precision backend was introduced.

| Selected final comparison | Maximum velocity/Uref | Maximum pressure/Pref |
|---|---:|---:|
| R1 exported-f64 numerical target | 1.075398839959988e-16 | 1.943511817682625e-16 |
| R2 independent exact-integral numerical target | 1.861216501617988e-12 | 3.354808230044388e-13 |

Pressure normalization remains the prospective
rho*Uref*Lref/dt+mu*Uref/Lref, never an observed pressure magnitude. The largest
R1-R2 perturbation gap is1.8612243077e-12 velocity and3.3553711834e-13 pressure.
The final solve error for R1 is near f64 representation error, while the
remaining larger R2 difference is dominated by the retained arithmetic-data
perturbation. Both meet the original fixed component criterion. Separate exact
represented analytic targets have maxima1.8612611452e-12 velocity and
3.3548921217e-13 normalized pressure. The2Pa affine pressure is an actual coupled
patch result here, unlike CV1's projected load coefficient alone; it still is
not general pressure-PDE convergence or physical free-surface acceptance.

The unscaled comparator fails the fixed component criterion in all6 square
cases (both dt values and all3 scenarios), and none of the other18 cases. Its
worst normalized velocity error is2.5492773239e-9 against R1 and2.5511340212e-9
against R2. Its pressure maxima remain9.5866185929e-12/9.9221557113e-12. These
negative comparator results remain retained. The separate owner postprocessing
receipt reports its maximum original fixed-scale residual9.92930057184e-12;
that residual passing1e-10 does not make its velocity accurate to1e-10.

Selected iteration0 already has maximum R1 velocity8.7383612213e-14 and pressure
1.5599417680e-14. Correction1 reaches the final R1 errors above; corrections2/3
retain them. This is a reported history, not a post-data iteration selection:
all3 corrections remain mandatory under this version. There was no failure-
driven change of solver, precision, scaling, cut support or tolerance.

For the square at dt.125/1s, raw A condition diagnostics are
5.5613665165e7/5.2426437431e7, scaled-A diagnostics9.8951540982/73.9158427930,
and scaled weighted-divergence Z diagnostics1.3900510251/1.4860568966.
The original full-M diagnostic remains recorded. Preserve its provenance:
previous CV1 reported6.5331614839e11 from exact-integral-rounded M, whereas this
CV2 report evaluates the unchanged native-f64 M and obtains6.5331738715e11.
Those are different rounded arrays/SVD estimates, not changed source, geometry,
support or mass: all65 native CSV hashes are identical. None of these finite
SVD diagnostics is a rigorous condition enclosure or uniform mesh/cut bound.

Every declared physical and factor gate passes. Independent maximum normalized
momentum-row residual is1.0625028755e-16; continuity6.1824554991e-17; original
closed-cell flux2.4980018054e-16; independently integrated physical closed-cell
flux1.0901687884e-16; direct boundary-power discrepancy1.8041124150e-16;
linear/angular momentum discrepancy1.9258654814e-15. Absolute raw work balance
is<=1.3781414281e-15, residual-/mass-skew-corrected balance<=6.5963090572e-16,
and the absolute pressure/residual-work bound<=3.6783735088e-15. All use the
prospective units/scales and full physical source plus Cartesian traces. No
source-face flux alone was substituted for a cell's full divergence balance.

All45 frozen evaluator-custody checks pass:

- Complete normal Python, `-O` and `-OO` reconstructions succeed scientifically
  and produce byte-identical full reports.
- 13 malformed fixtures each refuse in all3 modes at their prospectively named
  guard, including empty/truncated/duplicate summaries, NaNs, incomplete/duplicate
  factors/states/vectors/metrics, and wrong dt. Each has empty stdout and no
  numerical success artifact. Generic incidental rejection was insufficient.
- A wrong coefficient is placed at least1e-3 from its R1 target, and ALL associated
  original residual fields are updated consistently. Each interpreter still
  produces the full24 negative numerical report and rejects the actual R1
  velocity gate (observed0.0009999999999999913), rather than relying on stale
  residuals or a malformed record to fail.
- All178 original native CSV path hashes are unchanged before/after, including
  the identical65-file prior operator corpus. Prospective Python/oracle hashes
  remain unchanged and own acceptance guards have zero Assert AST nodes.

`evaluation-v1/report.json` binds interpreter/custody outcomes, exact commands,
exits and before/after input hashes. `evaluation-v1/complete-normal-result/analysis.json`
contains all deciding errors, exact ranks, high-precision target coefficients,
iteration/baseline histories, physical fields and factor diagnostics. The other
complete modes are byte-identical. `postprocess.py/json` is explicitly post-run
aggregation of those saved values, not part of the frozen scientific oracle and
not another selected experiment. Changed mutant payloads/logs and original-input
symlink references are retained. `SHA256SUMS-final` binds all regular deliverables.

The bounded conditioning/coupled-solve question is resolved positively for this
corpus. Next method/verification gates remain pressure approximation/stability
and PDE convergence for the chosen pair, plus any separately selected solid
boundary lifting and moving-interface transport/GCL. Current results authorize
no production promotion or assumption that those remaining gates already pass.
Parent/repository lead retain root lifecycle, source admission and merge custody.
