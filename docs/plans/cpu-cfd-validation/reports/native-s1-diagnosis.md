# Native S1 failure diagnosis

## Exact source and preserved original outcome

Native producer: `1b62ae378c2aaa01327055f121ebc0d869f2c4ac` (draft PR74).
Verifier: clean `aea43a54d7c8e51b897b21f7e8119c2a22cd1605` (draft PR73).
The reviewed tracked-only capture built and ran the native example successfully:

```sh
source /workspace/rheon-setup/env.sh
python3 tools/verify_compressible.py capture \
  --source-repo /workspace/Rheon-native-s1 \
  --native-commit 1b62ae378c2aaa01327055f121ebc0d869f2c4ac \
  --output /tmp/rheon-s1-1b62ae3-aea43a5-original
```

The original directory is retained unchanged. Its `origin.json` SHA-256 is
`7b53940fd168ae26e9cd0b29ffde0c191dab5b0d01a5be2953de21eae5c4dbec`.
`assessment-failed.json` records `sod/initial-cell-average`; no accepted receipt
was created. Output contains 142 frames, 241596 conservative-state rows and
134 accepted steps. Generated frames/logs/receipts remain outside Git under the
repository output policy. No source admission or prior receipt was refreshed.

## Constant-state oracle repair

At t=0 the Sod solution is two constant conservative states. For cell i of n,
the left-state fraction is `clip(n/2-i, 0, 1)`. Direct integration gives
`rho = 0.125 + 0.875*f`, zero momentum, and
`E = (0.1 + 0.9*f)/(gamma-1)`. This also handles an odd-n cell crossing the jump.
Gauss integration adds no information here and accumulated avoidable endpoint
and weight rounding: at n=48 the old reference energy differed from the native
constant by `4.884981308350689e-15`, beyond the unchanged `16*epsilon`
allowance (`3.552713678800501e-15`). Direct integration differs by exactly zero
at all native initial states n=24/48/64. This repairs the oracle; it widens no
numerical allowance. Positive-time split quadrature remains unchanged.

The focused test checks exact native constant values and an odd-n mixed cell.
All 24 tool tests passed normally and under `-O`. Separate read-only review
confirmed the formula, unchanged tolerance and focused tests. Applying the
corrected oracle diagnostically to the preserved fields reaches the independent
failure `acoustic/order-below-0.6`; this does not rewrite their origin record.

## Causal acoustic evidence

Keep the admitted cell-average continuum reference and both frozen order gates
strictly greater than 0.6. Native normalized density L1 and an independently
derived linearized LLF Fourier symbol give:

| n | Native error | Symbol error | Maximum native-minus-symbol density |
| --- | --- | --- | --- |
| 8 | 0.31274301614224453 | 0.312742938268773 | 2.883e-13 |
| 12 | 0.2490710346665802 | 0.24907096311347726 | 4.028e-13 |
| 20 | 0.1712388220900807 | 0.17123876906860458 | 5.838e-13 |

Native orders are `0.5614384036983551`, `0.7334771702681024`; symbol orders are
`0.5614384981046086`, `0.7334772140301444`. Maximum difference across all five
components is `2.153e-12`. A separate reviewer reproduced the symbol errors
with a scalar eigenmode calculation. The observed coarse-grid order failure
is consistent with the stated first-order scheme, rather than evidence of a
native indexing/EOS defect. This comparison does not prove every native path.

For rest-state rho=p=1 and c=sqrt(gamma), linearize the independent Euler flux
Jacobian A_d. Its nonzero entries are `A_d[0,d+1]=1`,
`A_d[d+1,4]=gamma-1`, `A_d[4,d+1]=gamma/(gamma-1)`. On a periodic uniform grid,

```text
L_h(k) = -c sum_d ((1-cos(theta_d))/h_d) I
         - i sum_d (sin(theta_d)/h_d) A_d
theta_d = 2*pi*k_d*h_d
Uhat_final = product_steps (I + dt_step*L_h) Uhat_initial
```

The calculation uses every actual native accepted dt and the exact analytic
cell-average sinc factors. It imports no native numerical flux. A diagnostic
point-sampled initializer/reference with the same dt sequence predicts orders
`0.692202099134634`, `0.7892144555742635`. Thus the change from point samples to
conservative cell averages materially changes the finite-grid slope. Point
samples are not an acceptable replacement for the admitted reference.
At n=8, mode (1,-1,2) has only four cells per shortest wavelength; the largest
modified-gradient relative defect `1-sin(theta)/theta` is 36.34%, falling to
17.30% at n=12 and 6.45% at n=20. This is a resolution diagnosis, not an error
bound or a reason to relax the threshold.

## Parent decision required before a new numerical admission

A prospective resolution replan could retain the original failed 8/12/20 study
and admit 16/24/32, using at least eight cells per shortest wavelength and a
maximum modified-gradient defect below 10% as a new resolution heuristic.
Keep the same cell averages, amplitudes, modes, final time, norm, CFL and >0.6
threshold. The linearized model forecasts errors 0.2034711687 / 0.1475330865 /
0.1152967813 and orders 0.7928471406 / 0.8569979812. These are forecasts,
not executed native cases or an order theorem. The heuristic and family require
explicit parent admission before edits. Alternatively retain the failed S1
result and qualify a higher-order S2 method under a separate admitted plan.
No grid change, threshold reduction, norm rescaling or time-step tuning is
implemented here. A6 remains blocked by the frozen acoustic gate.

## Other observations on the original native fields

These are separate diagnostics, not full-suite acceptance. All 142 retained
frames are finite and physically admissible under the reader. Worst normalized
five-component conservation drift is `1.6148698540002275e-16` against `2e-13`.
The separately labelled S1-D differential maximum is `5.551115123125783e-17`.

| Sod n | Density L1 | Pressure L1 in admitted interior |
| --- | --- | --- |
| 24 | 0.038436133232651666 | 0.05448703952865247 |
| 48 | 0.028884473581600432 | 0.04271663871527951 |
| 64 | 0.02823249991117378 | 0.037306094143767975 |

Density decreases; the finest pressure error is below 0.04. Comparing split
8/16-point quadrature gives maximum conservative-state difference
`1.3322676295501878e-15` for each mesh. This cannot override acoustic failure.
The admitted isolated speed gate still covers the first mode only; the existing
mode-two counterexample and scope limitation remain disclosed.

## Reproducible diagnostic source

The exact diagnostic source below has SHA-256
`427ceb6afa3ccc63b27bc71600c2d15f71159a29dd86117da425601cf9446e9c`.
Save its code block verbatim, including trailing newline, as
`/tmp/rheon-s1-acoustic-diagnosis.py`; it uses the stated workspace checkout and
NumPy 2.3.5. Run `python3 /tmp/rheon-s1-acoustic-diagnosis.py
/tmp/rheon-s1-1b62ae3-aea43a5-original/fields` to print its JSON report.
The prospective-family dt uses the documented rest-state CFL, not an executed
native step sequence; actual-family comparisons use captured dt values.

```python
"""Diagnostic linearized LLF symbol, not a continuum acceptance oracle."""
import hashlib,json,math,sys
from pathlib import Path
import numpy as np
sys.path.insert(0,'/workspace/Rheon-validation/tools')
import verify_compressible as v
root=Path(sys.argv[1]);suite=json.loads((root/'suite.json').read_text())
c=math.sqrt(v.GAMMA);background=np.array([1,0,0,0,1/(v.GAMMA-1)])
def field(n,dts,time,averaged):
    xyz=np.meshgrid(*[(np.arange(n)+.5)/n]*3,indexing='ij')
    value=np.broadcast_to(background,(n,n,n,5)).copy()
    for k,weight,phase in v.MODES:
        norm=math.sqrt(sum(q*q for q in k));theta=2*math.pi*np.array(k)/n
        b=np.array([1,*[c*q/norm for q in k],v.GAMMA/(v.GAMMA-1)])
        volume=math.prod(float(np.sinc(q/n)) for q in k) if averaged else 1.
        a=v.AMPLITUDE*weight*volume*np.exp(1j*phase)*b
        angle=2*math.pi*sum(q*x for q,x in zip(k,xyz))
        if dts is None:
            a=a*np.exp(-2j*math.pi*norm*c*time)
        else:
            matrix=-c*n*sum(1-np.cos(theta))*np.eye(5,dtype=complex)
            for d in range(3):
                jacobian=np.zeros((5,5));jacobian[0,d+1]=1
                jacobian[d+1,4]=v.GAMMA-1;jacobian[4,d+1]=v.GAMMA/(v.GAMMA-1)
                matrix-=1j*n*math.sin(theta[d])*jacobian
            for dt in dts:a=(np.eye(5)+dt*matrix)@a
        value+=np.imag(np.exp(1j*angle)[...,None]*a)
    return value
actual=[];point=[]
for case in suite['cases']:
    if case['kind']!='acoustic':continue
    n=case['shape'][0];dts=[f['accepted_dt_s'] for f in case['frames'][1:]]
    final=v.read_state(root/case['frames'][-1]['file'],case['shape'])
    predicted=field(n,dts,.08,True);exact=v.acoustic_average(tuple(case['shape']),.08)
    exact_point=field(n,None,.08,False);predicted_point=field(n,dts,.08,False)
    actual.append({'n':n,'native_density_l1':float(np.mean(abs(final[...,0]-exact[...,0]))/v.AMPLITUDE),
                   'linear_symbol_density_l1':float(np.mean(abs(predicted[...,0]-exact[...,0]))/v.AMPLITUDE),
                   'native_minus_symbol_max_per_component':np.max(abs(final-predicted),axis=(0,1,2)).tolist(),
                   'max_modified_gradient_relative_error':max(1-math.sin(a)/a for a in 2*math.pi*np.abs(np.array([1,1,2]))/n)})
    point.append({'n':n,'linear_symbol_point_normalized_error':float(np.mean(abs(predicted_point[...,0]-exact_point[...,0]))/v.AMPLITUDE)})
def orders(rows,key):return [math.log(a[key]/b[key])/math.log(b['n']/a['n']) for a,b in zip(rows,rows[1:])]
predictions=[]
for n in (8,12,16,20,24,32):
    dt=.25/(3*c*n);steps=int(.08/dt);dts=[dt]*steps
    if .08-steps*dt>0:dts.append(.08-steps*dt)
    predicted=field(n,dts,.08,True);exact=field(n,None,.08,True)
    predictions.append({'n':n,'linear_symbol_density_l1':float(np.mean(abs(predicted[...,0]-exact[...,0]))/v.AMPLITUDE)})
proposed=[r for r in predictions if r['n'] in (16,24,32)]
report={'diagnostic_source_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'scope':'Linearized constant-state LLF Fourier symbol using actual native dts. No continuum gate or acceptance replacement.',
        'actual_family':actual,'native_orders':orders(actual,'native_density_l1'),
        'symbol_orders':orders(actual,'linear_symbol_density_l1'),
        'point_comparison':point,'point_symbol_orders':orders(point,'linear_symbol_point_normalized_error'),
        'predicted_family':predictions,'proposed_16_24_32_symbol_orders':orders(proposed,'linear_symbol_density_l1')}
print(json.dumps(report,indent=2))
```
