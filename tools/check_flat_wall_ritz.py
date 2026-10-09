#!/usr/bin/env python3
"""Exact stored-coordinate oracle + ACTUAL native algebra comparisons. NO solve.

Reconstructs C, every requested endpoint/sector and physical surface moments
independently. Never trusts native rows as its matrix/reference source. Forcing
is compared with exact rational polynomial integrals, without native velocity
or physical load input. Runtime/body-object/bit/output bounds refuse failures.
"""
import argparse, hashlib, json, math, subprocess, sys, time
from fractions import Fraction as F
from itertools import product
from pathlib import Path
from flat_wall_force_source import coefficients, write as write_force

MAX_BYTES=16_000_000; MAX_BITS=4096; FILE_CAP=1<<20
WORKING_RESERVE=131072
def require(test,message):
    if not test:raise ValueError(message)
def rational(v):
    r=F(v)
    require(max(r.numerator.bit_length(),r.denominator.bit_length())<=MAX_BITS,'rational bit cap')
    return r
def file_digest(path):
    digest=hashlib.sha256()
    with path.open('rb') as stream:
        while chunk:=stream.read(8192):digest.update(chunk)
    return digest.hexdigest()
def managed(*objects):
    seen=set()
    def size(o):
        if id(o) in seen:return 0
        seen.add(id(o));total=sys.getsizeof(o)
        if isinstance(o,F):return total+size(o.numerator)+size(o.denominator)
        if isinstance(o,dict):return total+sum(size(k)+size(v) for k,v in o.items())
        if isinstance(o,(tuple,list,set)):return total+sum(size(v) for v in o)
        if isinstance(o,Model):return total+size(vars(o))
        return total
    data=sum(size(o) for o in objects)
    # The visitor's identity set is itself a live allocation. IDs are retained
    # Python integers; account their bodies plus the actual set table, along
    # with bounded parser/hash/formatting work. Stack/interpreter/RSS excluded.
    value=data+sys.getsizeof(seen)+sum(sys.getsizeof(i) for i in seen)+WORKING_RESERVE
    require(value<=MAX_BYTES,'comparison managed-object cap')
    return value
def nearest(actual,exact,scale,operations):
    actual=rational(actual);exact=rational(exact);scale=rational(scale)
    if scale==0:require(actual==exact==0,'nonzero primitive with zero independent scale');return
    # Bounded unit consistency tolerance from independent absolute contribution
    # sums. No absolute dimensional floor and no physical convergence claim.
    bound=F(16)*F.from_float(sys.float_info.epsilon)*(operations+1)*scale
    require(abs(actual-exact)<=bound,'nearest arithmetic primitive mismatch')
    if exact and abs(exact)>bound:require((actual>0)==(exact>0),'primitive sign mismatch')
def contained(pair,exact):
    require(len(pair)==2 and rational(pair[0])<=exact<=rational(pair[1]),'outward arithmetic interval misses exact value')

class Model:
    def __init__(self,n):
        self.n=n;self.m=n//3;self.h=3./n
        self.p=[float(i)*self.h for i in range(n+1)]
        self.c=[(float(i)+.5)*self.h for i in range(n)]
    def face_shape(self,a):return tuple(self.n+(d==a) for d in range(3))
    def index(self,a,p):
        s=self.face_shape(a);return p[0]+s[0]*(p[1]+s[1]*p[2])
    def coordinate(self,a,i):
        s=self.face_shape(a);return (i%s[0],(i//s[0])%s[1],i//(s[0]*s[1]))
    def active(self,a,p):
        m=self.m
        return p[a] not in (0,self.n) and not (m<=p[a]<=2*m and all(m<=p[d]<2*m for d in range(3) if d!=a))
    def area(self,a,p):
        if not self.active(a,p):return 0.
        d=[i for i in range(3) if i!=a];return (self.p[p[d[0]]+1]-self.p[p[d[0]]])*(self.p[p[d[1]]+1]-self.p[p[d[1]]])
    def position(self,a,p):return [self.p[p[d]] if d==a else self.c[p[d]] for d in range(3)]
    def columns(self):
        result=[];m=self.m
        for z,y,x in product(range(m,2*m),range(1,m),range(m+1,2*m)):
            terms=[]
            for a,p,sign in [(0,(x,y-1,z),1),(0,(x,y,z),-1),(1,(x-1,y,z),-1),(1,(x,y,z),1)]:
                terms.append([a,self.index(a,p),float(sign)/self.area(a,p)])
            result.append({'node':[x,y,z],'terms':terms})
        return result
    def sites(self):
        m=self.m;sites=[]
        for z,y,x in product(range(m,2*m),range(m),range(m,2*m)):
            sites.extend((a,a,[x,y,z],-1) for a in range(3))
        def edges(a,b,ranges,quads):
            for z,y,x in product(*ranges):
                for q in quads:sites.extend([(a,b,[x,y,z],q),(b,a,[x,y,z],q)])
        edges(0,1,(range(m,2*m),range(1,m),range(m,2*m+1)),range(4))
        edges(0,1,(range(m,2*m),[m],range(m+1,2*m)),range(2))
        edges(0,2,(range(m,2*m+1),range(m),range(m+1,2*m)),range(4))
        edges(1,2,(range(m,2*m+1),range(1,m),range(m,2*m)),range(4))
        return sites
    def row(self,site):
        a,d,p,q=site;m=self.m
        if a==d:
            lower=list(p);upper=list(p);upper[d]+=1
            widths=[self.p[p[i]+1]-self.p[p[i]] for i in range(3)]
            weight=(widths[0]*widths[1])*widths[2]
            positions=[self.position(a,lower),self.position(a,upper)]
            faces=[self.index(a,t) if self.active(a,t) else -1 for t in (lower,upper)]
        else:
            b0,b1=sorted((a,d));c=3-b0-b1
            sectors=[]
            for quadrant in range(4):
                cell=list(p)
                if not quadrant&1:cell[b0]-=1
                if not quadrant&2:cell[b1]-=1
                if not all(m<=i<2*m for i in cell):sectors.append((quadrant,cell))
            require(len(sectors) in (2,4),'oracle unsupported row')
            cell=dict(sectors)[q]
            weight=(abs(self.c[cell[b0]]-self.p[p[b0]])*abs(self.c[cell[b1]]-self.p[p[b1]]))*(self.p[p[c]+1]-self.p[p[c]])
            lower=list(p);lower[d]-=1;upper=list(p)
            positions=[self.position(a,lower),self.position(a,upper)]
            faces=[self.index(a,t) if self.active(a,t) else -1 for t in (lower,upper)]
            if len(sectors)==2:
                first,last=sectors[0][0],sectors[-1][0];normal=b1 if first^last==1 else b0
                if d==normal:
                    fluid=1 if first&(1 if d==b0 else 2) else 0;trace=1-fluid
                    positions[trace]=positions[fluid].copy();positions[trace][d]=self.p[p[d]];faces[trace]=-1
                else:
                    faces=[-1,-1]
                    for pos in positions:pos[a]=self.p[p[a]]
        inverse=1./(positions[1][d]-positions[0][d])
        return {'component':a,'derivative':d,'cell':p,'quadrant':q,'weight':weight,
                'endpoints':[{'face':face,'coefficient':sign*inverse,'position':pos} for face,sign,pos in zip(faces,[-1,1],positions)]}
    def integral(self,a,face,terms):
        p=self.coordinate(a,face)
        if not self.active(a,p):return None
        low=[];high=[]
        for d in range(3):
            l=self.c[p[d]-1] if d==a else self.p[p[d]]
            u=self.c[p[d]] if d==a else self.p[p[d]+1]
            low.append(max(l,[1.,0.,1.][d]));high.append(min(u,[2.,1.,2.][d]))
        if any(u<=l for l,u in zip(low,high)):return None
        value=F(0)
        for (component,i,j,k),coefficient in terms.items():
            if component!=a:continue
            t=coefficient
            for lo,hi,shift,power in zip(low,high,[1,0,1],[i,j,k]):
                l=rational(lo)-shift;u=rational(hi)-shift;t*= (u**(power+1)-l**(power+1))/(power+1)
            value+=t
        widths=[high[d]-low[d] for d in range(3)]
        geometric=(widths[0]*widths[1])*widths[2]
        stored=self.area(a,p)*(self.c[p[a]]-self.c[p[a]-1])
        return rational(value),geometric,stored

def traction(model,velocity,scheme):
    m=model.m;p=list(map(rational,model.p));c=list(map(rational,model.c));wall=p[m];force=[F(0)]*3;torque=[F(0)]*3
    def add(a,stress,area,position):
        f=stress*area;force[a]+=f;r=[x-F(3,2) for x in position]
        if a==0:torque[1]+=r[2]*f;torque[2]-=r[1]*f
        else:torque[0]-=r[2]*f;torque[2]+=r[0]*f
    for z,x in product(range(m,2*m),range(m+1,2*m)):
        v=velocity.get((0,model.index(0,(x,m-1,z))),F(0));tx=v/(wall-c[m-1])
        hl=p[x]-p[x-1];hr=p[x+1]-p[x];hat=(hl+hr)/2
        add(0,tx,hat*(p[z+1]-p[z]),[p[x]+(hr-hl)/3,wall,(p[z]+p[z+1])/2])
    for z,x in product(range(m,2*m),range(m,2*m)):
        v1=velocity.get((1,model.index(1,(x,m-1,z))),F(0));d1=wall-p[m-1]
        derivative=-v1/d1
        if scheme=='normal_p2':
            v2=velocity.get((1,model.index(1,(x,m-2,z))),F(0));d2=wall-p[m-2]
            derivative=-d2*v1/(d1*(d2-d1))+d1*v2/(d2*(d2-d1))
        add(1,-2*derivative,(p[x+1]-p[x])*(p[z+1]-p[z]),[(p[x]+p[x+1])/2,wall,(p[z]+p[z+1])/2])
    return force,torque

def compare(path,source_path,head,binary,physical=False):
    begin=time.monotonic();require(path.stat().st_size<=FILE_CAP,'record file cap')
    records=[]
    retained_records=0
    allowed_kinds={'header','source_term','column','row','matrix','source_integral','velocity','traction','divergence','solve','runner_bound'}
    with path.open('rb') as f:
        while line:=f.readline(8193):
            require(len(line)<=8192 and line.endswith(b'\n'),'record line cap')
            require(sum(line.count(c) for c in (b'[',b']',b'{',b'}',b',',b':'))<=128,'record structural-node cap')
            require(retained_records+sys.getsizeof(records)+WORKING_RESERVE<=MAX_BYTES,'preparse managed record cap')
            record=json.loads(line.decode('ascii'))
            require(isinstance(record,dict) and len(record)<=16 and record.get('kind') in allowed_kinds,'unsupported record schema')
            # Per-record tracking is bounded by the lexical node/line gate; its
            # workspace is already in the128KiB parser reserve. Sum record bodies
            # conservatively without cross-record deduplication before retention.
            record_bound=managed(record)-WORKING_RESERVE
            require(retained_records+record_bound+sys.getsizeof(records)+WORKING_RESERVE<=MAX_BYTES,'parsed record admission cap')
            retained_records+=record_bound;records.append(record)
    by={}
    for r in records:by.setdefault(r['kind'],[]).append(r)
    require((len(by.get('solve',[]))==1) if physical else ('solve' not in by),'solve record/mode mismatch')
    require(len(by.get('header',[]))==1,'missing/duplicate header');header=by['header'][0];n=header['n'];model=Model(n)
    require(n in (6,9,12) and header['h']==model.h,'geometry header')
    require(header['head']==head and header['mode']==('numerical_reduced_ritz_solve' if physical else 'supplied_synthetic_q_algebra_no_solve'),'source/mode mismatch')
    require(header['pressure_available'] is False and header['physical_qualified'] is False,'qualification claim')
    require(header['source_sha256']==file_digest(source_path),'source hash mismatch')
    journal=(path.parent/'acquisition.txt').read_text();require(('solver_report=Some(' in journal) if physical else ('solver_report=None' in journal),'solver provenance mismatch')
    require('binary_sha256='+file_digest(binary) in journal,'binary journal mismatch')
    columns=model.columns();require(len(by['column'])==len(columns),'column count')
    for i,(actual,expected) in enumerate(zip(by['column'],columns)):
        require(actual['index']==i and actual['node']==expected['node'] and actual['terms']==expected['terms'],'independent C mismatch')
    source_terms=coefficients()
    observed_terms={(r['component'],*r['powers']):rational(r['coefficient']) for r in by['source_term']}
    require(observed_terms==source_terms and len(by['source_term'])==len(source_terms),'forcing polynomial mismatch')
    del observed_terms
    expected_rows=[model.row(s) for s in model.sites()];require(len(by['row'])==len(expected_rows),'row count')
    for i,(actual,expected) in enumerate(zip(by['row'],expected_rows)):
        require(actual['index']==i,'row order')
        require({k:v for k,v in actual.items() if k not in ('kind','index')}==expected,'independent row primitive mismatch')
    nq=len(columns);cd=[{(a,face):rational(c) for a,face,c in col['terms']} for col in columns]
    matrix=[[F(0) for _ in cd] for _ in cd];scales=[[F(0) for _ in cd] for _ in cd];groups={}
    for row in expected_rows:
        a=row['component'];d=row['derivative'];p=tuple(row['cell']);q=row['quadrant']
        key=(a,a,p,q) if a==d else (*sorted((a,d)),p,q)
        groups.setdefault(key,[]).append(row)
    def row_values(row):
        a=row['component']
        return [sum((rational(e['coefficient'])*col.get((a,e['face']),F(0)) for e in row['endpoints'] if e['face']>=0),F(0)) for col in cd]
    for group in groups.values():
        row=group[0];g=row_values(row);w=rational(row['weight'])*(2 if row['component']==row['derivative'] else 1)
        if row['component']!=row['derivative']:
            require(len(group)==2,'missing reverse shear');g=[a+b for a,b in zip(g,row_values(group[1]))]
        for i,j in product(range(nq),repeat=2):
            v=w*g[i]*g[j];matrix[i][j]+=v;scales[i][j]+=abs(v)
    memory=managed(locals())
    force={};max_error=F(0);max_width=F(0);integrals={}
    for a in range(3):
        for face in range(math.prod(model.face_shape(a))):
            result=model.integral(a,face,source_terms)
            if result is not None:force[(a,face)]=result[0];integrals[(a,face)]=result
    memory=max(memory,managed(locals()))
    require(len(by['source_integral'])==len(integrals),'source dual roster')
    for r in by['source_integral']:
        key=(r['component'],r['face']);exact,geometric,stored=integrals.pop(key)
        contained(r['interval'],exact);contained(r['interval'],rational(r['force']))
        require(r['geometric_volume']==geometric and r['stored_volume']==stored,'dual/stored volume conflation')
        max_error=max(max_error,abs(rational(r['force'])-exact));max_width=max(max_width,rational(r['interval'][1])-rational(r['interval'][0]))
    require(not integrals,'missing integral')
    require(len(by['matrix'])==nq,'matrix row count')
    native_intervals={(r['component'],r['face']):tuple(map(rational,r['interval'])) for r in by['source_integral']}
    memory=max(memory,managed(locals()))
    max_matrix_error=F(0)
    for i,r in enumerate(by['matrix']):
        require(r['index']==i and len(r['values'])==nq,'matrix shape')
        for j,v in enumerate(r['values']):nearest(v,matrix[i][j],scales[i][j],len(groups));max_matrix_error=max(max_matrix_error,abs(rational(v)-matrix[i][j]))
        expected_rhs=sum((c*force.get(key,F(0)) for key,c in cd[i].items()),F(0))
        # Source arithmetic is independently enclosed, so use the native source
        # intervals for a projection enclosure ONLY after their exact verification.
        rhslo=F(0);rhshi=F(0)
        for key,c in cd[i].items():
            lo,hi=native_intervals.get(key,(F(0),F(0)));rhslo+=min(c*lo,c*hi);rhshi+=max(c*lo,c*hi)
        require(rhslo<=rational(r['rhs'])<=rhshi and rhslo<=expected_rhs<=rhshi,'projected source enclosure')
    q=header['q'];require(len(q)==nq and all(math.isfinite(v) for v in q),'q shape/finite')
    if not physical:require(q==[(i+1)/64 for i in range(nq)],'unit q control')
    velocity={}
    for col,value in zip(columns,q):
        for a,face,c in col['terms']:velocity[(a,face)]=velocity.get((a,face),0.)+c*value
    native_velocity={(r['component'],r['face']):r['value'] for r in by['velocity']}
    require(native_velocity=={k:v for k,v in velocity.items() if v},'actual owned curl values')
    velocity={k:rational(v) for k,v in velocity.items()}
    require(len(by['traction'])==2,'traction schemes')
    for r in by['traction']:
        require(r['scheme'] in ('p1','normal_p2'),'unknown traction scheme');ef,et=traction(model,velocity,r['scheme'])
        for i in range(3):contained(r['force_interval'][i],ef[i]);contained(r['torque_interval'][i],et[i]);contained(r['force_interval'][i],rational(r['force'][i]));contained(r['torque_interval'][i],rational(r['torque'][i]))
        if n in (6,12) and r['scheme']=='p1':require(et[2]==(F(1,2)-F(3,n))*ef[0],'lost P1 coarse failure')
        if n==6 and r['scheme']=='normal_p2':
            require(et[2]==-ef[0]/2,'lost P2 coarse law')
            if not physical:require(ef[0]<0 and et[2]>0,'lost P2 coarse wrong sign')
    dmax=F(0)
    for p in product(range(n),repeat=3):
        if all(model.m<=i<2*model.m for i in p):continue
        d=F(0)
        for a in range(3):
            upper=list(p);upper[a]+=1
            for point,sign in [(p,-1),(upper,1)]:
                face=model.index(a,point);d+=sign*rational(model.area(a,point))*velocity.get((a,face),F(0))
        dmax=max(dmax,abs(d))
    require(len(by['divergence'])==1 and rational(by['divergence'][0]['arithmetic_bound'])>=dmax,'divergence enclosure')
    memory=max(memory,managed(locals()))
    require(time.monotonic()-begin<=180,'comparison timeout')
    semantic=hashlib.sha256()
    for r in records:
        if r['kind']!='runner_bound':semantic.update((json.dumps(r,sort_keys=True)+'\n').encode())
    return {'n':n,'q':nq,'rows':len(expected_rows),'blocks':len(groups),'exact_source_duals':len(by['source_integral']),
            'max_source_nearest_error':str(max_error),'max_source_interval_width':str(max_width),'max_matrix_exact_stored_coefficient_error':str(max_matrix_error),
            'exact_rounded_field_integrated_divergence_max':str(dmax),'comparison_managed_bytes':memory,
            'native_managed_peak':by['runner_bound'][0]['managed_peak'],'native_record_sha256':file_digest(path),
            'semantic_sha256':semantic.hexdigest(),'physical_solve_executed':physical}

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--binary',type=Path,required=True);ap.add_argument('--head',required=True);ap.add_argument('--output',type=Path,required=True);args=ap.parse_args()
    repo=Path(__file__).resolve().parents[1];head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()
    require(head==args.head and not subprocess.check_output(['git','status','--porcelain=v1'],cwd=repo,text=True),'source must be frozen and clean')
    root=args.output.resolve();require(not root.is_relative_to(repo),'outputs must stay outside Git');root.mkdir(parents=True,exist_ok=False)
    source=root/'body-force.csv';source_receipt=write_force(source);reports=[]
    for n in (6,9,12):
        out=root/f'n{n}'
        with (root/f'n{n}.log').open('wb') as log:
            completed=subprocess.run([str(args.binary.resolve()),'--algebra',str(n),str(source),str(out)],stdout=log,stderr=log,timeout=180)
        require(completed.returncode==0,f'native algebra N{n} failed')
        require(sum(p.stat().st_size for p in out.iterdir())<=FILE_CAP,'native per-level total output cap')
        worker=repo/'tools/flat_wall_compare_worker.py';comparison=root/f'n{n}-comparison.json'
        with (root/f'n{n}-comparison.log').open('wb') as log:
            child=subprocess.run([sys.executable,str(worker),'--records',str(out/'records.jsonl'),'--source',str(source),'--head',head,'--binary',str(args.binary.resolve()),'--output',str(comparison)],stdout=log,stderr=log,timeout=180)
        require(child.returncode==0,f'exact comparison N{n} failed');reports.append(json.loads(comparison.read_text()))
    require(subprocess.check_output(['git','rev-parse','HEAD'],cwd=repo,text=True).strip()==head and not subprocess.check_output(['git','status','--porcelain=v1'],cwd=repo,text=True),'source changed during qualification')
    report={'source_head':head,'source_forcing':source_receipt,'native_binary_sha256':file_digest(args.binary),
            'mode':'bounded actual native supplied-q algebra; no physical solve','physical_solve_executed':False,'runs':reports}
    (root/'qualification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
