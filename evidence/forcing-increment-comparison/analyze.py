"""Bitwise B0 binding, A0 exact fixed fields and one changed-chart E1 outcome."""
from pathlib import Path
from fractions import Fraction as Q
import copy,hashlib,json,math,struct,sys
P=Path(__file__).resolve().parent
ROOT=P.parents[1]
def require(ok,message):
    if not ok:raise ValueError(message)
def bit(x):return struct.pack('>d',x)
def qstr(x):
    x=Q(x);return f'{x.numerator}/{x.denominator}'
def rows(name):return [json.loads(x) for x in (P/name).read_text().splitlines() if x.startswith('{')]
def same_float(a,b):
    if isinstance(a,list):
        require(isinstance(b,list) and len(a)==len(b),'matching captured float dimensions')
        return sum(same_float(x,y) for x,y in zip(a,b))
    require(bit(a)==bit(b),'actual bitwise capture binding including signed zero')
    return 1
def exact_rate(c,e,direct=False):
    out=[Q(0)]*22;h=Q(c['h']);a=[Q(.0625),Q(-.125),Q(.03125)]
    for i in range(16):
        for d in range(2):
            if direct:inertia=Q(e['end_mass'][i])*Q(e['end_velocity'][i][d])-Q(c['mass'][i])*Q(c['old'][i][d])
            else:
                increment=sum(Q(w)*(Q(z1)-Q(z0)) for w,z1,z0 in zip(e['r'][i][d],e['end_z'],e['start_z']))
                inertia=Q(e['end_mass'][i])*increment+(Q(e['end_mass'][i])-Q(c['mass'][i]))*Q(c['old'][i][d])
            force=h*(Q(e['end_force'][i][d])-Q(c['mass'][i])*a[d])
            for v in range(22):out[v]+=Q(e['r'][i][d][v])*(inertia+force)
    for f,(i,j) in enumerate(e['pairs']):
        for d in range(2):
            transported=Q(e['plus'][f])*Q(e['end_velocity'][i][d])-Q(e['minus'][f])*Q(e['end_velocity'][j][d])
            for v in range(22):out[v]+=(Q(e['r'][i][d][v])-Q(e['r'][j][d][v]))*transported
    return [x/h for x in out]
def exact_summary(c,e):
    stable=exact_rate(c,e);direct=exact_rate(c,e,True);sq=sum(x*x for x in stable);gap=[a-b for a,b in zip(stable,direct)]
    return dict(scope='exact arithmetic on these identical binary fields; no chart/geometry recomputation or native acceptance',
                stable_rate=list(map(qstr,stable)),direct_rate=list(map(qstr,direct)),squared_norm=qstr(sq),
                native_threshold_squared=qstr(Q(1e-13)**2),strict_exact_threshold_pass=sq<=Q(1e-13)**2,
                stable_norm_float=math.sqrt(float(sq)),direct_norm_float=math.sqrt(float(sum(x*x for x in direct))),
                stable_direct_exact_projected_rate_gap_norm=math.sqrt(float(sum(x*x for x in gap))),
                max_native_stable_component_difference=max(abs(float(x)-y) for x,y in zip(stable,e['rate'])))
def changed(a,b):
    if isinstance(a,list):return sum(changed(x,y) for x,y in zip(a,b))
    return int(bit(a)!=bit(b))
def main(full=False):
    packet=json.loads((ROOT/'evidence/forcing-increment-contract-supplement/selected-capture.json').read_text())
    c,native=packet['input_row'],packet['fixed_native_candidate'];record=rows('B0-native.log')
    b0=[x for x in record if x.get('stage')=='B0-16'][0];fine=[x for x in record if x.get('stage')=='B0-32'][0]
    fields=['r','start_z','end_z','start_mass','end_mass','end_velocity','end_force','end_b','end_d','plus','minus','rate','direct_rate','norm','end_eta']
    count=sum(same_float(b0[k],native[k]) for k in fields)
    count+=same_float(b0['end_q'],packet['post_window_observations'][0]['candidate_q'])
    require(b0['pairs']==native['pairs'] and b0['published'] is False and b0['newton_pass'] is False,'B0 remains refused')
    count+=same_float(fine['rate'],packet['post_window_observations'][1]['rate'])
    count+=same_float(fine['direct_rate'],packet['post_window_observations'][1]['direct_rate'])
    count+=same_float(fine['norm'],packet['post_window_observations'][1]['rate_norm'])
    require(any(x['event']=='B0_stage_complete' and x['e1_executed'] is False for x in record),'B0 stops before A0/E1')
    a0=exact_summary(c,b0)
    require(a0['strict_exact_threshold_pass'] is False,'A0 same fields retains exact refusal')
    result=dict(status='PASS_B0_AND_A0_DIAGNOSTIC_BINDING',prototype='78b7365513ccf317e554d36f968da36b55f04065',
        prototype_policy_sha256=hashlib.sha256((P/'staged-prototype-policy.json').read_bytes()).hexdigest(),
        selector=packet['selector'],B0=dict(bitwise_float_checks=count,coarse_norm=b0['norm'],fine_norm=fine['norm'],native_refusal=True),
        A0=a0,owner_advances=0,searches=0,acceptance_promotions=0,original_native_refusals=5,
        original_temporal_band='FAIL_ORIGINAL_TEMPORAL_BAND',E1_executed=False)
    if full:
        e=rows('E1-native.log')
        require([x for x in e if x.get('stage')=='B0-16'][0]==b0,'E1 invocation reproduces identical B0')
        failures=[x for x in e if x['event']=='E1_failure']
        require(not any(x.get('published') is True or x.get('owner_advances',0)!=0 for x in e),'no publication or owner advance')
        result['E1_executed']=True
        if failures:
            result['E1']=dict(scope='changed chart solution, same terminal unknowns/native geometry/downstream policy',failures=failures)
        else:
            coarse=[x for x in e if x.get('stage')=='E1-16'][0];fine_e=[x for x in e if x.get('stage')=='E1-32'][0]
            # Geometry/pressure-basis operators and known eta are native and exact-bit unchanged.
            for field in ['r','start_z','start_mass','end_q','end_eta','end_mass','end_d','end_b']:same_float(coarse[field],b0[field])
            require(coarse['pairs']==b0['pairs'],'same native topology')
            changes={k:changed(coarse[k],b0[k]) for k in ['end_z','end_velocity','end_force','plus','minus','rate','direct_rate']}
            require(any(changes.values()),'E1 is a changed-field comparison, not A0')
            gates=[x for x in e if x.get('event')=='isolated_planar_gates' and x.get('stage')=='E1']
            complete=[x for x in e if x['event']=='comparison_complete']
            require(len(gates)==len(complete)==1 and complete[0]['searches']==0,'one fixed comparison only')
            result['E1']=dict(scope='changed chart solution at same frozen terminal unknowns; native geometry and all downstream arithmetic retained',
                coarse_native_norm=coarse['norm'],fine_native_norm=fine_e['norm'],native_newton_threshold_pass=coarse['newton_pass'],
                endpoint_bit_changes=changes,unchanged_geometry_and_basis_bitwise=True,
                isolated_planar_gates=gates[0],chart_evaluations=complete[0]['chart_evaluations'],
                exact_diagnostic_on_E1_changed_fields=exact_summary(c,coarse),third_or_owner_step_qualified=False)
        result['status']='COMPLETED_SPECIFIED_NO_OWNER_COMPARISON'
    return result
if __name__=='__main__':print(json.dumps(main('--full' in sys.argv),indent=2,sort_keys=True))
