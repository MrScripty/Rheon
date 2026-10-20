"""Always-on structural/corpus gates; independent exact arithmetic is separate."""
import csv
import json
import math
import pathlib
import struct
import subprocess
import sys

def require(condition, message):
    if not condition:
        raise SystemExit('v3 input/gate refused: ' + message)

def rows(path):
    with path.open() as stream:
        return list(csv.DictReader(stream))

def number(bits):
    return struct.unpack('>d', struct.pack('>Q', int(bits)))[0]

root = pathlib.Path(sys.argv[1])
old = pathlib.Path(sys.argv[2])
finite = json.loads(subprocess.check_output([sys.executable, str(pathlib.Path(__file__).parent.parent/'q2/analyze.py'), str(root)]))
require(finite['native_geometry_candidate'], 'retained finite integration gates')
counts = dict(faces=0, regions=0, positive_faces=0)
regressions = [('disk',32,1,143), ('disk',64,0,2088), ('disk',128,0,8272), ('disk',128,1,2111), ('disk',128,1,14400)]
regression_results = []
for case in ['disk','square-0','square-17','square-31']:
    for n in ([32,64,128] if case == 'disk' else [16,32,64]):
        tag = f'{case}-n{n}'
        require((root/f'inputs-{tag}.csv').read_bytes() == (old/f'inputs-{tag}.csv').read_bytes(), tag+' unchanged represented inputs')
        a,b = [root/f'{tag}-mu{mu}' for mu in ['0.1','0.001002']]
        for filename in ['source.csv','moments.csv','faces.csv','topology.csv','domains.csv']:
            require((a/filename).read_bytes()==(b/filename).read_bytes(),tag+' mu-independent '+filename)
        moments = rows(a/'moments.csv'); domains=rows(a/'domains.csv'); topology=rows(a/'topology.csv'); faces=rows(a/'faces.csv')
        expected = {(kind,i) for kind,size in [('cell',n*n),('dual-x',n*(n+1)),('dual-y',n*(n+1)),('shear',(n+1)**2)] for i in range(size)}
        def keyed(data):
            result={(r['kind'],int(r['index'])):r for r in data}
            require(len(result)==len(data) and set(result)==expected,tag+' region association cardinality')
            return result
        mm,dd=keyed(moments),keyed(domains)
        for key,d in dd.items():
            value=number(d['volume_bits'])
            require(math.isfinite(value) and value>=0 and value==float(mm[key]['volume']),tag+' moment/domain volume identity')
            for axis in ['x','y']:
                lo,hi=[number(d[f'{bound}_{axis}_bits']) for bound in ['lo','hi']]
                require(math.isfinite(lo) and math.isfinite(hi) and lo<hi,tag+' ordered finite domain')
        tt={(int(t['axis']),int(t['index'])):t for t in topology}
        ff={(int(f['axis']),int(f['index'])):f for f in faces}
        require(len(tt)==len(topology)==2*n*(n+1),tag+' topology cardinality')
        require(set(tt)=={(axis,i) for axis in [0,1] for i in range(n*(n+1))},tag+' face IDs')
        require(len(ff)==len(faces),tag+' duplicate positive face')
        require(set(ff)<=set(tt),tag+' positive face IDs belong to topology')
        for (axis,index),t in tt.items():
            require(t['status'] in ['empty','point','positive'],tag+' face dimension enum')
            positive=t['status']=='positive'
            require(((axis,index) in ff)==positive,tag+' dimension/segment presence')
            length=float(t['represented_length'])
            require(math.isfinite(length) and (length>0 if positive else length==0),tag+' represented face length')
            dual=number(dd[('dual-x' if axis==0 else 'dual-y',index)]['volume_bits'])
            require(dual==float(t['dual_volume']),tag+' dual identity')
            if positive:
                require(length==float(ff[(axis,index)]['area']) and dual>0,tag+' positive face/dual')
                normal,tangent=(index%(n+1),index//(n+1)) if axis==0 else (index//n,index%n)
                require(0<normal<n,tag+' containing-grid boundary aperture')
                adjacent=[(normal-1)+n*tangent,normal+n*tangent] if axis==0 else [tangent+n*(normal-1),tangent+n*normal]
                require(all(float(mm[('cell',i)]['volume'])>0 for i in adjacent),tag+' positive face needs positive adjacent regions')
            if (case,n,axis,index) in regressions:
                require(not positive,tag+' retained artifact corrected')
                regression_results.append(dict(case=case,n=n,axis=axis,index=index,status=t['status'],length=length))
        counts['faces']+=len(tt);counts['regions']+=len(dd);counts['positive_faces']+=len(ff)
require(counts['faces']==76384 and counts['regions']==152780, 'complete fixed corpus')
require(len(regression_results)==5, 'all retained artifacts')
print(json.dumps(dict(native_structural_gate=True,counts=counts,regressions=regression_results,finite=finite,independent_exact_face_and_region_gate='REQUIRED',production_topology_or_coupled_water_admitted=False),indent=2,allow_nan=False))
