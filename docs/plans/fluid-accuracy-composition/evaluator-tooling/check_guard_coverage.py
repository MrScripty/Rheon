"""Check existing refusal logs; this never executes an evaluator or solver."""
import ast
import json
from pathlib import Path
import sys

root = Path(sys.argv[1])
here = Path(__file__).resolve().parent
expected = set()
for frozen in (here / 'frozen').glob('*.py'):
    for node in ast.walk(ast.parse(frozen.read_text())):
        if isinstance(node, ast.Assert):
            expected.add(f'evaluator input refused: {frozen.stem}: input contract at original line {node.lineno}')
covered = {}
for path in (root / 'logs').glob('*.stderr'):
    message = path.read_text().strip()
    if message not in expected:
        continue
    mode = '-OO' if path.name.endswith('--OO.stderr') else '-O' if path.name.endswith('--O.stderr') else 'normal'
    covered.setdefault(message, set()).add(mode)
if set(covered) != expected or any(modes != {'normal','-O','-OO'} for modes in covered.values()):
    raise SystemExit('incomplete always-on guard coverage')
report = dict(original_guards=len(expected), modes_per_guard=3, all_guards_all_modes=True,
              guards={k:sorted(v) for k,v in sorted(covered.items())})
(root / 'guard-coverage.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps({k:v for k,v in report.items() if k!='guards'}, indent=2))
