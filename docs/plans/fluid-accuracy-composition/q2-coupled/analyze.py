"""Fixed tiny aligned algebra corpus; independent discretization oracle required."""
import csv,json,math,pathlib,sys

def require(value,message):
    if not value: raise SystemExit('coupled input/gate refused: '+message)
def rows(p):
    with p.open() as stream:return list(csv.DictReader(stream))
root=pathlib.Path(sys.argv[1]); summary=rows(root/'summary.csv')
expected={(f,r,dt) for f in ['unit','anisotropic'] for r in [1.0,998.206] for dt in [.125,1.0]}
keys=[(r['fixture'],float(r['rho']),float(r['dt'])) for r in summary]
require(len(keys)==8 and len(set(keys))==8 and set(keys)==expected,'eight unique prescribed fixtures')
results=[]
for r in summary:
    fixture=r['fixture'];rho=float(r['rho']);dt=float(r['dt']);mu=.1 if rho==1 else .001002
    require(float(r['mu'])==mu,'prescribed mu')
    require(all(math.isfinite(float(v)) for k,v in r.items() if k!='fixture'),'finite summary')
    for field in ['momentum','continuity','work','corrected_work','residual_work_bound']:
        require(0<=float(r[field])<=1e-10,fixture+' fixed '+field)
    require(float(r['old_energy'])>0 and float(r['energy'])>=0 and float(r['dissipation'])>=0,fixture+' energy/dissipation')
    case=root/f'{fixture}-rho{rho:g}-dt{dt:g}';meta_rows=rows(case/'meta.csv');meta={v['key']:v['value'] for v in meta_rows}
    require(len(meta)==len(meta_rows),'duplicate metadata')
    require(all(math.isfinite(float(v)) for k,v in meta.items() if k!='fixture'),'finite metadata')
    require(meta['fixture']==fixture and float(meta['rho'])==rho and float(meta['mu'])==mu and float(meta['dt'])==dt,'source parameters')
    nv,nc,ns,components=[int(meta[k]) for k in ['nv','nc','ns','components']];nn=nv+nc-components
    require(0<nv<nn<=256 and nc>components>0 and ns>0,'bounded dimensions')
    faces=rows(case/'faces.csv');rr=rows(case/'rows.csv');terms=rows(case/'terms.csv');cells=rows(case/'cells.csv');mat=rows(case/'matrices.csv')
    require(all(math.isfinite(float(v)) for f in faces for v in f.values()),'finite face fields')
    require(all(math.isfinite(float(c[k])) for c in cells for k in ['volume','p']),'finite cell volume/pressure')
    require(len(faces)==nv and {int(f['active']) for f in faces}==set(range(nv)),'active IDs')
    require(len(rr)==ns and {int(t['row']) for t in rr}==set(range(ns)),'strain rows')
    require(all(0<=int(t['row'])<ns and 0<=int(t['active'])<nv and math.isfinite(float(t['coefficient'])) for t in terms),'strain term association')
    require(len({(t['row'],t['active']) for t in terms})==len(terms),'duplicate strain term')
    require(all(math.isfinite(float(t['weight'])) and float(t['weight'])>=0 for t in rr),'nonnegative finite weights')
    require(all(float(f['mass'])>0 and float(f['area'])>0 and float(f['distance'])>0 for f in faces),'positive mass/area/distance')
    echar=.5*sum(float(f['mass']) for f in faces)
    require(float(r['energy'])<=float(r['old_energy'])+1e-10*echar,'energy upper bound')
    pressure=[c for c in cells if float(c['volume'])>0];require(len(pressure)==nc,'pressure cells')
    require({int(c['pressure_index']) for c in pressure}==set(range(nc)) and sum(c['gauge']=='true' for c in pressure)==components,'pressure IDs/gauges')
    sizes={'B':(nc,nv),'K':(nv,nv),'A':(nv,nv),'coupled':(nn,nn)}
    actual={(m['block'],int(m['row']),int(m['column'])) for m in mat}
    wanted={(block,i,j) for block,(n,m) in sizes.items() for i in range(n) for j in range(m)}
    require(len(actual)==len(mat) and actual==wanted and all(math.isfinite(float(m['value'])) for m in mat),'complete finite blocks')
    results.append(dict(fixture=fixture,rho=rho,mu=mu,dt=dt,nv=nv,nc=nc,ns=ns,components=components,full_size=nn,metrics={k:float(v) for k,v in r.items() if k not in ['fixture','rho','mu','dt']}))
print(json.dumps(dict(native_tiny_aligned_gate=True,results=results,independent_blocks_solution_work_gate='REQUIRED',cut_free_surface_or_production_admitted=False),indent=2,allow_nan=False))
