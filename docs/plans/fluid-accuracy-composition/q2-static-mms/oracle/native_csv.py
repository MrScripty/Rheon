"""Prospective always-on CV3 schema/bit/corpus parser. No work on import."""
import csv
from fractions import Fraction as F
import hashlib
import math
from pathlib import Path
import struct

GATE = F(1, 10**10)
SOURCE_BITS = {
    'rectangle': ((0x3fc5c28f5c28f5c3,0x3fcd70a3d70a3d71),(0x3fea8f5c28f5c28f,0x3fcd70a3d70a3d71),
                  (0x3fea8f5c28f5c28f,0x3fe947ae147ae148),(0x3fc5c28f5c28f5c3,0x3fe947ae147ae148)),
    'square-17': ((0x3fdbfe86b0ab8be6,0x3fd2ab30509fa4da),(0x3fe72d7a4647bae3,0x3fd848410ee0cb62),
                  (0x3fe45ef1e727279f,0x3fe5525775625aa1),(0x3fd66175f26a655e,0x3fe283cf1641c75d)),
}
SOURCE_SHA = {'rectangle':'860b4140b7483797842e6c6bdeb1e661d1e8510fb0855cf9a69b772914208666',
              'square-17':'2a2938a1ae4b12bfcc6c33ff3cb8fc332fc55436e1247c32638e8b83fde1b6cc'}
SCHEMAS = {
 'source.csv':'vertex,x,y', 'input-meta.csv':'key,value', 'meta.csv':'key,value',
 'nodes.csv':'global,axis,face,i,j,x,y,active,old_dual,mass_diagonal,a,b,affine',
 'patches.csv':'patch,i,j,cell,lo_x_bits,lo_y_bits,hi_x_bits,hi_y_bits,area_num,area_den,vertices,triangles',
 'vertices.csv':'patch,index,x_num,x_den,y_num,y_den',
 'triangles.csv':'patch,index,ax,ay,bx,by,cx,cy,cross,cross_num,cross_den',
 'quadrature.csv':'q,patch,cell,x,y,weight', 'bulk-basis.csv':'q,active,global,axis,phi,dx,dy',
 'cells.csv':'cell,pressure,volume,x_center,y_center,q_pressure',
 'matrices.csv':'block,row,column,value',
 'maps.csv':'active,bulk_xx,bulk_xy,bulk_yy,trace_xx,trace_xy,trace_yy,moment_px,moment_py,moment_lz,full_load,omit_load',
 'segments.csv':'segment,kind,owner,axis,edge,ax,ay,bx,by',
 'trace-points.csv':'q,segment,x,y,nx_weight,ny_weight', 'trace-basis.csv':'q,active,phi',
 'metrics.csv':'name,value,gate',
}
TOP_SCHEMAS = {'input.csv':'key,value,bits','loads.csv':'index,body,boundary,rhs',
 'h1.csv':'row,column,value','traction-points.csv':'q,record,x,y,nx_weight,ny_weight,gx_weight,gy_weight',
 'traction-basis.csv':'q,index,phi','factors.csv':'block,row,column,value',
 'states.csv':'method,iteration,kind,index,value,original_residual', 'metrics.csv':'kind,name,value'}
NATIVE_GATES = {'original_momentum_rows','original_continuity_rows','physical_closed_cell_flux',
 'forced_raw_work','forced_residual_corrected_work','absolute_residual_work','physical_body_power',
 'physical_boundary_power','negative_viscous_dissipation','momentum_x','momentum_y','angular_momentum',
 'body_resultant_x','body_resultant_y','body_torque','boundary_resultant_x','boundary_resultant_y','boundary_torque'}
NATIVE_OBSERVATIONS = {'energy','dissipation','body_power','boundary_power','direct_body_power',
 'direct_boundary_power','pressure_work','residual_work','raw_work'} | {
 prefix+'_'+label for prefix in ('inertia','body_nodal','boundary_nodal','body_direct','boundary_direct')
 for label in ('x','y','torque')}


def require(condition, reason):
    if not condition:
        raise RuntimeError('CV3 input guard: '+reason)


def cases():
    return {f'{geometry}-{phase}-n{n}': (geometry,phase,n)
            for geometry,phases in (('rectangle',('origin',)),('square-17',('origin','quarter')))
            for phase in phases for n in (4,8,16)}


def real(value):
    try:
        out = float(value)
    except (ValueError, TypeError) as exc:
        raise RuntimeError('CV3 input guard: invalid float') from exc
    require(math.isfinite(out), 'nonfinite field')
    return out


def fraction(value):
    return F.from_float(real(value))


def integer(value):
    try:
        out = int(value)
    except (ValueError, TypeError) as exc:
        raise RuntimeError('CV3 input guard: invalid integer') from exc
    require(str(out) == value and out >= 0, 'canonical nonnegative integer')
    return out


def bits(value):
    return struct.unpack('>Q',struct.pack('>d',real(value)))[0]


def from_bits(value):
    return F.from_float(struct.unpack('>d',struct.pack('>Q',value))[0])


def read(path, schema):
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), 'missing/linked file '+str(path))
    with path.open(newline='') as handle:
        reader = csv.DictReader(handle)
        require(reader.fieldnames == schema.split(','), 'schema '+str(path))
        rows = list(reader)
    require(all(None not in row and all(v is not None for v in row.values()) for row in rows), 'ragged CSV '+str(path))
    return rows


def indexed(rows, key, count):
    out = {}
    for row in rows:
        i = integer(row[key]); require(i not in out, 'duplicate '+key)
        out[i] = row
    require(set(out) == set(range(count)), 'incomplete/extra '+key)
    return out


def keyed(rows, key):
    out = {}
    for row in rows:
        require(row[key] not in out, 'duplicate '+key); out[row[key]] = row
    return out


def dense(rows, shapes):
    out = {k:[[F(0)]*cols for _ in range(nr)] for k,(nr,cols) in shapes.items()}; seen = set()
    for row in rows:
        k = row['block']; require(k in shapes, 'known matrix block')
        i,j = integer(row['row']),integer(row['column'])
        require(i < shapes[k][0] and j < shapes[k][1] and (k,i,j) not in seen, 'block index/duplicate')
        seen.add((k,i,j)); out[k][i][j] = fraction(row['value'])
    require(len(seen) == sum(a*b for a,b in shapes.values()), 'complete dense blocks including zeros')
    return out


def hashes(root):
    return {str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(Path(root).rglob('*.csv'))}


def read_case(root):
    root = Path(root); name = root.name
    require(name in cases(), 'declared nine-case identity')
    geometry,phase,n = cases()[name]; origin = F(0) if phase == 'origin' else F(1,4*n)
    identity = keyed(read(root/'input.csv',TOP_SCHEMAS['input.csv']),'key')
    numeric = {'rho':1.,'mu':.1,'dt':1.,'Lref':1.,'Uref':1.,'Href':1.,'tau_ref':1.,'extent':1.,
               'origin_x':float(origin),'origin_y':float(origin)}
    text = {'case':name,'geometry':geometry,'phase':phase,'source_csv_sha256':SOURCE_SHA[geometry],
            'model':'CV3-A-static-polygon-MMS-v1','field_load':'global-polynomial-continuum-v1',
            'basis':'CV1-full-consistent-Q1-P0','solver':'CV2-scaled-Cholesky-QR-refinement3-cap544',
            'traction_rule':'split-three-point-Gauss-v1','pressure_gauge':'none','u_star':'zero','n':str(n)}
    vertices = {f'vertex_{i}_{d}':value for i,point in enumerate(SOURCE_BITS[geometry]) for d,value in enumerate(point)}
    require(set(identity) == set(numeric)|set(text)|set(vertices), 'complete primitive metadata')
    for key,value in numeric.items():
        reason = 'phase-identity' if key.startswith('origin_') else 'fixed primitive '+key
        require(real(identity[key]['value']) == value and integer(identity[key]['bits']) == bits(value), reason)
    for key,value in text.items():
        reason = 'phase-identity' if key in ('case','phase') else 'source-identity' if key in ('geometry','source_csv_sha256') else 'fixed method '+key
        require(identity[key]['value'] == value and identity[key]['bits'] == '', reason)
    for key,value in vertices.items():
        require(integer(identity[key]['bits']) == value and bits(identity[key]['value']) == value, 'source-identity')
    require(hashlib.sha256((root/'operators/source.csv').read_bytes()).hexdigest() == SOURCE_SHA[geometry], 'source-identity')
    source = [tuple(from_bits(v) for v in point) for point in SOURCE_BITS[geometry]]
    if (root/'refusal.txt').exists():
        return {'name':name,'root':root,'native_refused':(root/'refusal.txt').read_text(),'input_sha256':hashes(root)}
    expected = {f'operators/{k}' for k in SCHEMAS}|set(TOP_SCHEMAS)
    require(set(hashes(root)) == expected, 'complete24-file native case corpus')
    rows = {key:read(root/'operators'/key,schema) for key,schema in SCHEMAS.items()}
    top = {key:read(root/key,schema) for key,schema in TOP_SCHEMAS.items()}
    # Parse every represented numeric field, including diagnostic-only custody.
    categorical = {'block','kind','method','name','gate'}
    for key,rr in list(rows.items())+[(f'top/{k}',v) for k,v in top.items()]:
        if key in ('meta.csv','input-meta.csv','top/input.csv'):
            continue
        for row in rr:
            for field,value in row.items():
                if field not in categorical and value != '':
                    real(value)
    metadata = {k:r['value'] for k,r in keyed(rows['meta.csv'],'key').items()}
    require(set(metadata) == {'n','h','origin_x','origin_y','active','pressure_cells','patches','quadrature','segments','trace_points',
                              'rho','mu','depth','velocity_basis','pressure_basis','operators_only','pressure_gauge'}, 'complete operator metadata')
    for key in ('n','active','pressure_cells','patches','quadrature','segments','trace_points'):
        integer(metadata[key])
    for key in ('h','origin_x','origin_y','rho','mu','depth'):
        real(metadata[key])
    require(integer(metadata['patches']) == len(rows['patches.csv']) == 4*n*n
            and integer(metadata['quadrature']) == len(rows['quadrature.csv'])
            and integer(metadata['segments']) == len(rows['segments.csv'])
            and integer(metadata['trace_points']) == len(rows['trace-points.csv']), 'complete native geometry record counts')
    require((metadata['velocity_basis'],metadata['pressure_basis'],metadata['operators_only'],metadata['pressure_gauge']) ==
            ('staggered-continuous-Q1','primal-cut-P0','true','none'), 'fixed operator method metadata')
    require(fraction(metadata['rho']) == 1 and fraction(metadata['mu']) == F.from_float(.1) and fraction(metadata['depth']) == 1, 'fixed operator material primitives')
    imeta = {k:r['value'] for k,r in keyed(rows['input-meta.csv'],'key').items()}
    require(set(imeta) == {'n','origin_x','origin_y','extent','rho','mu','depth','model'}, 'complete operator input metadata')
    require(integer(imeta['n']) == n and fraction(imeta['origin_x']) == origin and fraction(imeta['origin_y']) == origin, 'phase-identity')
    require(fraction(imeta['extent']) == 1 and fraction(imeta['rho']) == 1 and fraction(imeta['mu']) == F.from_float(.1)
            and fraction(imeta['depth']) == 1 and imeta['model'] == 'CV1-operator-only', 'fixed operator input primitives')
    require(integer(metadata['n']) == n and fraction(metadata['h']) == F(1,n), 'phase-identity')
    require(fraction(metadata['origin_x']) == origin and fraction(metadata['origin_y']) == origin, 'phase-identity')
    nv,nc = integer(metadata['active']),integer(metadata['pressure_cells'])
    require(0 < nv <= 544 and 0 < nc <= 256 and nv+nc <= 800, 'bounded dimensions')
    nodes = indexed(rows['nodes.csv'],'global',2*n*(n+1))
    cells = indexed(rows['cells.csv'],'cell',n*n)
    mats = dense(rows['matrices.csv'],{'M':(nv,nv),'K':(nv,nv),'B':(nc,nv),'grid_flux':(nc,nv),'physical_flux':(nc,nv)})
    h1 = dense([dict(row,block='H1') for row in top['h1.csv']],{'H1':(nv,nv)})['H1']
    factor_shapes = {'A':(nv,nv),'scaled_A':(nv,nv),'L':(nv,nv),'Y':(nv,nc),'Z':(nv,nc),'Q':(nv,nc),
                     'R':(nc,nc),'D':(nv,1),'E':(nc,1)}
    factors = dense(top['factors.csv'],factor_shapes)
    loads = indexed(top['loads.csv'],'index',nv)
    loads = {k:[fraction(loads[i][k]) for i in range(nv)] for k in ('body','boundary','rhs')}
    states = {}
    for row in top['states.csv']:
        require(row['method'] == 'selected' and row['kind'] in ('u','p'), 'fixed state method/kind')
        it = integer(row['iteration']); i = integer(row['index']); kind = row['kind']
        require(it in range(4) and i < (nv if kind == 'u' else nc), 'fixed state iteration/index')
        target = states.setdefault((it,kind),{}); require(i not in target, 'duplicate state coefficient')
        target[i] = fraction(row['value']),fraction(row['original_residual'])
    require(set(states) == {(it,k) for it in range(4) for k in ('u','p')}, 'all four selected states')
    for (it,kind),vector in states.items():
        require(set(vector) == set(range(nv if kind == 'u' else nc)), 'complete state vector')
    metrics = keyed(top['metrics.csv'],'name')
    require(set(metrics) == NATIVE_GATES|NATIVE_OBSERVATIONS, 'complete native physical metrics')
    require(all(r['kind'] == ('gate' if k in NATIVE_GATES else 'observation') for k,r in metrics.items()), 'metric classification')
    metrics = {k:(r['kind'],fraction(r['value'])) for k,r in metrics.items()}
    return {'name':name,'root':root,'n':n,'origin':(origin,origin),'source':source,'nv':nv,'nc':nc,
            'rows':rows,'top':top,'nodes':nodes,'cells':cells,'native':mats,'h1':h1,'factors':factors,
            'loads':loads,'states':states,'metrics':metrics,'input_sha256':hashes(root)}
