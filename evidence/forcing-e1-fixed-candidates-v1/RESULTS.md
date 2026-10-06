# Two terminal E1 candidates: fixed-state diagnosis

The two original refusals remain refusals. Four fresh native equation evaluations (orders 16 and 32 at each frozen final authorized unknown) construct and advance zero accepted owners, perform zero controller iterations, and publish nothing. The order-16 norms reproduce the archived terminal norms bit for bit. Each original refusal followed seven corrections and 50 counted equations at h = 0.00078125; these budgets and the 1e-13 target remain unchanged. The third-component solve was never reached in the refused trajectories and is not invented here.

| Order-16 comparison | Case 41, initial forward | Case 47, pressure reversed |
|---|---:|---:|
| Actual native sequential norm | 1.0724458777318338e-13 | 1.3144896847477506e-13 |
| Exact final accumulation of rounded contributions | 1.0724458501434415e-13 | 1.314490211095982e-13 |
| Exact products/sums, stored stable equation | 1.0724469898375466e-13 | 1.3145195403802802e-13 |
| Exact stored direct equation | 1.394392931663723e-13 | 1.2947132889675134e-13 |
| Exact native force assembly and captured flux accumulation | 1.0701607453865519e-13 | 1.314368160164316e-13 |
| Actual 106-bit chart, inertia-only sensitivity | 1.047576648258133e-13 | 1.2563663896443561e-13 |
| Exact chart with stored knowns, inertia-only sensitivity | 1.047576648258133e-13 | 1.2563663896443564e-13 |
| Exact chart with continuous knowns, inertia-only sensitivity | 8.18553971748197e-14 | 6.205423361130985e-14 |
| Continuous knowns, consistent fixed-operator endpoint substitution | 8.16607051951688e-14 | 6.194941900524401e-14 |

All twelve predeclared variants, their full 22-component vectors and exact rational squared norms are in `analysis-80-normal.json`. Normal/optimized readers and 80/120-digit norm evaluation agree; classifications use exact rational comparisons against the original binary64 target squared. `native_norm` is the actual sequential Rust norm. The variant named `native_sequential` displays the exact norm of its already-rounded components, which can differ in the final displayed bit.

The native replay reconstructs all 22 planar equation components, assembled strain/pressure forces and captured flux quadrature bit for bit. The exact stored stable/direct comparison verifies

    stable - direct = R^T diag(m1) (e1 - e0) / h,  e = R z - U_stored.

Exact arithmetic here is over the captured binary inputs, not a continuum operator. The selected 15 chart equations solve exactly; other rows of the captured rounded geometry need not be exactly dependent. Their maximum residuals are about 1e-15. The accepted start chart is reproduced exactly. The actual 106-bit endpoint differs from the exact chart with the same rounded knowns by at most 1.7e-32; simply increasing factor precision therefore does not address the observed fixed-candidate obstruction. Rounded affine known increments differ from exact eta + h*a by roughly 1e-13 after division by h.

Inertia-only sensitivities retain stored donor velocity and forces, so they are not coherent acceptance candidates. The final fixed-operator variants substitute velocity consistently in every planar term, including forces and donors, while holding captured geometry, masses and flux integration fixed. They remain counterfactual endpoints: no native acceptance, physical acceptance gates, third solve, trajectory continuation or new integration was performed. Order 32 gives the same exact above/below-target classifications.

No validated trajectory remedy or arithmetic floor is established. The narrow next candidate direction supported by these comparisons is to defer rounding of affine known coordinates and carry ephemeral chart increments into inertia assembly, reusing the existing workspace. It must retain stored accepted endpoints and physical gates as authorities and receive coherent native, trajectory and fresh memory qualification before adoption. This packet implements no such repair and makes no claim of global minimality.

Native source: `00d7de13b82396836e8f1efad6e3049034f5c2e7`, tree `e6487beb81407818a0280513fd2a1bbbc986d883`. Passed preflight, readers and execution driver were frozen at `4ee70e4587cb4ce5f5f164bcfea59427905e1565`, tree `c1254024e893049c7290341916350b56ca262d8c`, before the E1 capture. Actual ELF SHA-256: `b07f3c9abeb0bb2cde6fa8c7d0b4fe816795f51fe6492457e2d0ca6a81cd1b8c`. Formatting, Rust compilation, Clippy with denied warnings, 23 native scalar groups, 36 exact scalar probes and fresh linked memory auditing passed. The additional-memory bound is 65,616 bytes against 67,584, leaving 1,968; the cap is unchanged. This allowance is not whole-process RSS or external evidence storage.

`arithmetic.svg` renders the captured comparisons. `receipt.json`, `verify.py` and subsequent `post-git-verification.json` bind this packet and the separate existing-endpoint geometry diagnosis. Original source, accepted evidence, Jacobi fixtures, research equations and Lean limitations are preserved. No production adoption, threshold relaxation, extra Newton iterations, persistent tails, memory expansion or liquid-simulator claim is made.
