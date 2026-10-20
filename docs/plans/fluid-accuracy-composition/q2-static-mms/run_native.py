"""Prospective serialized CV3-A native campaign owner; never runs on import.

This driver is prepared, not executed, under the preparation-only handoff.
An eventual run uses the separately reviewed complete preparation snapshot.
"""
import hashlib
import json
from pathlib import Path
import sys
import signal
import time
from resource_supervisor import require, run_child

CASES = tuple(f'{family}-n{n}' for family in (
    'rectangle-origin', 'square-17-origin', 'square-17-quarter') for n in (4, 8, 16))
BINARY = Path('/workspace/rheon-q2-static-mms-evidence/grid_static_mms_reference')
BINARY_SHA256 = '72f4245bf56a56da6e8a8c223c2a7a56220fc914491cf6997416c269945f9ad4'


def main():
    require(len(sys.argv) == 2, 'usage run_native.py FRESH_NATIVE_ROOT')
    root = Path(sys.argv[1]).absolute()
    require(not root.exists(), 'new native campaign path required')
    require(BINARY.is_file() and not BINARY.is_symlink(), 'frozen ordinary native executable')
    require(hashlib.sha256(BINARY.read_bytes()).hexdigest() == BINARY_SHA256, 'native preparation binary identity')
    root.mkdir()
    receipts = root  # receipt/log files stay inside the accounted corpus root
    deadline = time.monotonic()+900
    inventory = {'native_preparation_checkpoint': 'b9ec50c779122587a5da30318b29b42e1825ec10',
        'binary': str(BINARY), 'binary_sha256': BINARY_SHA256,
        'cases': list(CASES), 'completed': [], 'status': 'starting'}
    path = root/'campaign.json'
    def retain():
        path.write_text(json.dumps(inventory, indent=2, sort_keys=True)+'\n')
    retain()
    try:
        for case in CASES:
            argv = [str(BINARY), str(root/case), case]
            run_child(argv, root, receipts/(case+'-receipt.json'), kind='native',
                frozen_files={str(BINARY): BINARY_SHA256}, total_deadline=deadline)
            inventory['completed'].append(case)
            inventory['status'] = 'running'
            retain()
        require(len(inventory['completed']) == 9, 'complete fixed nine-case corpus')
        inventory['status'] = 'native-completed-independent-qualification-pending'
    except BaseException as error:
        inventory.update(status='stopped', reason=str(error))
        raise
    finally:
        retain()


if __name__ == '__main__':
    def terminated(signum, frame):
        raise InterruptedError('CV3 native campaign termination requested')
    previous = signal.signal(signal.SIGTERM, terminated)
    try:
        try:
            main()
        except Exception as error:
            print(str(error), file=sys.stderr)
            sys.exit(1)
    finally:
        signal.signal(signal.SIGTERM, previous)
