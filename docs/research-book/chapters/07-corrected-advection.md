# 7 Higher order transport with bounded failure

Linear interpolation is robust but dissipative. Let \(S_{\Delta t}\) denote a basic semi-Lagrangian step. A forward-backward correction computes

\[
q_f=S_{\Delta t}q,\qquad
q_b=S_{-\Delta t}q_f,\qquad
q_c=q_f+\tfrac12(q-q_b).
\]

This is the teaching companion's precise formula. Selle and collaborators analyze an unconditionally stable MacCormack construction [S6]. Several related formulas share that informal label; specify the exact correction and limiter rather than relying on the name.

## Overshoot and limiting

Forward and reverse interpolation average values. Their composition loses detail; adding half the estimated loss sharpens smooth fields. Subtraction can make effective weights negative, so the convex maximum principle no longer applies. Sharp interfaces can acquire negative concentration or excessive values.

The companion reverts an out-of-range corrected value to the forward value. Bounds come from the old departure interpolation stencil. Clamping is another policy with different truncation behavior. Neither automatically conserves scalar total. The limiter belongs in the method specification and saved configuration.

The inverse trace here reuses one velocity field. It does not invert a time-dependent physical trajectory. Near collisions, emitters and open boundaries, the paths may not represent reversible transport. Mark invalid traces and revert instead of treating arbitrary boundary values as error estimates.

## Executed refinement

The companion advects \(q(x)=\sin(2\pi x)\) once around a periodic unit interval at Courant number 0.5. At 32, 64, 128 and 256 cells, linear RMS errors are 0.187922, 0.101090, 0.052478 and 0.026743. Limited corrected errors are 0.037630, 0.009022, 0.003211 and 0.001022. These are executed measurements, not assumed order labels.

The corrected scheme retains more amplitude. Refinement ratios vary because the limiter activates near extrema. Do not claim uniform second-order convergence from the unbounded formula alone. Compute \(\log_2(E_h/E_{h/2})\), inspect limiter activity and include nonsmooth data. A different step-size sequence asks a different convergence question.

## Velocity and conservation

For material volume, use a bounded conservative finite-volume or geometric volume-of-fluid construction. A corrected sampler is an appearance tool unless additional conservation structure is established. Sharp plume images are not mass or momentum evidence.

Velocity components live on different grids and have no scalar positivity requirement. Componentwise limiting may suppress meaningful extrema. Projection removes divergence but cannot undo all momentum or energy errors from interpolation. Measure circulation, energy and vortex displacement as well as divergence. Prescribed-velocity tracer accuracy does not prove self-advected vortex accuracy.

## Buffer dependencies

Old data must survive both forward and backward passes; forward data survives correction. Overwriting it early creates an order-dependent in-place scheme. Per-component processing lowers peak memory only if velocity reconstruction does not read an already overwritten component. Immutable input views and separate output slices expose this dependency directly.

The implementation should count fallback samples and invalid backtraces. A method that reverts over most of the domain is functionally close to the first-order baseline, even if its configuration says high order. Report observed behavior rather than only the selected option.
