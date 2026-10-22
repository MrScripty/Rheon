# CV1 qualification record

Current disposition: CV1 operator/physical-trace reference accepted for the fixed
four-fixture corpus. Native and independent exact reconstruction pass, as do
normal/-O/-OO evaluator contracts. No coupled pressure solve, timestep, uniform
cut stability, PDE convergence or production water admission.

Frozen native source: 6d99e7794a6aa69f5886bb15cc662e40ea43cd40.
Binary SHA256: 377edcf4142104e44ef16d1d884d8d4c8a6df8e32ccdf8f1c18507e4871caf05.
Independent oracle freeze SHA256:
38d4b75a5bbcdc7991725aef091f5e2347f671fe0b19ce51a34e8a4ce1095d55.
The oracle was frozen at03:54:12UTC before the native campaign03:54:36UTC.
Native execution is release CPU on the retained Rust toolchain; independent
reconstruction uses Python Fraction/NumPy on CPU.

The single fresh campaign exited0 and completed the immutable four cases:

| Fixture | Active velocities | Positive pressures | Native maximum fixed discrepancy | Active with zero old dual |
|---|---:|---:|---:|---:|
| rectangle | 40 | 16 | 4.440892098500626e-15 | 16 |
| square-17 | 44 | 13 | 9.769962616701378e-15 | 18 |
| disk-64 | 104 | 36 | 6.750155989720952e-14 | 28 |
| corner-triangle | 8 | 1 | 1.110223024625157e-16 | 6 |

All native fixed gates use the prospective1e-10 budget. The corner liquid is
strictly inside one primal cell: Cartesian aperture is zero, yet eight basis
restrictions have positive mass. Six of their old h-width dual volumes are zero.
This is evidence for the declared wider basis support, not an aperture/default
extension policy for the earlier finite-difference method.

The full2Pa pressure-load coefficient is2Pa in every case to native rounding;
this is an affine nodal load discriminator, not PDE pressure recovery. Source
geometry, all positive halfcell pieces, quadrature, basis coefficients, matrices,
physical/grid traces, moments and nodal full/omitted-pressure loads are retained
as65 CSVfiles. No source, fixture, threshold or geometry was changed after data.

Final frozen source passed10 debug and10 release contracts, strict Clippy,
formatting and locked release build. Earlier pre-driver5d352e0 receipts are
retained separately and were never used for a native campaign. The final driver
continues all immutable cases and retains each refusal before nonzero exit.

Evidence owner: external `rheon-q2-cut-method-evidence`, including immutable
source archives, exact command/exit/time receipts, binary/source hashes,
prospectively frozen independent analyzer and the single native-v1 corpus.
Archive identity and custody are recorded in evidence-manifest.md.

Independent exact reconstruction confirms all640 common partition pieces,
all196 active coefficients and all43,724 dense M/B/K/flux entries. Exact pressure
ranks are16/16,13/13,36/36,1/1 in protocol order, with no pressure gauge removed.
Both exact-integral-rounded and native fullmass Cholesky succeed. The largest
independent normalized discrepancy is7.849623728795052e-15 (disk viscous work).
All65 retained native CSV hashes remain unchanged after independent evaluation.

| Fixture | Full M condition diagnostic | B condition diagnostic | Minimum native M diagonal |
|---|---:|---:|---:|
| rectangle | 4.021710993853908e4 | 4.976132295902294e1 | 1.960419555555552e-6 |
| square-17 | 6.533161483915378e11 | 8.114511387366868e3 | 1.701396215853621e-14 |
| disk-64 | 1.411514371285607e4 | 5.139203430543827e1 | 4.903299629761844e-6 |
| corner-triangle | 1.003311845462681e5 | 1 | 2.228228999999997e-5 |

The square's conditioning is a material next-gate finding. Conditions were
prospective diagnostics, so no criterion changed after data. A new bounded
conditioning/solver protocol is required before any coupled-step qualification;
CV1 does not supply pressure/velocity recovery, a timestep or convergence proof.

The frozen analyzer reconstructs all four cases in normal Python/-O/-OO with
byte-identical reports. Its45 subprocess checks include14 prospectively declared
malformed fixtures in all three modes; every mutant refuses intentionally and
creates no success artifact. Inputs are hashed before/after. This tests evaluator
custody only and executes no additional native campaign.

The primary-source-supported continuum derivation and independent review resolve
the declared basis/support/physical-trace reference gate. They do not establish
the particular shiftedQ1/P0 pair's uniform inf-sup stability or oblique no-slip
lifting. Earlier aligned, geometry, particle and transport failures remain intact.
No new Lean theorem was needed for this builder; the existing diagonal-mass proof
is not relabelled as a full-mass/IEEE implementation proof.
