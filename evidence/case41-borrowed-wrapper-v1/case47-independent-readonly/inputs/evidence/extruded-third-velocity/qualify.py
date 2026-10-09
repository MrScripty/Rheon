"""Record actual successor commands; earlier source-bound Rust runs are reused."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

P = Path(__file__).resolve().parent
ROOT = P.parents[1]
OUT = P / 'final-qualification'
OUT.mkdir(exist_ok=True)
phase = sys.argv[1]
source = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
records = []
env = dict(os.environ, OPENBLAS_NUM_THREADS='1')


def run(name, argv, suffix='log'):
    output = OUT / (name + '.' + suffix)
    error = OUT / (name + '-stderr.log')
    if output.exists() or error.exists():
        raise ValueError('refusing to overwrite qualification: ' + name)
    with output.open('wb') as stdout, error.open('wb') as stderr:
        answer = subprocess.run(argv, cwd=ROOT, env=env, stdout=stdout, stderr=stderr)
    records.append(dict(name=name, command=argv, exit=answer.returncode, stdout=str(output.relative_to(ROOT)), stderr=str(error.relative_to(ROOT))))
    (OUT / (phase + '-commands.json')).write_text(json.dumps(dict(qualified_source_commit=source, commands=records, status='PASS' if all(x['exit'] == 0 for x in records) else 'FAIL'), indent=2) + '\n')
    if answer.returncode:
        raise ValueError('actual command failed: ' + name)
    print('PASS ' + name, flush=True)


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


if phase == 'captures':
    run('native-default', ['cargo', 'run', '--release', '--quiet', '--example', 'extruded_third'], 'jsonl')
    run('native-no-default', ['cargo', 'run', '--release', '--quiet', '--no-default-features', '--example', 'extruded_third'], 'jsonl')
    run('legacy-default', ['cargo', 'run', '--release', '--quiet', '--example', 'coupled_discrete_identity'], 'jsonl')
elif phase in ['normal', 'optimized']:
    python = [sys.executable] + (['-O'] if phase == 'optimized' else [])
    run(phase + '-schema-tests', python + [str(P / 'test_replay.py')])
    run(phase + '-replay', python + [str(P / 'replay.py'), str(OUT / 'native-default.jsonl'), '--negative-self-test'], 'json')
elif phase == 'render':
    run('render', [sys.executable, str(P / 'render_native.py'), str(OUT / 'native-default.jsonl'), str(OUT / 'renders')])
    run('viewer-vm', ['node', str(P / 'check_viewer.cjs'), str(OUT / 'renders/published-third.html'), str(OUT / 'native-default.jsonl')], 'json')
    run('browser', [sys.executable, str(P / 'check_browser.py'), str(OUT / 'renders/published-third.html')], 'json')
elif phase == 'checks':
    require = lambda ok, message: None if ok else (_ for _ in ()).throw(ValueError(message))
    captures = [OUT / (x + '.jsonl') for x in ['native-default', 'native-no-default']]
    old = P / 'native-trials/mapped-native.jsonl'
    require(captures[0].read_bytes() == captures[1].read_bytes() == old.read_bytes(), 'actual publications changed between builds/modes')
    require((OUT / 'legacy-default.jsonl').read_bytes() == (P.parent / 'coupled-oracle-repair/oracle-qualification/native-default.jsonl').read_bytes(), 'legacy published source behavior changed')
    mode = [OUT / (x + '-replay.json') for x in ['normal', 'optimized']]
    require(mode[0].read_bytes() == mode[1].read_bytes(), 'normal/optimized qualified numerical evidence differs')
    result = json.loads(mode[0].read_text())
    require(result['status'] == 'PASS' and len(result['actual_corruption_rejections']) == 13, 'missing physical replay/controls')
    viewer = json.loads((OUT / 'viewer-vm.json').read_text())
    require(viewer['status'] == 'PASS' and viewer['publications'] == 268 and viewer['cases'] == 20, 'missing recorded UI execution')
    browser = json.loads((OUT / 'browser.json').read_text())
    require(browser['status'] in ['PASS', 'BLOCKED_BROWSER_SANDBOX'], 'unreported browser result')
    stages = [json.loads((OUT / (x + '-commands.json')).read_text()) for x in ['captures', 'normal', 'optimized', 'render']]
    require(all(x['qualified_source_commit'] == source and x['status'] == 'PASS' for x in stages), 'qualification sources differ')
    receipt = dict(status='PASS', qualified_source_commit=source,
        qualified_source_tree=subprocess.check_output(['git', 'rev-parse', source + '^{tree}'], cwd=ROOT, text=True).strip(),
        actual_commands=[c for stage in stages for c in stage['commands']],
        actual_publications=268, actual_constructors=20, actual_steps=248,
        actual_corruption_rejections=13, full_native_modes_identical=True, legacy_publications_byte_identical=True,
        normal_optimized_replays_byte_identical=True, browser_status=browser['status'],
        browser_rasterization_qualified=browser['status'] == 'PASS',
        reused_native_test_binding=str((P / 'full-feature-tests/source-binding.json').relative_to(ROOT)),
        reused_native_preflight_binding=str((P / 'native-preflight/source-binding.json').relative_to(ROOT)),
        maxima=result['maxima'], file_sha256={str(x.relative_to(ROOT)):sha(x) for x in sorted(OUT.rglob('*')) if x.is_file()},
        scope='Actual published fixed-spatial periodic-z-invariant three-component model; conditional temporal evidence. No new Lean/continuum/general-3D claim. Original rollback finding remains shared.')
    (OUT / 'receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps({k: v for k, v in receipt.items() if k not in ['file_sha256', 'actual_commands']}, indent=2))
else:
    raise ValueError('unknown qualification phase')
