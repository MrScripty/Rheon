# CV1 independent deciding review

Disposition: the four frozen cases pass the prospectively declared **operator /
physical-trace builder** gates. This does not admit a coupled timestep, pressure
PDE solve, general cut-cell stability, moving free surface, or production method.
No native rerun, source patch, altered threshold, floor, coefficient deletion,
gauge or stabilization was used by this reviewer.

Reviewed native source is `6d99e7794a6aa69f5886bb15cc662e40ea43cd40`.
The independent oracle was frozen before native data inspection/evaluation;
`oracle-freeze.json` SHA256 is
`38d4b75a5bbcdc7991725aef091f5e2347f671fe0b19ce51a34e8a4ce1095d55`.
Its candidate-file receipts match the final native source. The only prospective
review corrections were the native five-field refusal row and continued-case
driver custody, plus matching the oracle to the actual native `passed` status.
No fixed native campaign preceded those source corrections/final freeze. Earlier
pre-driver focused-contract receipts remain separately retained.

The ONE native campaign was produced by its owner. This reviewer executed only
frozen Python reconstruction/custody checks on the saved native-v1 CSV corpus.
Independent geometry and polynomial integration use exact represented-coordinate
Fractions and analytic standard-simplex factorial moments, not native Gauss
samples or candidate imports. Every active association, full matrix entry and
physical trace is rebuilt from the source polygon, declared lattice and common
partition. Physical loads are integrated from original source edges; they are
not manufactured as matrix actions.

| Case | Active velocity coefficients | Exact rank(B) / positive pressures | Full-M condition diagnostic | B condition diagnostic | Active zero-old-dual coefficients |
|---|---:|---:|---:|---:|---:|
| rectangle | 40 | 16/16 | 4.0217109939e4 | 49.76132296 | 16 |
| square-17 | 44 | 13/13 | 6.5331614839e11 | 8114.51138737 | 18 |
| disk-64 | 104 | 36/36 | 1.4115143713e4 | 51.39203431 | 28 |
| corner-triangle | 8 | 1/1 | 1.0033118455e5 | 1 | 6 |

Exact rational elimination gives full row rank without a pressure gauge or SVD
rank tolerance. Both the independently integrated full M and exported native M
pass finite, positive-diagonal Cholesky in all four cases. Condition numbers are
floating SVD diagnostics of the exact-integral matrices converted to f64; they
are not rigorous condition enclosures or a mesh-uniform stability theorem.
The square's minimum exact M diagonal is approximately1.7013962159e-14.
Its large condition number is a material unresolved next-solve issue, not a
failed CV1 threshold: conditioning was diagnostic and no coupled solve was run.
It forbids inferring trustworthy velocity/pressure forward error from these
builder identities or from a future residual alone. Any next solve needs a
prospective method/backend/scaling and accuracy/conditioning contract.

The independent reconstruction checked all368 native node associations,196
active restrictions,66 positive pressure cells,640 common regions (206 positive),
470 positive fan triangles,4230 bulk quadrature points,33840 bulk basis records,
502 split boundary/grid segments,1004 trace quadrature points,7292 nonzero trace
basis records,102 unique positive Cartesian faces, and43724 dense M/K/B/grid-flux/
physical-flux entries. Exact cut polygons, rational area witnesses, zero/positive
classifications and active/pressure masks have zero mismatches. All68 active
restrictions with zero old h-width dual area remain present. The corner has no
positive Cartesian face aperture but eight positive-mass basis restrictions,
six with zero old dual. That is a result for the declared wider Q1 support;
it does not validate an aperture-based finite-difference extension policy.

All fixed1e-10 finite geometry/moment, M/B/K, nodal derivative/trace, every-cell
physical flux, full mass/momentum maps, rigid/affine actions, full-stress/pressure
load, polarization and pressure/viscous work gates pass. The largest independent
normalized discrepancy is7.8496237288e-15 for disk viscous cross-work. All original
native fixed diagnostics also pass; their largest value is6.7501559897e-14.
The constant2Pa projected load coefficient and correct omitted-pressure sign are
retained entrywise, with nonzero nodal negative-control signal in every case.
These are affine physical-load discriminators, not PDE pressure recovery.
The reference remains two-dimensional per1m depth and has no 3D end-cap claim.

Frozen evaluator custody checks pass in ordinary Python, `-O` and `-OO`:

- All three complete reconstructions succeed with byte-identical analysis JSON.
- Fourteen prospectively fixed malformed fixtures each intentionally refuse in
  all three modes: empty/truncated/duplicate/NaN summaries; missing quadrature;
  duplicate bulk basis; NaN matrix/map; duplicate/misindexed node; invalid rational
  denominator; zero quadrature weight; missing trace point; duplicate metric.
- All45 subprocess checks pass. Malformed runs have empty stdout and create no
  success-artifact directory. Logs retain their guard reasons and exits.
- All65 original native CSV path hashes match before/after. Frozen Python sources
  have zero Assert AST nodes and retain their original hashes.

`evaluation-v1/report.json` is the interpreter/custody report;
`evaluation-v1/complete-normal-result/analysis.json` contains all case values,
exact-rank pivot indices, normalized discrepancies and original input hashes.
The other two analysis JSON files are byte-identical independent executions.
`command.json` records the replay invocation; `SHA256SUMS-final` binds retained
regular review files and changed malformed fixtures. Unchanged fixture files
are symlinks to the immutable65-file native corpus, whose hashes are separately
bound in the reports. No earlier P1/Q1/Q2 evidence or repository owner was edited.

The bounded CV1 information is complete. The next coupled/free-surface stage
remains blocked on a predeclared conditioning/solver and approximation contract,
along with pressure stability/convergence evidence appropriate to the selected
basis pair. Solid boundary lifting, transport/GCL and full three-dimensional
water remain separate unqualified dependencies. Do not extend this result into
a production winner or silently repair small support by deleting coefficients.
