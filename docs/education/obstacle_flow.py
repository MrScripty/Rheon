"""Independent rational matrix controls for the bounded obstacle-flow lab.

Pressure uses a closed stationary face-area graph. Shear uses only the admitted
fully developed extrusion, with exact rational backward-Euler solves as the
fixture oracle. Neither qualifies general cut-cell viscosity or continuum error.
"""
from fractions import Fraction as F
from pathlib import Path
import argparse, hashlib, json, math, shutil, subprocess
from static_obstacle import coordinates, index, intersect, require, sha

SOURCES = ['src/obstacle_pressure.rs','src/obstacle_shear.rs','src/static_obstacle.rs',
           'src/collision.rs','src/geometry.rs','src/lib.rs','examples/obstacle_flow.rs',
           'tests/obstacle_flow_contract.rs','Cargo.toml','Cargo.lock',
           'docs/education/obstacle_flow.py','docs/education/obstacle_flow.js','docs/education/static_obstacle.py']
PRESSURE = [('partial-gradient',[.25,.5,.75],[1.75,1.5,2.25],[1,2,3]),
            ('outside-gradient',[4]*3,[5]*3,[1,2,3]),
            ('separator-gradient',[1,0,0],[2,3,3],[1,2,3]),
            ('partial-rest',[.25,.5,.75],[1.75,1.5,2.25],[0,0,0])]
SHEAR = [('cut-no-slip',None,2,'density',2,1.5),
         ('cut-navier',4,2,'density',2,1.5),
         ('cut-free-slip',0,2,'density',2,1.5),
         ('channel-density-one',None,1,'density',1,.5),
         ('channel-density-two',None,2,'density',1,.5),
         ('channel-acceleration-two',None,2,'acceleration',1,.5)]
TRIANGLES = [[0,2,3],[0,3,1],[4,5,7],[4,7,6],[0,1,5],[0,5,4],[2,6,7],[2,7,3],[0,4,6],[0,6,2],[1,3,7],[1,7,5]]
EPS = F(math.ulp(1.0))

def exact(actual, expected, name):
    require(actual == expected, name)

def close(actual, expected, name, scale=1):
    require(type(actual) in (float,int,F) and math.isfinite(actual), 'finite '+name)
    require(abs(F(actual)-expected) <= 4096*EPS*max(F(scale),abs(expected)), 'rational mismatch '+name)

def geometry(c, dims, lo, hi, stamp):
    exact((c['counts'],c['spacing'],c['origin'],c['lower'],c['upper']), (dims,[1.0]*3,[0.0]*3,lo,hi), 'fixed geometry')
    exact(c['stamp'],[stamp,1],'retained source stamp')
    exact(c['vertices'],[[hi[d] if corner & (1<<d) else lo[d] for d in range(3)] for corner in range(8)],'retained source vertices')
    exact(c['triangles'],TRIANGLES,'retained source facets')
    volumes=[];areas=[];edges=[];box=[(F(lo[d]),F(hi[d])) for d in range(3)]
    for p in coordinates(dims):
        volumes.append(F(1)-math.prod(intersect((F(p[d]),F(p[d]+1)),box[d]) for d in range(3)))
    exact(c['volumes'],volumes,'owner cell volumes')
    parent=list(range(len(volumes)))
    def root(i):
        while parent[i]!=i:i=parent[i]
        return i
    for d in range(3):
        fd=dims.copy();fd[d]+=1;aa=[]
        for p in coordinates(fd):
            area=F(1)-(math.prod(intersect((F(p[a]),F(p[a]+1)),box[a]) for a in range(3) if a!=d) if box[d][0]<=p[d]<=box[d][1] else 0)
            aa.append(area)
            if 0<p[d]<dims[d] and area:
                low=list(p);low[d]-=1;a,b=index(dims,low),index(dims,p)
                require(volumes[a]>0 and volumes[b]>0,'active edge volumes')
                edges.append((d,index(fd,p),a,b,area));parent[root(b)]=root(a)
        areas.append(aa)
    exact(c['areas'],areas,'owner shared face areas')
    roots={};labels=[];gauges=[]
    for i,v in enumerate(volumes):
        if not v:labels.append(None)
        else:
            r=root(i)
            if r not in roots:roots[r]=len(roots);gauges.append(i)
            component=roots[r]
            # Independent exact-volume maximum; iteration order supplies the
            # stable lowest-index tie break, without copying the native gauge.
            if v>volumes[gauges[component]]:gauges[component]=i
            labels.append(component)
    exact(c['labels'],labels,'owner connectivity');exact(c['components'],len(roots),'component count')
    return volumes,areas,edges,labels,gauges

def finite_metric(value, name, nonnegative=False):
    require(type(value) in (float,int) and math.isfinite(value), 'finite '+name)
    require(not nonnegative or value>=0, 'nonnegative '+name)
    return F(value)

def roundoff_close(actual, expected, scale, name, factor=64):
    """Fixture arithmetic allowance, with no unit-sized absolute floor.

    Rational recomputation uses represented native inputs. 64 eps times the
    sum/magnitude of contributing terms covers the short binary64 expression
    and compensated reductions in these <=4-layer controls; this is a declared
    fixture check, not a universal IEEE theorem or an operator tolerance.
    """
    value=finite_metric(actual,name)
    require(abs(value-expected)<=factor*EPS*abs(scale), 'arithmetic mismatch '+name)

def check_budget(actual, multiplier, scale, name):
    """Check the documented formula, including its own short sum rounding.

    At most five nonnegative reported terms are added before multiplication by
    a power-of-two epsilon multiplier. 16 eps relative to that tiny budget is
    a conservative fixture allowance; max(1, budget) would defeat this check.
    """
    finite_metric(actual,name,nonnegative=True)
    expected=multiplier*EPS*scale
    roundoff_close(actual,expected,expected,name,factor=16)
    return expected

def gaussian(a,b):
    """Tiny exact dense solve, independent of Rust's Thomas/PCG algorithms."""
    a=[list(row)+[b[i]] for i,row in enumerate(a)];n=len(b)
    for j in range(n):
        pivot=next(i for i in range(j,n) if a[i][j])
        a[j],a[pivot]=a[pivot],a[j];value=a[j][j];a[j]=[x/value for x in a[j]]
        for i in range(n):
            if i!=j:
                value=a[i][j];a[i]=[a[i][k]-value*a[j][k] for k in range(n+1)]
    return [a[i][-1] for i in range(n)]

def controls(records):
    exact(records.get('schema'),'rheon-obstacle-flow-records-v1','flow schema')
    exact([c['id'] for c in records['pressure_cases']],[p[0] for p in PRESSURE],'pressure roster')
    exact([c['id'] for c in records['shear_cases']],[s[0] for s in SHEAR],'shear roster')
    for case,(c,(name,lo,hi,gradient)) in enumerate(zip(records['pressure_cases'],PRESSURE)):
        dims=[3]*3;volumes,areas,edges,labels,gauges=geometry(c,dims,lo,hi,case+1)
        exact((c['density'],c['dt'],c['gradient'],c['gauges']),(2,.5,gradient,gauges),'pressure parameters/gauges')
        exact(len(c['residual']),3,'pressure residual shape');exact(len(c['ledger']),7,'pressure ledger shape')
        for x in c['residual']:finite_metric(x,'pressure residual',nonnegative=True)
        for i,x in enumerate(c['ledger']):finite_metric(x,'pressure ledger',nonnegative=i in (0,1,2,3,6))
        rho=F(2);dt=F(1,2);pressure=list(map(F,c['pressure']));exact(len(pressure),len(volumes),'pressure length')
        before=[[F(0)]*len(a) for a in areas];after=[list(map(F,a)) for a in c['after']]
        exact([len(a) for a in after],[len(a) for a in areas],'corrected field lengths')
        for d,f,a,b,area in edges:before[d][f]=dt*F(gradient[d])/rho
        exact(c['before'],before,'analytic initial gradient')
        points=list(coordinates(dims))
        for i,p in enumerate(pressure):
            expected=0 if labels[i] is None else sum(F(gradient[d])*(points[i][d]-points[gauges[labels[i]]][d]) for d in range(3))
            close(p,expected,'affine pressure')
        for d,field in enumerate(after):
            for f,u in enumerate(field):
                require(math.isfinite(float(u)) and abs(u)<=F(1,10**10),'actual gradient removal')
                if not before[d][f]:close(u,0,'stationary face',scale=1)
        q0=[F(0)]*len(volumes);q1=q0.copy();lp=q0.copy();operator_scale=q0.copy();flux_scale=q0.copy();e0=F(0);e1=F(0);ec=F(0)
        for d,f,a,b,area in edges:
            old,new=before[d][f],after[d][f];jump=pressure[b]-pressure[a]
            close(new,old-dt*jump/rho,'pressure correction')
            q0[a]+=area*old;q0[b]-=area*old;q1[a]+=area*new;q1[b]-=area*new
            lp[a]-=area/rho*jump;lp[b]+=area/rho*jump
            operator_scale[a]+=abs(area/rho*jump);operator_scale[b]+=abs(area/rho*jump)
            flux_scale[a]+=abs(area*new);flux_scale[b]+=abs(area*new)
            mass=rho*area;e0+=mass*old*old/2;e1+=mass*new*new/2;ec+=mass*(new-old)**2/2
        residual=[-q0[i]/dt-lp[i] for i in range(len(volumes))]
        predicted=max((dt*abs(r)/volumes[i] for i,r in enumerate(residual) if volumes[i]),default=0)
        actual=max((abs(q1[i])/v for i,v in enumerate(volumes) if v),default=0)
        require(predicted<=F(1,10**10) and actual<=F(1,10**10),'independent cellwise divergence gate')
        residual_scale=[abs(q0[i]/dt)+operator_scale[i] for i in range(len(volumes))]
        roundoff_close(c['residual'][0],F(math.sqrt(float(sum(r*r for r in residual)))),sum(residual_scale),'true residual l2')
        roundoff_close(c['residual'][1],max(map(abs,residual)),max(residual_scale),'true residual max')
        roundoff_close(c['residual'][2],predicted,max((dt*residual_scale[i]/v for i,v in enumerate(volumes) if v),default=0),'scaled residual')
        work=dt*sum(pressure[i]*q1[i] for i in range(len(volumes)));identity=e1-e0+ec-work
        work_scale=dt*sum(abs(pressure[i])*flux_scale[i] for i in range(len(volumes)))
        energy_scale=e0+e1+ec+abs(work)
        scales=[max((flux_scale[i]/v for i,v in enumerate(volumes) if v),default=0),e0,e1,ec,work_scale,energy_scale+work_scale]
        for native,value,scale in zip(c['ledger'][:6],[actual,e0,e1,ec,work,identity],scales):roundoff_close(native,value,scale,'projection ledger')
        native=list(map(F,c['ledger']))
        check_budget(c['ledger'][6],1024,native[1]+native[2]+native[3]+abs(native[4]),'projection rounding budget')
        require(abs(identity)<=1024*EPS*energy_scale,'actual pressure energy identity')
        require(abs(native[5])<=native[6],'reported pressure energy identity')
        require(e1<=e0+abs(work)+1024*EPS*energy_scale,'actual pressure energy gate')
        require(type(c['iterations']) is int and 0<=c['iterations']<=300,'bounded pressure iterations')
        require(c['workspace_bytes']>=7*8*len(volumes)+8*sum(map(len,areas))+8*len(gauges) and c['workspace_bytes']<=1_000_000,'pressure capacity payload')
    for case,(c,(name,beta,rho,kind,value,cut)) in enumerate(zip(records['shear_cases'],SHEAR)):
        dims=[1,2,1] if cut==1.5 else [1,4,1]
        geometry(c,dims,[0.0]*3,[1.0,cut,1.0],case+101)
        exact((c['normal'],c['density'],c['viscosity'],c['beta'],c['force_kind'],c['force_value'],c['dt']),(1,rho,1,beta,kind,value,.125),'shear parameters')
        intervals=[(max(F(j),F(cut)),F(j+1)) for j in range(dims[1]) if j+1>cut]
        volumes=[b-a for a,b in intervals];centers=[(a+b)/2 for a,b in intervals];n=len(volumes);rho=F(rho);dt=F(1,8);q=F(value)*(rho if kind=='acceleration' else 1)
        exact(c['layer_volumes'],volumes,'summed owner layer volumes');exact(c['centers'],centers,'fluid centroids')
        interior=[1/(b-a) for a,b in zip(centers,centers[1:])]
        lowdist=centers[0]-F(cut);low=1/lowdist if beta is None else F(beta)/(1+F(beta)*lowdist);high=1/(dims[1]-centers[-1])
        for native,expected in zip(c['interior_conductances'],interior):close(native,expected,'shared area conductance')
        exact(len(c['interior_conductances']),n-1,'interior conductance count')
        exact(len(c['wall_conductances']),2,'wall conductance count')
        for native,expected in zip(c['wall_conductances'],[low,high]):close(native,expected,'wall distance/friction closure')
        k=[[F(0)]*n for _ in range(n)];k[0][0]+=low;k[-1][-1]+=high
        for j,conductance in enumerate(interior):k[j][j]+=conductance;k[j+1][j+1]+=conductance;k[j][j+1]-=conductance;k[j+1][j]-=conductance
        masses=[rho*v for v in volumes];a=[[dt*k[i][j]+(masses[i] if i==j else 0) for j in range(n)] for i in range(n)]
        exact([f['step'] for f in c['frames']],list(range(9)),'shear frame roster');exact(c['frames'][0]['velocity'],[0]*n,'initial rest')
        reference_old=[F(0)]*n
        native_old=[F(0)]*n
        native_interior=[finite_metric(x,'interior conductance',True) for x in c['interior_conductances']]
        native_low,native_high=[finite_metric(x,'wall conductance',True) for x in c['wall_conductances']]
        for frame in c['frames'][1:]:
            exact(len(frame['energy']),8,'shear energy shape');exact(len(frame['momentum']),6,'shear momentum shape')
            exact(len(frame['velocity']),n,'velocity length')
            for i,x in enumerate(frame['energy']):finite_metric(x,'shear energy report',nonnegative=i in (0,1,2,4,5,7))
            for i,x in enumerate(frame['momentum']):finite_metric(x,'shear momentum report',nonnegative=i==5)
            native_residual=finite_metric(frame['residual_max'],'shear residual maximum',nonnegative=True)
            native_new=[finite_metric(x,'native shear velocity') for x in frame['velocity']]
            # Preserve a separate exact reference trajectory. It never replaces
            # the actual stored old/new fields in the equation or ledger gates.
            reference_new=gaussian(a,[masses[i]*reference_old[i]+dt*q*volumes[i] for i in range(n)])
            for native,expected in zip(frame['velocity'],reference_new):close(native,expected,'exact backward Euler field')
            residuals=[];scales=[]
            for j,v in enumerate(native_new):
                inertia=masses[j]*(v-native_old[j])/dt
                stress=(native_interior[j-1]*(v-native_new[j-1]) if j else 0)
                stress+=(native_interior[j]*(v-native_new[j+1]) if j+1<n else 0)
                stress+=native_low*v if j==0 else 0
                stress+=native_high*v if j+1==n else 0
                load=q*volumes[j]
                residuals.append(abs(inertia+stress-load));scales.append(abs(inertia)+abs(stress)+abs(load))
            residual_max=max(residuals);residual_scale=max(scales)
            equation_budget=4096*EPS*residual_scale
            require(residual_max<=equation_budget,'actual shear momentum equation gate')
            require(native_residual<=equation_budget,'reported shear momentum equation gate')
            roundoff_close(frame['residual_max'],residual_max,residual_scale,'actual momentum residual report')
            e0=sum(m*u*u/2 for m,u in zip(masses,native_old));e1=sum(m*v*v/2 for m,v in zip(masses,native_new))
            inc=sum(m*(v-u)**2/2 for m,u,v in zip(masses,native_old,native_new))
            work=dt*q*sum(volume*v for volume,v in zip(volumes,native_new))
            boundary=dt*(native_low*native_new[0]**2+native_high*native_new[-1]**2)
            loss=boundary+dt*sum(g*(native_new[j+1]-native_new[j])**2 for j,g in enumerate(native_interior))
            energy_error=e1-e0+inc+loss-work;energy_scale=e0+e1+inc+loss+abs(work)
            require(abs(energy_error)<=2048*EPS*energy_scale,'actual shear energy identity gate')
            work_scale=dt*abs(q)*sum(abs(volume*v) for volume,v in zip(volumes,native_new))
            for native,value,scale in zip(frame['energy'][:7],[e0,e1,inc,work,loss,boundary,energy_error],[e0,e1,inc,work_scale,loss,boundary,energy_scale]):roundoff_close(native,value,scale,'actual shear energy ledger')
            energy=list(map(F,frame['energy']))
            check_budget(frame['energy'][7],2048,energy[0]+energy[1]+energy[2]+energy[4]+abs(energy[3]),'shear energy rounding budget')
            require(abs(energy[6])<=energy[7],'reported shear energy identity gate')
            p0=sum(m*u for m,u in zip(masses,native_old));p1=sum(m*v for m,v in zip(masses,native_new))
            body=dt*q*sum(volumes);wall=-dt*(native_low*native_new[0]+native_high*native_new[-1])
            momentum_error=p1-p0-body-wall;momentum_scale=abs(p0)+abs(p1)+abs(body)+abs(wall)
            require(abs(momentum_error)<=2048*EPS*momentum_scale,'actual shear momentum identity gate')
            p0_scale=sum(abs(m*u) for m,u in zip(masses,native_old));p1_scale=sum(abs(m*v) for m,v in zip(masses,native_new))
            wall_scale=dt*(abs(native_low*native_new[0])+abs(native_high*native_new[-1]))
            for native,value,scale in zip(frame['momentum'][:5],[p0,p1,body,wall,momentum_error],[p0_scale,p1_scale,abs(body),wall_scale,momentum_scale]):roundoff_close(native,value,scale,'actual shear momentum ledger')
            momentum=list(map(F,frame['momentum']))
            check_budget(frame['momentum'][5],2048,sum(abs(x) for x in momentum[:4]),'shear momentum rounding budget')
            require(abs(momentum[4])<=momentum[5],'reported shear momentum identity gate')
            reference_old=reference_new
            native_old=native_new
        require(6*8*n<=c['workspace_bytes']<=1_000_000,'shear capacity payload')
    return {'pressure_cases':4,'shear_cases':6,'pressure_projections':4,'shear_updates':48,
            'rational_linear_solves':48,'continuum_reference_integrations':0,
            'scope':'sealed stationary pressure graph; separate fully developed reduced shear',
            'field_allowance':'4096 * binary64 epsilon * max(1, abs(exact control))',
            'pressure_divergence_limit':1e-10,
            'pressure_gauge':'largest represented positive volume per component; lowest-index exact ties',
            'actual_field_equation_allowance':'4096 * epsilon * max(abs(inertia)+abs(stress)+abs(load))',
            'actual_field_energy_allowance':'1024 epsilon pressure; 2048 epsilon shear, times actual-field ledger term scale',
            'actual_field_momentum_allowance':'2048 * epsilon * (abs(Pold)+abs(Pnew)+abs(body impulse)+abs(wall impulse))',
            'reported_term_arithmetic_allowance':'64 * epsilon * contributing-term scale; no absolute floor',
            'budget_formula_arithmetic_allowance':'16 * epsilon * derived budget; no absolute floor'}

def source_hashes(repo):return {name:sha(Path(repo)/name) for name in SOURCES}

def qualify(repo,records,executable):
    repo=Path(repo);records=Path(records);result=controls(json.loads(records.read_text()))
    result.update(schema='rheon-obstacle-flow-qualification-v1',records_sha256=sha(records),sources=source_hashes(repo),elf_sha256=sha(executable),
                  source_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
                  source_tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip(),
                  clean=not subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True).strip())
    return result

def validate_packet(repo,directory):
    if directory is None:return None
    directory=Path(directory)
    require({p.name for p in directory.iterdir()}=={'records.json','qualification.json'} and all(p.is_file() and not p.is_symlink() for p in directory.iterdir()),'flow packet regular files')
    receipt=json.loads((directory/'qualification.json').read_text());require(receipt.get('schema')=='rheon-obstacle-flow-qualification-v1' and receipt.get('clean') is True,'frozen clean flow source')
    require(receipt['sources']==source_hashes(repo) and receipt['records_sha256']==sha(directory/'records.json'),'stale flow source/records')
    for key,query in [('source_head','HEAD'),('source_tree','HEAD^{tree}')]:require(receipt[key]==subprocess.check_output(['git','rev-parse',query],cwd=repo,text=True).strip(),'flow source identity')
    result=controls(json.loads((directory/'records.json').read_text()));require(all(receipt.get(k)==v for k,v in result.items()),'flow control receipt');return receipt

def publish(repo,output,directory=None):
    receipt=validate_packet(repo,directory);output=Path(output)
    (output/'obstacle-flow-presentation.json').write_text(json.dumps({'schema':'rheon-obstacle-flow-presentation-v1','included':receipt is not None,'qualification':receipt},indent=2)+'\n')
    if receipt is None:return '<h1>Pressure and wall shear.</h1><p>Recorded native obstacle-flow fields are absent in this source-only edition. No substitute was generated.</p><p><a href="implementation/static-obstacle-flow.html">Read the bounded operator contract →</a></p>'
    for name in ['records','qualification']:shutil.copy2(Path(directory)/(name+'.json'),output/('obstacle-flow-'+name+'.json'))
    shutil.copy2(Path(repo)/'docs/education/obstacle_flow.js',output/'obstacle_flow.js')
    return '''<p class="eyebrow">RECORDED NATIVE FIELD RESPONSE</p><h1>Pressure and wall shear.</h1><p class="lede">One retained triangle obstacle supplies collision geometry, fluid volumes and shared openings to two explicitly bounded operators.</p><p role="note">Saved native binary64 fields, checked against independent rational controls. The pressure projection has sealed stationary walls. The separate shear mode has invariant/periodic tangential directions, a stationary embedded lower wall and stationary upper wall. This is not a composed 3D fluid solver.</p><h2>Remove a pressure gradient</h2><label for="flow-pressure-case">Pressure control</label><select id="flow-pressure-case"></select><label for="flow-state">Face speeds</label><select id="flow-state"><option value="before">Before projection</option><option value="after">After projection</option></select><label for="flow-slice">Z cell slice</label><select id="flow-slice"><option>0</option><option>1</option><option>2</option></select><svg id="flow-pressure-view" viewBox="0 0 600 400" role="img" aria-label="Pressure cell values and in-plane face velocities on the selected Z slice" style="width:100%;max-height:420px"></svg><dl id="flow-pressure-metrics" aria-live="polite"></dl><h2>Respond to force and wall friction</h2><p>Compare fixed force density with fixed acceleration as density changes, or no-slip with finite Navier friction and free slip. Friction is not adhesion or contact-angle evolution.</p><label for="flow-shear-case">Reduced shear control</label><select id="flow-shear-case"></select><label for="flow-step">Saved backward-Euler step</label><input id="flow-step" type="range" min="0" max="8" step="1" value="8"><svg id="flow-shear-view" viewBox="0 0 600 350" role="img" aria-label="Recorded tangential velocity profile above the retained obstacle wall" style="width:100%;max-height:380px"></svg><dl id="flow-shear-metrics" aria-live="polite"></dl><p id="flow-status" role="status">Loading native fields…</p><p><a href="implementation/static-obstacle-flow.html">Equations, units, proof assumptions and open work →</a> · <a href="obstacle-flow-records.json">Native fields</a> · <a href="obstacle-flow-qualification.json">Rational controls and source binding</a></p>'''

def verify_published(repo,output):
    output=Path(output);presentation=json.loads((output/'obstacle-flow-presentation.json').read_text());require(presentation.get('schema')=='rheon-obstacle-flow-presentation-v1','flow presentation schema')
    require(type(presentation.get('included')) is bool,'flow inclusion flag')
    if not presentation['included']:
        require(presentation.get('qualification') is None and not (output/'obstacle-flow-records.json').exists(),'absent flow records');return None
    receipt=json.loads((output/'obstacle-flow-qualification.json').read_text());require(presentation['qualification']==receipt and receipt['clean'] is True,'flow presentation binding')
    require(receipt['sources']==source_hashes(repo) and receipt['records_sha256']==sha(output/'obstacle-flow-records.json'),'published flow source/record binding')
    for key,query in [('source_head','HEAD'),('source_tree','HEAD^{tree}')]:require(receipt[key]==subprocess.check_output(['git','rev-parse',query],cwd=repo,text=True).strip(),'published flow source identity')
    require(sha(output/'obstacle_flow.js')==sha(Path(repo)/'docs/education/obstacle_flow.js'),'published flow renderer binding')
    result=controls(json.loads((output/'obstacle-flow-records.json').read_text()));require(all(receipt.get(k)==v for k,v in result.items()),'published rational controls');return receipt

if __name__=='__main__':
    parser=argparse.ArgumentParser();parser.add_argument('--records',required=True);parser.add_argument('--executable',required=True);parser.add_argument('--receipt',required=True);a=parser.parse_args()
    result=qualify(Path(__file__).resolve().parents[2],a.records,a.executable)
    with Path(a.receipt).open('x') as f:json.dump(result,f,indent=2);f.write('\n')
    print(json.dumps({k:v for k,v in result.items() if k not in ('sources',)},indent=2))
