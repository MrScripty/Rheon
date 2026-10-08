"""Independent comparison to the frozen exact-rational formulation assembly.

No assert-dependent gates; -O has the same validation. This qualifies native
instantaneous arrays, not a physical finite trajectory or advancing simulator.
"""
import copy
from fractions import Fraction
import json
from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'evidence/fitted-height-periodic-repair'))
import reference as exact


def require(ok,message):
    if not ok: raise ValueError(message)


def close(a,b,message):
    b=float(b)
    require(abs(a-b)<=1e-12*(1+abs(b)),message)


def verify(data):
    require(data['scope']=='native instantaneous fitted assembly; no physical time advance','scope')
    require(data['allocated_bytes']==55408,'bounded payload')
    require(data['pressure_modes']==32,'pressure image dimension')
    points,micro,ids,_,_=exact.mesh()
    mass,rate,k,div,areas=exact.operators(points,micro)
    require(len(data['nodes'])==35 and data['triangles']==[list(t) for t in micro],'actual topology')
    for i,(node,p) in enumerate(zip(data['nodes'],points)):
        require(node[2]==ids[i],'periodic identification')
        for d in range(2):close(node[d],p[d].v,'physical position')
    for a,b in zip(data['mass'],exact.merge(mass,ids)):close(a,b,'liquid nodal mass')
    require(len(data['mass'])==32,'complete mass partition')
    first,second=data['samples']
    require(first['case']=='translation-third' and second['case']=='divergence-defect-pressure-work','cases')
    fractions=exact.witness()
    for key,want in [('mass',Fraction(57,16)),('volume',Fraction(19,16)),('strain_power',Fraction(fractions['strain_power'])),('advection_dissipation',Fraction(fractions['advection_dissipation']))]:
        close(first[key],want,key)
    for key in ['divergence_max','continuity_max','geometric_identity_error','pressure_adjoint_error','strain_work_error','convection_work_error','total_mass_rate']:
        close(first[key],0,key)
    for a,b in zip(first['mass_rates'],exact.merge(rate,ids)):close(a,b,'actual differentiated mass')
    require(len(first['mass_rates'])==32 and len(first['velocity'])==32,'full field arrays')
    flux=exact.dual_flux(points,micro,ids)
    given={(i,j):value for i,j,value in first['flux']}
    require(len(given)==len(first['flux']),'one shared oriented pair')
    require(set(flux)<=set(given),'complete geometric fluxes')
    for pair,f in given.items():close(f,flux.get(pair,0),'shared physical face flux')
    # Check all three conservative momentum fluxes and strain from exact local geometry.
    u=[[Fraction(v) for v in row] for row in first['velocity']]
    convection=[[Fraction(0)]*3 for _ in range(32)]
    for (i,j),f in flux.items():
        for d in range(3):
            g=f*(u[i][d]+u[j][d])/2+abs(f)*(u[i][d]-u[j][d])/2
            convection[i][d]+=g;convection[j][d]-=g
    raw_u=[value for i in range(35) for value in u[ids[i]]]
    ku=exact.mv(k,raw_u)
    force=[exact.merge([ku[3*i+d] for i in range(35)],ids) for d in range(3)]
    for i in range(32):
        for d in range(3):
            close(first['convection'][i][d],convection[i][d],'all-component shared transport')
            close(first['strain_force'][i][d],force[d][i],'full symmetric strain force')
    close(second['divergence_max'],1,'reported divergence defect')
    close(second['total_mass_rate'],Fraction(57,16),'physical volume-rate defect')
    require(second['continuity_max']>.1 and abs(second['pressure_work'])>.1,'defect not hidden')
    close(second['geometric_identity_error'],0,'defective-flow geometry identity')
    close(second['pressure_adjoint_error'],0,'nonzero adjoint work')
    # Independently integrate transpose force from native pressure reconstructed per triangle.
    q=[Fraction(p) for p in second['pressure_values']]
    require(len(q)==48,'pressure support')
    raw_force=[[Fraction(0)]*3 for _ in points]
    for t,row in enumerate(div):
        for i in range(35):
            for d in range(2):raw_force[i][d]-=areas[t]*q[t]*row[3*i+d]
    for d in range(3):
        merged=exact.merge([v[d] for v in raw_force],ids)
        for i in range(32):close(second['pressure_force'][i][d],merged[i],'actual transpose pressure force')
    return {'nodes':35,'triangles':48,'periodic_mass_nodes':32,'pressure_modes':32,'native_bytes':55408,'samples':2,
            'reference':'frozen exact-rational fitted-height-formulation/reference.py','all_components_checked':3,
            'scope':'instantaneous geometry/operators only'}


if __name__=='__main__':
    data=json.loads(Path(sys.argv[1]).read_text())
    summary=verify(data)
    if '--negative-self-test' in sys.argv:
        for kind in ['mass','flux','third_strain','pressure_force','scope','seam_geometry']:
            changed=copy.deepcopy(data)
            if kind=='mass':changed['mass'][0]+=.01
            elif kind=='flux':changed['samples'][0]['flux'][0][2]+=.01
            elif kind=='third_strain':changed['samples'][0]['strain_force'][0][2]+=.01
            elif kind=='pressure_force':changed['samples'][1]['pressure_force'][0][0]+=.01
            elif kind=='seam_geometry':
                changed['nodes'][19][1]=.5;changed['nodes'][30][1]=.5
            else:changed['scope']='advancing liquid simulation'
            try:verify(changed)
            except ValueError:print('REJECTED',kind)
            else:raise ValueError('corrupted native evidence accepted '+kind)
    print(json.dumps(summary,indent=2))
