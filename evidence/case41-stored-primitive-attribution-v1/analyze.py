"""Exact attribution from frozen stored E2 primitives; no native calls or new states."""
from pathlib import Path
from fractions import Fraction as Q
import hashlib,importlib.util,json,math,struct,sys
import mpmath as mp
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
def require(ok,msg):
    if not ok:raise ValueError(msg)
def bits(x):
    if isinstance(x,list):return [bits(v) for v in x]
    if isinstance(x,dict):return {k:bits(v) for k,v in x.items()}
    if isinstance(x,float):return struct.pack('>d',x).hex()
    return x
def records(path,prefix=''):
    return [(i+1,line,json.loads(line.removeprefix(prefix))) for i,line in enumerate(path.read_text().splitlines()) if line.startswith('{') or prefix and line.startswith(prefix)]
def bindings(e,eq,start,end,quad,c):
    fields=['r','rate','direct_rate','start_z','start_mass','start_velocity','end_z','end_velocity','end_mass','end_force','end_q','end_eta','end_d','end_b','pairs','plus','minus','physical_convection','endpoint_convection','maximum_constraints','sign_roots']
    require(all(bits(e[k])==bits(eq[k]) for k in fields),'21 archived equation fields bit-identical')
    maps={'start':{'z':'start_z','velocity':'start_velocity','mass':'start_mass','q':'start_q','eta':'start_eta'},'end':{'z':'end_z','velocity':'end_velocity','mass':'end_mass','force':'end_force','q':'end_q','eta':'end_eta','d':'end_d'}}
    for label,point in [('start',start),('end',end)]:
        require(all(bits(point[k])==bits(e[v]) for k,v in maps[label].items()),label+' point identity')
    require(len(quad)==16,'sixteen retained donor samples')
    require(bits(c['final_authorized_unknowns'])==bits(e['unknown']),'unchanged original unknown bits')
    for point in [start,end,*quad]:
        require(point['mode']=='E2' and point['index']==41 and point['order']==16 and point['owner_advances']==0 and not point['published'],'frozen point scope')
        require(bits(point['pairs'])==bits(e['pairs']),'same forty-face donor topology')
        t=point['time'];eta=[c['eta'][i]+t*c['final_authorized_unknowns'][i] for i in range(6)]
        q=[(c['q'][i]+t*c['eta'][i])+((.5*t)*t)*c['final_authorized_unknowns'][i] for i in range(3)]
        require(bits(eta)==bits(point['eta']) and bits(q)==bits(point['q']),'original geometry/known graph at retained time')
    return dict(equation_fields=fields,point_fields=maps,donor_samples=16,donor_faces=len(e['pairs']),start_end_bit_identical=True)
def run(dps):
    mp.mp.dps=dps;policy=json.loads((P/'protocol.json').read_text())
    for f,h in policy['sha256'].items():require(hashlib.sha256((ROOT/f).read_bytes()).hexdigest()==h,'frozen source/raw '+f)
    spec=importlib.util.spec_from_file_location('primitive_algebra',ROOT/'evidence/forcing-case41-scale-diagnosis-v1/analyze.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a);o=a.a
    current=records(ROOT/'evidence/case41-exploratory-seven-v1/native.log','test research_public_call::case41_paired_candidate_roster ... ')
    baseline=next(x for x in current if x[2]['event']=='fd_equation' and x[2]['column']==-1);e=baseline[2]
    archived=records(ROOT/'evidence/forcing-e2-fixed-candidates-v1/candidate-fixed.log')
    selected=[x for x in archived if x[2].get('index')==41 and x[2].get('order')==16]
    eq=next(x[2] for x in selected if x[2]['event']=='fixed_equation');points=[x[2] for x in selected if x[2]['event']=='fixed_point']
    start=next(x for x in points if x['label']=='start');end=next(x for x in points if x['label']=='end');quad=[x for x in points if x['label']=='quadrature']
    fixture=json.loads((ROOT/'evidence/case41-exploratory-seven-v1/fixture.json').read_text())['original_case']
    c=next(x for x in json.loads((ROOT/'evidence/forcing-e2-fixed-candidates-v1/inputs.json').read_text())['cases'] if x['index']==41)
    require(bits(c)==bits(fixture),'entire original fixture bit identity')
    bound=bindings(e,eq,start,end,quad,c);c=dict(c,acceleration=[.0625,-.125,.03125])
    native,_=o.native(c,e);require(bits(native)==bits(e['rate']),'native stored arithmetic bit replay')
    f_native,visc_native,pres_native=o.forces(c,end,end['velocity'],False)
    require(bits(f_native)==bits(e['end_force']),'native force assembly bit replay')
    plus_native,minus_native=o.transfers(quad,len(e['pairs']),False)
    require(bits(plus_native)==bits(e['plus']) and bits(minus_native)==bits(e['minus']),'native donor quadrature bit replay')
    f_exact,visc_exact,pres_exact=o.forces(c,end,end['velocity'],True)
    plus_exact,minus_exact=o.transfers(quad,len(e['pairs']),True)
    stored,groups=o.exact(c,e);primitive,primitive_groups=o.exact(c,e,force=f_exact,plus=plus_exact,minus=minus_exact)
    zeros=lambda:[[Q(0)]*3 for _ in range(16)]
    sub=lambda v,w:[x-y for x,y in zip(v,w)]
    sumv=lambda vs:[sum(v[i] for v in vs) for i in range(22)]
    nodal=lambda force:[sum(Q(e['r'][n][d][j])*Q(force[n][d]) for n in range(16) for d in range(2)) for j in range(22)]
    fgap=lambda x,y:nodal([[Q(xx)-Q(yy) for xx,yy in zip(row,col)] for row,col in zip(x,y)])
    force_gaps=dict(strain_assembly=fgap(visc_native,visc_exact),pressure_assembly=fgap(pres_native,pres_exact),combined_force_addition=fgap(e['end_force'],[[Q(x)+Q(y) for x,y in zip(v,p)] for v,p in zip(visc_native,pres_native)]))
    def donor_term(plus,minus):
        _,g=o.exact(c,e,plus=plus,minus=minus);return g['endpoint_donor_transport']
    plus_products=[sum(Q(float(Q(p['factor'])*Q(max(p['flux'][j],0.)))) for p in quad) for j in range(len(e['pairs']))]
    minus_products=[sum(Q(float(Q(p['factor'])*Q(max(-p['flux'][j],0.)))) for p in quad) for j in range(len(e['pairs']))]
    donor_native=donor_term(e['plus'],e['minus']);donor_products=donor_term(plus_products,minus_products);donor_exact=donor_term(plus_exact,minus_exact)
    donor_gaps=dict(product_rounding=sub(donor_products,donor_exact),sequential_sum_rounding=sub(donor_native,donor_products))
    accumulation=sub(list(map(Q,e['rate'])),stored)
    force_gap=sumv(force_gaps.values());donor_gap=sumv(donor_gaps.values())
    require(sumv([primitive,force_gap,donor_gap])==stored,'exact primitive-to-stored reconstruction')
    require(sumv([primitive,force_gap,donor_gap,accumulation])==list(map(Q,e['rate'])),'exact native residual reconstruction')
    M=[[sum(Q(e['end_mass'][n])*Q(e['r'][n][d][i])*Q(e['r'][n][d][j]) for n in range(16) for d in range(2)) for j in range(22)] for i in range(22)]
    W=[[x/Q(27,8) for x in row] for row in a.inverse(M)];B=a.transpose([[Q(x) for x in row] for row in e['end_b']]);BtW=a.mul(a.transpose(B),W);proj=a.mul(a.mul(B,a.inverse(a.mul(BtW,B))),BtW)
    complement=lambda v:sub(v,a.mv(proj,v));inner=lambda v,w:a.dot(v,a.mv(W,w));root=lambda q:float(mp.sqrt(mp.mpf(q.numerator)/q.denominator))
    rc=complement(stored);square=inner(rc,rc)
    def summary(v):
        pv=a.mv(proj,v);cv=complement(v);require(inner(v,v)==inner(pv,pv)+inner(cv,cv),'weighted Pythagoras')
        require(a.mv(BtW,cv)==[Q(0)]*16,'weighted complement orthogonality')
        return dict(raw_norm=o.normq(v),components=[float(x) for x in v],pressure_norm=root(inner(pv,pv)),complement_norm=root(inner(cv,cv)),signed_alignment_to_stored_complement=float(inner(rc,cv)/square),exact_raw_squared_norm=str(a.dot(v,v)))
    # Actual captured working chart parts are observed, but their use in inertia
    # instead of the stored endpoint is a counterfactual equation contribution.
    latent=list(map(o.scalar,end['chart_endpoint_parts']));rho=sub(list(map(Q,e['end_z'])),latent)
    require(all(bits(float(latent[j]))==bits(e['end_z'][j]) for j in o.UNKNOWN),'dependent H chart RN64 matches actual stored endpoint')
    known_rho=[x if j in o.KNOWN else Q(0) for j,x in enumerate(rho)];dependent_rho=sub(rho,known_rho)
    inertial=lambda dz:[x/Q(c['h']) for x in a.mv(M,dz)]
    endpoint_known=inertial(known_rho);endpoint_dependent=inertial(dependent_rho)
    latent_primitive,_=o.exact(c,e,z=latent,force=f_exact,plus=plus_exact,minus=minus_exact)
    require(sumv([latent_primitive,endpoint_known,endpoint_dependent,force_gap,donor_gap,accumulation])==list(map(Q,e['rate'])),'exact endpoint/primitive/native identity')
    actual_decomp=dict(primitive_replay_stored_endpoint=primitive,force_assembly_gap=force_gap,donor_quadrature_gap=donor_gap,native_accumulation_gap=accumulation)
    stored_decomp={k:v for k,v in actual_decomp.items() if k!='native_accumulation_gap'}
    require(sum(inner(rc,complement(v)) for v in stored_decomp.values())==square,'stored complement decomposition exact')
    cross={f'{x}__{y}':str(2*inner(complement(stored_decomp[x]),complement(stored_decomp[y]))) for i,x in enumerate(stored_decomp) for y in list(stored_decomp)[i+1:]}
    primitive_terms={k:v for k,v in primitive_groups.items() if k not in ['direct_inertia','captured_combined_force']}
    primitive_terms.update(strain_force=nodal(visc_exact),pressure_force=nodal(pres_exact))
    require(sumv(primitive_terms.values())==primitive,'consistent primitive residual terms exact')
    binding_records=[dict(source='current_baseline',line=baseline[0],raw_line_sha256=hashlib.sha256(baseline[1].encode()).hexdigest())]+[dict(source='archived_E2',line=i,event=row['event'],label=row.get('label'),raw_line_sha256=hashlib.sha256(line.encode()).hexdigest()) for i,line,row in selected]
    return dict(status='PASS_ARCHIVED_STORED_PRIMITIVE_ATTRIBUTION',precision_digits=dps,bindings=bound,binding_records=binding_records,
        native_baseline=summary(list(map(Q,e['rate']))),exact_stored_baseline=summary(stored),actual_stored_primitive_decomposition={k:summary(v) for k,v in actual_decomp.items()},
        force_assembly_subterms={k:summary(v) for k,v in force_gaps.items()},donor_rounding_subterms={k:summary(v) for k,v in donor_gaps.items()},
        consistent_primitive_residual_terms={k:summary(v) for k,v in primitive_terms.items()},exact_stored_complement_cross_terms=cross,
        endpoint_counterfactual=dict(known_eta_H_vs_public_inertia=summary(endpoint_known),dependent_H_to_stored_conversion_inertia=summary(endpoint_dependent),primitive_with_latent_inertia_only=summary(latent_primitive),
            scope='Observed H/stored endpoint differences; replacing only inertia is counterfactual. Stored donor velocities/geometry/forces/flux samples remain fixed; no consistent nonlinear state or acceptance.'),
        primitive_exact_above_Newton_gate=a.dot(primitive,primitive)>Q(1e-13)**2,
        primitive_latent_counterfactual_above_Newton_gate=a.dot(latent_primitive,latent_primitive)>Q(1e-13)**2,
        new_native_equations=0,controller_corrections=0,owner_advances=0,historical_memory_qualified=False,
        rank_analysis_repeated=False,FD_truncation_bound=None,arithmetic_floor_claimed=False,true_map_error_bound=None,
        scope='Exact replay of stored triangle/pressure and captured flux/factor primitives at bit-identical baseline state. Primitive replay is not unrounded geometry, fresh quadrature, real-map evaluation or nonlinear solve; signed alignments are cancellation-sensitive algebra, not error shares.')
if __name__=='__main__':print(json.dumps(run(int(sys.argv[1]) if len(sys.argv)>1 else 80),indent=2,sort_keys=True))
