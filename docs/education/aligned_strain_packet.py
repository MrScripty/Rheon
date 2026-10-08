"""Source-bound export of one real native aligned-box strain specimen.

This is an educational algebra packet: controls change an active face vector,
not a PDE solution or geometry. Every accepted packet rechecks its native TSV
against the independent rational oracle and rebuilds its JSON representation.
"""
from __future__ import annotations
from fractions import Fraction as F
from pathlib import Path
import argparse
import hashlib
import json
import math
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'tools'))
from aligned_strain_oracle import parse_dump, reference, verify, matrix, require

# Requalified native source: fresh full native/rational driver, unchanged strain
# and geometry modules; PR36 adds body/contact exports. Historical PR30 stays pinned.
PRIMARY_SOURCE = 'd31cf735645579357f9f58dcc55958e23f77af59'
PRIMARY_TREE = '68bfbefca7f83ead7468ce870b57353e2753e27d'
SCHEMA = 'rheon-aligned-strain-education-packet-v1'
RECEIPT_SCHEMA = 'rheon-aligned-strain-education-qualification-v1'
FILES = {'native.tsv', 'data.json', 'qualification.json', 'primary-qualification.json'}
PARAMETERS = ['3','3','3','1','1','1','0','0','0','1','1','1','2','2','2','1','1']


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, indent=2, allow_nan=False) + '\n'


def git(repo, *args):
    return subprocess.check_output(['git', *args], cwd=repo, text=True).strip()


def source_core(repo):
    require(git(repo, 'rev-parse', PRIMARY_SOURCE + '^{tree}') == PRIMARY_TREE,
            'immutable primary Git tree')
    require(subprocess.run(['git', 'merge-base', '--is-ancestor', PRIMARY_SOURCE, 'HEAD'],
                           cwd=repo, capture_output=True).returncode == 0,
            'education source must descend from qualified primary')
    paths = git(repo,'ls-tree','-r','--name-only',PRIMARY_SOURCE).splitlines()
    native_paths = [p for p in paths if p.startswith('src/') or p.startswith('rust-toolchain')
                    or p in ('Cargo.toml','Cargo.lock','examples/aligned_strain.rs',
                             'tools/aligned_strain_oracle.py')]
    native_sources = {}
    for name in native_paths:
        blob = subprocess.check_output(['git','show',PRIMARY_SOURCE+':'+name],cwd=repo)
        native_sources[name] = hashlib.sha256(blob).hexdigest()
        require(sha(Path(repo)/name) == native_sources[name], 'changed qualified native source '+name)
    require(len(native_sources)>5, 'primary native source inventory')
    return native_sources


def trusted_primary(repo, path):
    # The native executable is freshly built on the runner. Its hash is bound
    # by that run's qualification, not by a machine-specific historical ELF.
    path = Path(path)
    require(path.is_file() and not path.is_symlink(), 'primary receipt regular file')
    receipt = json.loads(path.read_text())
    require(receipt.get('schema') == 'rheon-reconstructed-aligned-strain-qualification-v1' and
            receipt.get('clean') is True and receipt.get('status') == 'passed',
            'clean passed native qualification')
    head, tree = receipt.get('source_head'), receipt.get('source_tree')
    require(type(head) is str and len(head)==40 and type(tree) is str and len(tree)==40,
            'native qualification source identity shape')
    require(git(repo,'rev-parse',head+'^{tree}') == tree and subprocess.run(
            ['git','merge-base','--is-ancestor',PRIMARY_SOURCE,head],cwd=repo,
            capture_output=True).returncode == 0, 'native qualification primary ancestry')
    sources = source_core(repo)
    require(all(receipt.get('source_sha256',{}).get(p)==digest for p,digest in sources.items()),
            'native qualification core source manifest')
    for name,digest in sources.items():
        blob = subprocess.check_output(['git','show',head+':'+name],cwd=repo)
        require(hashlib.sha256(blob).hexdigest() == digest, 'qualified commit core source '+name)
    digest = receipt.get('executable_sha256')
    require(type(digest) is str and len(digest)==64 and all(c in '0123456789abcdef' for c in digest),
            'native binary digest shape')
    return receipt, sources


def specimen(dump):
    g = dump.geometry
    require((g.counts, g.spacing, g.origin, g.lower, g.upper, g.density, g.viscosity) ==
            ((3,3,3), (F(1),)*3, (F(0),)*3, (1,1,1), (2,2,2), F(1), F(1)),
            'education specimen must be captured unit center box at rho=mu=1')


def basis_and_corners(faces, rows):
    """Control bases derive from the reference geometry, never from UI code."""
    n = len(faces)
    lookup = {(f.axis, f.p): i for i, f in enumerate(faces)}
    corner_a, corner_b = [F(0)] * n, [F(0)] * n
    corner_a[lookup[0, (2,2,1)]] = 1
    corner_b[lookup[1, (2,2,1)]] = 1
    normal_a = [f.position[0]-F(3,2) if f.axis == 0 else F(0) for f in faces]
    normal_b = [f.position[1]-F(3,2) if f.axis == 1 else F(0) for f in faces]
    shear_a = [f.position[1]-F(3,2) if f.axis == 0 else F(0) for f in faces]
    shear_b = [f.position[0]-F(3,2) if f.axis == 1 else F(0) for f in faces]
    rotation_a = [F(0)] * n
    for axis, p in ((0,(1,0,0)), (0,(1,1,0)), (1,(0,1,0)), (1,(1,1,0))):
        f = faces[lookup[axis,p]]
        rotation_a[lookup[axis,p]] = -(f.position[1]-1) if axis == 0 else f.position[0]-1
    def matching(family, p):
        return [i for i, r in enumerate(rows) if r.family == family and r.p == p]
    corner_focus = matching((0,1), (2,2,1))
    interior_focus = matching((0,1), (1,1,0))
    normal_focus = [i for i, r in enumerate(rows) if r.family == (0,0) and len(r.terms) == 2][:1]
    definitions = {
        'corner': ('Corner face samples', 'U · x face', 'V · y face', 'm/s', corner_a, corner_b,
                   'Two actual active samples beside the stationary obstacle corner.', corner_focus),
        'normal': ('Normal sample stretch', 'x stretch rate', 'y stretch rate', 's⁻¹', normal_a, normal_b,
                   'Affine component samples on active faces; stationary eliminated traces remain zero.', normal_focus),
        'shear': ('Cross-component sample shear', '∂y uₓ sample rate', '∂x uᵧ sample rate', 's⁻¹', shear_a, shear_b,
                  'Both velocity components contribute to engineering shear.', interior_focus),
        'rotation': ('Local rotation patch', 'ω · rotation rate', 'unused', 's⁻¹', rotation_a, [F(0)]*n,
                     'The selected full-fluid shear rows have zero engineering shear. Surrounding normal rows and stationary traces can produce positive global loss.', interior_focus),
    }
    presets = {key: {'label': label, 'primaryLabel': primary, 'secondaryLabel': secondary,
                    'units': units, 'basis': [[float(x) for x in a], [float(x) for x in b]],
                    'description': description, 'focusRowIds': focus,
                    'secondaryDisabled': key == 'rotation'}
               for key, (label, primary, secondary, units, a, b, description, focus) in definitions.items()}
    groups = {}
    for i, row in enumerate(rows):
        if row.closure == 2:
            groups.setdefault((*row.family, *row.p), []).append(i)
    corners = []
    for key, ids in sorted(groups.items()):
        selected = [rows[i] for i in ids]
        active_ids = sorted({i for row in selected for i, _ in row.terms})
        a, b, *p = key
        solid = next((sa,sb) for sa in (-1,1) for sb in (-1,1)
                     if (sa+1)//2+2*((sb+1)//2) not in {row.quadrant for row in selected})
        stiffness = matrix(selected, n)
        corners.append({'id': '-'.join(map(str,key)), 'axes': [a,b], 'coordinates': p,
                        'rowIds': ids, 'activeIds': active_ids, 'solidSigns': list(solid),
                        'unitPatchMatrix': [[float(stiffness.get((i,j),0)) for j in active_ids] for i in active_ids],
                        'basis': [[1.0 if i==active else 0.0 for i in range(n)] for active in active_ids]})
    return presets, corners


def packet_data(dump, source):
    specimen(dump)
    g = dump.geometry
    faces, reference_rows, _, _ = reference(g)
    # Preserve native order while matching each row to its independent key.
    canonical = {r.key: r for r in reference_rows}
    independent_rows = [canonical[r.key] for r in dump.rows]
    presets, corners = basis_and_corners(faces, independent_rows)
    return {'schema': SCHEMA,
            'scope': 'Finite aligned-box strain algebra; no time stepping or fluid solve.',
            'meta': {'counts': list(g.counts), 'spacing': list(map(float,g.spacing)),
                     'origin': list(map(float,g.origin)), 'lower': list(g.lower), 'upper': list(g.upper),
                     'density': float(g.density), 'captureViscosity': float(g.viscosity),
                     'outerBoundary': 'sealed free-slip', 'obstacleBoundary': 'stationary no-slip'},
            'active': [{'id': i, 'axis': f.axis, 'face': f.flat, 'coordinates': list(f.p),
                        'position': list(map(float,f.position)), 'area': float(f.area),
                        'distance': float(f.distance), 'mass': float(f.mass)} for i,f in enumerate(dump.faces)],
            'rows': [{'id': i, 'axes': list(r.family), 'coordinates': list(r.p),
                      'quadrant': r.quadrant, 'boundary': r.closure, 'weight': float(r.weight),
                      'terms': [{'active': face, 'coefficient': float(a)} for face,a in r.terms]}
                     for i,r in enumerate(dump.rows)],
            'presets': presets, 'cornerEdges': corners,
            'controlLimits': {'amplitude': [-2,2], 'secondary': [-2,2], 'mu': [0,2]},
            'defaultCornerSelection': '0-1-2-2-1',
            'source': source}


def external_new_directory(repo, output):
    output, repo = Path(output).resolve(), Path(repo).resolve()
    require(not output.is_relative_to(repo) and not repo.is_relative_to(output),
            'packet output must be outside the checkout and ancestors')
    existing = output
    while not existing.exists():
        existing = existing.parent
    require(subprocess.run(['git','-C',str(existing),'rev-parse','--is-inside-work-tree'],
                           capture_output=True).returncode != 0, 'packet output inside Git')
    require(not output.exists(), 'packet output must be fresh')
    return output


def build_packet(repo, output, executable, primary_qualification, records=None):
    repo, executable = Path(repo).resolve(), Path(executable).resolve()
    primary, sources = trusted_primary(repo, primary_qualification)
    require(executable.is_file() and sha(executable) == primary['executable_sha256'],
            'executable must match qualified immutable native binary')
    head, tree, status = git(repo,'rev-parse','HEAD'), git(repo,'rev-parse','HEAD^{tree}'), git(repo,'status','--porcelain')
    require(not status, 'packet capture requires a clean committed education source')
    output = external_new_directory(repo, output)
    output.mkdir(parents=True)
    native = output/'native.tsv'
    command = [str(executable), *PARAMETERS, str(native)]
    result = subprocess.run(command, cwd=repo, capture_output=True, text=True, timeout=60)
    require(result.returncode == 0, 'native specimen execution failed: ' + result.stderr)
    if records is not None:
        require(Path(records).read_bytes() == native.read_bytes(), 'provided records differ from fresh native execution')
    summary = verify(native)
    dump = parse_dump(native)
    specimen(dump)
    source = {'commit': primary['source_head'], 'tree': primary['source_tree'], 'clean': True,
              'baseCommit': PRIMARY_SOURCE, 'baseTree': PRIMARY_TREE,
              'binarySha256': primary['executable_sha256'], 'recordsSha256': sha(native),
              'oracleSummarySha256': hashlib.sha256(encoded(summary).encode()).hexdigest(),
              'primaryQualificationSha256': sha(primary_qualification), 'packetSchema': SCHEMA}
    data = packet_data(dump, source)
    (output/'data.json').write_text(encoded(data))
    shutil.copyfile(primary_qualification, output/'primary-qualification.json')
    require((git(repo,'rev-parse','HEAD'),git(repo,'rev-parse','HEAD^{tree}'),git(repo,'status','--porcelain')) ==
            (head,tree,status), 'education source changed during capture')
    latest_primary, _ = trusted_primary(repo, primary_qualification)
    require(latest_primary == primary, 'native qualification changed during capture')
    require(sha(executable) == primary['executable_sha256'], 'executable changed during capture')
    receipt = {'schema': RECEIPT_SCHEMA, 'status': 'passed', 'clean': True,
               'source_head': head, 'source_tree': tree, 'native_source_head': primary['source_head'],
               'native_source_tree': primary['source_tree'], 'native_source_sha256': sources,
               'executable_sha256': primary['executable_sha256'], 'records_sha256': sha(native),
               'data_sha256': sha(output/'data.json'), 'primary_qualification_sha256': sha(primary_qualification),
               'oracle_summary': summary, 'native_argv': command, 'native_exit': 0,
               'stepping_authorized': False, 'fluid_solves': 0}
    (output/'qualification.json').write_text(encoded(receipt))
    validate_packet(repo, output)
    return receipt


def validate_packet(repo, directory):
    if directory is None:
        return None
    repo, directory = Path(repo), Path(directory)
    require(directory.is_dir() and not directory.is_symlink(), 'packet regular directory')
    require({p.name for p in directory.iterdir()} == FILES and
            all(p.is_file() and not p.is_symlink() for p in directory.iterdir()), 'packet exact regular file roster')
    primary, sources = trusted_primary(repo, directory/'primary-qualification.json')
    receipt = json.loads((directory/'qualification.json').read_text())
    require(set(receipt) == {'schema','status','clean','source_head','source_tree','native_source_head',
            'native_source_tree','native_source_sha256','executable_sha256','records_sha256','data_sha256',
            'primary_qualification_sha256','oracle_summary','native_argv','native_exit','stepping_authorized',
            'fluid_solves'}, 'packet qualification exact field roster')
    require(receipt.get('schema') == RECEIPT_SCHEMA and receipt.get('status') == 'passed' and
            receipt.get('clean') is True, 'clean passed packet qualification')
    require(receipt.get('native_source_head') == primary['source_head'] and receipt.get('native_source_tree') == primary['source_tree'] and
            receipt.get('native_source_sha256') == sources and receipt.get('executable_sha256') == primary['executable_sha256'] and
            receipt.get('primary_qualification_sha256') == sha(directory/'primary-qualification.json'), 'packet native source binding')
    require(receipt.get('records_sha256') == sha(directory/'native.tsv') and
            receipt.get('data_sha256') == sha(directory/'data.json'), 'packet payload digest binding')
    argv = receipt.get('native_argv')
    require(type(argv) is list and len(argv)==19 and argv[1:-1] == PARAMETERS and
            all(type(x) is str for x in argv), 'packet native invocation specimen')
    require(receipt.get('native_exit') == 0 and receipt.get('stepping_authorized') is False and
            receipt.get('fluid_solves') == 0, 'packet scope or runtime qualification')
    require(receipt.get('source_tree') == git(repo,'rev-parse',receipt.get('source_head','')+'^{tree}'),
            'packet education source tree identity')
    require(subprocess.run(['git','merge-base','--is-ancestor',receipt['source_head'],'HEAD'],
                           cwd=repo,capture_output=True).returncode == 0, 'packet education source ancestry')
    summary = verify(directory/'native.tsv')
    # The summary includes a path; path relocation changes only this field.
    previous = dict(receipt.get('oracle_summary', {}))
    previous['dump'] = summary['dump']
    require(previous == summary, 'packet independent oracle summary')
    data = json.loads((directory/'data.json').read_text())
    source = {'commit': primary['source_head'], 'tree': primary['source_tree'], 'clean': True,
              'baseCommit': PRIMARY_SOURCE, 'baseTree': PRIMARY_TREE,
              'binarySha256': primary['executable_sha256'], 'recordsSha256': sha(directory/'native.tsv'),
              'oracleSummarySha256': hashlib.sha256(encoded(receipt['oracle_summary']).encode()).hexdigest(),
              'primaryQualificationSha256': sha(directory/'primary-qualification.json'), 'packetSchema': SCHEMA}
    require(data == packet_data(parse_dump(directory/'native.tsv'), source),
            'packet data differs from actual native rows and independent controls')
    return receipt


def publish(repo, output, directory=None):
    """Copy only validated packet inputs; the caller renders the returned data."""
    output, repo = Path(output), Path(repo)
    receipt = validate_packet(repo,directory)
    presentation = {'schema':'rheon-aligned-strain-presentation-v1','included':receipt is not None,
                    'qualification':receipt,'renderer_sha256':None}
    if receipt is not None:
        destination = output/'aligned-strain-packet'
        require(not destination.exists(),'published packet destination must be fresh')
        shutil.copytree(directory,destination)
        shutil.copyfile(Path(directory)/'data.json',output/'aligned-strain-packet.json')
        presentation['renderer_sha256'] = {name:sha(repo/'docs/education'/name) for name in
                                          ('aligned_strain.js','aligned_strain.css','aligned_strain_html.py')}
    with (output/'aligned-strain-presentation.json').open('x') as out:
        out.write(encoded(presentation))
    return json.loads((Path(directory)/'data.json').read_text()) if receipt is not None else None


def verify_published(repo,output):
    repo,output = Path(repo),Path(output)
    presentation = json.loads((output/'aligned-strain-presentation.json').read_text())
    require(set(presentation)=={'schema','included','qualification','renderer_sha256'} and
            presentation.get('schema')=='rheon-aligned-strain-presentation-v1' and
            type(presentation.get('included')) is bool,'aligned strain presentation schema')
    if not presentation['included']:
        require(presentation['qualification'] is None and presentation['renderer_sha256'] is None and
                not (output/'aligned-strain-packet').exists() and not (output/'aligned-strain-packet.json').exists(),
                'absent aligned strain packet')
        return None
    receipt = validate_packet(repo,output/'aligned-strain-packet')
    require(presentation['qualification']==receipt,'published aligned strain receipt binding')
    require((output/'aligned-strain-packet.json').read_bytes()==(output/'aligned-strain-packet/data.json').read_bytes(),
            'published aligned strain linked packet binding')
    expected = {name:sha(repo/'docs/education'/name) for name in
                ('aligned_strain.js','aligned_strain.css','aligned_strain_html.py')}
    require(presentation['renderer_sha256']==expected and all(sha(output/name)==expected[name] for name in
            ('aligned_strain.js','aligned_strain.css')),'published aligned strain renderer binding')
    return json.loads((output/'aligned-strain-packet/data.json').read_text())


def browser_expectation(packet, preset='corner', amplitude=1, secondary=1, mu=1, corner_selection=None):
    """Independent Fraction result from rederived geometry and browser inputs."""
    require(packet.get('schema') == SCHEMA and preset in packet['presets'], 'browser preset schema')
    for name, value in (('amplitude',amplitude),('secondary',secondary),('mu',mu)):
        require(type(value) in (int,float,F) and math.isfinite(float(value)) and
                packet['controlLimits'][name][0] <= value <= packet['controlLimits'][name][1],
                'browser control bounds ' + name)
    from aligned_strain_oracle import Geometry
    g = Geometry((3,3,3),(F(1),)*3,(F(0),)*3,(1,1,1),(2,2,2))
    faces, rows, _, coefficient_bound = reference(g)
    canonical = {r.key: r for r in rows}
    ordered_rows = [canonical[(*r['axes'],*r['coordinates'],r['quadrant'])] for r in packet['rows']]
    presets, corners = basis_and_corners(faces, ordered_rows)
    require(packet['presets'] == presets, 'browser control bases differ from independent reference')
    a, b, viscosity = F(amplitude), F(secondary), F(mu)
    require(packet['cornerEdges'] == corners, 'browser reflected corner bases differ from independent reference')
    corner_selection = corner_selection or packet['defaultCornerSelection']
    selected = next((edge for edge in corners if edge['id']==corner_selection),None)
    require(selected is not None, 'browser corner selection')
    first, second = selected['basis'] if preset=='corner' else presets[preset]['basis']
    field = [a*F(x)+b*F(y) for x,y in zip(first,second)]
    strains, action = [], [F(0)]*len(faces)
    normal, shear = F(0), F(0)
    for row in ordered_rows:
        strain = sum(coefficient*field[i] for i,coefficient in row.terms)
        strains.append(strain)
        loss = viscosity*row.weight*strain*strain
        if row.family[0] == row.family[1]: normal += loss
        else: shear += loss
        for i, coefficient in row.terms:
            action[i] += row.weight*coefficient*strain
    forces = [-viscosity*x for x in action]
    work = sum(v*f for v,f in zip(field,forces))
    return {'field': field, 'strains': strains, 'action': action, 'forces': forces,
            'normalD': normal, 'shearD': shear, 'D': normal+shear,
            'work': work, 'identity': normal+shear+work, 'B': coefficient_bound,
            'energy': sum(f.mass*v*v/2 for f,v in zip(faces,field))}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--executable',type=Path,required=True)
    parser.add_argument('--primary-qualification',type=Path,required=True)
    parser.add_argument('--records',type=Path)
    args = parser.parse_args()
    result = build_packet(ROOT,args.output,args.executable,args.primary_qualification,args.records)
    print(encoded({'status': result['status'], 'records_sha256':result['records_sha256'],
                   'data_sha256':result['data_sha256'], 'output':str(args.output.resolve())}),end='')
