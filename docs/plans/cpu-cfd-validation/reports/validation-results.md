# Executed validation and remaining native gate

This is an authored result summary; generated CSV/fields/logs/receipts remain
external under the repository output policy. No simulated frames are committed.

## Elementary CPU acceptance

Source commit `ef4f8e4a59d119533f0e41cf89fa11f3f94ab759`, clean working tree:

```sh
python3 validation/cpu-cfd/run.py --output /tmp/rheon-cpu-cfd-ef4f8e4-clean
```

Receipt status `accepted_elementary_numerics`. Raw CSV SHA-256
`6d14d7af98e2536401fc35492fdf6a3681ed2e8e7569964c01f69361873c71b8`.
All 12 points executed on local CPU through public Rust APIs, Rust 1.92.0.

| Study | Errors / observed orders | Claim limit |
| --- | --- | --- |
| Pressure Jacobi, n=6/12/24 | L∞ Pa 0.0417498206329 / 0.0111707287051 / 0.0028399168010; orders 1.9020467 / 1.9758027 | continuum cosine pressure in closed unit cube |
| Pressure SGS, same meshes | same displayed errors; orders 1.9020467 / 1.9758027 | same PDE and independent reference; no general speedup claim |
| Viscosity, n=8/16/32 | continuum velocity L∞ m/s 0.00105604045 / 0.000268689323 / 0.0000678954732; orders 1.9746542 / 1.9845517 | extruded free-slip Stokes mode, not full nonlinear flow |
| Viscosity fixed n=12, 16/32/64 steps | semidiscrete velocity L∞ m/s 0.000271635618 / 0.000135498609 / 0.0000676685231; orders 1.0033946 / 1.0017212 | time error relative to explicitly labelled semidiscrete reference |

Worst actual projected f32 divergence <4.41e-12 s⁻¹; worst all-step viscous
divergence <7.75e-6 s⁻¹. Normal-wall velocity, net boundary flux and positive
per-step energy-growth maxima were zero. Pressure residual <5.38e-13 in the
integrated operator units. Smaller dt at fixed viscous mesh moves toward its
spatial floor; continuum error and semidiscrete time error remain separate.

Meaningful checks passed:

- `cargo fmt --manifest-path validation/cpu-cfd/Cargo.toml --check`
- `cargo clippy --manifest-path validation/cpu-cfd/Cargo.toml --locked --all-targets -- -D warnings`
- `cargo test --manifest-path validation/cpu-cfd/Cargo.toml --locked`: 1 independent diagnostic test.
- `python3 -m unittest discover -s validation/cpu-cfd -v`: 12 tests, including exact rejection diagnostics and failed/timeout/source-change receipt behavior.
- `cargo test --locked --no-default-features --test pressure_contract --test viscosity_contract`: 10 pressure + 8 viscosity tests.
- `git diff --check`: passed for scoped changes.

No production source/Cargo/API change was made, so default/parallel/desktop
matrices are not claimed as newly executed by this worker. No Lean proof change
or hosted CI success is inferred from these local tests.

## Independent S1 verifier readiness

`tools/verify_compressible.py` and `tools/test_compressible_verifier.py` implement
the [proposed export/origin contract](s1-export-proposal.md). With existing pinned
NumPy 2.3.5, the following passed 19 tests each:

```sh
python3 -m unittest discover -s tools -p test_compressible_verifier.py -v
python3 -O -m unittest discover -s tools -p test_compressible_verifier.py -v
python3 -m py_compile tools/verify_compressible.py tools/test_compressible_verifier.py
```

This verifies analytic integrals, indexing, policy/origin diagnostics and the
sampled corruptions only. Synthetic positive probes are explicitly not native
simulation or S1 acceptance. Root/quadrature/linearization observations and
gamma_512 roundoff policy are in the proposal; thresholds remain those in
reviewed solver plan 67baa105. Native capture uses Cargo JSON's reported exact
example executable, fresh build/output, clean source commit/tree and artifact
hashes; the actual S1 path was subsequently executed as recorded below. Origin
pin checks prove identity against the external pin, not a
cryptographic authenticity claim.

## Native capture and outstanding action

Native source 1b62ae378c2aaa01327055f121ebc0d869f2c4ac is now available.
A clean reviewed-verifier aea43a5 capture built and ran it; original origin pin
is `7b53940fd168ae26e9cd0b29ffde0c191dab5b0d01a5be2953de21eae5c4dbec`.
The preserved original assessment failed Sod initial rounding. A direct analytic
t=0 repair leaves the tolerance unchanged and passes all 24 tests normally/-O.
The corrected assessment still fails frozen acoustic order (0.5614384 < 0.6),
independently reproduced by a linearized Fourier model. No accepted S1 receipt.
See [exact-source diagnosis](native-s1-diagnosis.md) for actual metrics, proposed
parent replan, preserved evidence and executable diagnostic source.

Fresh clean-verifier 6454acf capture and externally pinned read-only verify both
failed `acoustic/order-below-0.6`; origin SHA-256 is
`e7c098d13eb19fcc54abb37436c0f0e0ffc7e851cd30487e0a590a712c1866a7`.
Both failed runs are retained unchanged.

**Pending, not denied:** parent admission of a principled prospective
resolution/method revision before A6 acceptance.
S1-A/C remain the solver worker's public Rust API/rollback checks. Parent owns
checkpoint integration. No native source/exporter/core was changed here.

**Runtime disposition:** parent permits the existing NumPy as a separately
installed validation tool for this task; its bundled GCC/quadmath terms remain
disclosed. No runtime packaging/distribution is proposed or authorized; report
any future distribution before proceeding. No GPL-free runtime claim is made,
and Rheon's Apache-2.0 source license is unchanged.

## Hosted CI inspection at pinned PR head

At PR73 head 4621d69747740a950a90d78755fe3bb4e220efe4, the
[pull-request workflow](https://github.com/MrScripty/Rheon/actions/runs/38523280618)
and [push workflow](https://github.com/MrScripty/Rheon/actions/runs/38523275446)
both completed successfully. They run the existing bounded dense3D Rust and
normal/-O Python contracts. They do not execute the new S1 verifier, and cannot
substitute for its named local tests or the pending actual-native gate.
CodeRabbit's status is successful but its comment explicitly says the draft PR
was not reviewed; no independent code-review claim follows from that status.

## Independent review follow-up

The requested separate read-only review found and confirmed closure of three
provenance defects: sandbox marker handling, ignored compilation inputs and a
build-specific Rust flags override. Capture now builds an isolated tracked-only
export and refuses unsupported build configuration. Final 23 tests passed
normally and under `-O`; independent review confirmed all findings resolved.
The tiny native capture-protocol test writes no CFD state and supplies no S1
numerical acceptance. See [review](independent-review.md) for exact dispositions.
The admitted first-mode speed gate's mode-two limitation is reported/tested;
no numerical threshold or native format changed.

External experimental geometry/data rights/access and arbitrary moving-body
capability remain deferred as recorded in [staged validation](staged-validation.md).
No aircraft/submarine, experimental agreement or training-label acceptance;
no GPU, remote-worker or release-tag claim.
