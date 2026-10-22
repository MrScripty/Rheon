"""Prospective candidate-free native geometry/load/factor/physical audit."""
from collections import defaultdict
from fractions import Fraction as F
import exact_polynomial as ep
import exact_operators as eo
from continuum import fields, exact_loads
from native_csv import require, integer, fraction, from_bits, GATE
from validated import action, dot, multiply, transpose, norm_inf


class Gates:
    def __init__(self):
        self.values = {}

    def record(self, name, value, limit=GATE):
        value,limit = F(value),F(limit)
        require(value >= 0 and limit > 0, 'finite nonnegative gate '+name)
        old = self.values.get(name)
        value = max(value,old['value']) if old else value
        self.values[name] = {'value':value,'limit':limit,'pass':value <= limit}

    def diff(self, name, got, expected, scale):
        scale = F(scale); require(scale > 0, 'positive fixed scale '+name)
        self.record(name,abs(F(got)-F(expected))/scale)

    def matrix(self, name, got, expected):
        require(len(got) == len(expected) and all(len(a) == len(b) for a,b in zip(got,expected)), 'matching matrix '+name)
        scale = norm_inf(expected); require(scale > 0, 'positive matrix scale '+name)
        self.record(name,norm_inf([[a-b for a,b in zip(ar,br)] for ar,br in zip(got,expected)])/scale)

    def passed(self):
        return bool(self.values) and all(row['pass'] for row in self.values.values())


def structure(data, op, gates, budget):
    n,h = op['n'],op['h']; nv,nc = len(op['active']),len(op['cells'])
    require((nv,nc) == (data['nv'],data['nc']), 'exact positive support/pressure dimensions')
    amap = {g:i for i,g in enumerate(op['active'])}; cmap = {c:i for i,c in enumerate(op['cells'])}
    for g,node in enumerate(op['nodes']):
        r = data['nodes'][g]
        require(all(integer(r[k]) == node[k] for k in ('global','axis','face','i','j')), 'native MAC association')
        require((fraction(r['x']),fraction(r['y'])) == node['point'], 'phase-identity')
        require(r['active'] == (str(amap[g]) if g in amap else ''), 'exact positive support mask')
    for c,r in data['cells'].items():
        require(r['pressure'] == (str(cmap[c]) if c in cmap else ''), 'pressure-trace-binding')
        gates.diff('cell-volume',fraction(r['volume']),op['volumes'][c],h*h)
    patches = data['rows']['patches.csv']
    require(len(patches) == 4*n*n, 'complete common halfcell geometry')
    vertices = defaultdict(dict)
    for row in data['rows']['vertices.csv']:
        p,i = integer(row['patch']),integer(row['index'])
        require(p < len(patches) and i not in vertices[p], 'exact vertex association/duplicate')
        xn,xd,yn,yd = (int(row[k]) for k in ('x_num','x_den','y_num','y_den'))
        require(xd > 0 and yd > 0, 'positive rational geometry denominator')
        vertices[p][i] = F(xn,xd),F(yn,yd)
    for k,(row,patch) in enumerate(zip(patches,op['patches'])):
        budget.check()
        require(integer(row['patch']) == k and all(integer(row[d]) == patch[d] for d in ('i','j','cell')), 'common patch identity')
        require(tuple(from_bits(integer(row[key])) for key in ('lo_x_bits','lo_y_bits')) == patch['lo']
                and tuple(from_bits(integer(row[key])) for key in ('hi_x_bits','hi_y_bits')) == patch['hi'], 'exact common knot bit identity')
        count = integer(row['vertices']); require(set(vertices[k]) == set(range(count)), 'complete exact patch vertices')
        poly = [vertices[k][i] for i in range(count)]
        area = F(int(row['area_num']),int(row['area_den']))
        require(area == patch['area'] == ep.signed_twice_area(poly)/2, 'exact positive cut area')
        for point in poly:
            require(all(patch['lo'][d] <= point[d] <= patch['hi'][d] for d in range(2)), 'exact vertex clipping box')
            require(all(ep.cross((b[0]-a[0],b[1]-a[1]),(point[0]-a[0],point[1]-a[1])) >= 0
                        for a,b in zip(op['source'],op['source'][1:]+op['source'][:1])), 'exact vertex source containment')
    for name,expected in (('M',op['M']),('K',op['K']),('B',op['B']),('grid_flux',op['grid_flux']),('physical_flux',op['physical_flux'])):
        gates.matrix('exact-'+name,data['native'][name],expected)
    gates.matrix('exact-consistent-H1',data['h1'],op['H1'])
    maps = data['rows']['maps.csv']; require(len(maps) == nv, 'complete derivative/moment maps')
    for i,row in enumerate(maps):
        require(integer(row['active']) == i, 'map active identity')
        for d,key in enumerate(('bulk_xx','bulk_xy','bulk_yy')):
            gates.diff('exact-gradient-map',fraction(row[key]),op['gradient'][i][d],h)
        for d,key in enumerate(('trace_xx','trace_xy','trace_yy')):
            gates.diff('exact-physical-gradient-map',fraction(row[key]),op['trace_gradient'][i][d],h)
        for d,key in enumerate(('moment_px','moment_py','moment_lz')):
            gates.diff('exact-moment-map',fraction(row[key]),op['moment'][i][d],h*h)


def quadrature_and_traction(data, op, gates, budget):
    nv = len(op['active']); amap = {g:i for i,g in enumerate(op['active'])}; h = op['h']
    qr = data['rows']['quadrature.csv']; require(qr, 'nonempty native positive quadrature')
    qmap = {}
    for expected,row in enumerate(qr):
        require(integer(row['q']) == expected, 'complete quadrature indices')
        patch = integer(row['patch']); require(patch < len(op['patches']), 'quadrature patch range')
        p = op['patches'][patch]; require(p['area'] > 0 and integer(row['cell']) == p['cell'], 'quadrature positive patch association')
        point = fraction(row['x']),fraction(row['y']); weight = fraction(row['weight'])
        require(weight > 0 and all(p['lo'][d] < point[d] < p['hi'][d] for d in range(2)), 'positive interior native quadrature')
        qmap[expected] = point,weight,p
    bulk = defaultdict(dict)
    for row in data['rows']['bulk-basis.csv']:
        q,g = integer(row['q']),integer(row['global']); require(q in qmap and g not in bulk[q], 'bulk point/basis association duplicate')
        require(g in amap and integer(row['active']) == amap[g] and integer(row['axis']) == op['nodes'][g]['axis'], 'bulk basis native association')
        bulk[q][g] = row
    moments = defaultdict(lambda:defaultdict(F))
    for q,(point,weight,patch) in qmap.items():
        budget.check(); require(set(bulk[q]) == {node['global'] for node,phi,dx,dy in patch['local']}, 'complete eight local bulk bases')
        for node,phi,dx,dy in patch['local']:
            r = bulk[q][node['global']]
            for key,p,scale in (('phi',phi,F(1)),('dx',dx,1/h),('dy',dy,1/h)):
                gates.diff('native-bulk-reconstruction',fraction(r[key]),ep.evaluate(p,point),scale)
        for power in patch['moments']:
            moments[patch['i'],patch['j']][power] += weight*point[0]**power[0]*point[1]**power[1]
    for patch in op['patches']:
        if patch['area']:
            for power,expected in patch['moments'].items():
                gates.diff('native-degree4-volume-moments',moments[patch['i'],patch['j']][power],expected,h*h)
    segments = data['rows']['segments.csv']; require(len(segments) == len(op['segments']), 'complete split trace records')
    for i,(row,s) in enumerate(zip(segments,op['segments'])):
        require(integer(row['segment']) == i and row['kind'] == s['kind'] and integer(row['owner']) == s['owner']
                and integer(row['axis']) == s['axis'] and row['edge'] == (str(s['edge']) if s['edge'] is not None else ''), 'physical/cartesian trace association')
        for key,v in zip(('ax','ay','bx','by'),s['a']+s['b']):
            gates.diff('native-trace-endpoints',fraction(row[key]),v,h)
    # New three-point traction rule, independently check moments through degree5.
    tr = data['top']['traction-points.csv']; grouped = defaultdict(list); points = {}
    f = fields(op['mu']); native_boundary = [F(0)]*nv
    for i,row in enumerate(tr):
        require(integer(row['q']) == i, 'complete new traction point indices')
        record = integer(row['record']); require(record < len(segments) and segments[record]['kind'] == 'global', 'physical traction source record')
        s = segments[record]; a = fraction(s['ax']),fraction(s['ay']); b = fraction(s['bx']),fraction(s['by'])
        point = fraction(row['x']),fraction(row['y']); normal = b[1]-a[1],a[0]-b[0]
        axis = max(range(2),key=lambda d:abs(normal[d])); require(normal[axis] != 0, 'nonzero source segment')
        normal_weight = fraction(row['nx_weight']),fraction(row['ny_weight'])
        w = normal_weight[axis]/normal[axis]; require(w > 0, 'positive traction weight')
        tangent_axis = 1-axis; t = (point[tangent_axis]-a[tangent_axis])/(b[tangent_axis]-a[tangent_axis])
        require(0 < t < 1, 'interior traction parameter')
        for d in range(2):
            gates.diff('physical-traction-source-line',point[d],a[d]+t*(b[d]-a[d]),h)
            gates.diff('physical-traction-outward-normal',normal_weight[d],normal[d]*w,max(map(abs,normal)))
        grouped[record].append((t,w)); points[i] = point,normal_weight,row
        sx,sy,sz = (ep.evaluate(p,point) for p in f['sigma'])
        expected = (sx*normal_weight[0]+sy*normal_weight[1],sy*normal_weight[0]+sz*normal_weight[1])
        for key,value in zip(('gx_weight','gy_weight'),expected):
            gates.diff('exact-pressure-traction',fraction(row[key]),value,max(map(abs,normal_weight)))
    expected_records = {i for i,s in enumerate(segments) if s['kind'] == 'global'}
    require(set(grouped) == expected_records and all(len(v) == 3 for v in grouped.values()), 'complete three-point split traction rule')
    for group in grouped.values():
        for k in range(6):
            gates.diff('traction-degree4-moments',sum((w*t**k for t,w in group),F(0)),F(1,k+1),1)
    basis = defaultdict(dict)
    for row in data['top']['traction-basis.csv']:
        q,i = integer(row['q']),integer(row['index']); require(q in points and i < nv and i not in basis[q], 'traction basis association duplicate')
        basis[q][i] = fraction(row['phi'])
    for q,(point,nw,row) in points.items():
        expected = {i:ep.evaluate(eo.hat(op['nodes'][g],point,h),point) for i,g in enumerate(op['active'])}
        expected = {i:v for i,v in expected.items() if v}
        require(set(basis[q]) == set(expected), 'complete positive traction basis support')
        for i,value in expected.items():
            gates.diff('native-traction-basis',basis[q][i],value,1)
            axis = op['nodes'][op['active'][i]]['axis']
            native_boundary[i] += basis[q][i]*fraction(row['gx_weight' if axis == 0 else 'gy_weight'])
    for i,v in enumerate(native_boundary):
        gates.diff('physical-point-to-nodal-traction',v,data['loads']['boundary'][i],h)


def factors(data, op, gates, budget):
    f = data['factors']; nv,nc = data['nv'],data['nc']
    d = [r[0] for r in f['D']]; e = [r[0] for r in f['E']]
    require(all(v > 0 for v in d+e) and all(f['L'][i][i] > 0 for i in range(nv)), 'positive selected factor scales/pivots')
    require(all(f['R'][i][i] != 0 for i in range(nc)), 'nonzero QR pivots without floor')
    require(all(f['L'][i][j] == 0 for i in range(nv) for j in range(i+1,nv))
            and all(f['R'][i][j] == 0 for i in range(nc) for j in range(i)), 'actual triangular factors')
    expected_a = [[F.from_float(float(m)+float(k)) for m,k in zip(mr,kr)] for mr,kr in zip(data['native']['M'],data['native']['K'])]
    require(f['A'] == expected_a, 'original full rounded target A binding')
    scaled = [[d[i]*f['A'][i][j]*d[j] for j in range(nv)] for i in range(nv)]
    gates.matrix('factor-diagonal-congruence',f['scaled_A'],scaled)
    lower_symmetric = [[f['scaled_A'][max(i,j)][min(i,j)] for j in range(nv)] for i in range(nv)]
    gates.matrix('factor-Cholesky-lower-symmetric-target',multiply(f['L'],transpose(f['L']),budget),lower_symmetric)
    dbt = [[d[i]*data['native']['B'][c][i] for c in range(nc)] for i in range(nv)]
    gates.matrix('factor-Y',multiply(f['L'],f['Y'],budget),dbt)
    z = [[f['Y'][i][c]*e[c] for c in range(nc)] for i in range(nv)]
    gates.matrix('factor-pressure-congruence',f['Z'],z)
    gates.matrix('factor-QR',multiply(f['Q'],f['R'],budget),f['Z'])
    identity = [[F(i == j) for j in range(nc)] for i in range(nc)]
    gates.matrix('factor-orthogonality',multiply(transpose(f['Q']),f['Q'],budget),identity)


def physical(data, op, loads, gates):
    a,b = data['factors']['A'],data['native']['B']; nv,nc = data['nv'],data['nc']; volume = sum(op['volumes'],F(0))
    body,boundary,rhs = (data['loads'][k] for k in ('body','boundary','rhs'))
    for i in range(nv):
        require(rhs[i] == F.from_float(float(body[i])+float(boundary[i])), 'original rounded RHS binding')
    for it in range(4):
        u = [data['states'][it,'u'][i][0] for i in range(nv)]; p = [data['states'][it,'p'][i][0] for i in range(nc)]
        au,bu = action(a,u),action(b,u)
        ru = [rhs[i]-au[i]+sum((b[c][i]*p[c] for c in range(nc)),F(0)) for i in range(nv)]
        for i in range(nv):
            scale = sum(map(abs,a[i]),F(0))+sum((abs(b[c][i]) for c in range(nc)),F(0))+abs(rhs[i])
            require(scale > 0, 'positive momentum row scale')
            gates.diff('recorded-original-residual',data['states'][it,'u'][i][1],ru[i],scale)
            if it == 3:
                gates.record('original-momentum-rows',abs(ru[i])/scale)
        for c in range(nc):
            scale = sum(map(abs,b[c]),F(0)); require(scale > 0, 'positive continuity row scale')
            gates.diff('recorded-original-residual',data['states'][it,'p'][c][1],bu[c],scale)
            if it == 3:
                gates.record('original-continuity-rows',abs(bu[c])/scale)
    u = [data['states'][3,'u'][i][0] for i in range(nv)]; p = [data['states'][3,'p'][i][0] for i in range(nc)]
    bu = action(b,u); ru = [x-y for x,y in zip(rhs,action(a,u))]
    ru = [ru[i]+sum((b[c][i]*p[c] for c in range(nc)),F(0)) for i in range(nv)]
    gf,pf = action(data['native']['grid_flux'],u),action(data['native']['physical_flux'],u)
    for c in range(nc):
        gates.diff('physical-closed-cell-flux',gf[c]+pf[c],0,op['h'])
        gates.diff('physical-flux-bridge',gf[c]+pf[c],bu[c],op['h'])
    mu,ku = action(data['native']['M'],u),action(data['native']['K'],u)
    energy = dot(u,mu)/2; viscous = dot(u,ku); pb,pt = dot(u,body),dot(u,boundary)
    raw = 2*energy+viscous-pb-pt; pressure_work = dot(p,bu); residual_work = -dot(u,ru)
    gates.diff('forced-raw-work',raw,0,volume)
    gates.diff('forced-corrected-work',raw-pressure_work-residual_work,0,volume)
    gates.record('absolute-residual-work',(sum((abs(x*y) for x,y in zip(p,bu)),F(0))+sum((abs(x*y) for x,y in zip(u,ru)),F(0)))/volume)
    gates.diff('physical-body-power',pb,dot(u,loads['body']),volume)
    gates.diff('physical-boundary-power',pt,dot(u,loads['boundary']),volume)
    gates.record('negative-viscous-dissipation',max(F(0),-viscous)/(op['mu']*volume))
    observations = {'energy':energy,'dissipation':viscous,'body_power':pb,'boundary_power':pt,
                    'direct_body_power':dot(u,loads['body']),'direct_boundary_power':dot(u,loads['boundary']),
                    'pressure_work':pressure_work,'residual_work':residual_work,'raw_work':raw}
    for d,label in enumerate(('x','y','torque')):
        virtual = []
        for g in op['active']:
            node = op['nodes'][g]; axis = node['axis']; x,y = node['point']
            virtual.append(F(axis == d) if d < 2 else -y if axis == 0 else x)
        inertia = sum((op['moment'][i][d]*u[i] for i in range(nv)),F(0))
        direct_body,direct_boundary = dot(virtual,loads['body']),dot(virtual,loads['boundary'])
        gates.diff('physical-momentum-'+label,inertia,direct_body+direct_boundary,volume)
        for prefix,value,expected in (('body_nodal',dot(virtual,body),direct_body),('boundary_nodal',dot(virtual,boundary),direct_boundary)):
            gates.diff('physical-'+prefix+'-'+label,value,expected,volume)
        observations.update({f'inertia_{label}':inertia,f'body_direct_{label}':direct_body,f'boundary_direct_{label}':direct_boundary,
                             f'body_nodal_{label}':dot(virtual,body),f'boundary_nodal_{label}':dot(virtual,boundary)})
    for name,(kind,value) in data['metrics'].items():
        if kind == 'gate':
            require(value >= 0, 'nonnegative reported native gate')
            gates.record('reported-native:'+name,value)
        else:
            require(name in observations, 'independently reconstructed observation '+name)
            scale = volume*op['mu'] if name == 'dissipation' else volume
            gates.diff('reported-physical-observation:'+name,value,observations[name],scale)
    return observations


def audit(data, budget):
    op = eo.build(data['source'],data['n'],data['origin'],F(1),F(1),F.from_float(.1))
    gates = Gates(); structure(data,op,gates,budget)
    quadrature_and_traction(data,op,gates,budget)
    loads = exact_loads(op)
    for key,guard in (('body','exact-body-load'),('boundary','exact-total-traction-load')):
        scale = max(map(abs,loads[key])); require(scale > 0, 'positive continuum load scale')
        for a,b in zip(data['loads'][key],loads[key]):
            gates.diff(guard,a,b,scale)
    factors(data,op,gates,budget)
    observations = physical(data,op,loads,gates)
    return op,loads,gates,observations
