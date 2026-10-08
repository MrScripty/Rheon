"""Present checked native snapshots; never advance or interpolate a contact state."""
from pathlib import Path
import hashlib
import html
import json
import math
import shutil


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_packet(repo, packet):
    if packet is None:
        return None
    repo, packet = Path(repo), Path(packet)
    provenance = json.loads((packet/'packet-provenance.json').read_text())
    if provenance['source_head'] != 'd31cf735645579357f9f58dcc55958e23f77af59':
        raise ValueError('Contact playback requires the reviewed PR36 source')
    for name, digest in provenance['source_sha256'].items():
        if sha(repo/name) != digest:
            raise ValueError('Contact playback source changed: '+name)
    actual = {str(p.relative_to(packet)): sha(p) for p in packet.rglob('*')
              if p.is_file() and p.name != 'packet-provenance.json'}
    if actual != provenance['files_sha256'] or any(p.is_symlink() for p in packet.rglob('*')):
        raise ValueError('Contact playback packet changed')
    receipt = json.loads((packet/'receipt.json').read_text())
    if receipt['executable_sha256'] != provenance['executable_sha256']:
        raise ValueError('Contact playback executable identity changed')
    if (receipt['accepted'], receipt['refused'], receipt['checker_negative_probes']) != (19, 9, 11):
        raise ValueError('Contact playback oracle qualification incomplete')
    cases = []
    for case in receipt['cases']:
        name = case['name']
        path = packet/(name+'.native.json')
        if sha(path) != case['native_sha256'] or sha(packet/(name+'.input')) != case['input_sha256']:
            raise ValueError('Contact playback case binding changed: '+name)
        cases.append((name, json.loads(path.read_text())))
    return cases


def drawing(raw):
    # Fixed orthographic view. Circles are projections of the declared sphere;
    # polygons use the actual retained native vertices, never integrated poses.
    axis_x = (2**-.5, -2**-.5, 0)
    axis_y = (-1/math.sqrt(6), -1/math.sqrt(6), 2/math.sqrt(6))
    def project(p):
        return (sum(a*b for a,b in zip(axis_x,p)), -sum(a*b for a,b in zip(axis_y,p)))
    allpoints = raw['static_vertices']+raw['before']['vertices']+raw['after']['vertices']
    projected = [project(p) for p in allpoints]
    radius = raw['radius_m']
    lo = [min(p[d] for p in projected)-radius for d in range(2)]
    hi = [max(p[d] for p in projected)+radius for d in range(2)]
    scale = min(560/(hi[0]-lo[0]), 330/(hi[1]-lo[1]))
    def xy(p):
        q=project(p)
        return (20+(q[0]-lo[0])*scale, 20+(q[1]-lo[1])*scale)
    def polygon(points, color):
        coords=' '.join(f'{x:.6f},{y:.6f}' for x,y in map(xy,points))
        return f'<polygon points="{coords}" fill="{color}" fill-opacity=".15" stroke="{color}"/>'
    paths=[polygon([raw['static_vertices'][i] for i in t], '#52676d') for t in raw['static_triangles']]
    for state,color in [('before','#067d91'),('after','#aa5735')]:
        frame=raw[state]
        paths.extend(polygon([frame['vertices'][i] for i in t],color) for t in raw['moving_triangles'])
        x,y=xy(frame['center'])
        paths.append(f'<circle data-role="collider" cx="{x:.6f}" cy="{y:.6f}" r="{radius*scale:.6f}" fill="none" stroke="{color}" stroke-dasharray="4 3"/>')
    hit=raw['result']['hit'] if raw['result'] else None
    if hit is not None:
        x,y=xy(hit['point'])
        end=[p+radius*n for p,n in zip(hit['point'],hit['normal'])]
        ex,ey=xy(end)
        paths.append(f'<circle cx="{x:.6f}" cy="{y:.6f}" r="3" fill="#b12e42"/><path d="M {x:.6f} {y:.6f} L {ex:.6f} {ey:.6f}" stroke="#b12e42" stroke-width="2"/>')
    return '<svg viewBox="0 0 600 380" role="img" aria-label="Fixed orthographic projection of actual native static facets, before and after meshes, and declared sphere colliders">'+''.join(paths)+'</svg>'


def publish(repo, output, packet=None):
    cases=validate_packet(repo,packet)
    intro='''<p class="eyebrow">FINITE TRIANGLE / ONE IMPACT</p><h1>Inspect the actual sphere event.</h1><p><a href="implementation/static-sphere-contact.html">Derivation, numerical policy and Lean contracts</a> · <a href="implementation/release-learning-path.html">Learning path</a></p><p>These are saved native snapshots. The selector changes the displayed case; it runs no simulation and interpolates no pose. A unique hit stops after one frictionless sphere impact. There is no repeated/resting contact or fluid coupling.</p>'''
    if cases is None:
        return intro+'<p>Qualified sphere-contact packet is absent. No replacement simulation was run for this page.</p>'
    shutil.copytree(packet,Path(output)/'sphere-contact-packet')
    options=''.join(f'<option value="{i}">{html.escape(name)}</option>' for i,(name,_) in enumerate(cases))
    sections=[]
    for i,(name,raw) in enumerate(cases):
        before,after=raw['before'],raw['after']
        result=raw['result']
        hit=result['hit'] if result else None
        metrics={'feature':hit['feature'] if hit else ('refused' if raw['error'] else 'miss'),
                 'stored start time (s)':before['time_s'], 'stored end time (s)':after['time_s'],
                 'COM before (m)':before['center'], 'COM after (m)':after['center'],
                 'velocity before (m/s)':before['velocity'], 'velocity after (m/s)':after['velocity'],
                 'unused interval (s)':result['unused_interval_s'] if result else None,
                 'refusal':raw['error']}
        if hit is not None:
            impact=result['impact']
            metrics.update({'contact point (m)':hit['point'],'unit normal':impact['normal'],
                            'normal impulse (N s)':impact['impulse_n_s'],
                            'kinetic energy before (J)':impact['kinetic_before_j'],
                            'kinetic energy after (J)':impact['kinetic_after_j'],
                            'gap residual, unenclosed (m)':impact['gap_residual_m']})
        table=''.join('<tr><th scope="row">'+html.escape(k)+'</th><td>'+html.escape(json.dumps(v))+'</td></tr>' for k,v in metrics.items())
        sections.append(f'<section class="contact-case" data-case="{i}"><h2>{html.escape(name)}</h2>{drawing(raw)}<p>Teal: stored start. Brown: stored end. Gray: actual static facets. Red: actual contact point and normal. Dashed circles: orthographic projections of declared sphere colliders; solid polygons: retained render/traction meshes. Fixed world view, metres. Rounded diagnostics are unenclosed; measured fixture errors give no global accuracy bound.</p><table>{table}</table><p><a href="sphere-contact-packet/{name}.native.json">Actual native JSON</a> · <a href="sphere-contact-packet/{name}.oracle.json">Independent oracle</a></p></section>')
    return intro+'<label for="contact-case">Recorded case</label><select id="contact-case">'+options+'</select><p><a href="sphere-contact-packet/receipt.json">Numerical receipt and measured fixture errors</a> · <a href="sphere-contact-packet/packet-provenance.json">Source and file bindings</a></p><noscript><p>All recorded cases are shown below for reading and printing.</p></noscript>'+''.join(sections)+'''<script>const choice=document.querySelector('#contact-case');function show(){document.querySelectorAll('.contact-case').forEach((s,i)=>s.hidden=i!==Number(choice.value));}choice.onchange=show;show();</script>'''
