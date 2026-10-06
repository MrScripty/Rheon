"""Independent exact Fraction oracle for the actual fixed-precision Rust probes."""
from pathlib import Path
from fractions import Fraction as Q
import json
import sys

P=Path(__file__).resolve().parent
def require(ok,message):
    if not ok:raise ValueError(message)
def power(e):return Q(2**e) if e>=0 else Q(1,2**(-e))
def value(parts):
    sig,e,negative=parts
    return (-1 if negative else 1)*Q(int(sig))*power(e-105)
def rounded(x,negative_zero=False):
    if x==0:return ["0",0,negative_zero]
    neg=x<0;x=abs(x)
    e=x.numerator.bit_length()-x.denominator.bit_length()
    if x<power(e):e-=1
    units=x/power(e-105)
    q,r=divmod(units.numerator,units.denominator)
    if 2*r>units.denominator or (2*r==units.denominator and q%2):q+=1
    if q==2**106:q//=2;e+=1
    if e < -1022 or e>1023 or (e==1023 and q>((2**53-1)<<53)):return None
    return [str(q),e,neg]
def run(log):
    records=[json.loads(x) for x in log.read_text().splitlines() if x.startswith('{')]
    layout=[x for x in records if x['event']=='preflight_layout']
    require(len(layout)==1 and layout[0]['e1_not_executed'] is True,'native preflight before E1')
    rows=[x for x in records if x['event']=='scalar_probe']
    require(len(rows)==36 and {(x['case'],x['op']) for x in rows}=={(i,op) for i in range(12) for op in ('add','mul','div')},'closed 36-probe scalar set')
    controls=[]
    for row in rows:
        a,b=value(row['a']),value(row['b']);an,bn=row['a'][2],row['b'][2]
        if row['op']=='add':expected=rounded(a+b,a==b==0 and an and bn)
        elif row['op']=='mul':expected=rounded(a*b,an!=bn)
        elif b==0:expected=None
        else:expected=rounded(a/b,an!=bn)
        require(row['refused']==(expected is None),'exact range/underflow/division classification')
        if expected is not None:require(row['out']==expected,'exact 106-bit rounding/significand/zero comparison')
        controls.append(dict(case=row['case'],op=row['op'],expected=expected,refused=expected is None))
    d=layout[0]
    require(d['scalar_bytes']==32 and d['workspace_bytes']==62096 and d['borrow_descriptor_bytes']==8,'actual fixed native layout')
    require(d['workspace_bytes']+d['borrow_descriptor_bytes']+d['kernel_stack_ceiling']==d['additional_bytes_with_ceiling']==64152<=65536,'additional simultaneous reservation')
    require(d['baseline_forced_nominal_bytes']==474768 and d['baseline_step_stack_bytes']==392416
            and d['baseline_third_stack_bytes']==20672 and d['baseline_force_stack_bytes']==3776,'baseline reservations retained')
    return dict(status='PASS_SCALAR_ORACLE_AND_LAYOUT_ONLY',native_assertion_groups=d['conformance_groups'],exact_scalar_probes=36,
                layout=d,combined_baseline_and_additional_reservation_bytes=474768+64152,
                compiled_stack_proof_still_required=True,owner_advances=0,e1_executed=False,rows=controls)
if __name__=='__main__':
    try:print(json.dumps(run(Path(sys.argv[1]) if len(sys.argv)>1 else P/'preflight-native.log'),indent=2,sort_keys=True))
    except Exception as exc:
        print(json.dumps(dict(status='GENUINE_SCALAR_CONFORMANCE_FAILURE',error=str(exc),e1_forbidden=True)))
        raise
