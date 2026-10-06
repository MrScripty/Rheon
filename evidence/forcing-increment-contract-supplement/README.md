# Reviewed increment contract: additive coverage supplement

This packet adds the [specific capture and work-term specification](../../docs/research-book/implementation/forcing-increment-state-contract-supplement.md)
requested by review. All files at prior proposal `fd73aece1dc2a8c0308f9f6f80935276f86a08b5`
and verification `46a3d2986bc013ee1787f6af1d82b2d8c6ae50da` are unchanged.
All 6,959 base files are preserved; no production source, acceptance, initial
projection, equation, tolerance, trajectory or publication is changed.

`selected-capture.json` pins the exact pressure-forward `h=.00078125`, actual
accepted version 1/stamp id 131/time .00078125, then refused second step at
native norm `1.1895444217920825e-13`. It includes the original complete initial
and accepted publications, all three velocity components, geometry, masses,
accepted pressure, the distinct rejected candidate unknowns/pressure, and the
original terminal/16/32 post-window observations. Hashes and selected raw line
identities bind it to frozen source/evidence. The original native probe's dummy
qualification stamps are explicitly excluded as actual owner metadata.

`capture.py` reads these frozen files; it runs no new simulation. It also
independently checks the exact binary-input embedding defect: chart
reconstruction gap zero, max e0/e1 `5.551115123125783e-17`, projected rate norm
`7.359808400080196e-17`. These distinct measurements supply no cause, hidden
pass or repair for the refused residual.

`examples.py` adds two small exact examples and eight corruption controls.
The endpoint example has nonzero e1 and rN; **both** ledger-defect terms are
nonzero, with separate omission and sign-reversal rejections. The other uses
positive and negative nonzero GCL defects, checks their signed work, and rejects
omission/sign reversal. Normal and optimized actual results agree. These are
analytical identities, not native conservation acceptance or trajectories.

`comparison-policy.json` separates unchanged B0/native baseline, A0 exact
arithmetic on identical captured binary fields, and proposed E1 changed chart
solutions. E1 retains native qB/geometry/operator policy, specifies 106-bit
nearest-even scalar arithmetic/normalization, zero iterative refinement,
intermediate failures and one additional 65,536-byte fixed reservation with
61,888 bytes of simultaneous scalar slots. This is a proposed specification:
E1 is unimplemented/unexecuted, and cannot run until matching frozen source,
conformance and actual layout/live-memory receipts exist. No behavior-neutral
claim or tolerance/norm substitution is made. Alternate geometry is excluded.

Run the examples with ordinary and optimized Python, and `capture.py` for the
read-only selection. Run `verify.py` in either mode to check artifact hashes,
the full preserved base, exact results, eight capture corruption controls and
six policy corruption controls. In total the supplement exercises eight
analytical + eight capture + six policy rejections. The original eight examples
and eleven controls remain frozen and are not rewritten. Receipt production
hashes completed outputs only. Post-Git verification is additive and recorded
separately. No Rust/Lean build or new solver comparison is claimed by this packet.
