"""Publish source-bound recorded labs; never execute a numerical example."""
from pathlib import Path
import hashlib, html, json, re, shutil

GUIDES = ['release-learning-path', 'native-wall-force-sequence', 'column-wall-friction',
          'column-no-slip', 'column-poiseuille', 'static-obstacle-geometry',
          'static-obstacle-flow', 'reconstructed-aligned-strain',
          'aligned-strain-laboratory', 'experimental-aligned-stokes',
          'triangle-mesh-traction', 'rigid-mesh-impulse', 'spherical-rigid-motion',
          'static-sphere-contact', 'requirements-roadmap']

def metadata(repo):
    repo = Path(repo)
    data = json.loads((repo/'docs/education/native-sequence.json').read_text())
    pins = repo/'proofs/source-inventory.json'
    if hashlib.sha256(pins.read_bytes()).hexdigest() != data['proof_inventory_sha256']:
        raise ValueError('Current Lean inventory differs from reviewed current source; requalify.')
    for name, digest in json.loads(pins.read_text()).items():
        if hashlib.sha256((repo/'proofs'/name).read_bytes()).hexdigest() != digest:
            raise ValueError('Current Lean source differs from its inventory: '+name)
    if 'historical_native_proof' in data:
        historical=repo/'proofs/source-inventory-pr24.json'
        if hashlib.sha256(historical.read_bytes()).hexdigest()!=data['historical_native_proof']['proof_inventory_sha256']:
            raise ValueError('Historical PR24 source inventory changed')
    return data

def validate_proof_qualification(data, repo, directory):
    """New release bindings require the actual external kernel receipt and logs."""
    if not data.get('release_qualification_required'):
        return
    if directory is None:
        raise ValueError('PR36 requires the fresh current proof qualification bundle')
    directory=Path(directory)
    receipt=directory/'qualification.json'
    if hashlib.sha256(receipt.read_bytes()).hexdigest()!=data['local_kernel_receipt_sha256']:
        raise ValueError('Current proof qualification receipt differs from reviewed binding')
    actual=json.loads(receipt.read_text())
    if actual.get('status')!='passed' or actual.get('source_head')!=data['current_reconstruction_proof']['source_head']:
        raise ValueError('Current proof qualification source/status differs')
    if actual.get('proof_inventory_sha256')!=data['proof_inventory_sha256']:
        raise ValueError('Current proof qualification inventory differs')
    inventory=json.loads((Path(repo)/'proofs/source-inventory.json').read_text())
    if actual.get('proof_source_sha256')!=inventory or actual.get('lean_checked') is not True:
        raise ValueError('Current proof qualification omits reviewed proof sources')
    if actual.get('allowed_axioms')!=['propext','Classical.choice','Quot.sound'] or 'version 4.19.0,' not in actual.get('lean_version',''):
        raise ValueError('Current proof qualification compiler/axiom policy differs')
    pins={p['name']:p['rev'] for p in json.loads((Path(repo)/'proofs/lake-manifest.json').read_text())['packages']}
    if actual.get('dependency_pins')!=pins:
        raise ValueError('Current proof qualification dependency pins differ')
    required={('lake','build'),('lake','env','lean','AxiomAudit.lean'),
              ('python3','scripts/check_sources.py'),('python3','scripts/test_audit.py'),
              ('python3','-O','scripts/test_audit.py'),('python3','scripts/test_check_sources.py'),
              ('python3','-O','scripts/test_check_sources.py')}
    observed={tuple(c['command']) if isinstance(c['command'],list) else tuple(c['command'].split()) for c in actual['commands']}
    if observed!=required or len(actual['commands'])!=len(required):
        raise ValueError('Current proof qualification omits required commands')
    for name,digest in actual['proof_source_sha256'].items():
        if hashlib.sha256((Path(repo)/'proofs'/name).read_bytes()).hexdigest()!=digest:
            raise ValueError('Qualified proof source changed: '+name)
    for command in actual['commands']:
        if command['returncode']!=0 or hashlib.sha256((directory/command['log']).read_bytes()).hexdigest()!=command['log_sha256']:
            raise ValueError('Current proof qualification command/log differs')
    for key in ['public_theorems','explicit_expected_declarations','audited_declarations']:
        if actual[key]!=data[key]:
            raise ValueError('Current proof qualification count differs: '+key)

def validate_labs(data, directory):
    if directory is None:
        return
    directory = Path(directory)
    for lab in data['labs']:
        root = directory/lab['id']
        if any(not p.is_file() or p.is_symlink() for p in root.iterdir()):
            raise ValueError('Recorded lab packet must contain only declared regular files: '+lab['id'])
        actual = {p.name: hashlib.sha256(p.read_bytes()).hexdigest()
                  for p in root.iterdir() if p.is_file()}
        if actual != lab['files']:
            raise ValueError('Recorded lab bytes differ from accepted packet: '+lab['id'])

def publish(data, output, directory=None):
    output = Path(output)
    cards = []
    for i, lab in enumerate(data['labs'], 1):
        key = lab['id']
        if directory is not None:
            dest = output/'native-labs'/key
            shutil.copytree(Path(directory)/key, dest)
            if key == 'wall-friction':
                # Original HTML/CSV is preserved at input. This derived banner
                # states that case links show saved profiles, not live parameters.
                for p in dest.glob('*.html'):
                    text = p.read_text()
                    note = '<p role="note" class="snapshot-note"><strong>Snapshot playback.</strong> These pages display six fixed native runs. Case links select recorded results; they do not recompute parameters or run a live simulation. CSVs retain four actual saved times. The plot displays the final saved profile, not a general liquid solver.</p>'
                    text = re.sub(r'(<h1>.*?</h1>)', lambda m: m[0]+note, text, count=1)
                    text = text.replace('<meta charset=utf-8>', '<meta charset=utf-8><meta name="viewport" content="width=device-width,initial-scale=1"><style>body{font:16px system-ui;max-width:900px;margin:24px auto;padding:0 16px}svg{width:100%;height:auto}table{width:100%;font-size:.85rem;table-layout:fixed;overflow-wrap:anywhere}td,th{padding:5px!important}.snapshot-note{padding:12px;background:#eff7fa;line-height:1.5}</style>')
                    p.write_text(text)
            action = f'<a href="native-labs/{key}/index.html">Open recorded native lab →</a>'
        else:
            action = '<p>Recorded bundle is not included in this source-only build. No simulation was run to create a substitute.</p>'
        guide = {'wall-friction':'column-wall-friction', 'no-slip':'column-no-slip', 'poiseuille':'column-poiseuille'}[key]
        cards.append(f'<article><h2>{i} · {html.escape(key)}</h2><p>{html.escape(lab["model"])}</p><p>{lab["counts"]["cases"]} fixed cases; {lab["counts"]["saved_times"]} saved times.</p><p>Reference: {html.escape(lab["reference"])}</p>{action}<p><a href="implementation/{guide}.html">Equation, units and proof assumptions →</a></p></article>')
    (output/'native-sequence.json').write_text(json.dumps(data, indent=2)+'\n')
    receipt = {'schema':'rheon-native-presentation-v1', 'numerical_calls':0,
               'recorded_bundles_included':directory is not None,
               'accepted_inputs':data['labs'],
               'files':{str(p.relative_to(output)):hashlib.sha256(p.read_bytes()).hexdigest()
                        for p in sorted((output/'native-labs').rglob('*')) if p.is_file()}}
    (output/'native-presentation.json').write_text(json.dumps(receipt, indent=2)+'\n')
    return '<p class="eyebrow">THREE NATIVE STEPS / ONE FIXED SLAB</p><h1>Wall traction, exact trace, external force.</h1><p class="lede">Follow the same liquid dual mass and endpoint basis through finite Navier traction, compatible no-slip constraints, then uniform tangential forcing.</p><p role="note"><strong>Snapshot playback.</strong> Native controls select recorded cases and saved times. They do not provide live parameter recomputation or a general liquid solver. The original six 3D reference labs are separate analytical/stored mechanisms.</p><p><a href="implementation/native-wall-force-sequence.html">Read the progression</a> · <a href="implementation/requirements-roadmap.html">Remaining requirements and next geometry contract</a></p><div class="cards">'+''.join(cards)+'</div>'
