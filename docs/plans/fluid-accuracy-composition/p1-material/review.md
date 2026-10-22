# Independent review

Pre-freeze: read-only numerical reviewer /root/p1_numerical_review independently
derived the material potential, first-moment gravity, endpoint reaction, affine
traction lift, lumped inertia and positive incidence stiffness. It reviewed the
complete source, frozen scales, wrong controls and evaluator before ANY tests or
numerical experiment. It required dimensional independent m/Q oracle thresholds,
clear lumped-versus-consistent inertia, endpoint lifts, source-extrema branch
checks and separate translation probes; these are present before freeze.

Final disposition: no remaining mathematical/source/protocol blocker to the ONE
n8 static discriminator. Post-run independent source-integral and field review
remain required. It agrees that this is a substantial new material-mesh family,
not an SPH repair; recommendation is a bounded reference then grid Q1→Q2 through
the lead, with no automatic Q writes. No files edited or experiments run by the
reviewer at this phase. Production, nonlinear timestep, viscosity,3D,physical
and G1 claims are explicitly excluded.

Post-run A3: source-integral gate and frozen static necessary criteria PASS.
Independent80-digit Decimal closed-form source integration differs from native
mass by<=2.51615e-16 of rho0 L and first moment by<=1.57131e-16 of rho0 L², both
under the frozen1e-12 thresholds. Against physical source integrals, stored total
forces agree within3.28671e-10Pa and nodal masses within3.06287e-14kg/m².
The reviewer reconstructs stored force/geometry/moment fields and work, wall,
affine, stiffness and wrong-control diagnostics without importing/executing the
candidate. The full per-case/source/probe summary and input hashes are retained.

The8 smallest-step fixed-end Hessian differences remain failed native points.
Ideal Decimal increments give <=1.9821e-15 normalized error at1e-7s, while the
same analytic force with independently f64-rounded coordinate increments gives
1.25892e-6–1.36499e-6, reproducing the native1.21609e-6–1.39735e-6 failure scale.
No native rerun, smoothing, source or threshold change occurred. Raw displaced
force vectors were not recorded: no bitwise per-node reconstruction of the last
native subtraction is claimed. The two-consecutive-step rule passes unchanged.
Independent displaced extrema stay within1% (max compression.9722383%).

Retained independent reconstruction script SHA256:
b71ea21c20ab2276f3f5be27d1e54271d5f8c1798f1889d19221c172388502eb.
Summary SHA2565dcf05bb72eabf6ff2d8d145d238972dd29a1890af9877ae1ce7032dafce19a0;
full review SHA256c0e0950ae6ce88d39e4e7b0f2d7171dd79096d573ff20fa6769f11d694dea97d.
Root replay with explicit input/output directories reproduces the summary byte
for byte. This is independent code/reasoning/source verification, not an external
physical experiment, a Lean kernel result or3D water validation.

Final review endorses this reference-only closure and the concrete cost-based
Q1→Q2 recommendation. No remaining blocker to reporting the bounded result.
No P1 production method, nonlinear timestep or new Q/G1 ownership is admitted.
