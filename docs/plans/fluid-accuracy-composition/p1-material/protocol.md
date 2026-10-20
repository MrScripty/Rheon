# Frozen proposed minimal gate v1

Operation: one bounded static closure of volume/first moment/gravity/boundary
selection after PR123. Production promotion stops after this gate, pass or fail;
see route-decision.md. No parameter sweep or rest trajectory. Source/protocol and
criteria must be independently reviewed and committed before tests/computation.

Exclusive writes: examples/material_partition_gate.rs,
examples/material_partition_gate/*, tests/material_partition_gate_contract.rs,
and this p1-material directory. PR121/123 sources/reports/archives and all src,
proofs, workflows, Cargo/dependencies, canonical plan and G1 remain untouched.

One fixed n=8 material partition on[0,1]m: ordered X_i=i/8; perturbed interior
X_i=(i+.1 sin(i*2.399963229728653))/8; endpoints0,1. Two sources:
uniform rho_ref=1.005*998.2 kg/m³,g=0; hydro rho_ref=998.2 A exp(g(1-X)/1024),
g=9.80665,A=1+100/(998.2*1024),p_top=100Pa. Thus this derivative fixture can stay
inside the original1% compression envelope at c32, unlike PR123's deliberately
pressurized1000Pa probe. Assert source endpoint extrema/J satisfy rho0<rho<=1.01rho0
and J>0 on EVERY state/displaced probe. Do not retune if refused.

For each partition/source evaluate x=X and x=X+.001sin(pi X): 8 total states,
not trajectories. Only x=X claims hydrostatic equilibrium or exact wall loads;
deformed states check the same nonlinear energy derivative and geometry identities.
Fixed endpoints in directional differences: v_i=.01sin(i*sqrt2),v_0=v_8=0;
eps=1e-5,1e-6,1e-7s. Source moments freeze once. Separate translate ALL interfaces
by±eps*.01m to test volume invariance and gravity/mass translation; translated
source labels stay fixed. Recompute from one declared potential without resets.

Fixed scales: L_ref=1m, U_ref=.01m/s, rho_ref_scale=998.2kg/m³;
P_ref=998.2*1024*.005Pa uniform,100+998.2*9.80665Pa hydro;
M_ref=rho_ref_scale*L_ref, E_ref=P_ref*L_ref, power_ref=P_ref*U_ref.
Report SI residuals alongside normalized values.

Fixed necessary criteria:
- total volume telescope error/L_ref<=1e-12, mass assembly defect/M_ref<=1e-12,
  first-moment alias error/(M_ref L_ref)<=1e-12, volume translation defect<=1e-12.
- work defect/power_ref<=1e-12; global sumF+g sumM residual/P_ref<=1e-12.
- fixed-end directional potential derivative versus -F dot v, and uniform
  translation derivative versus g sumM*.01, normalized by power_ref<=1e-6 at TWO
  consecutive eps values; retain every raw value, including failures.
  Force directional difference versus -H v uses fixed scale rho0*c²*U_ref/L_ref;
  normalized error<=1e-6 at two consecutive eps, in both probe directions.
- BASE uniform interior E0/P_ref<=1e-10 and individual endpoint load defect/P_ref
  <=1e-10. BASE hydro interior residual/(g M_a)<=1e-10, and each physical endpoint
  reaction defect/P_ref<=1e-10. No criterion is applied to deformed equilibrium.
- manufactured affine pressure p=100+1000x with exact endpoint traction lift:
  max residual/(1000 L_ref)<=1e-12 at all states, including endpoints.
- two declared diagnostic WRONG controls at BASE hydro: equal-half source mass
  replacing first moments, and pressure-only wall load. Each must exceed1e-8 on
  at least one partition under the corresponding equilibrium/wall normalization;
  otherwise the test has failed to discriminate the missing first moment/load.
- assembled H_II has positive Cholesky pivots and matches independent edge-energy
  quadratic form to1e-12 relative (scale max(E_ref,|quadratic|)); scaled row bound
  finite positive. Alternating cell pressure100+10(-1)^e yields nonzero interior
  force, with exact target20Pa max (operator diagnostic, not a new acoustic claim).

Store 8 summaries,72 node rows,64 element rows,24 directional+24 translation
probes. Source masses and first moments compare against a separate retained
high-precision integral oracle before a positive mathematical disposition:
|m_native-m_oracle|/(rho0 L_e)<=1e-12 and
|Q_native-Q_oracle|/(rho0 L_e²)<=1e-12 on every element, frozen now. These compare
actual native rounding plus series truncation; the real-tail bound alone does
not supply that verification. The native evaluator outcome stays conditional on
this independent source-integral gate and review; it does not self-accept A3.
A1 new source tests check identities and refusal semantics. A2 actual gate checks
frozen criteria. A3 independent pre/post numerical review checks both the
candidate and whether controls discriminate. Fixed thresholds are necessary
reference-only criteria; original production P-W gates remain wholly unchanged.
