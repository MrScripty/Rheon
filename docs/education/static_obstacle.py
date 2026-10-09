"""Independent rational controls and publication of recorded native geometry.

No integration or pressure/reference solve. Exact represented endpoints are the
oracle inputs; non-dyadic binary64 assembly has a stated 64-epsilon allowance.
"""
from fractions import Fraction as F
from pathlib import Path
import argparse, hashlib, json, math, shutil, subprocess

SOURCES = ['src/static_obstacle.rs', 'src/collision.rs', 'src/geometry.rs',
           'src/lib.rs', 'examples/static_obstacle.rs', 'Cargo.toml', 'Cargo.lock',
           'docs/education/static_obstacle.py']
BASE = ([3,3,3], [1.0]*3, [0.0]*3)
FIXTURES = [
    ('partial', BASE, [0.25,0.5,0.75], [1.75,1.5,2.25]),
    ('separator-x', BASE, [1,0,0], [2,3,3]),
    ('separator-y', BASE, [0,1,0], [3,2,3]),
    ('separator-z', BASE, [0,0,1], [3,3,2]),
    ('dry', BASE, [-1]*3, [4]*3),
    ('outside', BASE, [4]*3, [5]*3),
    ('touch', BASE, [3,0,0], [4,3,3]),
    ('translated-scaled', ([3]*3,[0.5,1,2],[4,-8,16]), [4.25,-7.5,17], [5.25,-6.5,19]),
    ('non-dyadic', ([3,4,2],[0.3,0.2,0.7],[0.1,-0.2,0.4]), [0.21,-0.1,0.5], [0.62,0.37,1.3]),
]

def require(condition, message):
    if not condition:
        raise ValueError(message)

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def source_hashes(repo):
    return {name: sha(Path(repo)/name) for name in SOURCES}

def coordinates(dims):
    for k in range(dims[2]):
        for j in range(dims[1]):
            for i in range(dims[0]):
                yield (i,j,k)

def index(dims, p):
    return p[0]+dims[0]*(p[1]+dims[1]*p[2])

def intersect(a,b):
    return max(F(0), min(a[1],b[1])-max(a[0],b[0]))

def controls(records):
    require(records.get('schema')=='rheon-static-obstacle-records-v1' and records.get('fluid_advances')==0, 'record scope')
    require([c['id'] for c in records['cases']]==[f[0] for f in FIXTURES], 'original fixture roster')
    counts={'cases':0,'cells':0,'faces':0,'collision_probes':0,'flux_pairs':0,'topology_refusals':0}
    maximum=0.0
    for c, fixture in zip(records['cases'], FIXTURES):
        name,(dims,h,o),lo,hi=fixture
        require((c['counts'],c['spacing'],c['origin'],c['lower'],c['upper'])==(dims,h,o,lo,hi), 'changed fixture '+name)
        # Rational oracle integrates rectangles at the represented endpoints.
        edges=[[F(float(o[d]+k*h[d])) for k in range(dims[d]+1)] for d in range(3)]
        box=[(F(float(lo[d])),F(float(hi[d]))) for d in range(3)]
        vertices={tuple(F(float(hi[d] if corner & (1<<d) else lo[d])) for d in range(3)) for corner in range(8)}
        require(len(c['vertices'])==8 and {tuple(F(x) for x in v) for v in c['vertices']}==vertices, 'shared source vertices')
        require(c['stamp']==[counts['cases']+1,1], 'source stamp')
        require(len(c['triangles'])==12, 'closed source facets')
        edge_uses={}
        for triangle in c['triangles']:
            require(len(set(triangle))==3 and all(0<=v<8 for v in triangle), 'facet indices')
            a,b,z=[c['vertices'][v] for v in triangle]
            u=[F(b[d])-F(a[d]) for d in range(3)];v=[F(z[d])-F(a[d]) for d in range(3)]
            normal=[u[1]*v[2]-u[2]*v[1],u[2]*v[0]-u[0]*v[2],u[0]*v[1]-u[1]*v[0]]
            require(sum(normal[d]*(F(a[d])-(box[d][0]+box[d][1])/2) for d in range(3))>0, 'outward winding')
            for pair in zip(triangle,triangle[1:]+triangle[:1]):
                edge_uses.setdefault(tuple(sorted(pair)),[]).append(pair)
        require(all(len(uses)==2 and uses[0]==uses[1][::-1] for uses in edge_uses.values()), 'closed oriented edge incidence')
        def close(actual, exact, full):
            nonlocal maximum
            require(type(actual) in (float,int) and math.isfinite(actual), 'finite metric')
            error=abs(F(actual)-exact)
            allowance=F(0) if name!='non-dyadic' else F(64)*F(math.ulp(1.0))*full
            require(error<=allowance, 'analytic metric mismatch '+name)
            require((actual==0)==(exact==0), 'lost positive metric')
            maximum=max(maximum,float(error/full) if full else 0.0)
        expected_volumes=[]
        require(len(c['volumes'])==math.prod(dims) and len(c['labels'])==math.prod(dims), 'cell lengths')
        for p,actual in zip(coordinates(dims),c['volumes']):
            intervals=[(edges[d][p[d]],edges[d][p[d]+1]) for d in range(3)]
            full=math.prod(b-a for a,b in intervals)
            exact=full-math.prod(intersect(intervals[d],box[d]) for d in range(3))
            close(actual,exact,full);expected_volumes.append(exact);counts['cells']+=1
        domain=math.prod(edge[-1]-edge[0] for edge in edges)
        global_exact=domain-math.prod(intersect((edge[0],edge[-1]),box[d]) for d,edge in enumerate(edges))
        close(sum(c['volumes']),global_exact,domain)
        areas=[]
        parents=list(range(math.prod(dims)))
        def root(i):
            while parents[i]!=i:
                i=parents[i]
            return i
        for d in range(3):
            fd=dims.copy();fd[d]+=1
            require(len(c['areas'][d])==math.prod(fd), 'face length')
            axis_areas=[]
            for p,actual in zip(coordinates(fd),c['areas'][d]):
                tangents=[a for a in range(3) if a!=d]
                intervals=[(edges[a][p[a]],edges[a][p[a]+1]) for a in tangents]
                full=math.prod(b-a for a,b in intervals)
                blocked=math.prod(intersect(intervals[i],box[a]) for i,a in enumerate(tangents)) if box[d][0]<=edges[d][p[d]]<=box[d][1] else F(0)
                exact=full-blocked;close(actual,exact,full);axis_areas.append(exact);counts['faces']+=1
                if 0<p[d]<dims[d] and exact>0:
                    negative=list(p);negative[d]-=1
                    a,b=index(dims,negative),index(dims,p)
                    require(expected_volumes[a]>0 and expected_volumes[b]>0,'dry face neighbor')
                    parents[root(b)]=root(a)
            areas.append(axis_areas)
        roots={};labels=[]
        for i,volume in enumerate(expected_volumes):
            if not volume: labels.append(None)
            else:
                r=root(i);roots.setdefault(r,len(roots));labels.append(roots[r])
        require(c['labels']==labels and c['components']==len(roots), 'independent connectivity labels')
        require(len(c['probes'])==3,'collision probe roster')
        for d,probe in enumerate(c['probes']):
            require(probe['axis']==d and probe['stamp']==c['stamp'],'collision shared source')
            start,end=probe['start'],probe['end']
            require(start[d]<lo[d]<hi[d]<end[d] and all(start[a]==end[a] and lo[a]<start[a]<hi[a] for a in range(3) if a!=d),'crossing probe')
            t=(F(lo[d])-F(start[d]))/(F(end[d])-F(start[d]))
            require(abs(F(probe['t'])-t)<=F(64)*F(math.ulp(1.0)), 'analytic first hit')
            for a in range(3):
                x=F(start[a])+t*(F(end[a])-F(start[a]))
                require(abs(F(probe['position'][a])-x)<=F(64)*F(math.ulp(1.0))*max(F(1),abs(x)), 'analytic hit position')
            require(0<=probe['triangle']<12 and all(c['vertices'][v][d]==lo[d] for v in c['triangles'][probe['triangle']]), 'first facet is entry side')
            counts['collision_probes']+=1
        require(len(c['flux_controls'])==3,'flux roster')
        for d,flux in enumerate(c['flux_controls']):
            p=[1,1,1];fd=dims.copy();fd[d]+=1;negative=p.copy();negative[d]-=1
            require(flux['axis']==d and flux['coordinate']==p and flux['negative']==index(dims,negative) and flux['positive']==index(dims,p) and flux['speed']==2,'shared incidence')
            area=c['areas'][d][index(fd,p)]
            require(flux['area']==area and flux['outward']==[2*area,-2*area] and sum(flux['outward'])==0,'opposite flux cancellation')
            counts['flux_pairs']+=1
        cells=math.prod(dims);faces=sum(len(a) for a in c['areas']);retained,peak=c['allocation']
        require(retained>=480+16*cells+8*faces and peak-retained>=8*cells and peak<=1_000_000,'managed capacity scope')
        counts['cases']+=1
    require(len(records['refusals'])==3,'refusal roster')
    for d,refusal in enumerate(records['refusals']):
        lo=[0.0]*3;hi=[3.0]*3;lo[d]=1.25;hi[d]=1.75
        require(refusal['axis']==d and refusal['lower']==lo and refusal['upper']==hi and 'UnresolvedCellTopology' in refusal['error'],'unresolved separator refusal')
        counts['topology_refusals']+=1
    return {'checks':counts,'max_relative_assembly_error':maximum,'non_dyadic_allowance':'64 * binary64 epsilon * physical cell/face/domain measure','dyadic_allowance':0,'fluid_advances':0,'reference_integrations':0}

def qualify(repo, records, executable):
    repo=Path(repo);records=Path(records)
    result=controls(json.loads(records.read_text()))
    result.update(schema='rheon-static-obstacle-qualification-v1',records_sha256=sha(records),sources=source_hashes(repo),elf_sha256=sha(executable),
                  source_head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip(),
                  source_tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip(),
                  clean=not subprocess.check_output(['git','status','--porcelain'],cwd=repo,text=True).strip())
    return result

def validate_packet(repo, directory):
    if directory is None: return None
    directory=Path(directory)
    require({p.name for p in directory.iterdir()}=={'records.json','qualification.json'} and all(p.is_file() and not p.is_symlink() for p in directory.iterdir()),'geometry packet regular files')
    receipt=json.loads((directory/'qualification.json').read_text())
    require(receipt.get('schema')=='rheon-static-obstacle-qualification-v1' and receipt.get('clean') is True,'frozen clean geometry source')
    require(receipt['sources']==source_hashes(repo) and receipt['records_sha256']==sha(directory/'records.json'),'stale geometry source/records')
    head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
    require(receipt['source_head']==head and receipt['source_tree']==subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip(),'geometry source identity')
    result=controls(json.loads((directory/'records.json').read_text()))
    require(all(receipt.get(k)==v for k,v in result.items()),'geometry control receipt')
    return receipt

def publish(repo, output, directory=None):
    receipt=validate_packet(repo,directory)
    output=Path(output)
    (output/'obstacle-presentation.json').write_text(json.dumps({'schema':'rheon-obstacle-presentation-v1','included':receipt is not None,'qualification':receipt,'fluid_advances':0},indent=2)+'\n')
    if receipt is None:
        return '<h1>One obstacle, one geometry.</h1><p>Native static geometry records are absent in this source-only edition. No substitute was generated.</p><p><a href="implementation/static-obstacle-geometry.html">Read the bounded geometry contract →</a></p>'
    shutil.copy2(Path(directory)/'records.json',output/'obstacle-records.json')
    shutil.copy2(Path(directory)/'qualification.json',output/'obstacle-qualification.json')
    shutil.copy2(Path(repo)/'docs/education/obstacle.js',output/'obstacle.js')
    return '''<p class="eyebrow">RECORDED NATIVE STATIC GEOMETRY</p><h1>One obstacle, one geometry.</h1><p class="lede">The same closed triangle source supplies collision hits, fluid volumes, shared face openings and connected regions.</p><p role="note">These controls inspect nine saved geometry constructions. They do not advance fluid, solve pressure or recompute arbitrary parameters.</p><label for="obstacle-case">Geometry control</label><select id="obstacle-case"></select><section class="laboratory"><div id="obstacle-scene" tabindex="0" aria-label="Three dimensional grid, solid box and collision segment. Drag to rotate; scroll to zoom."></div><div class="lab-panel"><label for="obstacle-cell">Cell</label><select id="obstacle-cell"></select><label for="obstacle-axis">Shared face / collision direction</label><select id="obstacle-axis"><option value="0">X</option><option value="1">Y</option><option value="2">Z</option></select><dl id="obstacle-metrics" aria-live="polite"></dl><p>Blue and green distinguish connected fluid components. Grey cells are dry. Gold outlines the selected cell; cyan marks the fixed shared control face. Red is the original closed solid. The segment and first hit come from native collision queries on that solid.</p><button id="obstacle-reset">Reset camera</button></div></section><p id="obstacle-status" role="status">Loading recorded geometry…</p><h2>When one cell is too coarse</h2><p>A thin box spanning a cell in two directions can split its fluid into two pieces. The constructor refuses that unresolved topology instead of merging the pieces.</p><ul id="obstacle-refusals"></ul><p><a href="implementation/static-obstacle-geometry.html">Equations, assumptions and remaining solver gates →</a> · <a href="obstacle-records.json">Native records</a> · <a href="obstacle-qualification.json">Independent rational controls and source bindings</a></p>'''

def verify_published(repo, output):
    output=Path(output)
    presentation=json.loads((output/'obstacle-presentation.json').read_text())
    require(presentation.get('schema')=='rheon-obstacle-presentation-v1' and presentation.get('fluid_advances')==0,'geometry presentation scope')
    if not presentation['included']:
        require(presentation['qualification'] is None and not (output/'obstacle-records.json').exists(),'unexpected source-only geometry records')
        return None
    receipt=json.loads((output/'obstacle-qualification.json').read_text())
    require(receipt==presentation['qualification'] and receipt['clean'] is True,'geometry qualification binding')
    require(receipt['sources']==source_hashes(repo) and receipt['records_sha256']==sha(output/'obstacle-records.json'),'published geometry source/record binding')
    require(receipt['source_head']==subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip() and receipt['source_tree']==subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=repo,text=True).strip(),'published geometry source identity')
    require(sha(output/'obstacle.js')==sha(Path(repo)/'docs/education/obstacle.js'),'published geometry renderer binding')
    result=controls(json.loads((output/'obstacle-records.json').read_text()))
    require(all(receipt.get(k)==v for k,v in result.items()),'published geometry controls')
    return receipt

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--records',required=True,type=Path);parser.add_argument('--executable',required=True,type=Path);parser.add_argument('--receipt',required=True,type=Path)
    args=parser.parse_args()
    receipt=qualify(Path(__file__).resolve().parents[2],args.records,args.executable)
    with args.receipt.open('x') as file: file.write(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:v for k,v in receipt.items() if k!='sources'},indent=2))
