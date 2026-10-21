# Two independent necessary properties

Let fixed m_i>0, fixed h, sorted sites y, smooth fixed symmetric support and
closed walls b<t. K is PR121's exact tangential marginal. Set
 S_i=sum_j m_j K(y_i-y_j), gamma_i=F(y_i-b)-F(y_i-t),
 rho_i=(S_i+C_i)/gamma_i, V_i=m_i/rho_i, q_i=p_i/rho_i².
Choose C_i=rho_initial_i gamma_initial_i-S_initial_i ONCE.
Negative C is permitted only while S+C>0; no positivity claim beyond measured
probe configurations is made. This is a new diagnostic map, not a reproduction
of Mayrhofer's k=0 continuity or a claim of offset-free consistency.

With gamma'_i=K(y_i-b)-K(y_i-t), exact directional continuity is
 rho_dot_i=[sum_j m_j(v_i-v_j)K'_ij-rho_i v_i gamma'_i]/gamma_i.
Differentiating U=sum_i m_i e(rho_i) gives
 F_i=-m_i sum_j m_j(q_i/gamma_i+q_j/gamma_j)K'_ij
     +m_i p_i gamma'_i/(rho_i gamma_i).
The pair force is antisymmetric. Wall reactions are
 R_b=sum_i m_i p_i/(rho_i gamma_i)*[-K(y_i-b)],
 R_t=sum_i m_i p_i/(rho_i gamma_i)*K(y_i-t).
Thus sumF+R_b+R_t=0 and sum_i[v_i F_i+m_i q_i rho_dot_i]=0.
For moving walls their shape derivatives enter density rate and their velocities
enter work. This probe uses stationary walls only. No pressure work from a
fictitious wall ghost velocity is used. Compared with the former k=0 scheme,
matching this map changes BOTH volume coefficients and gamma surface terms;
merely halving a surface term while retaining the old continuity is incoherent.

For ANY positive differentiable volume map rho_i=m_i/V_i,
 dU=-sum_i p_i dV_i, F_k=sum_i p_i partial_k V_i.
At uniform positive p*, F_k=p* partial_k(sum_i V_i).
Exact work therefore does not imply zero E0 force: the latter requires an exact
position-independent volume partition (or an equivalent identity). Calibrated
offsets enforce density values, not derivatives or hydrostatic stationarity.

Clipped 1D Voronoi reference: faces f_0=b,f_n=t,
 f_j=(y_{j-1}+y_j)/2, V_i=f_{i+1}-f_i. Strict ordering/interior sites makes V>0;
sumV=t-b identically. F_k=sum_i p_i partial_k V_i, R_b=-p_0,R_t=p_{n-1}.
Interior F_k=(p_{k-1}-p_{k+1})/2; endpoints F_0=(p_0-p_1)/2,
 F_last=(p_penultimate-p_last)/2. Continuity rho_dot=-rho Vdot/V.
Constant pressure cancels exactly. Linear point-pressure has exact interior
force -p' V_i, including irregular spacing, but this does NOT establish wall
consistency or the consistency of cell-mean hydro pressures with site forces.
Alternating pressure is an interior null mode: no undisclosed cure is allowed.

The linear EOS and e'=p/rho² are inherited unchanged from PR121. Exact exponential
hydrostatics, the c=32 compressibility/Mach restrictions and unqualified production
acoustic dt remain unchanged. No new formal theorem is claimed. Existing Lean
conditional pair/work lemmas remain valid under their original hypotheses; they
do not establish these maps' consistency, positivity or acoustic spectrum.

## Primary basis and scope

[Mayrhofer et al. 2013](https://arxiv.org/abs/1304.3692), equations 8–11 and
37–38, motivate the gamma adjoint issue; the map here is independently derived.
[Hess and Springel 2010](https://arxiv.org/abs/0912.0629), sections 2.1–2.3,
derive Lagrangian pressure from volume derivatives. Their section 5.3 discusses
the Nyquist pressure mode. Their ideal-gas applications do not validate water.
[Dehnen and Aly 2012](https://arxiv.org/abs/1204.2471) distinguish kernel sampling
consistency from conservation. We do not transplant astrophysical acceptance.
[Frontiere et al. 2017](https://arxiv.org/abs/1605.00725), Appendix B, shows that
conservative reproducing-kernel momentum need not reproduce linear pressure.
An RK gradient cannot simply replace this force while keeping its work proof.
