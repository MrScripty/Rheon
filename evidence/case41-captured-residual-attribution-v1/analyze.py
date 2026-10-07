"""Attribute captured baseline terms in the frozen mass metric; no numerical call."""
from pathlib import Path
from fractions import Fraction as Q
import importlib.util
import json
import hashlib
import sys
import mpmath as mp
P=Path(__file__).resolve().parent
ROOT=P.parents[1]
def require(ok,msg):
    if not ok:
        raise ValueError(msg)
def run(dps):
    mp.mp.dps=dps
    policy=json.loads((P/'protocol.json').read_text())
    for f,h in policy['sha256'].items():
        require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'bound source/data '+f)
    inv=json.loads((ROOT/'evidence/case41-exploratory-seven-v1/result-inventory.json').read_text())
    for f,h in inv['sha256'].items():
        require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'original result '+f)
    spec=importlib.util.spec_from_file_location('attribution_algebra',ROOT/'evidence/forcing-case41-scale-diagnosis-v1/analyze.py')
    a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a);oracle=a.a
    prefix='test research_public_call::case41_paired_candidate_roster ... '
    rows=[json.loads(line.removeprefix(prefix)) for line in (ROOT/'evidence/case41-exploratory-seven-v1/native.log').read_text().splitlines() if line.startswith(prefix) or line.startswith('{')]
    e=next(row for row in rows if row['event']=='fd_equation' and row['column']==-1)
    c=dict(json.loads((ROOT/'evidence/case41-exploratory-seven-v1/fixture.json').read_text())['original_case'],acceleration=[.0625,-.125,.03125])
    r,groups=oracle.exact(c,e)
    M=[[sum(Q(e['end_mass'][n])*Q(e['r'][n][d][i])*Q(e['r'][n][d][j]) for n in range(16) for d in range(2)) for j in range(22)] for i in range(22)]
    W=[[x/Q(27,8) for x in row] for row in a.inverse(M)]
    B=a.transpose([[Q(x) for x in row] for row in e['end_b']]);BtW=a.mul(a.transpose(B),W)
    proj=a.mul(a.mul(B,a.inverse(a.mul(BtW,B))),BtW)
    complement=lambda v:[x-y for x,y in zip(v,a.mv(proj,v))]
    inner=lambda x,y:a.dot(x,a.mv(W,y))
    root=lambda q:float(mp.sqrt(mp.mpf(q.numerator)/q.denominator))
    sumvec=lambda vals:[sum(v[j] for v in vals) for j in range(22)]
    z=list(map(Q,e['end_z']));D=tuple(map(tuple,e['end_d']))
    storedknown_chart=list(oracle.chart(D,tuple(z[j] for j in oracle.KNOWN)))
    etaideal=[Q(v)+Q(c['h'])*Q(alpha) for v,alpha in zip(e['start_eta'],e['unknown'][:6])]
    idealknown=[(-etaideal[oracle.COORD[k]] if k==6 else etaideal[oracle.COORD[k]]) for k in range(7)]
    idealknown_chart=list(oracle.chart(D,tuple(idealknown)))
    force=lambda dz:[x/Q(c['h']) for x in a.mv(M,dz)]
    dependent=force([x-y for x,y in zip(z,storedknown_chart)])
    known=force([x-y for x,y in zip(storedknown_chart,idealknown_chart)])
    remainder=[x-y-w for x,y,w in zip(r,dependent,known)]
    replay,_=oracle.exact(c,e,z=idealknown_chart)
    require(remainder==replay,'exact frozen-inertia known/dependent identity')
    native=list(map(Q,e['rate']));native_error=[x-y for x,y in zip(native,r)]
    rc=complement(r);rcsquare=inner(rc,rc)
    def summary(v):
        pv=a.mv(proj,v);cv=complement(v)
        require(inner(v,v)==inner(pv,pv)+inner(cv,cv),'exact weighted decomposition')
        return dict(raw_norm=oracle.normq(v),components=[float(x) for x in v],weighted_pressure_norm=root(inner(pv,pv)),weighted_complement_norm=root(inner(cv,cv)),
                    signed_complement_alignment=float(inner(rc,cv)/rcsquare),signed_alignment_scope='Signed dot(Cr,Cv)/||Cr||_W^2; cancellation-sensitive algebra, not error percentage.')
    active={k:v for k,v in groups.items() if k!='direct_inertia'}
    require(sumvec(active.values())==r,'all actual stable residual terms sum exactly')
    for value in active.values():require(a.mv(BtW,complement(value))==[Q(0)]*16,'term complement orthogonal')
    require(sum(inner(rc,complement(v)) for v in active.values())==rcsquare,'signed actual-term projections sum exactly')
    require(sumvec([dependent,known,remainder])==r,'stored endpoint splitting exact')
    split=dict(dependent_selected_chart_rounding_inertia=dependent,known_eta_endpoint_rounding_inertia=known,frozen_fields_with_ideal_known_selected_chart=remainder)
    require(sum(inner(rc,complement(v)) for v in split.values())==rcsquare,'signed endpoint split projections sum exactly')
    return dict(status='PASS_ARCHIVED_BASELINE_COMPLEMENT_ATTRIBUTION_ONLY',precision_digits=dps,
                reused_rank_conditioning='No new rank/SVD computation; see 455af0e35c25037d2ff904bb92fd71b5d6b1f5c9 / model source 95f620f5a47b7c9c9afd6eab4d5e9940049f09cf.',
                baseline_native=summary(native),baseline_exact_stored=summary(r),native_accumulation_gap=summary(native_error),
                actual_stable_residual_terms={k:summary(v) for k,v in active.items()},
                frozen_endpoint_inertia_split={k:summary(v) for k,v in split.items()},
                exact_split_cross_terms={f'{x}__{y}':str(2*inner(complement(split[x]),complement(split[y]))) for i,x in enumerate(split) for y in list(split)[i+1:]},
                ideal_known_eta=[str(x) for x in etaideal],captured_minus_ideal_known_eta=[float(Q(x)-y) for x,y in zip(e['end_eta'],etaideal)],
                exact_stored_above_Newton_gate=a.dot(r,r)>Q(1e-13)**2,
                frozen_ideal_endpoint_remainder_above_Newton_gate=a.dot(remainder,remainder)>Q(1e-13)**2,
                new_native_equations=0,controller_corrections=0,owner_advances=0,
                original_newton_gate=1e-13,original_physical_gate=1e-11,historical_memory_qualified=False,
                FD_truncation_bound=None,arithmetic_floor_claimed=False,true_map_residual_error_bound=None,
                scope='Actual retained baseline fields only. Exact selected chart uses frozen binary D; hypothetical inertia reconciliation leaves force, donor, mass, geometry and accepted state fixed. Neither reconstructed endpoint is a consistent nonlinear candidate or real-map oracle. No physical error share, root distance, accepted step, gate or controller change.')
if __name__=='__main__':
    print(json.dumps(run(int(sys.argv[1]) if len(sys.argv)>1 else 80),indent=2,sort_keys=True))
