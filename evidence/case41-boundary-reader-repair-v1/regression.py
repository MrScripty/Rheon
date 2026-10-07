"""Two-side negative regression on real frozen graph fixtures; no native call."""
from pathlib import Path
import copy
import gzip
import hashlib
import json
from boundary import check_boundaries, require

P = Path(__file__).resolve().parent
F = P.parent / 'case41-paired-observation-v1'


def run():
    raw = gzip.decompress((F / 'memory-normal.json.gz').read_bytes())
    frozen = json.loads(raw)
    actual = check_boundaries(frozen)
    controls = []
    for kind in ['candidate', 'reference']:
        for fault in ['serialization', 'stdout']:
            altered = copy.deepcopy(frozen)
            frames = altered['frames'][kind]
            root = next(a for a, row in frames.items()
                        if row['name'].endswith('::Work::case41_paired_equation'))
            if fault == 'serialization':
                serializer = next(int(a) for a, row in frames.items()
                                  if row['name'].endswith('::Work::case41_serialize'))
                altered['graphs'][kind][root].append(serializer)
            else:
                frames[root]['transfers'].append(dict(binding=dict(names=['std::io::stdio::_print'])))
            try:
                check_boundaries(altered)
            except ValueError as error:
                require(str(error).startswith(kind + ':'), 'correct side rejects the injected fault')
                controls.append(dict(side=kind, fault=fault, rejected=True, reason=str(error)))
            else:
                raise ValueError(kind + ': injected fault escaped check')
    return dict(status='PASS_BOTH_ARCHIVED_BOUNDARIES_AND_FOUR_NEGATIVES',
                archived_graph_sha256=hashlib.sha256(raw).hexdigest(),
                boundary_checks=actual, negative_controls=controls,
                complete_external_memory_accounting=False,
                historical_memory_qualified=False, native_equations=0)


if __name__ == '__main__':
    print(json.dumps(run(), indent=2, sort_keys=True))
