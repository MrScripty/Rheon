# Independent native prospective S1 results

## Exact execution subjects and outcome

The actual fresh native run satisfies the independent **exported-field
S1-B/D/E/F/G checks** under the admitted prospective study. This is local CPU
execution and independent raw-state verification, not a Fourier forecast.
S1-A/C public Rust API/rollback evidence belongs to the native owner and parent;
this report does not newly execute or infer those contracts.

| Subject | Exact identity |
| --- | --- |
| Parent-admitted canonical plan, PR75 | `fc2270a85e38080d9943048aaf689d99545d3715` |
| Reviewed study/oracle policy, PR73 | `764ca2c7cba1fed123a7f9097fad819098479efc` |
| Clean reviewed capture verifier | `94a831770cb96f0c030b15565e2b5f7cbeb6c916` |
| Clean revised native source, PR74 | `5a049ed604b99817a428d10bbc0140732d3421d6` |
| Native source tree | `565318387acba7ef7612e3d5aa98a1042cd78867` |
| Actual executable SHA-256 | `62a319429206b25be911f616c973d7327a62675188a64157931489d8251b9646` |
| External `origin.json` SHA-256 | `61885d7df4cd0827a023af39b6905afeac7ac906b7e8e21c835ffc36072d9455` |
| `assessment.json` SHA-256 | `1adc1d3cfb1477a1bfdc30cf84c336cde91495368fda6175689d2113fd10f3e9` |
| Study ID | `s1-cell-average-acoustic-16-24-32-v2` |

Output directory `/tmp/rheon-s1-5a049ed-94a8317-prospective` is retained-protected
under validation ownership until parent checkpoint/disposition. No generated
fields, executable, log or receipt is committed. The separate repeat-verification
stdout is `/tmp/rheon-s1-5a049ed-94a8317-verify.json`; its bytes equal the original
assessment exactly. This later authored-report commit does not refresh the
capture's verifier identity. The origin binds the tracked-only archive, all
exported source files, toolchain/build recipe, executable, requested study,
actual native invocation and every raw artifact. Its external hash supplies
identity, not malicious-host attestation or a physical-correctness proof.

## Reproduction

Execute from clean verifier commit 94a8317, with clean native input checkout
`/workspace/Rheon-native-prospective` at exact 5a049ed. Rust is pinned to 1.92.0,
Python 3.12.14, NumPy 2.3.5. Capture uses a fresh tracked-only source export and
Cargo-reported release/core-only example executable. For a new run choose a
fresh external directory; never overwrite the retained evidence below.

```sh
source /workspace/rheon-setup/env.sh
python3 tools/verify_compressible.py capture \
  --source-repo /workspace/Rheon-native-prospective \
  --native-commit 5a049ed604b99817a428d10bbc0140732d3421d6 \
  --study-id s1-cell-average-acoustic-16-24-32-v2 \
  --output /tmp/rheon-s1-5a049ed-94a8317-prospective
python3 tools/verify_compressible.py verify \
  --run /tmp/rheon-s1-5a049ed-94a8317-prospective \
  --origin-sha256 61885d7df4cd0827a023af39b6905afeac7ac906b7e8e21c835ffc36072d9455 \
  --native-commit 5a049ed604b99817a428d10bbc0140732d3421d6
```

Both commands completed with status `satisfied`. The independent second command
used the external origin pin, recomputed artifact hashes and all numerical gates;
it did not rewrite the retained assessment. All eight cases export CFL 0.5 in
the native incident-face convention. Source comparison to the original 1b62ae3
shows no changes under `src` or `tests`; the native fixture example changed.
No criterion, CFL, method, amplitude, mode, norm, averaging or final time was
tuned during this execution.

## Measured numerical results

Acoustic error is the admitted volume-mean absolute density error divided by
1e-6, relative to analytic conservative cell averages at t=0.08.

| n | Cells per shortest coordinate wavelength | Normalized density L1 | Order to next mesh |
| --- | --- | --- | --- |
| 16 | 8 | 0.20347124808517567 | 0.7928470981796805 |
| 24 | 12 | 0.14753314663362563 | 0.8569980147372906 |
| 32 | 16 | 0.11529682712965958 | — |

Both errors decrease and both orders exceed the unchanged 0.6 threshold using
ratios 24/16 and 32/24. The eight-cell statement is coordinate resolution, not
wave-normal resolution or a universal convergence guarantee.

| n | First-mode relative speed error | First-mode amplitude ratio | Second-mode relative speed error | Second-mode amplitude ratio |
| --- | --- | --- | --- | --- |
| 16 | 0.007788383112755981 | 0.7256930721642721 | 0.040881359098354886 | 0.5303585242886318 |
| 24 | 0.003417660424172464 | 0.8073364551933938 | 0.01818127874983777 | 0.6531741039400679 |
| 32 | 0.001911304158718985 | 0.8516573568857542 | 0.010223938753778916 | 0.7259737398351822 |

The finest first-mode error is 0.1911304%, below 8%. Only that isolated speed
is gated. Both-mode attenuation/speed are disclosed; the admitted aggregate
criteria do not separately guarantee second-mode speed. Substantial first-order
attenuation remains visible despite satisfying this elementary coarse gate.

| Sod n | Interior density L1 | Interior pressure L1 |
| --- | --- | --- |
| 24 | 0.038436133232651666 | 0.05448703952865247 |
| 48 | 0.028884473581600432 | 0.04271663871527951 |
| 64 | 0.02823249991117378 | 0.037306094143767975 |

Density decreases and finest pressure L1 is below unchanged 0.04 in 0.2<x<0.8.
The conservative-average oracle's 8/16-point split-quadrature maximum difference
is `1.3322676295501878e-15` on each mesh; star-root bracket width is
`5.551115123125783e-17`. The direct analytic t=0 repair leaves the original
16-epsilon allowance unchanged.

All 179 retained frames (1746044 cell-state rows) are finite/admissible. Every
one of 171 accepted steps was checked. Worst normalized five-component weighted
conservation drift is `1.6148698540002275e-16`, below fixture-sized `2e-13`.
The separately labelled S1-D differential maximum is `5.551115123125783e-17`.
Acoustic accepted steps are 19/28/37; Sod steps are 15/30/40; uniform and
differential each take one. Finest native managed capacity is 4194304 bytes;
this is the exported owned-payload observation, not total process memory.

The sampled local nonlinear acoustic PDE defect remains
`3.9722182496520564e-11`, with amplitude-doubling ratio `4.000006313929694`.
This observation is not a global solution-error bound, and the finite-amplitude
linear acoustic reference is not claimed to be an exact nonlinear Euler solution.

## Review and compatibility evidence

Independent read-only review found no defects in the capture-selector follow-up.
All 43 verifier tests passed normally and under `-O` in worker and reviewer,
including real original/prospective argument forwarding, study mismatch before
numerical assessment, all inherited numerical/layout/time negatives and the
unchanged order gate. Python compilation and scoped diff checks passed.
The numerical assessment code is unchanged from reviewed 764ca2c.

At exact verifier 94a8317, hosted [push workflow
38528609431](https://github.com/MrScripty/Rheon/actions/runs/38528609431) and
[PR workflow 38528615652](https://github.com/MrScripty/Rheon/actions/runs/38528615652)
passed. These bounded dense3D compatibility jobs do not execute S1. CodeRabbit
skipped the draft; that status supplies no independent review evidence.

## Preserved failures and limits

Original origin hashes are still exactly
`7b53940fd168ae26e9cd0b29ffde0c191dab5b0d01a5be2953de21eae5c4dbec` and
`e7c098d13eb19fcc54abb37436c0f0e0ffc7e851cd30487e0a590a712c1866a7`, with no
accepted assessment in either directory. Their failures remain unchanged as
recorded in [diagnosis](native-s1-diagnosis.md). This successful prospective
study does not retroactively pass 8/12/20.

Current acceptance covers bounded periodic inviscid ideal-gas numerical fixtures.
No experimental agreement, arbitrary exterior flow, viscosity/turbulence,
moving-body force/moment, passenger-jet/submarine or training-label qualification
follows. Parent owns remaining source admission, combined public-API evidence,
integration and compatible checkpoint coordination. No S2, release tags, runtime
packaging/distribution, GPU or remote-worker result is included.
