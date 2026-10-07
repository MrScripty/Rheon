# Case47 preserved reference endpoint comparison

The [archived-data comparison](../../../evidence/case47-archived-endpoint-comparison-v1/RESULTS.md)
uses the original pressure_state/nonconstant/reversed paired DOP853 endpoints
and unchanged error metrics. At the original nominal t=0.1 pure-E2 endpoint,
full velocity lumped L2 error is 6.288313524301006e-05, third velocity error is
6.288070229134095e-05 and maximum cap-coordinate error is
2.2147831915675376e-08. The original strict reference-consistency check passes.

The native initial state is bitwise identical to the original native fixture;
reference canonical q/eta/third coefficients match exactly. Independent nodal
rounding differences and native clock rounding drift are reported separately.
No integration, trajectory, FD equation, instantaneous momentum equation or
native test/build ran. This one endpoint comparison establishes neither
new temporal order nor broader geometry qualification. All earlier failed
bands, refusals, memory evidence and proof limitations remain unchanged.
