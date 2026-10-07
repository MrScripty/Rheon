"""Check both archived crate graphs; make no external-memory qualification."""


def require(ok, message):
    if not ok:
        raise ValueError(message)


def check_boundaries(memory):
    results = {}
    for kind in ['candidate', 'reference']:
        frames = {int(a): row for a, row in memory['frames'][kind].items()}
        graph = {int(a): targets for a, targets in memory['graphs'][kind].items()}
        roots = [a for a, row in frames.items()
                 if row['name'].endswith('::Work::case41_paired_equation')]
        require(len(roots) == 1, kind + ': one numerical root')
        pending = roots[:]
        seen = set()
        while pending:
            address = pending.pop()
            if address in seen:
                continue
            require(address in frames and address in graph, kind + ': full reached graph')
            seen.add(address)
            pending.extend(int(a) for a in graph[address])
        require(not any(frames[a]['name'].endswith('::Work::case41_serialize') for a in seen),
                kind + ': serialization outside numerical boundary')
        require(not any('std::io::stdio::_print' in str(edge.get('binding'))
                        for a in seen for edge in frames[a]['transfers']),
                kind + ': no stdout transfer inside numerical call')
        results[kind] = dict(root=roots[0], reached_crate_functions=len(seen),
                             serialization_outside_numerical_boundary=True,
                             stdout_outside_numerical_boundary=True)
    require(set(results) == {'candidate', 'reference'}, 'both checks finish before returning')
    return results
