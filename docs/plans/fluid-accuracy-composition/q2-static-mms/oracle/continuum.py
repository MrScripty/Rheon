"""Prospective exact CV3 continuum loads and reconstructed physical fields."""
from fractions import Fraction as F
import exact_polynomial as ep
import exact_operators as eo
from validated import require
from bounded_norms import fan


def fields(mu):
    mu = F(mu); x, y = ep.X, ep.Y; one = ep.constant(1)
    xx, yy, xy = ep.multiply(x, x), ep.multiply(y, y), ep.multiply(x, y)
    u = (ep.add(xx, ep.scale(xy, 2)), ep.add(ep.scale(xy, -2), ep.scale(yy, -1)))
    p = ep.add(one, x, ep.scale(y, 2), xy)
    visc = ep.scale(ep.add(x, y), 4*mu)
    sigma = (ep.add(ep.scale(p, -1), visc), ep.scale(ep.add(x, ep.scale(y, -1)), 2*mu), ep.add(ep.scale(p, -1), ep.scale(visc, -1)))
    div_sigma = (ep.add(ep.derivative(sigma[0], 0), ep.derivative(sigma[1], 1)), ep.add(ep.derivative(sigma[1], 0), ep.derivative(sigma[2], 1)))
    body = tuple(ep.add(v, ep.scale(d, -1)) for v, d in zip(u, div_sigma))
    require(ep.add(ep.derivative(u[0], 0), ep.derivative(u[1], 1)) == {}, 'exact continuum incompressibility')
    return {'u': u, 'p': p, 'sigma': sigma, 'body': body}


def integral(poly, patch):
    return sum((v*patch['moments'][e] for e, v in poly.items()), F(0))


def exact_loads(op):
    f = fields(op['mu']); nv = len(op['active']); amap = {g: i for i, g in enumerate(op['active'])}
    body = [F(0)]*nv; boundary = [F(0)]*nv
    for patch in op['patches']:
        if not patch['area']:
            continue
        for node, phi, dx, dy in patch['local']:
            body[amap[node['global']]] += integral(ep.multiply(phi, f['body'][node['axis']]), patch)
    for segment in op['segments']:
        if segment['kind'] != 'global':
            continue
        a, b = segment['a'], segment['b']; nx, ny = segment['normal']
        traction_measure = (ep.add(ep.scale(f['sigma'][0], nx), ep.scale(f['sigma'][1], ny)),
                            ep.add(ep.scale(f['sigma'][1], nx), ep.scale(f['sigma'][2], ny)))
        midpoint = tuple((a[d]+b[d])/2 for d in range(2))
        for i, g in enumerate(op['active']):
            node = op['nodes'][g]
            phi = eo.hat(node, midpoint, op['h'])
            boundary[i] += ep.segment_integral_parameter(ep.multiply(phi, traction_measure[node['axis']]), a, b)
    return {'body': body, 'boundary': boundary, 'rhs': [a+b for a, b in zip(body, boundary)]+[F(0)]*len(op['cells']), 'fields': f}


def reconstruction(op, velocity, pressure):
    require(len(velocity) == len(op['active']) and len(pressure) == len(op['cells']), 'complete physical state')
    amap = {g: i for i, g in enumerate(op['active'])}; cmap = {c: i for i, c in enumerate(op['cells'])}
    out = []
    for patch in op['patches']:
        if not patch['area']:
            continue
        u = [{}, {}]
        for node, phi, dx, dy in patch['local']:
            d = node['axis']; u[d] = ep.add(u[d], ep.scale(phi, velocity[amap[node['global']]]))
        p = ep.constant(pressure[cmap[patch['cell']]])
        mu = op['mu']
        sigma = (ep.add(ep.scale(ep.derivative(u[0], 0), 2*mu), ep.scale(p, -1)),
                 ep.scale(ep.add(ep.derivative(u[0], 1), ep.derivative(u[1], 0)), mu),
                 ep.add(ep.scale(ep.derivative(u[1], 1), 2*mu), ep.scale(p, -1)))
        out.append({'patch': patch, 'u': tuple(u), 'p': p, 'sigma': sigma})
    return out


def volume_items(reconstructed, field, exact):
    items = []
    for piece in reconstructed:
        got = piece[field] if field == 'u' else (piece[field],)
        expected = exact if field == 'u' else (exact,)
        error = tuple(ep.add(a, ep.scale(b, -1)) for a, b in zip(got, expected))
        items.extend(('triangle', error, tri) for tri in fan(piece['patch']['poly']))
    return items


def boundary_pieces(op, reconstructed):
    # Clip every physical source edge by actual common boxes. A source segment
    # lying on a box boundary could have two owners; choose its liquid-side
    # interior by exact CCW orientation, not epsilon displacements.
    out = []
    for edge, (a, b) in enumerate(zip(op['source'], op['source'][1:]+op['source'][:1])):
        nx, ny = b[1]-a[1], a[0]-b[0]
        cuts = []
        for piece in reconstructed:
            patch = piece['patch']; segment = ep.clip_segment(a, b, patch['lo'], patch['hi'])
            if segment is None:
                continue
            pa, pb = segment; mid = tuple((pa[d]+pb[d])/2 for d in range(2))
            # The inward direction (-nx,-ny) must enter this patch or remain
            # tangential. This excludes only the dry-side ownership at a knot.
            admissible = all(not (mid[d] == patch['lo'][d] and (nx, ny)[d] > 0)
                             and not (mid[d] == patch['hi'][d] and (nx, ny)[d] < 0) for d in range(2))
            if admissible:
                cuts.append((pa, pb, piece))
        # Exact coverage, no duplicated ownership or unrepresented length.
        axis = 0 if b[0] != a[0] else 1
        intervals = sorted((((pa[axis]-a[axis])/(b[axis]-a[axis]), (pb[axis]-a[axis])/(b[axis]-a[axis]), pa, pb, piece)
                            for pa, pb, piece in cuts), key=lambda r: (r[0], r[1]))
        cursor = F(0)
        for t0, t1, pa, pb, piece in intervals:
            require(t0 == cursor and t0 < t1, 'exact boundary ownership/coverage')
            cursor = t1
            out.append({'edge': edge, 'a': pa, 'b': pb, 'source_a': a, 'source_b': b,
                        'parameter_weight': t1-t0, 'normal_measure': (nx, ny), 'piece': piece})
        require(cursor == 1, 'complete physical source edge coverage')
    return out
