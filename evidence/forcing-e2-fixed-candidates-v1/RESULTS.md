# E2: one fixed candidate below target, one still above

The authorized chart-only experiment completed exactly four fixed equations. Case 41 remains above the original Newton target at both quadrature orders. Case 47 is below it at both orders. Both paired native planar qualifiers passed. These are fixed-candidate observations, not completed public steps or repaired trajectories. No Newton correction, accepted owner construction/advance, third solve, publication or new reference/trajectory integration occurred.

| Frozen case | Order | Original E1 native norm | E2 native stable norm | Original 1e-13 Newton test | E2 native direct norm |
|---|---:|---:|---:|---|---:|
| 41, initial forward | 16 | 1.0724458777318338e-13 | 1.0688520423549861e-13 | above | 1.0732717109648156e-13 |
| 41, initial forward | 32 | 1.0726596703982124e-13 | 1.069176205977044e-13 | above | 1.0736058954327475e-13 |
| 47, pressure reversed | 16 | 1.3144896847477506e-13 | 8.486535289007615e-14 | below | 1.077051674641148e-13 |
| 47, pressure reversed | 32 | 1.3142230998091432e-13 | 8.485631545099067e-14 | below | 1.0773050087925686e-13 |

The original Newton graph tests stable norm at 1e-13. The native planar qualifier retains both stable/direct physical limits at 1e-11; case 47's direct norm exceeds the Newton target but satisfies the unchanged direct physical gate. This distinction is inherited source behavior, not a relaxed limit. A stable planar pass cannot replace the unexecuted third block or final full publication gates.

Before E2, the actual frozen E1 binary reproduced every numerical JSON byte in the two original fixed captures, including both original order-16 refusals. Only the nondeterministic libtest wall-time footer is excluded from byte comparison. Baseline runtime journals and original trajectories are unchanged.

## What changed and what is authoritative

The candidate body differs from the accepted E1 private source in exactly one expression: its chart hook supplies eta0/unknown/time instead of already-rounded known targets. The existing guarded 106-bit kernel computes working affine values in existing coordinate slots, then uses the existing increment factorization. Native q, eta, all public known coefficients, masses and native stored-difference inertia retain their original operation graphs. After RN64 of dependent coefficients, every point, sign partition, positive/negative transfer, force, rate and planar gate is recomputed from the actual stored candidate. Baseline captures supply comparisons, never qualification fluxes.

Both endpoints change 12 dependent coefficient bit patterns and 12 nodal velocity-component values; public knowns/q/eta/mass have zero changed bits. Force components change (22 for case 41; 23 for case 47), and both directional transfer arrays change at many faces. These are changed stored candidates at the same frozen unknowns, not identical-field summation fixes. The complete native equation, assembled forces and actual quadrature transfers replay bit for bit from those changed fields.

The working latent endpoint is discarded for qualification. For deltaB stored and deltaL working, rho=deltaB-deltaL is retained in the exact comparison:

    stored stable rate - latent inertia-only rate = R^T diag(m1) R rho / h.

The exact reconciliation contribution has norm 3.6339084548091394e-14 for case 41 and 4.3752530225227224e-14 for case 47, at both orders. Omitting it would change the qualified finite equation. Case 41's isolated latent-inertia norm is below target, while its actual stored native and exact stored-input norms remain above; this directly demonstrates why the counterfactual must not qualify the endpoint.

Exact complete stored-input stable norms are 1.0688253531784157e-13 / 1.069149552142577e-13 for case 41 and 8.486791219061359e-14 / 8.485891782470648e-14 for case 47 (orders 16/32). Exact rational squared-norm classifications agree with native stable classifications. The actual H solution differs from a rational chart solve using its actual H knowns by at most 1.7e-32; this is measured fixed-case factorization error, not a global bound. Native scalar affine parts match independent per-operation rational rounding at every captured point.

The paired native qualifiers report:

| Case | Ledger error | Unchanged work allowance | 16/32 quadrature difference | Full constraints | Maximum GCL defect |
|---|---:|---:|---:|---:|---:|
| 41 | 5.936006894358137e-17 | 4.566143727135339e-14 | 3.6283450389026006e-20 | 2.253941120116537e-15 | 1.2541759927696166e-16 |
| 47 | 5.117434254131581e-17 | 4.529933410043764e-14 | 9.867933835512599e-20 | 2.2743321677722635e-15 | 1.8624878081582776e-16 |

Independent exact nodal assembly also checks the full direct projection, changing-mass work identity and both stored embedding-defect work terms for all four equations. This uses captured rounded nodal force pairing; it is not a claim that exact force pairing equals the separately rounded native strain/pressure powers. No discrepancy is added as an allowance or physical loss.

## Qualification, identities and limits

Native source: `cbeaa72b403cb1f3af4f95727cc29ec660c99ca3`, tree `2539d960f279d5f50aa213f3374651d7c51b5957`. Passed preflight/readers: `c2ee6acc8a783161a844e3dee0566af71732628d`, tree `a59508d87e97e2fb97f5d52d34d187924bbf0d92`, frozen and pushed before execution. Actual ELF SHA-256: `f7a85747e4dfc3cdd8815554fdc5f12a81bd8d875596cc4ff4fbacbb8f0f3983`. Preflight receipt SHA-256: `11727197c95aa464e63c9a413ac975d62cbde6b78a20263d2fda7e66f3c603a8`.

Rust formatting, release compilation and Clippy with denied warnings passed. There are 23 native scalar groups, 36 exact primitive probes and six actual affine scalar probes, including checked subnormal/underflow/overflow refusal. Fresh actual linked kernel, capture, planar-gate, public and constructor auditing bounds additional live storage at **65,840 bytes**, against **67,584**, with **1,744 bytes** remaining. Workspace is 62,096 bytes plus its 8-byte loan descriptor; kernel peak is 1,696. No baseline shrink/slack credit is used. The allowance is not whole-process RSS or external evidence storage.

Four actual analysis commands pass at 80/120 digits in normal/optimized modes, with complete outcome parity. Read-only nodal work identities also agree in normal/optimized modes. `fixed-equations.svg` is preserved; `fixed-equations-v2.svg` moves the legend outside the plotted bars, changing presentation only. No additional native execution produces either plot.

`verify.py`, `receipt.json` and subsequent post-Git bindings close the actual source, preflight, baseline, E2 journals, readers, identities and render. The two original runtime refusals and all eight original geometry-band failures remain unresolved. Case 41 is still refused at these frozen unknowns; case 47 only has fixed planar eligibility. No trajectory rerun, extra iteration, threshold/metric change, checkpoint, retained tail, production adoption, arithmetic-floor proof or general liquid-simulator qualification follows.
